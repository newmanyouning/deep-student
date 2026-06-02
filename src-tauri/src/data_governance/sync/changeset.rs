use super::classification;
use super::conflict_resolver;
use super::field_merge;
use super::hlc;
use super::manifest::{
    parse_flexible_timestamp_public, ApplyChangesResult, ChangeLogEntry, ChangeLogStats,
    ChangeOperation, ConflictRecord, DatabaseSyncState,
    MergeApplicationResult, MergeStrategy, PendingChanges, SyncChangeWithData,
    SyncError, SYNC_FIELD_DELTAS_KEY,
};
use super::SyncManager;
use crate::cloud_storage::CloudStorage;
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::collections::{HashMap, HashSet};

/// 记录并跳过迭代中的错误，避免静默丢弃
fn log_and_skip_err<T, E: std::fmt::Display>(result: Result<T, E>) -> Option<T> {
    match result {
        Ok(v) => Some(v),
        Err(e) => {
            tracing::warn!("[Sync] Row parse error (skipped): {}", e);
            None
        }
    }
}

type IdAliasMap = HashMap<(String, String), String>;

#[derive(Debug, Clone)]
struct ForeignKeyColumn {
    child_column: String,
    parent_table: String,
    parent_column: String,
}

impl SyncManager {
    // ========================================================================
    // 核心同步方法
    // ========================================================================

    /// 获取待同步的变更
    ///
    /// 查询 __change_log 表中 sync_version = 0 的所有记录。
    ///
    /// # 参数
    /// * `conn` - 数据库连接
    /// * `table_filter` - 可选的表名过滤器，为 None 时查询所有表
    /// * `limit` - 可选的返回数量限制
    ///
    /// # 返回
    /// * `PendingChanges` - 待同步的变更集合
    pub fn get_pending_changes(
        conn: &Connection,
        table_filter: Option<&str>,
        limit: Option<usize>,
    ) -> Result<PendingChanges, SyncError> {
        let has_field_deltas = Self::table_has_column(conn, "__change_log", "field_deltas_json");
        let mut sql =
            String::from("SELECT id, table_name, record_id, operation, changed_at, sync_version");
        if has_field_deltas {
            sql.push_str(", field_deltas_json");
        }
        sql.push_str(
            "
             FROM __change_log
             WHERE sync_version = 0",
        );

        if table_filter.is_some() {
            sql.push_str(" AND table_name = ?1");
        }

        sql.push_str(" ORDER BY changed_at ASC");

        if let Some(limit_val) = limit {
            sql.push_str(&format!(" LIMIT {}", limit_val));
        }

        let entries: Vec<ChangeLogEntry> = if let Some(table_name) = table_filter {
            let mut stmt = conn
                .prepare(&sql)
                .map_err(|e| SyncError::Database(format!("准备查询语句失败: {}", e)))?;

            let rows = stmt
                .query_map(params![table_name], ChangeLogEntry::from_row)
                .map_err(|e| SyncError::Database(format!("执行查询失败: {}", e)))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| SyncError::Database(format!("解析结果失败: {}", e)))?
        } else {
            let mut stmt = conn
                .prepare(&sql)
                .map_err(|e| SyncError::Database(format!("准备查询语句失败: {}", e)))?;

            let rows = stmt
                .query_map([], ChangeLogEntry::from_row)
                .map_err(|e| SyncError::Database(format!("执行查询失败: {}", e)))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| SyncError::Database(format!("解析结果失败: {}", e)))?
        };

        Ok(PendingChanges::from_entries(entries))
    }

    /// 按同步分类过滤待上传变更。
    ///
    /// `get_pending_changes` 是底层 change-log 读取函数，仍然允许 synthetic schema
    /// 和旧测试库直接读取任意表；真实云同步上传路径必须调用本函数，只发布
    /// registry 中声明为 RowSync 的表，避免派生表、运行态表和备份表被误上传。
    pub fn filter_pending_changes_for_database(
        pending: PendingChanges,
        database_name: &str,
    ) -> PendingChanges {
        let row_sync_tables: HashSet<&'static str> =
            classification::TableClassification::row_sync_tables()
                .into_iter()
                .filter(|entry| entry.database == database_name)
                .map(|entry| entry.table_name)
                .collect();

        PendingChanges::from_entries(
            pending
                .entries
                .into_iter()
                .filter(|entry| row_sync_tables.contains(entry.table_name.as_str()))
                .collect(),
        )
    }

    /// 标记变更已同步
    ///
    /// 更新 __change_log 表中指定记录的 sync_version 字段。
    ///
    /// # 参数
    /// * `conn` - 数据库连接
    /// * `change_ids` - 要标记的变更日志 ID 列表
    /// * `sync_version` - 同步版本号（通常使用时间戳或递增版本）
    ///
    /// # 返回
    /// * 更新的记录数量
    pub fn mark_synced(
        conn: &Connection,
        change_ids: &[i64],
        sync_version: i64,
    ) -> Result<usize, SyncError> {
        if change_ids.is_empty() {
            return Ok(0);
        }

        // 构建 IN 子句的占位符
        let placeholders: Vec<String> = (1..=change_ids.len())
            .map(|i| format!("?{}", i + 1))
            .collect();
        let placeholders_str = placeholders.join(", ");

        let sql = format!(
            "UPDATE __change_log SET sync_version = ?1 WHERE id IN ({})",
            placeholders_str
        );

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| SyncError::Database(format!("准备更新语句失败: {}", e)))?;

        // 构建参数列表
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> =
            Vec::with_capacity(change_ids.len() + 1);
        params_vec.push(Box::new(sync_version));
        for id in change_ids {
            params_vec.push(Box::new(*id));
        }

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|b| b.as_ref()).collect();

        let updated = stmt
            .execute(params_refs.as_slice())
            .map_err(|e| SyncError::Database(format!("更新同步版本失败: {}", e)))?;

        Ok(updated)
    }

    /// 批量标记变更已同步（使用当前时间戳作为版本）
    ///
    /// # 参数
    /// * `conn` - 数据库连接
    /// * `change_ids` - 要标记的变更日志 ID 列表
    ///
    /// # 返回
    /// * 更新的记录数量
    pub fn mark_synced_with_timestamp(
        conn: &Connection,
        change_ids: &[i64],
    ) -> Result<usize, SyncError> {
        // 使用秒级时间戳，与上传文件 key 版本保持同一版本空间
        let sync_version = chrono::Utc::now().timestamp();

        // 兼容修复：将历史毫秒级 sync_version 归一化为秒级，避免 data_version 卡在毫秒量级
        Self::normalize_existing_millis_sync_versions(conn);

        Self::mark_synced(conn, change_ids, sync_version)
    }

    /// 一次性修复历史毫秒级 sync_version 值
    ///
    /// 如果 __change_log 中存在 sync_version > 1e11 的记录，
    /// 将它们除以 1000 归一化为秒级，防止 data_version (MAX) 卡在毫秒量级。
    fn normalize_existing_millis_sync_versions(conn: &Connection) {
        const MILLIS_THRESHOLD: i64 = 100_000_000_000; // 1e11
        match conn.execute(
            "UPDATE __change_log SET sync_version = sync_version / 1000 WHERE sync_version > ?1",
            rusqlite::params![MILLIS_THRESHOLD],
        ) {
            Ok(count) if count > 0 => {
                tracing::info!("[sync] 归一化了 {} 条历史毫秒级 sync_version 到秒级", count);
            }
            Ok(_) => {} // 没有需要修复的记录
            Err(e) => {
                tracing::warn!("[sync] 归一化 sync_version 失败（非致命）: {}", e);
            }
        }
    }

    /// 清理已同步的变更日志
    ///
    /// 删除 sync_version > 0 且早于指定时间的变更日志记录。
    /// 这可以在同步完成后调用，以防止变更日志表无限增长。
    ///
    /// # 参数
    /// * `conn` - 数据库连接
    /// * `older_than` - 删除早于此时间的记录（ISO 8601 格式）
    ///
    /// # 返回
    /// * 删除的记录数量
    pub fn cleanup_synced_changes(conn: &Connection, older_than: &str) -> Result<usize, SyncError> {
        let deleted = conn
            .execute(
                "DELETE FROM __change_log WHERE sync_version > 0 AND changed_at < ?1",
                params![older_than],
            )
            .map_err(|e| SyncError::Database(format!("清理变更日志失败: {}", e)))?;

        Ok(deleted)
    }

    /// 重建同步基线（用于 ZIP 备份恢复后）
    ///
    /// 从 ZIP 备份恢复数据后，`__change_log` 表的状态可能：
    /// - 完全缺失（老备份不包含变更日志）
    /// - 包含源设备的历史变更（sync_version 混合）
    ///
    /// 无论哪种情况，都需要把整个库视为"已同步"的快照，避免把恢复的数据
    /// 当作"新变更"再次推送到云端，产生时光倒流式的数据覆盖。
    ///
    /// 此函数执行以下操作：
    /// 1. 截断 `__change_log` 表（删除所有历史变更记录）
    /// 2. 更新所有业务表的 `sync_version = local_version`（所有现存记录标记为"已同步"）
    /// 3. 清除任何未解决的冲突记录（`__sync_conflicts` 表）
    ///
    /// 调用方需要**负责重新执行一次完整的 upload 同步**以发布设备清单，
    /// 否则云端仍会认为此设备的 data_version 是恢复前的状态。
    ///
    /// # 参数
    /// * `conn` - 已打开的数据库连接（应在事务内调用以确保原子性）
    ///
    /// # 返回
    /// * `(truncated_changes, reset_records)` - 清理的变更日志条数 + 重置 sync_version 的业务记录条数
    pub fn reset_sync_baseline_after_restore(
        conn: &Connection,
    ) -> Result<(usize, usize), SyncError> {
        // 注意步骤顺序：必须先 UPDATE 业务表（touch local_version），
        // 再 DELETE __change_log。因为业务表上通常装有 trg_upd 触发器，
        // UPDATE 会重新向 __change_log 写一批新条目——如果先清 __change_log 再 UPDATE，
        // 清理就白做了。

        // 1. 找出所有装配了同步字段的业务表，将 sync_version 对齐到 local_version。
        //    这样恢复后的数据会被视为"当前设备上的已同步快照"，不会把备份里原有记录
        //    误判为新的本地修改再次推送。
        let mut table_stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master
                 WHERE type='table'
                   AND name NOT LIKE 'sqlite_%'
                   AND name NOT LIKE '\\_\\_%' ESCAPE '\\'",
            )
            .map_err(|e| SyncError::Database(format!("查询业务表失败: {}", e)))?;

        let table_names: Vec<String> = table_stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| SyncError::Database(format!("扫描业务表失败: {}", e)))?
            .filter_map(|r| r.ok())
            .collect();
        drop(table_stmt);

        let mut reset_count = 0usize;
        for table in table_names {
            // 仅处理同时具备 local_version / sync_version 的业务表。
            let col_names: Vec<String> = match conn.prepare(&format!(
                "SELECT name FROM pragma_table_info('{}')",
                table.replace('\'', "''")
            )) {
                Ok(mut stmt) => stmt
                    .query_map([], |row| row.get::<_, String>(0))
                    .map(|iter| iter.filter_map(|r| r.ok()).collect())
                    .unwrap_or_default(),
                Err(_) => continue,
            };

            let has_local_version = col_names.iter().any(|c| c == "local_version");
            let has_sync_version = col_names.iter().any(|c| c == "sync_version");
            if !has_local_version || !has_sync_version {
                continue;
            }

            // 安全引用表名（仅允许标识符字符，双重保险）
            if !table.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                continue;
            }

            let sql = format!(
                "UPDATE \"{}\" \
                 SET sync_version = local_version \
                 WHERE local_version IS NOT NULL \
                   AND (sync_version IS NULL OR sync_version != local_version)",
                table
            );
            match conn.execute(&sql, []) {
                Ok(n) => reset_count += n,
                Err(e) => {
                    tracing::warn!(
                        "[sync] Touch local_version 失败（表 {}，非致命）: {}",
                        table,
                        e
                    );
                }
            }
        }

        // 2. 截断 __change_log（此步必须在 UPDATE 业务表之后，
        //    否则 trg_upd 触发器会把 UPDATE 重新记录进来）
        let truncated = conn
            .execute("DELETE FROM __change_log", [])
            .map_err(|e| SyncError::Database(format!("清理变更日志失败: {}", e)))?;

        // 3. 清除未解决的冲突记录（若表存在）
        let _ = conn.execute("DELETE FROM __sync_conflicts", []);
        let _ = conn.execute("DELETE FROM __sync_id_aliases", []);

        tracing::info!(
            "[sync] reset_sync_baseline_after_restore: cleaned __change_log {} rows, touched {} business records for re-upload",
            truncated,
            reset_count
        );

        Ok((truncated, reset_count))
    }

    /// 应用合并策略
    ///
    /// 根据指定的合并策略处理本地和云端的冲突记录，决定保留哪一方的数据。
    ///
    /// # 参数
    /// * `strategy` - 合并策略
    /// * `conflicts` - 冲突记录列表
    ///
    /// # 返回
    /// * `MergeApplicationResult` - 合并应用结果，包含需要推送/拉取的记录列表
    pub fn apply_merge_strategy(
        strategy: MergeStrategy,
        conflicts: &[ConflictRecord],
    ) -> Result<MergeApplicationResult, SyncError> {
        let mut kept_local = 0;
        let mut used_cloud = 0;
        let mut records_to_push = Vec::new();
        let mut records_to_pull = Vec::new();
        let mut errors = Vec::new();

        for conflict in conflicts {
            match strategy {
                MergeStrategy::KeepLocal => {
                    // 保留本地数据，需要将本地数据推送到云端
                    records_to_push.push(conflict.record_id.clone());
                    kept_local += 1;
                }
                MergeStrategy::UseCloud => {
                    // 使用云端数据，需要从云端拉取数据到本地
                    records_to_pull.push(conflict.record_id.clone());
                    used_cloud += 1;
                }
                MergeStrategy::KeepLatest => {
                    // 比较更新时间，保留最新的
                    match Self::compare_timestamps(
                        &conflict.local_updated_at,
                        &conflict.cloud_updated_at,
                    ) {
                        std::cmp::Ordering::Greater | std::cmp::Ordering::Equal => {
                            // 本地更新或相同，推送到云端
                            records_to_push.push(conflict.record_id.clone());
                            kept_local += 1;
                        }
                        std::cmp::Ordering::Less => {
                            // 云端更新，从云端拉取
                            records_to_pull.push(conflict.record_id.clone());
                            used_cloud += 1;
                        }
                    }
                }
                MergeStrategy::Manual => {
                    // 手动模式不自动处理，记录错误
                    errors.push(format!("记录 {} 需要手动处理", conflict.record_id));
                }
            }
        }

        if !errors.is_empty() && strategy == MergeStrategy::Manual {
            return Err(SyncError::ManualResolutionRequired {
                count: errors.len(),
            });
        }

        let mut result = MergeApplicationResult::success(kept_local, used_cloud);
        result.records_to_push = records_to_push;
        result.records_to_pull = records_to_pull;

        Ok(result)
    }

    /// 比较两个时间戳字符串
    ///
    /// 兼容两种常见格式：
    /// - RFC 3339: `"2026-02-27T12:34:56+00:00"` (Rust chrono 生成)
    /// - SQLite:   `"2026-02-27 12:34:56"`       (datetime('now') 生成)
    ///
    /// [P1 Fix] 引入 2 秒容差（CLOCK_SKEW_TOLERANCE_SECS），当两端时间差
    /// 小于该阈值时视为 Equal，避免设备间微小时钟偏差导致 KeepLatest 做出
    /// 错误决策。对于差距 > 容差的情况仍正常比较。
    const CLOCK_SKEW_TOLERANCE_SECS: i64 = 2;

    pub(crate) fn compare_timestamps(local: &str, cloud: &str) -> std::cmp::Ordering {
        // HLC fast-path：若两端都是 HLC 字符串（fixed-width millis-counter 格式），
        // 直接按 HLC 自然序比较——它已经内置了 "同毫秒内 counter tie-break"，
        // 比 ISO 秒级比较 + 容差粗糙得多更精确，尤其适合同一物理时钟里爆发式的两端写入。
        if let (Some(hl), Some(hc)) = (hlc::Hlc::parse(local), hlc::Hlc::parse(cloud)) {
            return hl.cmp(&hc);
        }

        let local_dt = Self::parse_flexible_timestamp(local);
        let cloud_dt = Self::parse_flexible_timestamp(cloud);

        match (local_dt, cloud_dt) {
            (Some(l), Some(c)) => {
                let diff_secs = (l - c).num_seconds().abs();
                if diff_secs <= Self::CLOCK_SKEW_TOLERANCE_SECS {
                    // 差距在容差范围内，视为相同时间（本地优先）
                    std::cmp::Ordering::Equal
                } else {
                    l.cmp(&c)
                }
            }
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (None, None) => local.cmp(cloud),
        }
    }

    /// 灵活解析时间戳，兼容 RFC 3339 和 SQLite datetime('now') 格式
    /// 应用合并策略到数据库（实际执行更新）
    ///
    /// 根据合并结果，执行实际的数据库更新操作。
    /// 采用"DELETE + INSERT"策略处理 UPDATE，确保数据完整性。
    ///
    /// # 参数
    /// * `conn` - 数据库连接
    /// * `table_name` - 表名
    /// * `records_to_pull` - 需要从云端拉取更新的记录 ID 列表
    /// * `cloud_data` - 云端数据映射（record_id -> JSON 数据）
    /// * `id_column` - 主键列名
    ///
    /// # 策略
    /// 1. 开启事务
    /// 2. 对于每条记录：DELETE 旧数据 + INSERT 新数据
    /// 3. 提交事务（失败则回滚）
    ///
    /// # 返回
    /// * 成功应用的记录数量
    pub fn apply_merge_to_database(
        conn: &Connection,
        table_name: &str,
        records_to_pull: &[String],
        cloud_data: &HashMap<String, serde_json::Value>,
        id_column: &str,
    ) -> Result<usize, SyncError> {
        if records_to_pull.is_empty() {
            return Ok(0);
        }

        let mut updated = 0;

        for record_id in records_to_pull {
            if let Some(data) = cloud_data.get(record_id) {
                match Self::apply_single_record(conn, table_name, record_id, data, None) {
                    Ok(()) => {
                        updated += 1;
                        tracing::debug!(
                            "[sync] 成功应用记录 {}.{} = {}",
                            table_name,
                            id_column,
                            record_id
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            "[sync] 应用记录失败 {}.{} = {}: {}",
                            table_name,
                            id_column,
                            record_id,
                            e
                        );
                        // 继续处理其他记录，记录失败
                    }
                }
            } else {
                tracing::warn!(
                    "[sync] 云端数据缺失 {}.{} = {}，跳过",
                    table_name,
                    id_column,
                    record_id
                );
            }
        }

        Ok(updated)
    }

    /// 获取指定表的 UPSERT 冲突目标子句（不含 DO UPDATE SET 部分）。
    ///
    /// 用于处理业务唯一键冲突。当一张表除了主键 `id` 之外还有额外的 UNIQUE 约束
    /// （如 `resources.hash`、`review_plans.question_id`、`files.sha256`、
    /// `folder_items(folder_id,item_type,item_id)`），需使用对应的冲突目标来正确合并数据，
    /// 而非在插入新 `id` 时遭遇 UNIQUE 约束违反。

    /// 生成业务键冲突时的回落 UPSERT SQL。
    ///
    /// 当主 UPSERT（基于 id 或业务唯一键）因 UNIQUE 约束违反失败时，
    /// 根据表类型构造替代冲突目标的 UPSERT，确保数据正确合并：
    /// - `review_plans`：从 question_id 回落至 id
    /// - `resources`：从 id 回落至 hash（合并相同内容记录）
    /// - `files`：从 id 回落至 sha256
    /// - `folder_items`：从 id 回落至 (folder_id, item_type, item_id)
    fn registry_business_unique_columns(
        database_name: Option<&str>,
        table_name: &str,
    ) -> Vec<String> {
        if let Some(database) = database_name {
            return classification::TableClassification::get_business_unique_keys(
                database, table_name,
            );
        }

        let mut distinct = classification::sync_classification_registry()
            .into_iter()
            .filter(|c| c.table_name == table_name)
            .filter(|c| !c.business_unique_keys.trim().is_empty())
            .map(|c| c.business_unique_keys.trim().to_string())
            .collect::<Vec<_>>();
        distinct.sort();
        distinct.dedup();

        if distinct.len() == 1 {
            distinct[0]
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    }

    fn unique_index_column_groups(
        conn: &Connection,
        table_name: &str,
    ) -> Result<Vec<Vec<String>>, SyncError> {
        Self::ensure_table_allowed_and_exists(conn, table_name)?;
        let table_ident = Self::quote_identifier(table_name)?;
        let sql = format!("PRAGMA index_list({})", table_ident);
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| SyncError::Database(format!("查询唯一索引失败: {}", e)))?;

        let index_names: Vec<String> = stmt
            .query_map([], |row| {
                let name: String = row.get(1)?;
                let unique: i64 = row.get(2)?;
                Ok((name, unique))
            })
            .map_err(|e| SyncError::Database(format!("读取唯一索引失败: {}", e)))?
            .filter_map(log_and_skip_err)
            .filter_map(|(name, unique)| if unique != 0 { Some(name) } else { None })
            .collect();

        let mut groups = Vec::new();
        for index_name in index_names {
            let index_ident = Self::quote_identifier(&index_name)?;
            let index_sql = format!("PRAGMA index_info({})", index_ident);
            let mut index_stmt = conn
                .prepare(&index_sql)
                .map_err(|e| SyncError::Database(format!("查询唯一索引列失败: {}", e)))?;
            let cols: Vec<String> = index_stmt
                .query_map([], |row| {
                    let cid: i64 = row.get(1)?;
                    let name: Option<String> = row.get(2)?;
                    Ok((cid, name))
                })
                .map_err(|e| SyncError::Database(format!("读取唯一索引列失败: {}", e)))?
                .filter_map(log_and_skip_err)
                .filter_map(|(cid, name)| if cid >= 0 { name } else { None })
                .collect();
            if !cols.is_empty() && !groups.iter().any(|g| g == &cols) {
                groups.push(cols);
            }
        }

        Ok(groups)
    }

    fn business_unique_key_groups(
        conn: &Connection,
        database_name: Option<&str>,
        table_name: &str,
    ) -> Result<Vec<Vec<String>>, SyncError> {
        let registered = Self::registry_business_unique_columns(database_name, table_name);
        if registered.is_empty() {
            return Ok(Vec::new());
        }

        let registered_set: HashSet<&str> = registered.iter().map(|s| s.as_str()).collect();

        let mut groups: Vec<Vec<String>> = Self::unique_index_column_groups(conn, table_name)?
            .into_iter()
            .filter(|cols| cols.iter().all(|c| registered_set.contains(c.as_str())))
            .collect();

        groups.sort();
        groups.dedup();
        Ok(groups)
    }

    fn get_fallback_upsert_sql(
        conn: &Connection,
        database_name: Option<&str>,
        table_name: &str,
        table_ident: &str,
        columns: &str,
        placeholders: &str,
        columns_list: &[&str],
        pk_columns: &[String],
    ) -> Result<String, SyncError> {
        let business_key_groups =
            Self::business_unique_key_groups(conn, database_name, table_name)?;
        if business_key_groups.is_empty() {
            return Err(SyncError::Database(format!(
                "表 {} 没有已注册且可验证的业务唯一键，id 冲突需要人工处理",
                table_name
            )));
        }

        let mut protected_cols: HashSet<String> =
            business_key_groups.into_iter().flatten().collect();
        for pk in pk_columns {
            protected_cols.insert(pk.to_string());
        }

        let update_set = columns_list
            .iter()
            .filter(|c| {
                let raw = c.trim_matches('"').replace("\"\"", "\"");
                !protected_cols.contains(&raw)
            })
            .map(|c| {
                let quoted = (*c).to_string();
                format!(
                    "{}=COALESCE(excluded.{}, {}.{})",
                    quoted, quoted, table_ident, quoted
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        let action = if update_set.is_empty() {
            "DO NOTHING".to_string()
        } else {
            format!("DO UPDATE SET {}", update_set)
        };

        Ok(format!(
            "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT {}",
            table_ident, columns, placeholders, action
        ))
    }

    /// 返回某表需要字段级合并的列清单（在 UPSERT 之前抓取原始本地值用）。
    /// 仅在本地行原先就存在时调用（INSERT 新行没有"原始本地值"这一说）。
    fn field_merge_column_picklist(table_name: &str) -> Vec<&'static str> {
        match table_name {
            "resources" | "chat_v2_resources" => vec!["ref_count", "metadata_json"],
            "questions" => vec![
                "attempt_count",
                "correct_count",
                "user_note",
                "ai_feedback",
                "is_favorite",
                "is_bookmarked",
                "tags",
                "images_json",
                "options_json",
                "ai_score",
            ],
            "notes" => vec!["tags", "is_favorite"],
            "files" => vec!["tags_json", "is_favorite", "bookmarks_json", "preview_json"],
            "review_plans" => vec![
                "total_reviews",
                "total_correct",
                "ease_factor",
                "interval_days",
                "consecutive_failures",
            ],
            "todo_items" => vec!["estimated_pomodoros", "completed_pomodoros", "tags_json"],
            "chat_v2_sessions" => vec!["metadata_json"],
            "chat_v2_messages" => vec![
                "block_ids_json",
                "meta_json",
                "variants_json",
                "shared_context_json",
                "attachments_json",
            ],
            "chat_v2_blocks" => vec!["citations_json", "tool_input_json", "tool_output_json"],
            "chat_v2_session_groups" => vec!["default_skill_ids_json", "pinned_resource_ids_json"],
            "essays" => vec!["grading_result_json", "dimension_scores_json"],
            "mindmaps" => vec!["settings"],
            "exam_sheets" => vec!["metadata_json", "preview_json"],
            "translations" => vec!["metadata_json"],
            "mistakes" => vec![
                "tags",
                "question_images",
                "analysis_images",
                "chat_metadata",
            ],
            "chat_messages" => vec![
                "rag_sources",
                "memory_sources",
                "graph_sources",
                "web_search_sources",
                "image_paths",
                "image_base64",
                "doc_attachments",
                "tool_call",
                "tool_result",
                "overrides",
                "relations",
                "metadata",
            ],
            "review_analyses" => vec!["tags", "mistake_ids", "temp_session_data"],
            "review_chat_messages" => vec![
                "rag_sources",
                "memory_sources",
                "graph_sources",
                "web_search_sources",
                "image_paths",
                "image_base64",
                "doc_attachments",
                "tool_call",
                "tool_result",
                "overrides",
                "relations",
                "metadata",
            ],
            "anki_cards" => vec!["tags_json", "images_json", "extra_fields_json"],
            _ => vec![],
        }
    }

    /// 应用单条下载变更到本地数据库。
    fn apply_single_record(
        conn: &Connection,
        table_name: &str,
        record_id: &str,
        data: &serde_json::Value,
        database_name: Option<&str>,
    ) -> Result<(), SyncError> {
        Self::ensure_table_allowed_and_exists(conn, table_name)?;

        let table_ident = Self::quote_identifier(table_name)?;

        let mut obj = data
            .as_object()
            .ok_or_else(|| {
                SyncError::Database(format!("记录数据不是有效的 JSON 对象: {}", record_id))
            })?
            .clone();

        let field_deltas = match obj.remove(SYNC_FIELD_DELTAS_KEY) {
            Some(serde_json::Value::Object(map)) => Some(map),
            Some(serde_json::Value::Null) | None => None,
            Some(other) => {
                return Err(SyncError::Database(format!(
                    "字段增量元数据格式错误: {} = {}",
                    SYNC_FIELD_DELTAS_KEY, other
                )))
            }
        };

        if obj.is_empty() {
            return Err(SyncError::Database(format!("记录数据为空: {}", record_id)));
        }

        // 只把 deleted_at 的显式 null 作为"复活意图"处理，其他 null 字段走 COALESCE
        let revive_record = matches!(obj.get("deleted_at"), Some(serde_json::Value::Null))
            && Self::table_has_column(conn, table_name, "deleted_at");

        let pk_columns = Self::primary_key_columns(conn, table_name)?;
        let pk_values = Self::parse_record_key_values(table_name, record_id, &pk_columns)?;
        let pk_predicate = Self::build_primary_key_predicate(&pk_columns)?;

        // [安全校验] payload 里的主键必须与 record_id 一致，避免恶意或损坏的 change
        // 用不匹配的 payload 覆盖另一条记录。
        let payload_key_values: Option<Vec<String>> = if pk_columns.len() == 1 {
            obj.get(&pk_columns[0]).map(|value| {
                vec![Self::json_value_to_alias_key(value).unwrap_or_else(|| value.to_string())]
            })
        } else if pk_columns.iter().all(|col| obj.contains_key(col)) {
            let mut values = Vec::with_capacity(pk_columns.len());
            for col in &pk_columns {
                let value = obj.get(col).expect("checked contains_key above");
                values.push(
                    Self::json_value_to_alias_key(value).unwrap_or_else(|| value.to_string()),
                );
            }
            Some(values)
        } else {
            None
        };
        if let Some(payload_values) = payload_key_values {
            if payload_values != pk_values {
                return Err(SyncError::Database(format!(
                    "payload 主键不一致: record_id='{}', payload_pk='{}'。这可能是云端数据损坏或重放攻击，已拒绝。",
                    record_id,
                    payload_values.join(":")
                )));
            }
        }

        // build_insert_parts 已经跳过所有 null，因此 deleted_at=null 不会参与 INSERT/COALESCE
        let (columns, placeholders, values) = Self::build_insert_parts(&obj)?;
        let columns_list: Vec<&str> = columns.split(", ").collect();

        // 字段级合并准备：在 UPSERT 改写本地值之前，先读取字段级合并列的"原始本地值"。
        // 否则 UPSERT 的 COALESCE 语义会把远端值直接写进本地，local_val 读取到的
        // 就是"刚被改写后的值"（即 == remote_val），merge_field 永远检测不到冲突。
        let local_before: HashMap<String, serde_json::Value> = {
            let picklist = Self::field_merge_column_picklist(table_name);
            let picklist: Vec<&'static str> = picklist
                .into_iter()
                .filter(|col| Self::table_has_column(conn, table_name, col))
                .collect();
            let mut m = HashMap::new();
            if picklist.is_empty() {
                m
            } else {
                let cols_sql: Vec<String> = picklist
                    .iter()
                    .map(|c| Self::quote_identifier(c))
                    .collect::<Result<Vec<_>, _>>()?;
                let read_sql = format!(
                    "SELECT {} FROM {} WHERE {}",
                    cols_sql.join(","),
                    table_ident,
                    pk_predicate
                );
                if let Ok(mut stmt) = conn.prepare(&read_sql) {
                    let params_refs: Vec<&dyn rusqlite::ToSql> = pk_values
                        .iter()
                        .map(|v| v as &dyn rusqlite::ToSql)
                        .collect();
                    if let Ok(Some(row)) = stmt
                        .query_row(params_refs.as_slice(), |row| -> rusqlite::Result<_> {
                            let mut map = HashMap::new();
                            for (i, col) in picklist.iter().enumerate() {
                                map.insert(col.to_string(), Self::sqlite_value_to_json(row, i));
                            }
                            Ok(map)
                        })
                        .optional()
                    {
                        m = row;
                    }
                }
                m
            }
        };

        let upsert_sql = if table_name == "review_plans" {
            let pk_ident = Self::quote_identifier("id")?;
            let update_set = columns_list
                .iter()
                .filter(|c| **c != pk_ident.as_str())
                .map(|c| format!("{}=COALESCE(excluded.{}, {}.{})", c, c, table_ident, c))
                .collect::<Vec<_>>()
                .join(", ");
            let action = if update_set.is_empty() {
                "DO NOTHING".to_string()
            } else {
                format!("DO UPDATE SET {}", update_set)
            };
            format!(
                "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT(question_id) WHERE question_id IS NOT NULL {}",
                table_ident, columns, placeholders, action
            )
        } else if table_name == "resources" {
            let pk_ident = Self::quote_identifier("id")?;
            let update_set = columns_list
                .iter()
                .filter(|c| **c != pk_ident.as_str())
                .map(|c| format!("{}=COALESCE(excluded.{}, {}.{})", c, c, table_ident, c))
                .collect::<Vec<_>>()
                .join(", ");
            let action = if update_set.is_empty() {
                "DO NOTHING".to_string()
            } else {
                format!("DO UPDATE SET {}", update_set)
            };
            format!(
                "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT(id) {}",
                table_ident, columns, placeholders, action
            )
        } else if table_name == "files" {
            let pk_ident = Self::quote_identifier("id")?;
            let update_set = columns_list
                .iter()
                .filter(|c| **c != pk_ident.as_str())
                .map(|c| format!("{}=COALESCE(excluded.{}, {}.{})", c, c, table_ident, c))
                .collect::<Vec<_>>()
                .join(", ");
            let action = if update_set.is_empty() {
                "DO NOTHING".to_string()
            } else {
                format!("DO UPDATE SET {}", update_set)
            };
            format!(
                "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT(sha256) {}",
                table_ident, columns, placeholders, action
            )
        } else if table_name == "folder_items" {
            let pk_ident = Self::quote_identifier("id")?;
            let update_set = columns_list
                .iter()
                .filter(|c| **c != pk_ident.as_str())
                .map(|c| format!("{}=COALESCE(excluded.{}, {}.{})", c, c, table_ident, c))
                .collect::<Vec<_>>()
                .join(", ");
            let action = if update_set.is_empty() {
                "DO NOTHING".to_string()
            } else {
                format!("DO UPDATE SET {}", update_set)
            };
            format!(
                "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT(folder_id, item_type, item_id) WHERE deleted_at IS NULL {}",
                table_ident, columns, placeholders, action
            )
        } else {
            let pk_ident_list = pk_columns
                .iter()
                .map(|c| Self::quote_identifier(c))
                .collect::<Result<Vec<_>, _>>()?;
            let update_set = columns_list
                .iter()
                .filter(|c| {
                    let raw = c.trim_matches('"').replace("\"\"", "\"");
                    !pk_columns.iter().any(|pk| pk == &raw)
                })
                .map(|c| format!("{}=COALESCE(excluded.{}, {}.{})", c, c, table_ident, c))
                .collect::<Vec<_>>()
                .join(", ");

            let action = if update_set.is_empty() {
                "DO NOTHING".to_string()
            } else {
                format!("DO UPDATE SET {}", update_set)
            };

            format!(
                "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT({}) {}",
                table_ident,
                columns,
                placeholders,
                pk_ident_list.join(", "),
                action
            )
        };

        let params_refs: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v.as_ref()).collect();
        match conn.execute(&upsert_sql, params_refs.as_slice()) {
            Ok(_) => {}
            Err(e) => {
                let err_msg = e.to_string();
                if !err_msg.contains("UNIQUE constraint failed") {
                    return Err(SyncError::Database(format!(
                        "UPSERT (OnConflict) 记录失败: {}",
                        e
                    )));
                }

                let fallback_sql = Self::get_fallback_upsert_sql(
                    conn,
                    database_name,
                    table_name,
                    &table_ident,
                    &columns,
                    &placeholders,
                    &columns_list,
                    &pk_columns,
                )?;

                conn.execute("SAVEPOINT sp_upsert_fallback", [])
                    .map_err(|e| SyncError::Database(format!("创建 SAVEPOINT 失败: {}", e)))?;
                match conn.execute(&fallback_sql, params_refs.as_slice()) {
                    Ok(_) => {
                        conn.execute("RELEASE SAVEPOINT sp_upsert_fallback", [])
                            .map_err(|e| {
                                SyncError::Database(format!("释放 SAVEPOINT 失败: {}", e))
                            })?;
                    }
                    Err(e2) => {
                        let _ = conn.execute("ROLLBACK TO SAVEPOINT sp_upsert_fallback", []);
                        let _ = conn.execute("RELEASE SAVEPOINT sp_upsert_fallback", []);
                        return Err(SyncError::Database(format!(
                            "UPSERT (业务键回落) 记录失败: {}",
                            e2
                        )));
                    }
                }
            }
        }

        // 复活意图：清空 deleted_at
        //
        // **优化**：只在本地 deleted_at 实际非 NULL 时才运行 UPDATE。
        // 否则 trg_upd 触发器会产生无谓的 __change_log 条目（虽被回声抑制但仍污染日志表）。
        if revive_record {
            let null_sql = format!(
                "UPDATE {} SET \"deleted_at\" = NULL WHERE {} AND \"deleted_at\" IS NOT NULL",
                table_ident, pk_predicate
            );
            let pk_params: Vec<&dyn rusqlite::ToSql> = pk_values
                .iter()
                .map(|v| v as &dyn rusqlite::ToSql)
                .collect();
            conn.execute(&null_sql, pk_params.as_slice())
                .map_err(|e| SyncError::Database(format!("复活软删记录失败: {}", e)))?;
        }

        // 字段级合并策略（在 UPSERT 之后、使用 UPSERT 前保存的原始本地值）
        // COALESCE UPSERT 已经把远端有值的列写入了本地。此步骤用 UPSERT 之前抓取的
        // 本地值 (local_before) 与远端值做 domain-aware 合并，弥补 COALESCE 无法表达
        // 的计数器、标签合集、布尔 OR、JSON deep merge 等语义。
        if !local_before.is_empty() {
            for (col_name, original_local) in &local_before {
                let remote_val = match obj.get(col_name.as_str()) {
                    Some(v) if !v.is_null() => v,
                    _ => continue,
                };
                let (merged_val, was_merged, _conflict) =
                    if let Some(deltas) = field_deltas.as_ref() {
                        if field_merge::supports_counter_delta(table_name, col_name) {
                            if let Some(delta_value) = deltas.get(col_name) {
                                let local_count = original_local.as_i64().ok_or_else(|| {
                                    SyncError::Database(format!(
                                        "counter 字段不是整数: {}.{} = {}",
                                        table_name, col_name, original_local
                                    ))
                                })?;
                                let delta = delta_value.as_i64().ok_or_else(|| {
                                    SyncError::Database(format!(
                                        "counter delta 不是整数: {}.{} = {}",
                                        table_name, col_name, delta_value
                                    ))
                                })?;
                                let merged = local_count.saturating_add(delta).max(0);
                                (serde_json::Value::Number(merged.into()), delta != 0, false)
                            } else {
                                field_merge::merge_field(
                                    table_name,
                                    col_name,
                                    Some(original_local),
                                    Some(remote_val),
                                )
                            }
                        } else {
                            field_merge::merge_field(
                                table_name,
                                col_name,
                                Some(original_local),
                                Some(remote_val),
                            )
                        }
                    } else {
                        field_merge::merge_field(
                            table_name,
                            col_name,
                            Some(original_local),
                            Some(remote_val),
                        )
                    };

                if was_merged {
                    let merge_pk_predicate =
                        Self::build_primary_key_predicate_from(&pk_columns, 2)?;
                    let col_ident = Self::quote_identifier(col_name)?;
                    let merge_sql = format!(
                        "UPDATE {} SET {} = ?1 WHERE {}",
                        table_ident, col_ident, merge_pk_predicate
                    );
                    let Some(sql_value) = Self::json_value_to_sql_param(&merged_val) else {
                        continue;
                    };
                    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![sql_value];
                    for value in &pk_values {
                        params_vec.push(Box::new(value.clone()));
                    }
                    let params_refs: Vec<&dyn rusqlite::ToSql> =
                        params_vec.iter().map(|v| v.as_ref()).collect();
                    let affected =
                        conn.execute(&merge_sql, params_refs.as_slice())
                            .map_err(|e| {
                                SyncError::Database(format!(
                                    "字段级合并写入失败: {}.{} {}",
                                    table_name, col_name, e
                                ))
                            })?;
                    if affected == 0 {
                        return Err(SyncError::Database(format!(
                            "字段级合并未命中目标记录: {}.{} record_id={}",
                            table_name, col_name, record_id
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// 从 JSON 对象构建 INSERT 语句的各部分
    ///
    /// # 返回
    /// * `(列名列表, 占位符列表, 参数值列表)`
    ///
    /// ## NULL 处理（P0 修复）
    /// 对于值为 JSON `null` 的字段，**直接跳过不写入**。
    /// 原因：
    /// 1. 避免 INSERT 路径触发 NOT NULL 约束违规（即使 UPSERT 会走 UPDATE 分支，
    ///    SQLite 仍会先校验 VALUES 列的约束）
    /// 2. 语义上等价于"保留本地既有值"（符合项目里原本的 COALESCE 意图）
    /// 3. 对于真正需要"清空字段"的场景，应显式传递空字符串/空数组，而不是 null
    fn build_insert_parts(
        obj: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<(String, String, Vec<Box<dyn rusqlite::ToSql>>), SyncError> {
        let mut columns = Vec::new();
        let mut placeholders = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        let mut idx = 0usize;
        for (key, value) in obj.iter() {
            if matches!(value, serde_json::Value::Null) {
                continue;
            }
            idx += 1;
            columns.push(Self::quote_identifier(key)?);
            placeholders.push(format!("?{}", idx));

            // 根据 JSON 值类型转换为 SQLite 参数
            let sql_value: Box<dyn rusqlite::ToSql> = match value {
                serde_json::Value::Null => unreachable!("已在上面跳过"),
                serde_json::Value::Bool(b) => Box::new(*b),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Box::new(i)
                    } else if let Some(f) = n.as_f64() {
                        Box::new(f)
                    } else {
                        Box::new(n.to_string())
                    }
                }
                serde_json::Value::String(s) => Box::new(s.clone()),
                // 数组和对象序列化为 JSON 字符串存储
                serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                    Box::new(serde_json::to_string(value).unwrap_or_default())
                }
            };
            values.push(sql_value);
        }

        if columns.is_empty() {
            // 调用方（apply_single_record）应保证至少有一个非 null 字段。
            // 此分支保留作为防御：全 null 输入时返回错误。
            return Err(SyncError::Database(
                "UPSERT: 全部字段为 NULL，调用方应先用独立 UPDATE 处理".to_string(),
            ));
        }

        Ok((columns.join(", "), placeholders.join(", "), values))
    }

    /// 将不可信的表名/列名安全地用于 SQL（标识符引用）
    ///
    /// - 使用双引号引用标识符，并对内部 `"` 做转义（`""`）
    /// - 拒绝空标识符与包含 `\0` 的输入
    fn quote_identifier(identifier: &str) -> Result<String, SyncError> {
        let ident = identifier.trim();
        if ident.is_empty() {
            return Err(SyncError::Database("SQL 标识符不能为空".to_string()));
        }
        if ident.contains('\0') {
            return Err(SyncError::Database("SQL 标识符包含非法字符".to_string()));
        }
        Ok(format!("\"{}\"", ident.replace('"', "\"\"")))
    }

    /// 防御性约束：仅允许对"业务表"应用下载变更
    ///
    /// - 拒绝 `sqlite_*` 系统表
    /// - 拒绝 `__*` 内部元数据表（如 __change_log）
    /// - 要求表在本地数据库中存在
    fn ensure_table_allowed_and_exists(
        conn: &Connection,
        table_name: &str,
    ) -> Result<(), SyncError> {
        let t = table_name.trim();
        if t.starts_with("sqlite_") {
            return Err(SyncError::Database(format!(
                "禁止同步到系统表: {}",
                table_name
            )));
        }
        if t.starts_with("__") {
            return Err(SyncError::Database(format!(
                "禁止同步到内部元数据表: {}",
                table_name
            )));
        }

        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
                params![t],
                |row| row.get(0),
            )
            .map_err(|e| SyncError::Database(format!("检查表是否存在失败: {}", e)))?;

        if !exists {
            return Err(SyncError::Database(format!("目标表不存在: {}", table_name)));
        }

        Ok(())
    }

    pub(crate) fn collect_foreign_key_violations(
        conn: &Connection,
        limit: usize,
    ) -> Result<Vec<String>, SyncError> {
        let mut stmt = conn
            .prepare("PRAGMA foreign_key_check")
            .map_err(|e| SyncError::Database(format!("准备 foreign_key_check 失败: {}", e)))?;

        let rows = stmt
            .query_map([], |row| {
                let table: String = row.get(0)?;
                let rowid: rusqlite::types::Value = row.get(1)?;
                let parent: String = row.get(2)?;
                let fkid: rusqlite::types::Value = row.get(3)?;
                Ok(format!(
                    "table={}, rowid={:?}, parent={}, fkid={:?}",
                    table, rowid, parent, fkid
                ))
            })
            .map_err(|e| SyncError::Database(format!("执行 foreign_key_check 失败: {}", e)))?;

        let mut violations = Vec::new();
        for (idx, r) in rows.enumerate() {
            if idx >= limit {
                break;
            }
            violations
                .push(r.map_err(|e| SyncError::Database(format!("读取外键检查结果失败: {}", e)))?);
        }
        Ok(violations)
    }

    fn foreign_key_columns(
        conn: &Connection,
        table_name: &str,
    ) -> Result<Vec<ForeignKeyColumn>, SyncError> {
        Self::ensure_table_allowed_and_exists(conn, table_name)?;
        let table_ident = Self::quote_identifier(table_name)?;
        let sql = format!("PRAGMA foreign_key_list({})", table_ident);
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| SyncError::Database(format!("查询外键失败: {}", e)))?;

        let columns = stmt
            .query_map([], |row| {
                Ok(ForeignKeyColumn {
                    parent_table: row.get(2)?,
                    child_column: row.get(3)?,
                    parent_column: row.get(4)?,
                })
            })
            .map_err(|e| SyncError::Database(format!("读取外键失败: {}", e)))?
            .filter_map(log_and_skip_err)
            .collect();

        Ok(columns)
    }

    fn primary_key_columns(conn: &Connection, table_name: &str) -> Result<Vec<String>, SyncError> {
        Self::ensure_table_allowed_and_exists(conn, table_name)?;
        let table_ident = Self::quote_identifier(table_name)?;
        let sql = format!("PRAGMA table_info({})", table_ident);
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| SyncError::Database(format!("查询主键列失败: {}", e)))?;

        let mut columns = stmt
            .query_map([], |row| {
                let name: String = row.get(1)?;
                let pk_order: i64 = row.get(5)?;
                Ok((pk_order, name))
            })
            .map_err(|e| SyncError::Database(format!("读取主键列失败: {}", e)))?
            .filter_map(log_and_skip_err)
            .filter(|(pk_order, _)| *pk_order > 0)
            .collect::<Vec<_>>();

        columns.sort_by_key(|(pk_order, _)| *pk_order);
        Ok(columns.into_iter().map(|(_, name)| name).collect())
    }

    fn json_value_to_alias_key(value: &serde_json::Value) -> Option<String> {
        match value {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            _ => None,
        }
    }

    fn json_value_to_sql_param(value: &serde_json::Value) -> Option<Box<dyn rusqlite::ToSql>> {
        match value {
            serde_json::Value::Null => None,
            serde_json::Value::Bool(b) => Some(Box::new(*b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Some(Box::new(i))
                } else if let Some(f) = n.as_f64() {
                    Some(Box::new(f))
                } else {
                    Some(Box::new(n.to_string()))
                }
            }
            serde_json::Value::String(s) => Some(Box::new(s.clone())),
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                Some(Box::new(serde_json::to_string(value).unwrap_or_default()))
            }
        }
    }

    fn resolve_alias(
        aliases: &IdAliasMap,
        table_name: &str,
        record_id: &str,
    ) -> Result<String, SyncError> {
        let mut current = record_id.to_string();
        let mut seen = HashSet::new();
        loop {
            if !seen.insert(current.clone()) {
                return Err(SyncError::Database(format!(
                    "ID 别名存在循环: {}.{}",
                    table_name, record_id
                )));
            }
            match aliases.get(&(table_name.to_string(), current.clone())) {
                Some(next) => current = next.clone(),
                None => return Ok(current),
            }
        }
    }

    fn insert_alias(
        aliases: &mut IdAliasMap,
        table_name: &str,
        remote_id: &str,
        canonical_id: &str,
    ) -> Result<bool, SyncError> {
        if remote_id == canonical_id {
            return Ok(false);
        }

        let key = (table_name.to_string(), remote_id.to_string());
        if let Some(existing) = aliases.get(&key) {
            if existing == canonical_id {
                return Ok(false);
            }
            return Err(SyncError::Database(format!(
                "ID 别名冲突: {}.{} -> {} / {}",
                table_name, remote_id, existing, canonical_id
            )));
        }

        aliases.insert(key, canonical_id.to_string());
        Ok(true)
    }

    fn ensure_id_alias_table(conn: &Connection) -> Result<(), SyncError> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS __sync_id_aliases (
                table_name TEXT NOT NULL,
                remote_id TEXT NOT NULL,
                canonical_id TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                PRIMARY KEY (table_name, remote_id)
            );
            CREATE INDEX IF NOT EXISTS idx__sync_id_aliases_canonical
                ON __sync_id_aliases(table_name, canonical_id);
            "#,
        )
        .map_err(|e| SyncError::Database(format!("创建 __sync_id_aliases 失败: {}", e)))
    }

    fn load_id_aliases(conn: &Connection) -> Result<IdAliasMap, SyncError> {
        Self::ensure_id_alias_table(conn)?;
        let mut stmt = conn
            .prepare("SELECT table_name, remote_id, canonical_id FROM __sync_id_aliases")
            .map_err(|e| SyncError::Database(format!("读取 __sync_id_aliases 失败: {}", e)))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| SyncError::Database(format!("扫描 __sync_id_aliases 失败: {}", e)))?;

        let mut aliases = IdAliasMap::new();
        for row in rows {
            let (table_name, remote_id, canonical_id) =
                row.map_err(|e| SyncError::Database(format!("读取 ID 别名失败: {}", e)))?;
            Self::insert_alias(&mut aliases, &table_name, &remote_id, &canonical_id)?;
        }
        Ok(aliases)
    }

    fn persist_id_aliases(conn: &Connection, aliases: &IdAliasMap) -> Result<(), SyncError> {
        Self::ensure_id_alias_table(conn)?;
        for ((table_name, remote_id), canonical_id) in aliases {
            conn.execute(
                "INSERT INTO __sync_id_aliases (table_name, remote_id, canonical_id)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(table_name, remote_id) DO UPDATE SET
                    canonical_id = excluded.canonical_id",
                params![table_name, remote_id, canonical_id],
            )
            .map_err(|e| SyncError::Database(format!("写入 ID 别名失败: {}", e)))?;
        }
        Ok(())
    }

    fn remap_foreign_keys_in_object(
        conn: &Connection,
        table_name: &str,
        obj: &mut serde_json::Map<String, serde_json::Value>,
        aliases: &IdAliasMap,
    ) -> Result<(), SyncError> {
        for fk in Self::foreign_key_columns(conn, table_name)? {
            let parent_pk = Self::primary_key_columns(conn, &fk.parent_table)?;
            if parent_pk.len() != 1 || parent_pk[0] != fk.parent_column {
                continue;
            }

            let current = match obj.get(&fk.child_column) {
                Some(value) => match Self::json_value_to_alias_key(value) {
                    Some(v) => v,
                    None => continue,
                },
                None => continue,
            };
            let canonical = Self::resolve_alias(aliases, &fk.parent_table, &current)?;
            if canonical != current {
                obj.insert(fk.child_column, serde_json::Value::String(canonical));
            }
        }
        Ok(())
    }

    fn find_canonical_id_by_business_key(
        conn: &Connection,
        database_name: Option<&str>,
        table_name: &str,
        id_column: &str,
        obj: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<Option<String>, SyncError> {
        let key_groups = Self::business_unique_key_groups(conn, database_name, table_name)?;
        if key_groups.is_empty() {
            return Ok(None);
        }

        let table_ident = Self::quote_identifier(table_name)?;
        let id_col_ident = Self::quote_identifier(id_column)?;

        for group in key_groups {
            let mut where_parts = Vec::new();
            let mut values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
            for col in &group {
                let value = match obj.get(col) {
                    Some(value) if !value.is_null() => value,
                    _ => {
                        values.clear();
                        break;
                    }
                };
                let Some(sql_value) = Self::json_value_to_sql_param(value) else {
                    values.clear();
                    break;
                };
                where_parts.push(format!("{} = ?", Self::quote_identifier(col)?));
                values.push(sql_value);
            }
            if values.len() != group.len() {
                continue;
            }

            let sql = format!(
                "SELECT {} FROM {} WHERE {} LIMIT 1",
                id_col_ident,
                table_ident,
                where_parts.join(" AND ")
            );
            let params_refs: Vec<&dyn rusqlite::ToSql> =
                values.iter().map(|v| v.as_ref()).collect();
            let canonical = conn
                .query_row(&sql, params_refs.as_slice(), |row| {
                    Ok(Self::json_value_to_alias_key(&Self::sqlite_value_to_json(
                        row, 0,
                    )))
                })
                .optional()
                .map_err(|e| SyncError::Database(format!("查询业务键 canonical id 失败: {}", e)))?
                .flatten();

            if canonical.is_some() {
                return Ok(canonical);
            }
        }

        Ok(None)
    }

    fn build_download_id_aliases(
        conn: &Connection,
        changes: &[SyncChangeWithData],
        id_column_map: Option<&HashMap<String, String>>,
    ) -> Result<IdAliasMap, SyncError> {
        let mut aliases = Self::load_id_aliases(conn)?;

        loop {
            let before = aliases.len();
            for change in changes {
                if !matches!(
                    change.operation,
                    ChangeOperation::Insert | ChangeOperation::Update
                ) {
                    continue;
                }
                let Some(data) = &change.data else {
                    continue;
                };
                let Some(source_obj) = data.as_object() else {
                    continue;
                };

                Self::ensure_table_allowed_and_exists(conn, &change.table_name)?;
                let id_column = id_column_map
                    .and_then(|m| m.get(&change.table_name))
                    .map(|s| s.as_str())
                    .unwrap_or("id");

                let mut obj = source_obj.clone();
                Self::remap_foreign_keys_in_object(conn, &change.table_name, &mut obj, &aliases)?;

                let remote_id =
                    Self::resolve_alias(&aliases, &change.table_name, &change.record_id)?;
                let Some(canonical_id) = Self::find_canonical_id_by_business_key(
                    conn,
                    change.database_name.as_deref(),
                    &change.table_name,
                    id_column,
                    &obj,
                )?
                else {
                    continue;
                };

                Self::insert_alias(&mut aliases, &change.table_name, &remote_id, &canonical_id)?;
                Self::insert_alias(
                    &mut aliases,
                    &change.table_name,
                    &change.record_id,
                    &canonical_id,
                )?;
            }

            if aliases.len() == before {
                break;
            }
        }

        Ok(aliases)
    }

    fn remap_change_with_aliases(
        conn: &Connection,
        change: &SyncChangeWithData,
        id_column: &str,
        aliases: &IdAliasMap,
    ) -> Result<SyncChangeWithData, SyncError> {
        let mut remapped = change.clone();
        let canonical_record_id =
            Self::resolve_alias(aliases, &change.table_name, &change.record_id)?;

        if canonical_record_id != change.record_id {
            remapped.record_id = canonical_record_id.clone();
        }

        if let Some(serde_json::Value::Object(obj)) = remapped.data.as_mut() {
            if canonical_record_id != change.record_id && obj.contains_key(id_column) {
                obj.insert(
                    id_column.to_string(),
                    serde_json::Value::String(canonical_record_id),
                );
            }
            Self::remap_foreign_keys_in_object(conn, &change.table_name, obj, aliases)?;
        }

        Ok(remapped)
    }

    /// 应用下载的变更到数据库
    ///
    /// 批量应用从云端下载的变更，支持事务处理。
    ///
    /// # 参数
    /// * `conn` - 数据库连接
    /// * `changes` - 带完整数据的变更列表
    /// * `id_column_map` - 表名到主键列名的映射（默认使用 "id"）
    ///
    /// # 返回
    /// * `ApplyChangesResult` - 应用结果
    pub fn apply_downloaded_changes(
        conn: &Connection,
        changes: &[SyncChangeWithData],
        id_column_map: Option<&HashMap<String, String>>,
    ) -> Result<ApplyChangesResult, SyncError> {
        if changes.is_empty() {
            return Ok(ApplyChangesResult::empty());
        }

        let mut result = ApplyChangesResult::empty();

        // 原子性保证：任何错误都应回滚，避免"半套数据"落地。
        //
        // 同时为了避免跨表写入顺序导致的外键约束问题，这里在事务内临时关闭外键检查，
        // 写入完成后使用 `PRAGMA foreign_key_check` 做一次强校验，失败则回滚。
        let original_fk: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap_or(1);

        // 注意：SQLite 在事务内修改 foreign_keys 是无操作（no-op），
        // 必须在 BEGIN 之前修改，或者使用 defer_foreign_keys = ON。
        conn.execute_batch("PRAGMA defer_foreign_keys = ON;")
            .map_err(|e| SyncError::Database(format!("开启延迟外键检查失败: {}", e)))?;

        conn.execute_batch("BEGIN IMMEDIATE;")
            .map_err(|e| SyncError::Database(format!("开始事务失败: {}", e)))?;

        let apply_result: Result<(), SyncError> = (|| {
            let id_aliases = Self::build_download_id_aliases(conn, changes, id_column_map)?;

            for change in changes {
                let id_column = id_column_map
                    .and_then(|m| m.get(&change.table_name))
                    .map(|s| s.as_str())
                    .unwrap_or("id");
                let change_to_apply =
                    Self::remap_change_with_aliases(conn, change, id_column, &id_aliases)?;

                let suppress = change.suppress_change_log.unwrap_or(false);

                let pre_log_max_id = if suppress {
                    conn.query_row("SELECT COALESCE(MAX(id), 0) FROM __change_log", [], |row| {
                        row.get::<_, i64>(0)
                    })
                    .ok()
                } else {
                    None
                };

                let applied = Self::apply_single_change(conn, &change_to_apply, id_column)?;
                if applied {
                    result.success_count += 1;
                    result
                        .applied_keys
                        .insert((change.table_name.clone(), change.record_id.clone()));
                } else {
                    result.skipped_count += 1;
                }

                // 精确抑制：标记由本次回放产生的、匹配当前 table+record 的所有
                // change_log 条目为已同步。
                //
                // 这里**不限制 operation**，因为 apply_single_record 内部
                // 对 "payload 含 deleted_at: null" 的情况会先 UPSERT 再做一次独立 UPDATE，
                // 后者产生的 `operation = UPDATE` 条目也必须被视为回放产物。
                //
                // 并发安全性：apply_downloaded_changes 在一个事务内执行，用户手动写入
                // 使用独立连接无法并发产生同事务内的 __change_log 条目，所以不会误标记。
                if let Some(max_id) = pre_log_max_id {
                    let sync_version = chrono::Utc::now().timestamp();
                    let _ = conn.execute(
                        "UPDATE __change_log SET sync_version = ?1 \
                         WHERE id > ?2 AND sync_version = 0 \
                         AND table_name = ?3 AND record_id = ?4",
                        params![
                            sync_version,
                            max_id,
                            &change_to_apply.table_name,
                            &change_to_apply.record_id,
                        ],
                    );
                }
            }

            Self::persist_id_aliases(conn, &id_aliases)?;

            // 强校验：必须没有任何外键违规
            let violations = Self::collect_foreign_key_violations(conn, 20)?;
            if !violations.is_empty() {
                return Err(SyncError::Database(format!(
                    "外键约束检查失败（示例最多 20 条）: {}",
                    violations.join("; ")
                )));
            }

            Ok(())
        })();

        match apply_result {
            Ok(()) => {
                if let Err(e) = conn.execute_batch("COMMIT;") {
                    let _ = conn.execute_batch("ROLLBACK;");
                    let _ = if original_fk == 0 {
                        conn.execute_batch("PRAGMA foreign_keys = OFF;")
                    } else {
                        conn.execute_batch("PRAGMA foreign_keys = ON;")
                    };
                    return Err(SyncError::Database(format!("提交事务失败: {}", e)));
                }
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK;");
                // 恢复外键开关（best-effort）
                let _ = if original_fk == 0 {
                    conn.execute_batch("PRAGMA foreign_keys = OFF;")
                } else {
                    conn.execute_batch("PRAGMA foreign_keys = ON;")
                };
                return Err(e);
            }
        }

        // 恢复外键开关（best-effort）
        let _ = if original_fk == 0 {
            conn.execute_batch("PRAGMA foreign_keys = OFF;")
        } else {
            conn.execute_batch("PRAGMA foreign_keys = ON;")
        };

        tracing::info!(
            "[sync] 变更应用完成: success={}, failed={}, skipped={}",
            result.success_count,
            result.failure_count,
            result.skipped_count
        );

        Ok(result)
    }

    /// 以冲突感知方式应用变更（修复 #3 #4 #20）
    ///
    /// 与 `apply_downloaded_changes` 不同：
    /// 1. 对每条下载的变更，先用 `ConflictResolver::resolve_one` 判定是否冲突
    /// 2. 若冲突：
    ///    - 把败方数据写入 `__sync_conflicts` 表（永不丢失）
    ///    - 胜方是 Cloud → 正常应用云端变更到数据库
    ///    - 胜方是 Local → 跳过应用，但仍写胜方本地值到冲突表作为留痕
    /// 3. 无冲突：直接应用
    ///
    /// 使用一次事务保证要么全部成功要么回滚；若整体失败不写冲突表。
    pub fn apply_downloaded_changes_with_conflict_guard(
        conn: &Connection,
        changes: &[SyncChangeWithData],
        id_column_map: Option<&HashMap<String, String>>,
        policy: conflict_resolver::ConflictPolicy,
        cloud_device_id: Option<&str>,
        local_device_id: Option<&str>,
    ) -> Result<
        (
            ApplyChangesResult,
            conflict_resolver::ConflictAwareApplyResult,
        ),
        SyncError,
    > {
        use conflict_resolver::{ConflictResolver, ConflictSide};

        if changes.is_empty() {
            return Ok((
                ApplyChangesResult::empty(),
                conflict_resolver::ConflictAwareApplyResult::default(),
            ));
        }

        // 保证冲突表存在（幂等）
        ConflictResolver::ensure_conflict_table(conn)?;

        let original_fk: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap_or(1);
        conn.execute_batch("PRAGMA defer_foreign_keys = ON;")
            .map_err(|e| SyncError::Database(format!("开启延迟外键检查失败: {}", e)))?;
        conn.execute_batch("BEGIN IMMEDIATE;")
            .map_err(|e| SyncError::Database(format!("开始事务失败: {}", e)))?;

        let resolver = ConflictResolver::new(policy);
        let mut apply_result = ApplyChangesResult::empty();
        let mut conflict_result = conflict_resolver::ConflictAwareApplyResult::default();

        let inner: Result<(), SyncError> = (|| {
            for change in changes {
                let id_column = id_column_map
                    .and_then(|m| m.get(&change.table_name))
                    .map(|s| s.as_str())
                    .unwrap_or("id");

                match resolver.resolve_one(conn, change, id_column)? {
                    None => {
                        // 非冲突，正常 UPSERT
                        let suppress = change.suppress_change_log.unwrap_or(false);
                        let pre_log_max_id = if suppress {
                            conn.query_row(
                                "SELECT COALESCE(MAX(id), 0) FROM __change_log",
                                [],
                                |row| row.get::<_, i64>(0),
                            )
                            .ok()
                        } else {
                            None
                        };

                        let applied = Self::apply_single_change(conn, change, id_column)?;
                        if applied {
                            apply_result.success_count += 1;
                            apply_result
                                .applied_keys
                                .insert((change.table_name.clone(), change.record_id.clone()));
                        } else {
                            apply_result.skipped_count += 1;
                        }

                        if let Some(max_id) = pre_log_max_id {
                            let sync_version = chrono::Utc::now().timestamp();
                            let _ = conn.execute(
                                "UPDATE __change_log SET sync_version = ?1 \
                                 WHERE id > ?2 AND sync_version = 0 \
                                 AND table_name = ?3 AND record_id = ?4",
                                params![
                                    sync_version,
                                    max_id,
                                    &change.table_name,
                                    &change.record_id,
                                ],
                            );
                        }
                    }
                    Some(outcome) => {
                        // 落败方先进冲突表（两端都各存一份，便于 UI 三路展示）
                        ConflictResolver::save_conflict_record(
                            conn,
                            conflict_resolver::ConflictRecordToSave {
                                table_name: &change.table_name,
                                record_id: &change.record_id,
                                side: outcome.loser,
                                data: &outcome.loser_data,
                                winning_device_id: if outcome.winner == ConflictSide::Cloud {
                                    cloud_device_id
                                } else {
                                    local_device_id
                                },
                                losing_device_id: if outcome.loser == ConflictSide::Cloud {
                                    cloud_device_id
                                } else {
                                    local_device_id
                                },
                            },
                        )?;

                        // 同时把胜方的快照也记录一份（side=winner），方便 UI 同时看到两份
                        ConflictResolver::save_conflict_record(
                            conn,
                            conflict_resolver::ConflictRecordToSave {
                                table_name: &change.table_name,
                                record_id: &change.record_id,
                                side: outcome.winner,
                                data: &outcome.winner_data,
                                winning_device_id: if outcome.winner == ConflictSide::Cloud {
                                    cloud_device_id
                                } else {
                                    local_device_id
                                },
                                losing_device_id: if outcome.loser == ConflictSide::Cloud {
                                    cloud_device_id
                                } else {
                                    local_device_id
                                },
                            },
                        )?;

                        conflict_result.conflicts_saved += 2;
                        *conflict_result
                            .conflicts_by_table
                            .entry(change.table_name.clone())
                            .or_insert(0) += 1;

                        if outcome.winner == ConflictSide::Cloud {
                            // Cloud 胜，按云端数据写入本地（但要抑制回声）
                            let mut cloud_change = change.clone();
                            cloud_change.suppress_change_log = Some(true);

                            let pre_log_max_id = conn
                                .query_row(
                                    "SELECT COALESCE(MAX(id), 0) FROM __change_log",
                                    [],
                                    |row| row.get::<_, i64>(0),
                                )
                                .ok();

                            // 冲突已裁决为 Cloud 胜，绕过 LWW 门强制应用
                            let applied =
                                Self::apply_single_change_force(conn, &cloud_change, id_column)?;
                            if applied {
                                apply_result.success_count += 1;
                                apply_result
                                    .applied_keys
                                    .insert((change.table_name.clone(), change.record_id.clone()));
                                conflict_result.applied += 1;
                            } else {
                                apply_result.skipped_count += 1;
                            }

                            if let Some(max_id) = pre_log_max_id {
                                let sync_version = chrono::Utc::now().timestamp();
                                let _ = conn.execute(
                                    "UPDATE __change_log SET sync_version = ?1 \
                                     WHERE id > ?2 AND sync_version = 0 \
                                     AND table_name = ?3 AND record_id = ?4",
                                    params![
                                        sync_version,
                                        max_id,
                                        &change.table_name,
                                        &change.record_id,
                                    ],
                                );
                            }
                        } else {
                            // Local 胜，跳过应用云端变更；但记录为 rejected，上层会在下一轮把本地值上传
                            conflict_result.rejected += 1;
                            apply_result.skipped_count += 1;
                        }
                    }
                }
            }

            let violations = Self::collect_foreign_key_violations(conn, 20)?;
            if !violations.is_empty() {
                return Err(SyncError::Database(format!(
                    "外键约束检查失败（示例最多 20 条）: {}",
                    violations.join("; ")
                )));
            }

            Ok(())
        })();

        match inner {
            Ok(()) => {
                if let Err(e) = conn.execute_batch("COMMIT;") {
                    let _ = conn.execute_batch("ROLLBACK;");
                    let _ = if original_fk == 0 {
                        conn.execute_batch("PRAGMA foreign_keys = OFF;")
                    } else {
                        conn.execute_batch("PRAGMA foreign_keys = ON;")
                    };
                    return Err(SyncError::Database(format!("提交事务失败: {}", e)));
                }
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK;");
                let _ = if original_fk == 0 {
                    conn.execute_batch("PRAGMA foreign_keys = OFF;")
                } else {
                    conn.execute_batch("PRAGMA foreign_keys = ON;")
                };
                return Err(e);
            }
        }

        let _ = if original_fk == 0 {
            conn.execute_batch("PRAGMA foreign_keys = OFF;")
        } else {
            conn.execute_batch("PRAGMA foreign_keys = ON;")
        };

        tracing::info!(
            "[sync] 冲突感知应用完成: applied={}, rejected={}, conflicts_saved={}",
            conflict_result.applied,
            conflict_result.rejected,
            conflict_result.conflicts_saved
        );

        Ok((apply_result, conflict_result))
    }

    /// 检测"云端变更断层"：
    /// 返回 `true` 表示 `since_version` 所指向的变更文件在云端 `min_available_version` 之前
    /// 已被 prune 删除，**客户端无法只靠增量恢复到一致**。调用方应：
    /// - 引导用户走一次 full-snapshot 同步（重新拉取每张表的最新记录）
    /// - 或者退化到只同步"当前快照"而抛弃中间断层
    pub fn has_prune_gap(since_version: u64, min_available_version: Option<u64>) -> bool {
        match min_available_version {
            Some(min) => since_version > 0 && since_version < min,
            None => false,
        }
    }

    /// 获取云端当前可用的最小变更版本号（用于断层检测）
    pub async fn get_min_available_change_version(
        storage: &dyn CloudStorage,
    ) -> Result<Option<u64>, SyncError> {
        let files = storage
            .list(Self::CHANGES_PREFIX)
            .await
            .map_err(|e| SyncError::Network(format!("列出变更文件失败: {}", e)))?;

        let mut min_version: Option<u64> = None;
        for file in &files {
            if let Some(raw) = Self::parse_version_from_key(&file.key) {
                let v = Self::normalize_version_to_seconds(raw);
                min_version = Some(match min_version {
                    Some(cur) => cur.min(v),
                    None => v,
                });
            }
        }
        Ok(min_version)
    }

    /// 检查表是否拥有指定列
    fn table_has_column(conn: &Connection, table_name: &str, col_name: &str) -> bool {
        let table_ident = match Self::quote_identifier(table_name) {
            Ok(t) => t,
            Err(_) => return false,
        };
        let sql = format!("PRAGMA table_info({})", table_ident);
        let mut stmt = match conn.prepare(&sql) {
            Ok(s) => s,
            Err(_) => return false,
        };
        stmt.query_map([], |row| row.get::<_, String>(1))
            .map(|rows| rows.filter_map(|r| r.ok()).any(|name| name == col_name))
            .unwrap_or(false)
    }

    /// 获取列的声明类型（用于 tombstone 写入时选择 INTEGER vs TEXT）
    ///
    /// 返回 `PRAGMA table_info` 里的 type 列（原始声明，如 "TEXT" / "INTEGER" / ""）。
    /// SQLite 的 type affinity 规则：只要声明类型包含 "INT" 就是 INTEGER affinity。
    fn get_column_declared_type(
        conn: &Connection,
        table_name: &str,
        col_name: &str,
    ) -> Option<String> {
        let table_ident = Self::quote_identifier(table_name).ok()?;
        let sql = format!("PRAGMA table_info({})", table_ident);
        let mut stmt = conn.prepare(&sql).ok()?;
        let rows = stmt
            .query_map([], |row| {
                let name: String = row.get(1)?;
                let ty: String = row.get(2)?;
                Ok((name, ty))
            })
            .ok()?;
        for r in rows.flatten() {
            if r.0 == col_name {
                return Some(r.1);
            }
        }
        None
    }

    /// 应用单条变更
    ///
    /// # 返回
    /// * `Ok(true)` - 成功应用
    /// * `Ok(false)` - 跳过（保留兼容语义，当前分支通常不使用）
    /// * `Err` - 应用失败
    fn apply_single_change(
        conn: &Connection,
        change: &SyncChangeWithData,
        id_column: &str,
    ) -> Result<bool, SyncError> {
        Self::apply_single_change_inner(conn, change, id_column, false)
    }

    /// 同 `apply_single_change`，但跳过 LWW 时间戳门（用于 conflict_guard 已决策场景）
    fn apply_single_change_force(
        conn: &Connection,
        change: &SyncChangeWithData,
        id_column: &str,
    ) -> Result<bool, SyncError> {
        Self::apply_single_change_inner(conn, change, id_column, true)
    }

    fn apply_single_change_inner(
        conn: &Connection,
        change: &SyncChangeWithData,
        id_column: &str,
        skip_lww: bool,
    ) -> Result<bool, SyncError> {
        match change.operation {
            ChangeOperation::Delete => {
                Self::ensure_table_allowed_and_exists(conn, &change.table_name)?;
                let table_ident = Self::quote_identifier(&change.table_name)?;
                let has_tombstone = Self::table_has_column(conn, &change.table_name, "deleted_at");
                let pk_columns = Self::primary_key_columns(conn, &change.table_name)?;
                let pk_values = Self::parse_record_key_values(
                    &change.table_name,
                    &change.record_id,
                    &pk_columns,
                )?;
                let pk_predicate = Self::build_primary_key_predicate(&pk_columns)?;
                let pk_predicate_after_set_value =
                    Self::build_primary_key_predicate_from(&pk_columns, 2)?;

                // [LWW + HLC drift 保护 - DELETE]
                // 1. 如果云端 changed_at 超出 wall clock "未来 60 秒" → 视为可疑漂移，跳过
                // 2. 如果本地记录的 updated_at 严格晚于云端 DELETE 的 changed_at → 跳过（LWW）
                if !skip_lww && has_tombstone {
                    if let Some(cloud_ts) = parse_flexible_timestamp_public(&change.changed_at) {
                        // ─── HLC drift check ───
                        let now = chrono::Utc::now();
                        if (cloud_ts - now).num_milliseconds() > hlc::MAX_DRIFT_MS {
                            tracing::warn!(
                                "[sync] 跳过 DELETE（时间戳漂移过大）: {}.{} = {}, drift_ms={}",
                                change.table_name,
                                id_column,
                                change.record_id,
                                (cloud_ts - now).num_milliseconds()
                            );
                            return Ok(false);
                        }

                        // ─── LWW check ───
                        if Self::table_has_column(conn, &change.table_name, "updated_at") {
                            let local_ts_opt = Self::get_record_data(
                                conn,
                                &change.table_name,
                                &change.record_id,
                                id_column,
                            )
                            .ok()
                            .flatten()
                            .and_then(|row| {
                                row.get("updated_at").and_then(|v| {
                                    v.as_i64()
                                        .and_then(
                                            chrono::DateTime::<chrono::Utc>::from_timestamp_millis,
                                        )
                                        .or_else(|| {
                                            v.as_str().and_then(parse_flexible_timestamp_public)
                                        })
                                })
                            });

                            if let Some(local_ts) = local_ts_opt {
                                if local_ts > cloud_ts {
                                    tracing::debug!(
                                        "[sync] LWW skip DELETE: {}.{} = {} (本地 update 更新)",
                                        change.table_name,
                                        id_column,
                                        change.record_id
                                    );
                                    return Ok(false);
                                }
                            }
                        }
                    }
                }

                let affected = if has_tombstone {
                    // [修复] deleted_at 列可能是 TEXT（ISO 字符串）或 INTEGER（毫秒时间戳）。
                    // 检测列的声明类型后用匹配的值写入，避免把 '2026-05-01T...' 写到 INTEGER 列
                    // 导致后续 `row.get::<_, i64>(...)` panic。
                    //
                    // [幂等性修复] 使用 `change.changed_at`（来自云端变更日志）而不是 `now()`，
                    // 确保同一 DELETE 变更被多次回放时写入相同时间戳（否则 checksum 每次都变）。
                    let col_type =
                        Self::get_column_declared_type(conn, &change.table_name, "deleted_at")
                            .unwrap_or_else(|| "TEXT".to_string());
                    let sql = format!(
                        "UPDATE {} SET \"deleted_at\" = ?1 WHERE {} AND \"deleted_at\" IS NULL",
                        table_ident, pk_predicate_after_set_value
                    );
                    let upper = col_type.to_uppercase();
                    if upper.contains("INT") {
                        // 尝试把 changed_at 解析成毫秒时间戳；失败则回落到当前时间
                        let ts_ms = chrono::DateTime::parse_from_rfc3339(&change.changed_at)
                            .map(|dt| dt.timestamp_millis())
                            .unwrap_or_else(|_| chrono::Utc::now().timestamp_millis());
                        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(ts_ms)];
                        for value in &pk_values {
                            params_vec.push(Box::new(value.clone()));
                        }
                        let params_refs: Vec<&dyn rusqlite::ToSql> =
                            params_vec.iter().map(|v| v.as_ref()).collect();
                        conn.execute(&sql, params_refs.as_slice())
                            .map_err(|e| SyncError::Database(format!("软删除记录失败: {}", e)))?
                    } else {
                        // 规范化为 RFC3339 字符串（保留 changed_at 来源但统一格式）
                        let ts = chrono::DateTime::parse_from_rfc3339(&change.changed_at)
                            .map(|dt| dt.with_timezone(&chrono::Utc).to_rfc3339())
                            .unwrap_or_else(|_| change.changed_at.clone());
                        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(ts)];
                        for value in &pk_values {
                            params_vec.push(Box::new(value.clone()));
                        }
                        let params_refs: Vec<&dyn rusqlite::ToSql> =
                            params_vec.iter().map(|v| v.as_ref()).collect();
                        conn.execute(&sql, params_refs.as_slice())
                            .map_err(|e| SyncError::Database(format!("软删除记录失败: {}", e)))?
                    }
                } else {
                    let sql = format!("DELETE FROM {} WHERE {}", table_ident, pk_predicate);
                    let params_refs: Vec<&dyn rusqlite::ToSql> = pk_values
                        .iter()
                        .map(|v| v as &dyn rusqlite::ToSql)
                        .collect();
                    conn.execute(&sql, params_refs.as_slice())
                        .map_err(|e| SyncError::Database(format!("删除记录失败: {}", e)))?
                };

                tracing::debug!(
                    "[sync] DELETE(tombstone={}) {}.{} = {}, affected={}",
                    has_tombstone,
                    change.table_name,
                    id_column,
                    change.record_id,
                    affected
                );
                Ok(true)
            }
            ChangeOperation::Insert | ChangeOperation::Update => {
                // INSERT/UPDATE 操作：使用 UPSERT (ON CONFLICT DO UPDATE)
                let data = match &change.data {
                    Some(d) => d,
                    None => {
                        // 兼容旧版下载格式（v1）：仅含变更元数据，不含完整行数据。
                        // 对这类历史数据跳过而非失败，避免旧云端数据导致整次同步回滚。
                        if change.database_name.is_none() {
                            tracing::warn!(
                                "[sync] INSERT/UPDATE 缺少数据（旧格式兼容），跳过: {}.{} = {}",
                                change.table_name,
                                id_column,
                                change.record_id
                            );
                            return Ok(false);
                        }

                        return Err(SyncError::Database(format!(
                            "INSERT/UPDATE 缺少 data 字段: {}.{} = {}",
                            change.table_name, id_column, change.record_id
                        )));
                    }
                };

                // [LWW 保护] 比较云端 payload 的 updated_at 和本地记录的 updated_at。
                // 若本地更新，跳过应用 —— 避免旧云端变更覆盖较新的本地值（这是 chaos test 暴露的
                // 核心收敛性 bug：没有时间戳门的 UPSERT 会让 "较早的云端 change 在较晚本地写入之后
                // 抵达" 的场景产生分叉）。
                //
                // 跳过的判定需要**双方的 updated_at 都能解析**，否则保持原有行为（直接 UPSERT）。
                if !skip_lww
                    && Self::should_skip_stale_update(
                        conn,
                        &change.table_name,
                        &change.record_id,
                        id_column,
                        data,
                    )
                {
                    tracing::debug!(
                        "[sync] LWW skip: {}.{} = {} (本地更新)",
                        change.table_name,
                        id_column,
                        change.record_id
                    );
                    return Ok(false);
                }

                Self::apply_single_record(
                    conn,
                    &change.table_name,
                    &change.record_id,
                    data,
                    change.database_name.as_deref(),
                )?;
                Ok(true)
            }
        }
    }

    /// 判断是否应当跳过这条云端 UPSERT
    ///
    /// 两道防线：
    /// 1. **HLC 漂移保护（防恶意超前时间戳）**：如果云端 `updated_at` 比本地 wall clock
    ///    晚超过 `hlc::MAX_DRIFT_MS`（60 秒），视为可疑并跳过，避免一个时钟错乱的
    ///    设备永久压制其他设备。参考 CockroachDB / YugabyteDB 的 MAX_OFFSET 设计。
    /// 2. **LWW 比较（防过时变更覆盖较新本地值）**：如果本地 `updated_at` 严格晚于
    ///    云端 payload，跳过这条云端 change。这保证最终一致性收敛（chaos test 暴露的关键 bug）。
    fn should_skip_stale_update(
        conn: &Connection,
        table_name: &str,
        record_id: &str,
        id_column: &str,
        cloud_data: &serde_json::Value,
    ) -> bool {
        // 云端 payload 必须带 updated_at
        // 兼容 TEXT（ISO 8601 / HLC 串）和 INTEGER（毫秒时间戳）两种形式——
        // 项目里 resources / chat_v2_todo_lists 等表用的是 INTEGER ms。
        let cloud_str: String = match cloud_data.get("updated_at") {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(serde_json::Value::Number(n)) => {
                // 数值 updated_at：统一转成字符串供下游解析
                n.to_string()
            }
            _ => return false,
        };
        let cloud_str = cloud_str.as_str();

        // ─── HLC 快速路径 ───
        // 若云端 updated_at 是 HLC 字符串，直接按 HLC 比较；跳过 timestamp 漂移检查
        // （HLC 内部的 receive() 已经有 MAX_DRIFT_MS 守护，且发送端如果伪造
        // millis 会被收端直接拒绝）。
        let cloud_hlc = hlc::Hlc::parse(cloud_str);
        if let Some(cloud_hlc_val) = cloud_hlc {
            // HLC 漂移 check：云端 millis 比本地 wall clock 超前过多 → 跳过
            let now_ms = chrono::Utc::now().timestamp_millis() as u64;
            if cloud_hlc_val.millis as i64 - now_ms as i64 > hlc::MAX_DRIFT_MS {
                tracing::warn!(
                    "[sync] 跳过云端变更（HLC 漂移过大）: table={}, id={}, cloud_hlc={}, now_ms={}, drift_ms={}",
                    table_name,
                    record_id,
                    cloud_str,
                    now_ms,
                    cloud_hlc_val.millis as i64 - now_ms as i64
                );
                return true;
            }
            // LWW by HLC：查本地 updated_at 并尝试解析为 HLC
            if !Self::table_has_column(conn, table_name, "updated_at") {
                return false;
            }
            let table_ident = match Self::quote_identifier(table_name) {
                Ok(s) => s,
                Err(_) => return false,
            };
            let id_col = match Self::quote_identifier(id_column) {
                Ok(s) => s,
                Err(_) => return false,
            };
            let sql = format!(
                "SELECT \"updated_at\" FROM {} WHERE {} = ?1",
                table_ident, id_col
            );
            let local_hlc_opt: Option<hlc::Hlc> = conn
                .query_row(&sql, params![record_id], |row| {
                    if let Ok(s) = row.get::<_, String>(0) {
                        return Ok(hlc::Hlc::parse(&s));
                    }
                    Ok(None)
                })
                .ok()
                .flatten();
            if let Some(local_hlc_val) = local_hlc_opt {
                return local_hlc_val > cloud_hlc_val;
            }
            // 本地是非 HLC 格式，继续走常规时间戳路径（降级比较）
        }

        let cloud_ts = match parse_flexible_timestamp_public(cloud_str) {
            Some(t) => t,
            None => return false,
        };

        // ─── 防线 1：HLC 漂移 sanity check ───
        // 如果云端时间戳超出本地 wall clock "未来 60 秒"，视为恶意/故障，跳过。
        let now = chrono::Utc::now();
        let drift_from_now = cloud_ts - now;
        if drift_from_now.num_milliseconds() > hlc::MAX_DRIFT_MS {
            tracing::warn!(
                "[sync] 跳过云端变更（时间戳漂移过大）: table={}, id={}, cloud_ts={}, now={}, drift_ms={}",
                table_name,
                record_id,
                cloud_ts,
                now,
                drift_from_now.num_milliseconds()
            );
            return true;
        }

        // ─── 防线 2：LWW ───
        // 查本地当前 updated_at（要求该表也有 updated_at 列）
        if !Self::table_has_column(conn, table_name, "updated_at") {
            return false;
        }
        let local_ts_opt = Self::get_record_data(conn, table_name, record_id, id_column)
            .ok()
            .flatten()
            .and_then(|row| {
                row.get("updated_at").and_then(|updated_at| {
                    if let Some(s) = updated_at.as_str() {
                        parse_flexible_timestamp_public(s)
                    } else if let Some(ms) = updated_at.as_i64() {
                        chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ms)
                    } else {
                        None
                    }
                })
            });

        // 本地不存在 → 不跳过（需要 INSERT）
        let local_ts = match local_ts_opt {
            Some(t) => t,
            None => return false,
        };

        // 严格晚于才跳过；相等时允许 UPSERT（可能是幂等回放）
        local_ts > cloud_ts
    }

    /// 获取记录的完整数据
    ///
    /// 从指定表中获取记录的完整 JSON 数据。
    ///
    /// # 参数
    /// * `conn` - 数据库连接
    /// * `table_name` - 表名
    /// * `record_id` - 记录 ID
    /// * `id_column` - 主键列名
    ///
    /// # 返回
    /// * `Option<serde_json::Value>` - 记录数据（如果存在）
    pub fn get_record_data(
        conn: &Connection,
        table_name: &str,
        record_id: &str,
        id_column: &str,
    ) -> Result<Option<serde_json::Value>, SyncError> {
        let columns = Self::get_table_columns(conn, table_name)?;
        if columns.is_empty() {
            return Ok(None);
        }
        Self::get_record_data_with_columns(conn, table_name, record_id, id_column, &columns)
    }

    /// 内部辅助：使用预取的列信息查询单条记录，避免重复 PRAGMA 查询
    fn get_record_data_with_columns(
        conn: &Connection,
        table_name: &str,
        record_id: &str,
        _id_column: &str,
        columns: &[String],
    ) -> Result<Option<serde_json::Value>, SyncError> {
        Self::ensure_table_allowed_and_exists(conn, table_name)?;
        let table_ident = Self::quote_identifier(table_name)?;
        let columns_str = columns
            .iter()
            .map(|c| Self::quote_identifier(c))
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");
        let pk_columns = Self::primary_key_columns(conn, table_name)?;
        let values = Self::parse_record_key_values(table_name, record_id, &pk_columns)?;
        let predicate = Self::build_primary_key_predicate(&pk_columns)?;
        let sql = format!(
            "SELECT {} FROM {} WHERE {}",
            columns_str, table_ident, predicate
        );

        let mut result: Option<serde_json::Value> = conn
            .query_row(&sql, rusqlite::params_from_iter(values.iter()), |row| {
                let mut obj = serde_json::Map::new();
                for (i, col) in columns.iter().enumerate() {
                    let value = Self::sqlite_value_to_json(row, i);
                    obj.insert(col.clone(), value);
                }
                Ok(serde_json::Value::Object(obj))
            })
            .optional()
            .map_err(|e| SyncError::Database(format!("查询记录失败: {}", e)))?;

        if result.is_none() && table_name == "questions" {
            let fallback_sql = format!(
                "SELECT {} FROM {} WHERE exam_id = ?1",
                columns_str, table_ident
            );

            let mut stmt = conn
                .prepare(&fallback_sql)
                .map_err(|e| SyncError::Database(format!("查询 questions 兼容记录失败: {}", e)))?;

            let mut rows = stmt
                .query(params![record_id])
                .map_err(|e| SyncError::Database(format!("查询 questions 兼容记录失败: {}", e)))?;

            if let Some(row) = rows
                .next()
                .map_err(|e| SyncError::Database(format!("读取 questions 兼容记录失败: {}", e)))?
            {
                let obj = {
                    let mut obj = serde_json::Map::new();
                    for (i, col) in columns.iter().enumerate() {
                        let value = Self::sqlite_value_to_json(row, i);
                        obj.insert(col.clone(), value);
                    }
                    obj
                };

                if rows
                    .next()
                    .map_err(|e| {
                        SyncError::Database(format!("读取 questions 兼容记录失败: {}", e))
                    })?
                    .is_none()
                {
                    result = Some(serde_json::Value::Object(obj));
                }
            }
        }

        Ok(result)
    }

    /// 获取表的所有列名
    fn get_table_columns(conn: &Connection, table_name: &str) -> Result<Vec<String>, SyncError> {
        Self::ensure_table_allowed_and_exists(conn, table_name)?;
        let table_ident = Self::quote_identifier(table_name)?;
        let sql = format!("PRAGMA table_info({})", table_ident);
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| SyncError::Database(format!("获取表结构失败: {}", e)))?;

        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| SyncError::Database(format!("查询列名失败: {}", e)))?
            .filter_map(log_and_skip_err)
            .collect();

        Ok(columns)
    }

    fn parse_llm_usage_daily_record_id(
        record_id: &str,
    ) -> Result<(String, String, String, String), SyncError> {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(record_id) {
            if let Some(obj) = value.as_object() {
                let date = obj
                    .get("date")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let caller_type = obj
                    .get("caller_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let model = obj
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let provider = obj
                    .get("provider")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                if !date.is_empty()
                    && !caller_type.is_empty()
                    && !model.is_empty()
                    && !provider.is_empty()
                {
                    return Ok((date, caller_type, model, provider));
                }
            }
        }

        let parts: Vec<&str> = record_id.splitn(4, '_').collect();
        if parts.len() == 4 {
            return Ok((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
                parts[3].to_string(),
            ));
        }

        Err(SyncError::Database(format!(
            "llm_usage_daily 记录ID格式无效: {}",
            record_id
        )))
    }

    fn parse_record_key_values(
        table_name: &str,
        record_id: &str,
        pk_columns: &[String],
    ) -> Result<Vec<String>, SyncError> {
        if pk_columns.is_empty() {
            return Err(SyncError::Database(format!(
                "表 {} 没有可用主键列",
                table_name
            )));
        }

        if pk_columns.len() == 1 {
            return Ok(vec![record_id.to_string()]);
        }

        if table_name == "llm_usage_daily" {
            if let Ok((date, caller_type, model, provider)) =
                Self::parse_llm_usage_daily_record_id(record_id)
            {
                return Ok(vec![date, caller_type, model, provider]);
            }
        }

        if let Ok(value) = serde_json::from_str::<serde_json::Value>(record_id) {
            if let Some(obj) = value.as_object() {
                let mut values = Vec::with_capacity(pk_columns.len());
                for col in pk_columns {
                    let Some(raw) = obj.get(col) else {
                        return Err(SyncError::Database(format!(
                            "复合主键 record_id 缺少字段: {}.{} -> {}",
                            table_name, col, record_id
                        )));
                    };
                    let value =
                        Self::json_value_to_alias_key(raw).unwrap_or_else(|| raw.to_string());
                    values.push(value);
                }
                return Ok(values);
            }

            if let Some(arr) = value.as_array() {
                if arr.len() == pk_columns.len() {
                    let mut values = Vec::with_capacity(pk_columns.len());
                    for raw in arr {
                        values.push(
                            Self::json_value_to_alias_key(raw).unwrap_or_else(|| raw.to_string()),
                        );
                    }
                    return Ok(values);
                }
            }
        }

        let parts: Vec<&str> = record_id.split(':').collect();
        if parts.len() == pk_columns.len() {
            return Ok(parts.into_iter().map(|s| s.to_string()).collect());
        }

        Err(SyncError::Database(format!(
            "复合主键 record_id 无法解析: {}.{} = {}",
            table_name,
            pk_columns.join(","),
            record_id
        )))
    }

    fn build_primary_key_predicate(pk_columns: &[String]) -> Result<String, SyncError> {
        Self::build_primary_key_predicate_from(pk_columns, 1)
    }

    fn build_primary_key_predicate_from(
        pk_columns: &[String],
        first_index: usize,
    ) -> Result<String, SyncError> {
        let mut parts = Vec::with_capacity(pk_columns.len());
        for (idx, col) in pk_columns.iter().enumerate() {
            parts.push(format!(
                "{} = ?{}",
                Self::quote_identifier(col)?,
                first_index + idx
            ));
        }
        Ok(parts.join(" AND "))
    }

    /// 将 SQLite 行值转换为 JSON
    fn sqlite_value_to_json(row: &Row, index: usize) -> serde_json::Value {
        // 尝试不同类型的提取
        if let Ok(v) = row.get::<_, i64>(index) {
            return serde_json::Value::Number(v.into());
        }
        if let Ok(v) = row.get::<_, f64>(index) {
            return serde_json::Number::from_f64(v)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null);
        }
        if let Ok(v) = row.get::<_, String>(index) {
            // 尝试解析为 JSON（处理存储的 JSON 字符串）
            if v.starts_with('{') || v.starts_with('[') {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&v) {
                    return parsed;
                }
            }
            return serde_json::Value::String(v);
        }
        if let Ok(v) = row.get::<_, Vec<u8>>(index) {
            // BLOB 类型，转为 base64 字符串
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&v);
            return serde_json::Value::String(encoded);
        }
        // 默认返回 null
        serde_json::Value::Null
    }

    /// 批量获取变更日志条目的完整记录数据
    ///
    /// 为每个变更日志条目获取其对应记录的完整数据。
    ///
    /// # 参数
    /// * `conn` - 数据库连接
    /// * `entries` - 变更日志条目列表
    /// * `id_column_map` - 表名到主键列名的映射
    ///
    /// # 返回
    /// * 带完整数据的变更列表
    pub fn enrich_changes_with_data(
        conn: &Connection,
        entries: &[ChangeLogEntry],
        id_column_map: Option<&HashMap<String, String>>,
    ) -> Result<Vec<SyncChangeWithData>, SyncError> {
        let mut result = Vec::with_capacity(entries.len());
        // Schema 缓存：避免对同一张表重复执行 PRAGMA table_info (N+1 → 1)
        let mut columns_cache: HashMap<String, Vec<String>> = HashMap::new();

        for entry in entries {
            let id_column = id_column_map
                .and_then(|m| m.get(&entry.table_name))
                .map(|s| s.as_str())
                .unwrap_or("id");

            let data = if entry.operation == ChangeOperation::Delete {
                None
            } else {
                let columns = if let Some(cached) = columns_cache.get(&entry.table_name) {
                    cached
                } else {
                    let cols = Self::get_table_columns(conn, &entry.table_name)?;
                    columns_cache
                        .entry(entry.table_name.clone())
                        .or_insert(cols)
                };

                if columns.is_empty() {
                    None
                } else {
                    Self::get_record_data_with_columns(
                        conn,
                        &entry.table_name,
                        &entry.record_id,
                        id_column,
                        columns,
                    )?
                }
            };

            result.push(SyncChangeWithData::from_entry_with_data(entry, data));
        }

        Ok(result)
    }

    /// 获取数据库的同步状态
    ///
    /// 计算数据库的当前同步状态，包括 schema 版本、数据版本和 checksum。
    ///
    /// # 参数
    /// * `conn` - 数据库连接
    /// * `database_name` - 数据库名称
    ///
    /// # 返回
    /// * `DatabaseSyncState` - 数据库同步状态
    pub fn get_database_sync_state(
        conn: &Connection,
        database_name: &str,
    ) -> Result<DatabaseSyncState, SyncError> {
        // 获取 schema 版本（从 refinery_schema_history 表——迁移系统的权威数据源）
        // 注意：历史版本曾使用 __schema_migrations 表，这里统一到 refinery 权威表，
        // 避免同步状态与迁移系统判定不一致导致伪冲突。
        let schema_version: u32 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM refinery_schema_history",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // 获取数据版本（基于 __change_log 的最大 sync_version，跨库可比较）
        let raw_data_version: u64 = conn
            .query_row(
                "SELECT COALESCE(MAX(sync_version), 0) FROM __change_log",
                [],
                |row| row.get::<_, i64>(0).map(|v| v as u64),
            )
            .unwrap_or(0);
        // 兼容：如果历史 sync_version 被写入了毫秒值（>1e12），归一化为秒
        let data_version = Self::normalize_version_to_seconds(raw_data_version);

        // 获取最后更新时间
        let last_updated_at: Option<String> = conn
            .query_row("SELECT MAX(changed_at) FROM __change_log", [], |row| {
                row.get(0)
            })
            .ok();

        // 计算简单的 checksum（基于表数量和记录数）
        // 实际应用中可能需要更复杂的 checksum 算法
        let checksum = Self::calculate_simple_checksum(conn, database_name)?;

        Ok(DatabaseSyncState {
            schema_version,
            data_version,
            checksum,
            last_updated_at,
        })
    }

    /// 计算数据库 checksum（跨 Rust 版本稳定）
    ///
    /// 使用 SHA-256 代替 DefaultHasher，确保不同编译版本产生一致的哈希值。
    ///
    /// [P1 Fix] 除了 COUNT 之外，还包含 MAX(updated_at)（如果表存在该列），
    /// 避免 "删 1 + 插 1 → COUNT 不变 → checksum 不变" 的伪阴性问题。
    fn calculate_simple_checksum(
        conn: &Connection,
        database_name: &str,
    ) -> Result<String, SyncError> {
        let classifications = classification::sync_classification_registry();

        let tables: Vec<String> = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='table'
                 AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '\\_\\_%' ESCAPE '\\'
                 ORDER BY name",
            )
            .map_err(|e| SyncError::Database(format!("查询表列表失败: {}", e)))?
            .query_map([], |row| row.get(0))
            .map_err(|e| SyncError::Database(format!("获取表名失败: {}", e)))?
            .filter_map(|r: Result<String, _>| r.ok())
            .filter(|table_name| {
                // Exclude FTS5 shadow tables when the FTS virtual table is in DerivedRebuild
                if let Some(base) = table_name
                    .strip_suffix("_content")
                    .or_else(|| table_name.strip_suffix("_docsize"))
                    .or_else(|| table_name.strip_suffix("_config"))
                    .or_else(|| table_name.strip_suffix("_idx"))
                    .or_else(|| table_name.strip_suffix("_segdir"))
                    .or_else(|| table_name.strip_suffix("_segments"))
                    .or_else(|| table_name.strip_suffix("_stat"))
                    .or_else(|| table_name.strip_suffix("_data"))
                {
                    let is_fts_base = classifications.iter().any(|c| {
                        c.database == database_name
                            && c.table_name == base
                            && c.primary_key == "(virtual)"
                    });
                    if is_fts_base {
                        return false;
                    }
                }

                // Only include tables classified as RowSync or FileSync for checksum
                classifications.iter().any(|c| {
                    c.database == database_name
                        && c.table_name == table_name.as_str()
                        && matches!(
                            c.category,
                            classification::SyncCategory::RowSync
                                | classification::SyncCategory::FileSync
                        )
                })
            })
            .collect();

        let mut hasher_input = format!("{}:", database_name);

        for table in &tables {
            let quoted = Self::quote_identifier(table)?;
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {}", quoted), [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);

            // [P1 Fix] 追加 MAX(updated_at) 以捕获记录内容变化
            let max_updated: String = if Self::table_has_column(conn, table, "updated_at") {
                conn.query_row(
                    &format!("SELECT COALESCE(MAX(\"updated_at\"), '') FROM {}", quoted),
                    [],
                    |row| row.get(0),
                )
                .unwrap_or_default()
            } else {
                String::new()
            };

            hasher_input.push_str(&format!("{}={},{};", table, count, max_updated));
        }

        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(hasher_input.as_bytes());
        Ok(hex::encode(&hash[..16]))
    }

    /// 获取变更日志统计信息
    pub fn get_change_log_stats(conn: &Connection) -> Result<ChangeLogStats, SyncError> {
        let total_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM __change_log", [], |row| row.get(0))
            .map_err(|e| SyncError::Database(format!("查询变更日志总数失败: {}", e)))?;

        let pending_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM __change_log WHERE sync_version = 0",
                [],
                |row| row.get(0),
            )
            .map_err(|e| SyncError::Database(format!("查询待同步数量失败: {}", e)))?;

        let synced_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM __change_log WHERE sync_version > 0",
                [],
                |row| row.get(0),
            )
            .map_err(|e| SyncError::Database(format!("查询已同步数量失败: {}", e)))?;

        Ok(ChangeLogStats {
            total_count: total_count as usize,
            pending_count: pending_count as usize,
            synced_count: synced_count as usize,
        })
    }
}
