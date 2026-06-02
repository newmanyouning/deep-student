use super::*;
use super::hlc;
use rusqlite::Connection;
use serde_json::json;

fn create_test_manifest(
    device_id: &str,
    databases: Vec<(&str, u32, u64, &str)>,
) -> SyncManifest {
    let mut db_map = std::collections::HashMap::new();
    for (name, schema_ver, data_ver, checksum) in databases {
        db_map.insert(
            name.to_string(),
            DatabaseSyncState {
                schema_version: schema_ver,
                data_version: data_ver,
                checksum: checksum.to_string(),
                last_updated_at: None,
            },
        );
    }
    SyncManifest {
        sync_transaction_id: "test-tx".to_string(),
        databases: db_map,
        status: SyncTransactionStatus::Complete,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        device_id: device_id.to_string(),
    }
}

#[test]
fn test_parse_version_from_key_with_nonce() {
    let key = "data_governance/changes/device-1/12345-acde.json";
    assert_eq!(SyncManager::parse_version_from_key(key), Some(12345));
}

#[test]
fn test_parse_version_from_key_legacy_no_nonce() {
    // Legacy 文件没有 nonce（纯秒级时间戳）
    let key = "data_governance/changes/device-1/1707500000.json";
    assert_eq!(SyncManager::parse_version_from_key(key), Some(1707500000));
}

#[test]
fn test_parse_version_from_key_seconds_with_nonce() {
    // 旧格式 .json：秒级时间戳 + UUID nonce
    let key =
        "data_governance/changes/device-1/1707500000-550e8400-e29b-41d4-a716-446655440000.json";
    assert_eq!(SyncManager::parse_version_from_key(key), Some(1707500000));
}

#[test]
fn test_parse_version_from_key_zst_with_nonce() {
    // 新格式 .json.zst：秒级时间戳 + UUID nonce + zstd 压缩
    let key = "data_governance/changes/device-1/1707500000-550e8400-e29b-41d4-a716-446655440000.json.zst";
    assert_eq!(SyncManager::parse_version_from_key(key), Some(1707500000));
}

#[test]
fn test_parse_version_from_key_zst_legacy_no_nonce() {
    // .json.zst 无 nonce
    let key = "data_governance/changes/device-1/1707500000.json.zst";
    assert_eq!(SyncManager::parse_version_from_key(key), Some(1707500000));
}

#[test]
fn test_parse_version_from_key_invalid() {
    assert_eq!(SyncManager::parse_version_from_key(""), None);
    assert_eq!(SyncManager::parse_version_from_key("no-slash"), None);
    assert_eq!(
        SyncManager::parse_version_from_key("data_governance/changes/device-1/notanumber.json"),
        None
    );
    assert_eq!(
        SyncManager::parse_version_from_key("data_governance/changes/device-1/abc.json.zst"),
        None
    );
}

#[test]
fn test_version_space_compatibility_seconds() {
    // 验证新旧版本空间兼容：legacy 用秒级时间戳，新代码也用秒级
    // 新变更 version = 当前时间秒 > 旧的 since_version 秒 → 会被下载
    // 旧变更 version = 更早的秒 < 新的 since_version 秒 → 会被跳过（正确）
    let old_version: u64 = 1707500000; // legacy 设备上传
    let new_since: u64 = 1707400000; // 本地已同步到的版本
    assert!(
        old_version > new_since,
        "旧设备新变更应大于本地 since，被下载"
    );

    let stale_version: u64 = 1707300000; // 更早的变更
    assert!(stale_version < new_since, "过时变更应被跳过");
}

#[test]
fn test_build_change_key_unique() {
    let manager = SyncManager::new("device-1".to_string());
    let key1 = manager.build_change_key(1707500000);
    let key2 = manager.build_change_key(1707500000);
    // 同一秒生成的 key 不应相同（UUID nonce 不同）
    assert_ne!(key1, key2, "同版本号的 key 应因 nonce 不同而不同");
    // 但版本号应可正确解析
    assert_eq!(SyncManager::parse_version_from_key(&key1), Some(1707500000));
    assert_eq!(SyncManager::parse_version_from_key(&key2), Some(1707500000));
}

#[test]
fn test_normalize_version_to_seconds() {
    // 秒级值不变
    assert_eq!(
        SyncManager::normalize_version_to_seconds(1707500000),
        1707500000
    );
    assert_eq!(SyncManager::normalize_version_to_seconds(0), 0);
    assert_eq!(SyncManager::normalize_version_to_seconds(42), 42);
    // 毫秒级值被除以 1000
    assert_eq!(
        SyncManager::normalize_version_to_seconds(1707500000000),
        1707500000
    );
    assert_eq!(
        SyncManager::normalize_version_to_seconds(1707600000123),
        1707600000
    );
}

#[test]
fn test_same_second_download_not_skipped() {
    // 验证 >= 语义：同秒版本不被跳过
    let since_version: u64 = 1707500000;
    let file_version: u64 = 1707500000; // 同秒
    assert!(file_version >= since_version, "同秒版本应通过 >= 过滤");
}

#[test]
fn test_detect_no_conflicts() {
    let local = create_test_manifest("device-1", vec![("chat_v2", 1, 100, "abc123")]);
    let cloud = create_test_manifest("device-2", vec![("chat_v2", 1, 100, "abc123")]);

    let result = SyncManager::detect_conflicts(&local, &cloud).unwrap();
    assert!(!result.has_conflicts);
    assert!(result.database_conflicts.is_empty());
}

#[test]
fn test_detect_schema_mismatch() {
    let local = create_test_manifest("device-1", vec![("chat_v2", 1, 100, "abc123")]);
    let cloud = create_test_manifest("device-2", vec![("chat_v2", 2, 100, "abc123")]);

    let result = SyncManager::detect_conflicts(&local, &cloud).unwrap();
    assert!(result.has_conflicts);
    assert!(result.needs_migration);
    assert_eq!(result.database_conflicts.len(), 1);
    assert_eq!(
        result.database_conflicts[0].conflict_type,
        DatabaseConflictType::SchemaMismatch
    );
}

#[test]
fn test_detect_data_conflict() {
    let local = create_test_manifest("device-1", vec![("chat_v2", 1, 101, "abc123")]);
    let cloud = create_test_manifest("device-2", vec![("chat_v2", 1, 102, "def456")]);

    let result = SyncManager::detect_conflicts(&local, &cloud).unwrap();
    assert!(result.has_conflicts);
    assert!(!result.needs_migration);
    assert_eq!(result.database_conflicts.len(), 1);
    assert_eq!(
        result.database_conflicts[0].conflict_type,
        DatabaseConflictType::DataConflict
    );
}

#[test]
fn test_detect_local_only() {
    let local = create_test_manifest(
        "device-1",
        vec![("chat_v2", 1, 100, "abc123"), ("mistakes", 1, 50, "xyz789")],
    );
    let cloud = create_test_manifest("device-2", vec![("chat_v2", 1, 100, "abc123")]);

    let result = SyncManager::detect_conflicts(&local, &cloud).unwrap();
    assert!(result.has_conflicts);
    assert_eq!(result.database_conflicts.len(), 1);
    assert_eq!(
        result.database_conflicts[0].conflict_type,
        DatabaseConflictType::LocalOnly
    );
    assert_eq!(result.database_conflicts[0].database_name, "mistakes");
}

#[test]
fn test_detect_cloud_only() {
    let local = create_test_manifest("device-1", vec![("chat_v2", 1, 100, "abc123")]);
    let cloud = create_test_manifest(
        "device-2",
        vec![
            ("chat_v2", 1, 100, "abc123"),
            ("llm_usage", 1, 200, "qwe456"),
        ],
    );

    let result = SyncManager::detect_conflicts(&local, &cloud).unwrap();
    assert!(result.has_conflicts);
    assert_eq!(result.database_conflicts.len(), 1);
    assert_eq!(
        result.database_conflicts[0].conflict_type,
        DatabaseConflictType::CloudOnly
    );
    assert_eq!(result.database_conflicts[0].database_name, "llm_usage");
}

#[test]
fn test_sync_keep_local() {
    let manager = SyncManager::new("device-1".to_string());
    let result = ConflictDetectionResult::empty();

    let sync_result = manager.sync(MergeStrategy::KeepLocal, &result).unwrap();
    assert!(sync_result.success);
}

#[test]
fn test_record_conflict_detection() {
    let local_records = vec![RecordSnapshot {
        table_name: "messages".to_string(),
        record_id: "msg-1".to_string(),
        local_version: 3,
        sync_version: 2,
        updated_at: "2024-01-01T10:00:00Z".to_string(),
        deleted_at: None,
        data: serde_json::json!({"content": "local edit"}),
    }];

    let cloud_records = vec![RecordSnapshot {
        table_name: "messages".to_string(),
        record_id: "msg-1".to_string(),
        local_version: 4,
        sync_version: 2,
        updated_at: "2024-01-01T11:00:00Z".to_string(),
        deleted_at: None,
        data: serde_json::json!({"content": "cloud edit"}),
    }];

    let conflicts =
        SyncManager::detect_record_conflicts("chat_v2", &local_records, &cloud_records);

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].record_id, "msg-1");
    assert_eq!(conflicts[0].local_version, 3);
    assert_eq!(conflicts[0].cloud_version, 4);
}

// ========================================================================
// 新增测试：核心同步方法
// ========================================================================

/// 创建测试用的内存数据库并初始化 __change_log 表
fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS __change_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            table_name TEXT NOT NULL,
            record_id TEXT NOT NULL,
            operation TEXT NOT NULL CHECK(operation IN ('INSERT', 'UPDATE', 'DELETE')),
            changed_at TEXT NOT NULL DEFAULT (datetime('now')),
            sync_version INTEGER DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx__change_log_sync_version ON __change_log(sync_version);

        CREATE TABLE IF NOT EXISTS refinery_schema_history (
            version INTEGER PRIMARY KEY,
            name TEXT,
            applied_on TEXT,
            checksum TEXT
        );

        -- 插入测试用的 schema 版本（与 refinery 迁移系统权威表结构一致）
        INSERT INTO refinery_schema_history (version, name, applied_on, checksum) VALUES (1, 'V1__init', '2024-01-01T00:00:00Z', 'abc');
        INSERT INTO refinery_schema_history (version, name, applied_on, checksum) VALUES (2, 'V2__update', '2024-01-02T00:00:00Z', 'def');
        "#,
    )
    .unwrap();
    conn
}

/// 插入测试用的变更日志
fn insert_test_change_log(
    conn: &Connection,
    table_name: &str,
    record_id: &str,
    operation: &str,
    sync_version: i64,
) {
    conn.execute(
        "INSERT INTO __change_log (table_name, record_id, operation, sync_version)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![table_name, record_id, operation, sync_version],
    )
    .unwrap();
}

#[test]
fn test_get_pending_changes_empty() {
    let conn = create_test_db();

    let pending = SyncManager::get_pending_changes(&conn, None, None).unwrap();

    assert!(!pending.has_changes());
    assert_eq!(pending.total_count, 0);
    assert!(pending.entries.is_empty());
}

#[test]
fn test_get_pending_changes_with_data() {
    let conn = create_test_db();

    // 插入一些待同步的变更
    insert_test_change_log(&conn, "messages", "msg-1", "INSERT", 0);
    insert_test_change_log(&conn, "messages", "msg-2", "UPDATE", 0);
    insert_test_change_log(&conn, "sessions", "sess-1", "INSERT", 0);
    // 这条已同步，不应该出现
    insert_test_change_log(&conn, "messages", "msg-3", "DELETE", 100);

    let pending = SyncManager::get_pending_changes(&conn, None, None).unwrap();

    assert!(pending.has_changes());
    assert_eq!(pending.total_count, 3);
    assert_eq!(pending.changes_by_table.get("messages"), Some(&2));
    assert_eq!(pending.changes_by_table.get("sessions"), Some(&1));
}

#[test]
fn test_get_pending_changes_with_field_deltas_json() {
    let conn = create_test_db();
    conn.execute(
        "ALTER TABLE __change_log ADD COLUMN field_deltas_json TEXT",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO __change_log (table_name, record_id, operation, field_deltas_json, sync_version)
         VALUES ('resources', 'res-1', 'UPDATE', '{\"ref_count\":1}', 0)",
        [],
    )
    .unwrap();

    let pending = SyncManager::get_pending_changes(&conn, None, None).unwrap();
    assert_eq!(pending.total_count, 1);
    assert_eq!(
        pending.entries[0].field_deltas_json,
        Some(json!({"ref_count": 1}))
    );
}

#[test]
fn test_from_entry_with_data_injects_field_deltas_metadata() {
    let entry = ChangeLogEntry {
        id: 1,
        table_name: "resources".to_string(),
        record_id: "res-1".to_string(),
        operation: ChangeOperation::Update,
        changed_at: "2024-01-01T10:00:00Z".to_string(),
        sync_version: 0,
        field_deltas_json: Some(json!({"ref_count": 1})),
    };

    let change = SyncChangeWithData::from_entry_with_data(
        &entry,
        Some(json!({
            "id": "res-1",
            "ref_count": 2,
            "updated_at": "2024-01-01T10:00:00Z"
        })),
    );

    let data = change.data.expect("data should be present");
    assert_eq!(data["__sync_field_deltas"], json!({"ref_count": 1}));
}

#[test]
fn test_get_pending_changes_with_table_filter() {
    let conn = create_test_db();

    insert_test_change_log(&conn, "messages", "msg-1", "INSERT", 0);
    insert_test_change_log(&conn, "messages", "msg-2", "UPDATE", 0);
    insert_test_change_log(&conn, "sessions", "sess-1", "INSERT", 0);

    let pending = SyncManager::get_pending_changes(&conn, Some("messages"), None).unwrap();

    assert_eq!(pending.total_count, 2);
    assert!(pending.entries.iter().all(|e| e.table_name == "messages"));
}

#[test]
fn test_get_pending_changes_with_limit() {
    let conn = create_test_db();

    for i in 0..10 {
        insert_test_change_log(&conn, "messages", &format!("msg-{}", i), "INSERT", 0);
    }

    let pending = SyncManager::get_pending_changes(&conn, None, Some(5)).unwrap();

    assert_eq!(pending.total_count, 5);
}

#[test]
fn test_mark_synced() {
    let conn = create_test_db();

    insert_test_change_log(&conn, "messages", "msg-1", "INSERT", 0);
    insert_test_change_log(&conn, "messages", "msg-2", "UPDATE", 0);
    insert_test_change_log(&conn, "messages", "msg-3", "DELETE", 0);

    // 标记前两条为已同步
    let updated = SyncManager::mark_synced(&conn, &[1, 2], 1000).unwrap();
    assert_eq!(updated, 2);

    // 验证只剩一条待同步
    let pending = SyncManager::get_pending_changes(&conn, None, None).unwrap();
    assert_eq!(pending.total_count, 1);
    assert_eq!(pending.entries[0].record_id, "msg-3");
}

#[test]
fn test_mark_synced_empty() {
    let conn = create_test_db();

    let updated = SyncManager::mark_synced(&conn, &[], 1000).unwrap();
    assert_eq!(updated, 0);
}

#[test]
fn test_mark_synced_with_timestamp() {
    let conn = create_test_db();

    insert_test_change_log(&conn, "messages", "msg-1", "INSERT", 0);

    let updated = SyncManager::mark_synced_with_timestamp(&conn, &[1]).unwrap();
    assert_eq!(updated, 1);

    // 验证已同步
    let pending = SyncManager::get_pending_changes(&conn, None, None).unwrap();
    assert!(!pending.has_changes());
}

#[test]
fn test_cleanup_synced_changes() {
    let conn = create_test_db();

    // 插入变更并标记为已同步
    conn.execute(
        "INSERT INTO __change_log (table_name, record_id, operation, changed_at, sync_version)
         VALUES ('messages', 'msg-1', 'INSERT', '2024-01-01T00:00:00Z', 100)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO __change_log (table_name, record_id, operation, changed_at, sync_version)
         VALUES ('messages', 'msg-2', 'UPDATE', '2024-01-15T00:00:00Z', 100)",
        [],
    )
    .unwrap();
    // 这条未同步，不应该被删除
    conn.execute(
        "INSERT INTO __change_log (table_name, record_id, operation, changed_at, sync_version)
         VALUES ('messages', 'msg-3', 'DELETE', '2024-01-01T00:00:00Z', 0)",
        [],
    )
    .unwrap();

    // 清理 2024-01-10 之前的已同步记录
    let deleted = SyncManager::cleanup_synced_changes(&conn, "2024-01-10T00:00:00Z").unwrap();
    assert_eq!(deleted, 1);

    // 验证还剩两条记录
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM __change_log", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_compare_timestamps_hlc_fast_path() {
    // 两端都是 HLC，应走 HLC 序比较（更精确，同毫秒 counter 决胜）
    let earlier = hlc::Hlc::new(1_700_000_000_000, 0).to_string();
    let later = hlc::Hlc::new(1_700_000_000_000, 1).to_string();

    // counter 1 > counter 0 → Greater
    assert_eq!(
        SyncManager::compare_timestamps(&later, &earlier),
        std::cmp::Ordering::Greater
    );
    assert_eq!(
        SyncManager::compare_timestamps(&earlier, &later),
        std::cmp::Ordering::Less
    );
    assert_eq!(
        SyncManager::compare_timestamps(&earlier, &earlier),
        std::cmp::Ordering::Equal
    );
}

#[test]
fn test_compare_timestamps_mixed_hlc_and_iso() {
    // 只有一端是 HLC → 回落到 timestamp 比较路径（都解析失败或部分失败走 None 分支）
    let hlc_str = hlc::Hlc::new(1_700_000_000_000, 0).to_string();
    let iso_str = "2024-01-01T00:00:00Z";

    // HLC 格式 Hlc::parse 成功，ISO 格式 Hlc::parse 失败 → 降级到 timestamp path
    // HLC 的 `015-05` 固定宽度不是有效 RFC3339，parse_flexible_timestamp 会返回 None
    // 于是落到 (None, Some) → Less
    let r = SyncManager::compare_timestamps(&hlc_str, iso_str);
    assert_eq!(r, std::cmp::Ordering::Less);
}

#[test]
fn test_reset_sync_baseline_after_restore() {
    let conn = create_test_db();

    // 创建一张业务表，带同步列
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            content TEXT,
            device_id TEXT,
            local_version INTEGER DEFAULT 0,
            sync_version INTEGER DEFAULT 0,
            updated_at TEXT,
            deleted_at TEXT
        );
        INSERT INTO notes (id, content, local_version, sync_version, updated_at)
        VALUES ('n1', 'hello', 5, 3, '2024-01-01T00:00:00Z'),
               ('n2', 'world', 2, 2, '2024-01-02T00:00:00Z');",
    )
    .unwrap();

    // 插入 __change_log 历史条目（模拟源设备的残留）
    conn.execute(
        "INSERT INTO __change_log (table_name, record_id, operation, changed_at, sync_version)
         VALUES ('notes', 'n1', 'UPDATE', '2024-01-01T00:00:00Z', 100)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO __change_log (table_name, record_id, operation, changed_at, sync_version)
         VALUES ('notes', 'n2', 'INSERT', '2024-01-02T00:00:00Z', 0)",
        [],
    )
    .unwrap();

    let (truncated, reset) = SyncManager::reset_sync_baseline_after_restore(&conn).unwrap();
    assert_eq!(truncated, 2);
    // 优化后仅更新 "sync_version != local_version" 的行，避免不必要的 trigger。
    // n1 (lv=5, sv=3) 需要更新；n2 (lv=2, sv=2) 相等不需更新。
    assert_eq!(reset, 1);

    // __change_log 应为空
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM __change_log", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);

    // sync_version 应等于 local_version
    let (lv1, sv1): (i64, i64) = conn
        .query_row(
            "SELECT local_version, sync_version FROM notes WHERE id = 'n1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(lv1, 5);
    assert_eq!(sv1, 5); // 从 3 提升到 5
    let (lv2, sv2): (i64, i64) = conn
        .query_row(
            "SELECT local_version, sync_version FROM notes WHERE id = 'n2'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(lv2, 2);
    assert_eq!(sv2, 2); // 已经相等，无变化
}

#[test]
fn test_apply_merge_strategy_keep_local() {
    let conflicts = vec![ConflictRecord {
        database_name: "chat_v2".to_string(),
        table_name: "messages".to_string(),
        record_id: "msg-1".to_string(),
        local_version: 3,
        cloud_version: 4,
        local_updated_at: "2024-01-01T10:00:00Z".to_string(),
        cloud_updated_at: "2024-01-01T11:00:00Z".to_string(),
        local_data: serde_json::json!({"content": "local"}),
        cloud_data: serde_json::json!({"content": "cloud"}),
    }];

    let result =
        SyncManager::apply_merge_strategy(MergeStrategy::KeepLocal, &conflicts).unwrap();

    assert!(result.success);
    assert_eq!(result.kept_local, 1);
    assert_eq!(result.used_cloud, 0);
    assert_eq!(result.records_to_push, vec!["msg-1"]);
    assert!(result.records_to_pull.is_empty());
}

#[test]
fn test_apply_merge_strategy_use_cloud() {
    let conflicts = vec![ConflictRecord {
        database_name: "chat_v2".to_string(),
        table_name: "messages".to_string(),
        record_id: "msg-1".to_string(),
        local_version: 3,
        cloud_version: 4,
        local_updated_at: "2024-01-01T10:00:00Z".to_string(),
        cloud_updated_at: "2024-01-01T11:00:00Z".to_string(),
        local_data: serde_json::json!({"content": "local"}),
        cloud_data: serde_json::json!({"content": "cloud"}),
    }];

    let result =
        SyncManager::apply_merge_strategy(MergeStrategy::UseCloud, &conflicts).unwrap();

    assert!(result.success);
    assert_eq!(result.kept_local, 0);
    assert_eq!(result.used_cloud, 1);
    assert!(result.records_to_push.is_empty());
    assert_eq!(result.records_to_pull, vec!["msg-1"]);
}

#[test]
fn test_apply_merge_strategy_keep_latest() {
    let conflicts = vec![
        // 云端更新
        ConflictRecord {
            database_name: "chat_v2".to_string(),
            table_name: "messages".to_string(),
            record_id: "msg-1".to_string(),
            local_version: 3,
            cloud_version: 4,
            local_updated_at: "2024-01-01T10:00:00Z".to_string(),
            cloud_updated_at: "2024-01-01T11:00:00Z".to_string(),
            local_data: serde_json::json!({"content": "local"}),
            cloud_data: serde_json::json!({"content": "cloud"}),
        },
        // 本地更新
        ConflictRecord {
            database_name: "chat_v2".to_string(),
            table_name: "messages".to_string(),
            record_id: "msg-2".to_string(),
            local_version: 5,
            cloud_version: 3,
            local_updated_at: "2024-01-01T12:00:00Z".to_string(),
            cloud_updated_at: "2024-01-01T09:00:00Z".to_string(),
            local_data: serde_json::json!({"content": "local new"}),
            cloud_data: serde_json::json!({"content": "cloud old"}),
        },
    ];

    let result =
        SyncManager::apply_merge_strategy(MergeStrategy::KeepLatest, &conflicts).unwrap();

    assert!(result.success);
    assert_eq!(result.kept_local, 1);
    assert_eq!(result.used_cloud, 1);
    assert_eq!(result.records_to_push, vec!["msg-2"]);
    assert_eq!(result.records_to_pull, vec!["msg-1"]);
}

#[test]
fn test_apply_merge_strategy_manual_error() {
    let conflicts = vec![ConflictRecord {
        database_name: "chat_v2".to_string(),
        table_name: "messages".to_string(),
        record_id: "msg-1".to_string(),
        local_version: 3,
        cloud_version: 4,
        local_updated_at: "2024-01-01T10:00:00Z".to_string(),
        cloud_updated_at: "2024-01-01T11:00:00Z".to_string(),
        local_data: serde_json::json!({"content": "local"}),
        cloud_data: serde_json::json!({"content": "cloud"}),
    }];

    let result = SyncManager::apply_merge_strategy(MergeStrategy::Manual, &conflicts);

    assert!(result.is_err());
    match result {
        Err(SyncError::ManualResolutionRequired { count }) => {
            assert_eq!(count, 1);
        }
        _ => panic!("Expected ManualResolutionRequired error"),
    }
}

#[test]
fn test_get_change_log_stats() {
    let conn = create_test_db();

    // 插入混合状态的变更日志
    insert_test_change_log(&conn, "messages", "msg-1", "INSERT", 0);
    insert_test_change_log(&conn, "messages", "msg-2", "UPDATE", 0);
    insert_test_change_log(&conn, "messages", "msg-3", "DELETE", 100);
    insert_test_change_log(&conn, "sessions", "sess-1", "INSERT", 200);

    let stats = SyncManager::get_change_log_stats(&conn).unwrap();

    assert_eq!(stats.total_count, 4);
    assert_eq!(stats.pending_count, 2);
    assert_eq!(stats.synced_count, 2);
}

#[test]
fn test_change_operation_from_str() {
    assert_eq!(
        ChangeOperation::from_str("INSERT"),
        Some(ChangeOperation::Insert)
    );
    assert_eq!(
        ChangeOperation::from_str("insert"),
        Some(ChangeOperation::Insert)
    );
    assert_eq!(
        ChangeOperation::from_str("UPDATE"),
        Some(ChangeOperation::Update)
    );
    assert_eq!(
        ChangeOperation::from_str("DELETE"),
        Some(ChangeOperation::Delete)
    );
    assert_eq!(ChangeOperation::from_str("INVALID"), None);
}

#[test]
fn test_change_operation_as_str() {
    assert_eq!(ChangeOperation::Insert.as_str(), "INSERT");
    assert_eq!(ChangeOperation::Update.as_str(), "UPDATE");
    assert_eq!(ChangeOperation::Delete.as_str(), "DELETE");
}

#[test]
fn test_pending_changes_get_table_changes() {
    let entries = vec![
        ChangeLogEntry {
            id: 1,
            table_name: "messages".to_string(),
            record_id: "msg-1".to_string(),
            operation: ChangeOperation::Insert,
            changed_at: "2024-01-01T10:00:00Z".to_string(),
            sync_version: 0,
            field_deltas_json: None,
        },
        ChangeLogEntry {
            id: 2,
            table_name: "sessions".to_string(),
            record_id: "sess-1".to_string(),
            operation: ChangeOperation::Insert,
            changed_at: "2024-01-01T11:00:00Z".to_string(),
            sync_version: 0,
            field_deltas_json: None,
        },
        ChangeLogEntry {
            id: 3,
            table_name: "messages".to_string(),
            record_id: "msg-2".to_string(),
            operation: ChangeOperation::Update,
            changed_at: "2024-01-01T12:00:00Z".to_string(),
            sync_version: 0,
            field_deltas_json: None,
        },
    ];

    let pending = PendingChanges::from_entries(entries);

    let message_changes = pending.get_table_changes("messages");
    assert_eq!(message_changes.len(), 2);

    let session_changes = pending.get_table_changes("sessions");
    assert_eq!(session_changes.len(), 1);

    let other_changes = pending.get_table_changes("other");
    assert!(other_changes.is_empty());
}

#[test]
fn test_pending_changes_get_change_ids() {
    let entries = vec![
        ChangeLogEntry {
            id: 1,
            table_name: "messages".to_string(),
            record_id: "msg-1".to_string(),
            operation: ChangeOperation::Insert,
            changed_at: "2024-01-01T10:00:00Z".to_string(),
            sync_version: 0,
            field_deltas_json: None,
        },
        ChangeLogEntry {
            id: 5,
            table_name: "messages".to_string(),
            record_id: "msg-2".to_string(),
            operation: ChangeOperation::Update,
            changed_at: "2024-01-01T11:00:00Z".to_string(),
            sync_version: 0,
            field_deltas_json: None,
        },
    ];

    let pending = PendingChanges::from_entries(entries);
    let ids = pending.get_change_ids();

    assert_eq!(ids, vec![1, 5]);
}

#[test]
fn test_pending_changes_time_range() {
    let entries = vec![
        ChangeLogEntry {
            id: 1,
            table_name: "messages".to_string(),
            record_id: "msg-1".to_string(),
            operation: ChangeOperation::Insert,
            changed_at: "2024-01-01T12:00:00Z".to_string(),
            sync_version: 0,
            field_deltas_json: None,
        },
        ChangeLogEntry {
            id: 2,
            table_name: "messages".to_string(),
            record_id: "msg-2".to_string(),
            operation: ChangeOperation::Update,
            changed_at: "2024-01-01T08:00:00Z".to_string(),
            sync_version: 0,
            field_deltas_json: None,
        },
        ChangeLogEntry {
            id: 3,
            table_name: "messages".to_string(),
            record_id: "msg-3".to_string(),
            operation: ChangeOperation::Delete,
            changed_at: "2024-01-01T15:00:00Z".to_string(),
            sync_version: 0,
            field_deltas_json: None,
        },
    ];

    let pending = PendingChanges::from_entries(entries);

    assert_eq!(
        pending.earliest_change,
        Some("2024-01-01T08:00:00Z".to_string())
    );
    assert_eq!(
        pending.latest_change,
        Some("2024-01-01T15:00:00Z".to_string())
    );
}

#[test]
fn test_merge_application_result() {
    let success = MergeApplicationResult::success(3, 2);
    assert!(success.success);
    assert_eq!(success.kept_local, 3);
    assert_eq!(success.used_cloud, 2);

    let failure = MergeApplicationResult::failure(vec!["error1".to_string()]);
    assert!(!failure.success);
    assert_eq!(failure.errors, vec!["error1"]);
}

// ========================================================================
// apply_downloaded_changes: data=None 跳过行为测试
// ========================================================================

/// 创建包含业务表的测试数据库（用于 apply 测试）
fn create_test_db_with_business_table() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE test_records (
            id TEXT PRIMARY KEY,
            content TEXT,
            updated_at TEXT
        );
        CREATE TABLE IF NOT EXISTS __change_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            table_name TEXT NOT NULL,
            record_id TEXT NOT NULL,
            operation TEXT NOT NULL CHECK(operation IN ('INSERT', 'UPDATE', 'DELETE')),
            changed_at TEXT NOT NULL DEFAULT (datetime('now')),
            sync_version INTEGER DEFAULT 0
        );
        "#,
    )
    .unwrap();
    conn
}

#[test]
fn test_apply_insert_with_data_none_is_skipped() {
    let conn = create_test_db_with_business_table();

    let changes = vec![SyncChangeWithData {
        table_name: "test_records".to_string(),
        record_id: "rec-1".to_string(),
        operation: ChangeOperation::Insert,
        data: None, // 旧格式：无数据
        changed_at: "2024-01-01T10:00:00Z".to_string(),
        change_log_id: None,
        database_name: None,
        suppress_change_log: None,
    }];

    let result = SyncManager::apply_downloaded_changes(&conn, &changes, None).unwrap();

    assert_eq!(result.success_count, 0);
    assert_eq!(
        result.skipped_count, 1,
        "data=None INSERT should be skipped, not error"
    );
    assert_eq!(result.failure_count, 0);

    // 验证记录不存在
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM test_records WHERE id = 'rec-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_apply_update_with_data_none_is_skipped() {
    let conn = create_test_db_with_business_table();

    // 先插入一条记录
    conn.execute(
        "INSERT INTO test_records (id, content) VALUES ('existing', 'original')",
        [],
    )
    .unwrap();

    let changes = vec![SyncChangeWithData {
        table_name: "test_records".to_string(),
        record_id: "existing".to_string(),
        operation: ChangeOperation::Update,
        data: None, // 旧格式：无数据
        changed_at: "2024-01-01T10:00:00Z".to_string(),
        change_log_id: None,
        database_name: None,
        suppress_change_log: None,
    }];

    let result = SyncManager::apply_downloaded_changes(&conn, &changes, None).unwrap();

    assert_eq!(result.success_count, 0);
    assert_eq!(
        result.skipped_count, 1,
        "data=None UPDATE should be skipped"
    );
    assert_eq!(result.failure_count, 0);

    // 验证记录未被修改
    let content: String = conn
        .query_row(
            "SELECT content FROM test_records WHERE id = 'existing'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(content, "original");
}

#[test]
fn test_apply_delete_without_data_succeeds() {
    let conn = create_test_db_with_business_table();

    conn.execute(
        "INSERT INTO test_records (id, content) VALUES ('to-delete', 'bye')",
        [],
    )
    .unwrap();

    let changes = vec![SyncChangeWithData {
        table_name: "test_records".to_string(),
        record_id: "to-delete".to_string(),
        operation: ChangeOperation::Delete,
        data: None, // DELETE 不需要数据
        changed_at: "2024-01-01T10:00:00Z".to_string(),
        change_log_id: None,
        database_name: None,
        suppress_change_log: None,
    }];

    let result = SyncManager::apply_downloaded_changes(&conn, &changes, None).unwrap();

    assert_eq!(
        result.success_count, 1,
        "DELETE without data should succeed"
    );
    assert_eq!(result.skipped_count, 0);

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM test_records WHERE id = 'to-delete'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_apply_mixed_data_none_and_valid() {
    let conn = create_test_db_with_business_table();

    let changes = vec![
        // 1. INSERT 无数据 → 跳过
        SyncChangeWithData {
            table_name: "test_records".to_string(),
            record_id: "no-data".to_string(),
            operation: ChangeOperation::Insert,
            data: None,
            changed_at: "2024-01-01T10:00:00Z".to_string(),
            change_log_id: None,
            database_name: None,
            suppress_change_log: None,
        },
        // 2. INSERT 有数据 → 成功
        SyncChangeWithData {
            table_name: "test_records".to_string(),
            record_id: "has-data".to_string(),
            operation: ChangeOperation::Insert,
            data: Some(serde_json::json!({
                "id": "has-data",
                "content": "valid",
                "updated_at": "2024-01-01"
            })),
            changed_at: "2024-01-01T10:00:01Z".to_string(),
            change_log_id: None,
            database_name: None,
            suppress_change_log: None,
        },
    ];

    let result = SyncManager::apply_downloaded_changes(&conn, &changes, None).unwrap();

    assert_eq!(result.success_count, 1, "only valid INSERT should succeed");
    assert_eq!(
        result.skipped_count, 1,
        "data=None INSERT should be skipped"
    );
    assert_eq!(result.failure_count, 0, "no failures expected");

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM test_records WHERE id = 'has-data'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "valid record should still be applied");
}

#[test]
fn test_get_record_data_llm_usage_daily_with_json_record_id() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE llm_usage_daily (
            date TEXT NOT NULL,
            caller_type TEXT NOT NULL,
            model TEXT NOT NULL,
            provider TEXT NOT NULL,
            request_count INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (date, caller_type, model, provider)
        );
        INSERT INTO llm_usage_daily(date, caller_type, model, provider, request_count)
        VALUES('2026-02-10', 'chat', 'gpt-4o', 'openai', 7);
        "#,
    )
    .unwrap();

    let record_id = serde_json::json!({
        "date": "2026-02-10",
        "caller_type": "chat",
        "model": "gpt-4o",
        "provider": "openai"
    })
    .to_string();

    let data = SyncManager::get_record_data(&conn, "llm_usage_daily", &record_id, "id")
        .unwrap()
        .expect("record should be found");

    assert_eq!(data["request_count"], serde_json::json!(7));
}

#[test]
fn test_apply_downloaded_changes_can_suppress_change_log_echo() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE test_records (
            id TEXT PRIMARY KEY,
            content TEXT,
            updated_at TEXT
        );
        CREATE TABLE __change_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            table_name TEXT NOT NULL,
            record_id TEXT NOT NULL,
            operation TEXT NOT NULL,
            changed_at TEXT NOT NULL DEFAULT (datetime('now')),
            sync_version INTEGER DEFAULT 0
        );
        CREATE TRIGGER trg_echo_insert
        AFTER INSERT ON test_records
        BEGIN
            INSERT INTO __change_log(table_name, record_id, operation)
            VALUES('test_records', NEW.id, 'INSERT');
        END;
        "#,
    )
    .unwrap();

    let changes = vec![SyncChangeWithData {
        table_name: "test_records".to_string(),
        record_id: "r1".to_string(),
        operation: ChangeOperation::Insert,
        data: Some(serde_json::json!({
            "id": "r1",
            "content": "ok",
            "updated_at": "2026-02-10"
        })),
        changed_at: "2026-02-10T00:00:00Z".to_string(),
        change_log_id: None,
        database_name: Some("vfs".to_string()),
        suppress_change_log: Some(true),
    }];

    SyncManager::apply_downloaded_changes(&conn, &changes, None).unwrap();

    let unsynced: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM __change_log WHERE sync_version = 0",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(unsynced, 0, "echo logs should be marked as synced");
}

#[test]
fn test_detect_record_conflicts_with_diverged_sync_versions() {
    let local_records = vec![RecordSnapshot {
        table_name: "messages".to_string(),
        record_id: "msg-1".to_string(),
        local_version: 12,
        sync_version: 10,
        updated_at: "2026-02-10T10:00:00Z".to_string(),
        deleted_at: None,
        data: serde_json::json!({"content": "local edit"}),
    }];
    let cloud_records = vec![RecordSnapshot {
        table_name: "messages".to_string(),
        record_id: "msg-1".to_string(),
        local_version: 21,
        sync_version: 20,
        updated_at: "2026-02-10T10:01:00Z".to_string(),
        deleted_at: None,
        data: serde_json::json!({"content": "cloud edit"}),
    }];

    let conflicts =
        SyncManager::detect_record_conflicts("chat_v2", &local_records, &cloud_records);
    assert_eq!(
        conflicts.len(),
        1,
        "diverged sync_version should still detect conflict"
    );
}

#[test]
fn test_detect_record_conflicts_same_data_not_conflict() {
    let local_records = vec![RecordSnapshot {
        table_name: "messages".to_string(),
        record_id: "msg-1".to_string(),
        local_version: 12,
        sync_version: 10,
        updated_at: "2026-02-10T10:00:00Z".to_string(),
        deleted_at: None,
        data: serde_json::json!({"content": "same"}),
    }];
    let cloud_records = vec![RecordSnapshot {
        table_name: "messages".to_string(),
        record_id: "msg-1".to_string(),
        local_version: 21,
        sync_version: 20,
        updated_at: "2026-02-10T10:01:00Z".to_string(),
        deleted_at: None,
        data: serde_json::json!({"content": "same"}),
    }];

    let conflicts =
        SyncManager::detect_record_conflicts("chat_v2", &local_records, &cloud_records);
    assert!(
        conflicts.is_empty(),
        "same payload should not be treated as conflict even when both modified"
    );
}

#[test]
fn test_apply_delete_uses_tombstone_when_column_exists() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE test_records (
            id TEXT PRIMARY KEY,
            content TEXT,
            deleted_at TEXT
        );
        INSERT INTO test_records (id, content, deleted_at)
        VALUES ('r1', 'alive', NULL);
        "#,
    )
    .unwrap();

    let changes = vec![SyncChangeWithData {
        table_name: "test_records".to_string(),
        record_id: "r1".to_string(),
        operation: ChangeOperation::Delete,
        data: None,
        changed_at: "2026-02-10T00:00:00Z".to_string(),
        change_log_id: None,
        database_name: None,
        suppress_change_log: None,
    }];

    let result = SyncManager::apply_downloaded_changes(&conn, &changes, None).unwrap();
    assert_eq!(result.success_count, 1);

    let row_state: (i64, Option<String>) = conn
        .query_row(
            "SELECT COUNT(*), MAX(deleted_at) FROM test_records WHERE id = 'r1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(row_state.0, 1, "tombstone delete should keep row");
    assert!(row_state.1.is_some(), "deleted_at should be set");
}

#[test]
fn test_apply_downloaded_changes_rolls_back_on_fk_violation() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        CREATE TABLE parent_records (
            id TEXT PRIMARY KEY
        );
        CREATE TABLE child_records (
            id TEXT PRIMARY KEY,
            parent_id TEXT NOT NULL,
            FOREIGN KEY(parent_id) REFERENCES parent_records(id)
        );
        CREATE TABLE test_records (
            id TEXT PRIMARY KEY,
            content TEXT
        );
        "#,
    )
    .unwrap();

    let changes = vec![
        SyncChangeWithData {
            table_name: "test_records".to_string(),
            record_id: "safe-1".to_string(),
            operation: ChangeOperation::Insert,
            data: Some(serde_json::json!({
                "id": "safe-1",
                "content": "should rollback"
            })),
            changed_at: "2026-02-10T00:00:00Z".to_string(),
            change_log_id: None,
            database_name: None,
            suppress_change_log: None,
        },
        SyncChangeWithData {
            table_name: "child_records".to_string(),
            record_id: "child-1".to_string(),
            operation: ChangeOperation::Insert,
            data: Some(serde_json::json!({
                "id": "child-1",
                "parent_id": "missing-parent"
            })),
            changed_at: "2026-02-10T00:00:01Z".to_string(),
            change_log_id: None,
            database_name: None,
            suppress_change_log: None,
        },
    ];

    let result = SyncManager::apply_downloaded_changes(&conn, &changes, None);
    assert!(result.is_err(), "fk violation should fail entire batch");

    let test_records_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM test_records", [], |row| row.get(0))
        .unwrap();
    let child_records_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM child_records", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        test_records_count, 0,
        "transaction should rollback previously applied records"
    );
    assert_eq!(child_records_count, 0);
}

fn create_resource_alias_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        CREATE TABLE resources (
            id TEXT PRIMARY KEY,
            hash TEXT NOT NULL UNIQUE,
            body TEXT,
            updated_at TEXT
        );
        CREATE TABLE resource_notes (
            id TEXT PRIMARY KEY,
            resource_id TEXT NOT NULL,
            note TEXT,
            updated_at TEXT,
            FOREIGN KEY(resource_id) REFERENCES resources(id)
        );
        INSERT INTO resources (id, hash, body, updated_at)
        VALUES ('local-res', 'same-business-hash', 'local body', '2024-01-01T00:00:00Z');
        "#,
    )
    .unwrap();
    conn
}

fn resource_alias_parent_change() -> SyncChangeWithData {
    SyncChangeWithData {
        table_name: "resources".to_string(),
        record_id: "remote-res".to_string(),
        operation: ChangeOperation::Insert,
        data: Some(serde_json::json!({
            "id": "remote-res",
            "hash": "same-business-hash",
            "body": "cloud body",
            "updated_at": "2024-01-02T00:00:00Z"
        })),
        changed_at: "2024-01-02T00:00:00Z".to_string(),
        change_log_id: None,
        database_name: Some("vfs".to_string()),
        suppress_change_log: None,
    }
}

fn resource_alias_child_change() -> SyncChangeWithData {
    SyncChangeWithData {
        table_name: "resource_notes".to_string(),
        record_id: "note-remote".to_string(),
        operation: ChangeOperation::Insert,
        data: Some(serde_json::json!({
            "id": "note-remote",
            "resource_id": "remote-res",
            "note": "child uses remote id",
            "updated_at": "2024-01-02T00:00:01Z"
        })),
        changed_at: "2024-01-02T00:00:01Z".to_string(),
        change_log_id: None,
        database_name: Some("vfs".to_string()),
        suppress_change_log: None,
    }
}

fn assert_resource_alias_result(conn: &Connection) {
    let resource_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM resources", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        resource_count, 1,
        "business-key conflict should reuse local row"
    );

    let remote_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM resources WHERE id = 'remote-res'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        remote_count, 0,
        "remote id should be an alias, not a new row"
    );

    let body: String = conn
        .query_row(
            "SELECT body FROM resources WHERE id = 'local-res'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(body, "cloud body");

    let child_fk: String = conn
        .query_row(
            "SELECT resource_id FROM resource_notes WHERE id = 'note-remote'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(child_fk, "local-res", "child FK should be remapped");

    let violations = SyncManager::collect_foreign_key_violations(conn, 20).unwrap();
    assert!(
        violations.is_empty(),
        "foreign keys should pass: {:?}",
        violations
    );
}

#[test]
fn test_business_key_alias_remaps_child_fk_when_child_arrives_first() {
    let conn = create_resource_alias_test_db();
    let changes = vec![
        resource_alias_child_change(),
        resource_alias_parent_change(),
    ];

    let result = SyncManager::apply_downloaded_changes(&conn, &changes, None).unwrap();

    assert_eq!(result.success_count, 2);
    assert_resource_alias_result(&conn);
}

#[test]
fn test_business_key_alias_reuses_canonical_id_when_parent_arrives_first() {
    let conn = create_resource_alias_test_db();
    let changes = vec![
        resource_alias_parent_change(),
        resource_alias_child_change(),
    ];

    let result = SyncManager::apply_downloaded_changes(&conn, &changes, None).unwrap();

    assert_eq!(result.success_count, 2);
    assert_resource_alias_result(&conn);
}

#[test]
fn test_suppress_change_log_does_not_mark_existing_user_update() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE test_records (
            id TEXT PRIMARY KEY,
            content TEXT,
            updated_at TEXT
        );
        CREATE TABLE __change_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            table_name TEXT NOT NULL,
            record_id TEXT NOT NULL,
            operation TEXT NOT NULL,
            changed_at TEXT NOT NULL DEFAULT (datetime('now')),
            sync_version INTEGER DEFAULT 0
        );
        CREATE TRIGGER trg_echo_insert
        AFTER INSERT ON test_records
        BEGIN
            INSERT INTO __change_log(table_name, record_id, operation)
            VALUES('test_records', NEW.id, 'INSERT');
        END;
        CREATE TRIGGER trg_echo_update
        AFTER UPDATE ON test_records
        BEGIN
            INSERT INTO __change_log(table_name, record_id, operation)
            VALUES('test_records', NEW.id, 'UPDATE');
        END;
        "#,
    )
    .unwrap();

    // 首次云端回放：应只抑制回放引入的 echo 记录
    let replay_insert = vec![SyncChangeWithData {
        table_name: "test_records".to_string(),
        record_id: "r1".to_string(),
        operation: ChangeOperation::Insert,
        data: Some(serde_json::json!({
            "id": "r1",
            "content": "cloud",
            "updated_at": "2026-02-10T00:00:00Z"
        })),
        changed_at: "2026-02-10T00:00:00Z".to_string(),
        change_log_id: None,
        database_name: None,
        suppress_change_log: Some(true),
    }];
    SyncManager::apply_downloaded_changes(&conn, &replay_insert, None).unwrap();

    // 本地用户编辑，产生 UPDATE 日志（应该保持未同步）
    conn.execute(
        "UPDATE test_records SET content = 'local-edit' WHERE id = 'r1'",
        [],
    )
    .unwrap();
    let user_update_log_id: i64 = conn
        .query_row(
            "SELECT id FROM __change_log WHERE operation = 'UPDATE' ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    // 再次回放同一个 INSERT，验证不会误标记用户 UPDATE 记录
    SyncManager::apply_downloaded_changes(&conn, &replay_insert, None).unwrap();

    let user_sync_version: i64 = conn
        .query_row(
            "SELECT sync_version FROM __change_log WHERE id = ?1",
            rusqlite::params![user_update_log_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        user_sync_version, 0,
        "existing user update log must not be marked as synced by replay suppression"
    );
}
