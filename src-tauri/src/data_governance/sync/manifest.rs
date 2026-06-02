/// 公开的时间戳解析函数（供 conflict_resolver 等子模块复用）
pub fn parse_flexible_timestamp_public(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    use chrono::{DateTime, NaiveDateTime, Utc};
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(naive.and_utc());
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Some(naive.and_utc());
    }
    // 纯数字串：尝试作为毫秒时间戳解析
    // （resources / chat_v2_todo_lists 等表用 INTEGER ms 存储 updated_at）
    if let Ok(ms) = s.parse::<i64>() {
        // 秒级 (1e9 ~ 1e10) vs 毫秒级 (1e12 ~ 1e13) 用阈值区分，避免
        // 2038 前后年份的数值被误当毫秒
        const MS_THRESHOLD: i64 = 100_000_000_000; // 1e11
        if ms >= MS_THRESHOLD {
            return DateTime::<Utc>::from_timestamp_millis(ms);
        } else if ms >= 1_000_000_000 {
            return DateTime::<Utc>::from_timestamp(ms, 0);
        }
    }
    None
}

use rusqlite::{types::Type, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 同步清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncManifest {
    /// 同步事务 ID
    pub sync_transaction_id: String,
    /// 各数据库状态
    pub databases: HashMap<String, DatabaseSyncState>,
    /// 状态
    pub status: SyncTransactionStatus,
    /// 创建时间
    pub created_at: String,
    /// 设备 ID
    pub device_id: String,
}

/// 数据库同步状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSyncState {
    /// Schema 版本
    pub schema_version: u32,
    /// 数据版本（最大 local_version）
    pub data_version: u64,
    /// Checksum
    pub checksum: String,
    /// 最后更新时间
    #[serde(default)]
    pub last_updated_at: Option<String>,
}

/// 同步事务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncTransactionStatus {
    /// 完成
    Complete,
    /// 部分完成（需要修复）
    Partial,
    /// 失败
    Failed,
}

/// 数据库级冲突
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConflict {
    /// 数据库名称
    pub database_name: String,
    /// 冲突类型
    pub conflict_type: DatabaseConflictType,
    /// 本地状态
    pub local_state: Option<DatabaseSyncState>,
    /// 云端状态
    pub cloud_state: Option<DatabaseSyncState>,
}

/// 数据库冲突类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatabaseConflictType {
    /// Schema 版本不匹配（需要迁移）
    SchemaMismatch,
    /// 数据版本冲突（双方都有修改）
    DataConflict,
    /// Checksum 不匹配（数据内容不同）
    ChecksumMismatch,
    /// 本地有，云端没有
    LocalOnly,
    /// 云端有，本地没有
    CloudOnly,
}

/// 冲突记录（记录级别）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecord {
    /// 数据库名称
    pub database_name: String,
    /// 表名
    pub table_name: String,
    /// 记录 ID
    pub record_id: String,
    /// 本地版本
    pub local_version: u64,
    /// 云端版本
    pub cloud_version: u64,
    /// 本地更新时间
    pub local_updated_at: String,
    /// 云端更新时间
    pub cloud_updated_at: String,
    /// 本地数据（JSON）
    pub local_data: serde_json::Value,
    /// 云端数据（JSON）
    pub cloud_data: serde_json::Value,
}

/// 冲突检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetectionResult {
    /// 数据库级冲突
    pub database_conflicts: Vec<DatabaseConflict>,
    /// 记录级冲突（需要进一步查询数据库）
    pub record_conflicts: Vec<ConflictRecord>,
    /// 是否有冲突
    pub has_conflicts: bool,
    /// 是否需要迁移
    pub needs_migration: bool,
}

impl ConflictDetectionResult {
    /// 创建空的检测结果（无冲突）
    pub fn empty() -> Self {
        Self {
            database_conflicts: Vec::new(),
            record_conflicts: Vec::new(),
            has_conflicts: false,
            needs_migration: false,
        }
    }

    /// 冲突总数
    pub fn total_conflicts(&self) -> usize {
        self.database_conflicts.len() + self.record_conflicts.len()
    }
}

/// 合并策略
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MergeStrategy {
    /// 保留本地
    KeepLocal,
    /// 使用云端
    UseCloud,
    /// 保留最新（按 updated_at）
    KeepLatest,
    /// 手动合并（用户选择）
    Manual,
}

/// 同步结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// 是否成功
    pub success: bool,
    /// 同步的数据库数量
    pub synced_databases: usize,
    /// 解决的冲突数量
    pub resolved_conflicts: usize,
    /// 需要手动处理的冲突
    pub pending_manual_conflicts: Vec<ConflictRecord>,
    /// 错误信息（如果有）
    pub errors: Vec<String>,
}

impl SyncResult {
    /// 创建成功结果
    pub fn success(synced_databases: usize, resolved_conflicts: usize) -> Self {
        Self {
            success: true,
            synced_databases,
            resolved_conflicts,
            pending_manual_conflicts: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// 创建需要手动处理的结果
    pub fn needs_manual(conflicts: Vec<ConflictRecord>) -> Self {
        Self {
            success: false,
            synced_databases: 0,
            resolved_conflicts: 0,
            pending_manual_conflicts: conflicts,
            errors: Vec::new(),
        }
    }

    /// 创建失败结果
    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            success: false,
            synced_databases: 0,
            resolved_conflicts: 0,
            pending_manual_conflicts: Vec::new(),
            errors,
        }
    }
}

/// 同步错误
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Conflict detected: {count} records")]
    Conflict { count: usize },

    #[error("Schema mismatch: local={local}, cloud={cloud}")]
    SchemaMismatch { local: u32, cloud: u32 },

    #[error("Partial sync: {completed}/{total} databases")]
    PartialSync { completed: usize, total: usize },

    #[error("Manual resolution required: {count} conflicts")]
    ManualResolutionRequired { count: usize },

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

/// 同步字段 SQL（用于需要同步的表）
pub const SYNC_FIELDS_SQL: &str = r#"
    -- 添加同步字段
    ALTER TABLE {table} ADD COLUMN device_id TEXT;
    ALTER TABLE {table} ADD COLUMN local_version INTEGER DEFAULT 0;
    ALTER TABLE {table} ADD COLUMN sync_version INTEGER DEFAULT 0;
    ALTER TABLE {table} ADD COLUMN updated_at TEXT DEFAULT (datetime('now'));
    ALTER TABLE {table} ADD COLUMN deleted_at TEXT;  -- tombstone，非 NULL 表示已删除

    -- 创建索引
    CREATE INDEX IF NOT EXISTS idx_{table}_local_version ON {table}(local_version);
    CREATE INDEX IF NOT EXISTS idx_{table}_sync_version ON {table}(sync_version);
    CREATE INDEX IF NOT EXISTS idx_{table}_deleted_at ON {table}(deleted_at);
"#;

/// 工作区数据库云同步清单（ws_*.db 文件级同步）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspacesManifest {
    /// ws_id → 条目
    pub entries: HashMap<String, WorkspaceEntry>,
    #[serde(default)]
    pub updated_at: String,
}

/// 单个工作区数据库的同步条目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkspaceEntry {
    pub sha256: String,
    pub size: u64,
    pub updated_at: String,
}

/// VFS blob 云同步清单（内容寻址）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlobsManifest {
    /// content_hash → 条目
    pub entries: HashMap<String, BlobEntry>,
    #[serde(default)]
    pub updated_at: String,
}

/// 单个 blob 的同步条目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlobEntry {
    /// 相对路径（相对于 vfs_blobs/），如 "ab/abc123....pdf"
    pub relative_path: String,
    pub size: u64,
}

/// VFS Blob 同步结果，区分完全成功与部分失败
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlobSyncOutcome {
    pub uploaded: usize,
    pub downloaded: usize,
    pub upload_failures: Vec<String>,
    pub download_failures: Vec<String>,
}

impl BlobSyncOutcome {
    pub fn has_failures(&self) -> bool {
        !self.upload_failures.is_empty() || !self.download_failures.is_empty()
    }

    pub fn failure_summary(&self) -> Option<String> {
        if !self.has_failures() {
            return None;
        }
        Some(format!(
            "附件同步部分失败：{} 个上传失败，{} 个下载失败",
            self.upload_failures.len(),
            self.download_failures.len()
        ))
    }
}

/// 通用资产目录云同步清单（images/documents/...）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssetDirsManifest {
    /// key -> 条目，key 形如 "active/images/a.png" 或 "app_data/pdf_ocr_sessions/x.json"
    pub entries: HashMap<String, AssetFileEntry>,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetFileEntry {
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetSyncOutcome {
    pub uploaded: usize,
    pub downloaded: usize,
    pub upload_failures: Vec<String>,
    pub download_failures: Vec<String>,
}

impl AssetSyncOutcome {
    pub fn has_failures(&self) -> bool {
        !self.upload_failures.is_empty() || !self.download_failures.is_empty()
    }

    pub fn failure_summary(&self) -> Option<String> {
        if !self.has_failures() {
            return None;
        }
        Some(format!(
            "资产目录同步部分失败：{} 个上传失败，{} 个下载失败",
            self.upload_failures.len(),
            self.download_failures.len()
        ))
    }
}

/// 下载变更结果（包含非致命解析告警）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DownloadChangesResult {
    pub changes: Vec<SyncChangeWithData>,
    pub decode_failures: Vec<String>,
}

/// 变更日志统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeLogStats {
    /// 总记录数
    pub total_count: usize,
    /// 待同步数量
    pub pending_count: usize,
    /// 已同步数量
    pub synced_count: usize,
}

/// 记录快照（用于冲突检测）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordSnapshot {
    /// 表名
    pub table_name: String,
    /// 记录 ID
    pub record_id: String,
    /// 本地版本
    pub local_version: u64,
    /// 同步版本
    pub sync_version: u64,
    /// 更新时间
    pub updated_at: String,
    /// 删除时间（tombstone）
    pub deleted_at: Option<String>,
    /// 记录数据（JSON）
    pub data: serde_json::Value,
}

/// 冲突解决方式
///
/// 注意：此类型包含 serde_json::Value，无法自动导出 TypeScript 类型。
/// 在 TypeScript 中手动定义为：
/// ```typescript
/// type ConflictResolution = "KeepLocal" | "UseCloud" | { Merge: any };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// 保留本地
    KeepLocal,
    /// 使用云端
    UseCloud,
    /// 手动合并的数据
    Merge(serde_json::Value),
}

/// 已解决的记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedRecord {
    /// 数据库名称
    pub database_name: String,
    /// 表名
    pub table_name: String,
    /// 记录 ID
    pub record_id: String,
    /// 解决后的数据
    pub resolved_data: serde_json::Value,
    /// 新版本号
    pub new_version: u64,
    /// 解决时间
    pub resolved_at: String,
    /// 解决设备 ID
    pub resolved_by: String,
}

/// 变更日志操作类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeOperation {
    /// 插入
    Insert,
    /// 更新
    Update,
    /// 删除
    Delete,
}

impl ChangeOperation {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "INSERT" => Some(Self::Insert),
            "UPDATE" => Some(Self::Update),
            "DELETE" => Some(Self::Delete),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Insert => "INSERT",
            Self::Update => "UPDATE",
            Self::Delete => "DELETE",
        }
    }
}

/// 带完整数据的同步变更
///
/// 扩展 ChangeLogEntry，包含完整的记录数据，用于云同步时传输完整记录。
/// 上传时必须携带 `data`（INSERT/UPDATE），下载后可直接回放，无需再查库。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncChangeWithData {
    /// 表名
    pub table_name: String,
    /// 记录 ID
    pub record_id: String,
    /// 操作类型
    pub operation: ChangeOperation,
    /// 完整记录数据（JSON 格式）
    /// - INSERT/UPDATE: 包含完整记录
    /// - DELETE: None
    pub data: Option<serde_json::Value>,
    /// 变更时间
    pub changed_at: String,
    /// 变更日志 ID（可选，用于追踪）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_log_id: Option<i64>,
    /// 来源数据库名称（用于多库同步时按库路由）
    /// 值为 DatabaseId::as_str()，如 "chat_v2"、"vfs"、"mistakes"、"llm_usage"
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub database_name: Option<String>,
    /// 回放时是否抑制写入 __change_log（防止下载回放形成回声同步）
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub suppress_change_log: Option<bool>,
}

impl SyncChangeWithData {
    /// 从 ChangeLogEntry 创建（不含数据，兼容旧链路）
    ///
    /// **注意**：此方法仅用于兼容旧格式下载数据。新上传链路应使用
    /// `enrich_changes_with_data` 确保 INSERT/UPDATE 携带完整数据。
    pub fn from_entry(entry: &ChangeLogEntry) -> Self {
        Self {
            table_name: entry.table_name.clone(),
            record_id: entry.record_id.clone(),
            operation: entry.operation,
            data: None,
            changed_at: entry.changed_at.clone(),
            change_log_id: Some(entry.id),
            database_name: None,
            suppress_change_log: None,
        }
    }

    /// 从 ChangeLogEntry 创建并附加数据
    pub fn from_entry_with_data(entry: &ChangeLogEntry, data: Option<serde_json::Value>) -> Self {
        let mut data = data;
        if let Some(field_deltas) = entry.field_deltas_json.as_ref() {
            if let Some(serde_json::Value::Object(obj)) = data.as_mut() {
                obj.insert(SYNC_FIELD_DELTAS_KEY.to_string(), field_deltas.clone());
            }
        }

        Self {
            table_name: entry.table_name.clone(),
            record_id: entry.record_id.clone(),
            operation: entry.operation,
            data,
            changed_at: entry.changed_at.clone(),
            change_log_id: Some(entry.id),
            database_name: None,
            suppress_change_log: None,
        }
    }
}

/// 应用变更的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyChangesResult {
    /// 成功应用的变更数
    pub success_count: usize,
    /// 失败的变更数
    pub failure_count: usize,
    /// 跳过的变更数（保留字段，当前主要用于非致命跳过场景）
    pub skipped_count: usize,
    /// 失败的详情
    pub failures: Vec<ApplyChangeFailure>,
    /// 实际成功落地的记录 key (table_name, record_id)
    /// 用于上层精确计算"已被云端覆盖"的本地待上传项
    pub applied_keys: std::collections::HashSet<(String, String)>,
}

impl ApplyChangesResult {
    /// 创建空结果
    pub fn empty() -> Self {
        Self {
            success_count: 0,
            failure_count: 0,
            skipped_count: 0,
            failures: Vec::new(),
            applied_keys: std::collections::HashSet::new(),
        }
    }

    /// 合并另一个结果
    pub fn merge(&mut self, other: ApplyChangesResult) {
        self.success_count += other.success_count;
        self.failure_count += other.failure_count;
        self.skipped_count += other.skipped_count;
        self.failures.extend(other.failures);
        self.applied_keys.extend(other.applied_keys);
    }
}

/// 单条变更应用失败的详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyChangeFailure {
    /// 表名
    pub table_name: String,
    /// 记录 ID
    pub record_id: String,
    /// 操作类型
    pub operation: String,
    /// 错误信息
    pub error: String,
}

/// 变更日志条目（来自 __change_log 表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeLogEntry {
    /// 记录 ID（自增）
    pub id: i64,
    /// 表名
    pub table_name: String,
    /// 记录 ID
    pub record_id: String,
    /// 操作类型
    pub operation: ChangeOperation,
    /// 变更时间
    pub changed_at: String,
    /// 同步版本（0 表示未同步）
    pub sync_version: i64,
    /// 字段增量元数据（用于 counter merge 等需要 old/new 的场景）
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub field_deltas_json: Option<serde_json::Value>,
}

impl ChangeLogEntry {
    /// 从数据库行解析
    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        let operation_str: String = row.get(3)?;
        let operation =
            ChangeOperation::from_str(&operation_str).unwrap_or(ChangeOperation::Update);

        Ok(Self {
            id: row.get(0)?,
            table_name: row.get(1)?,
            record_id: row.get(2)?,
            operation,
            changed_at: row.get(4)?,
            sync_version: row.get(5)?,
            field_deltas_json: if row.as_ref().column_count() > 6 {
                match row.get::<_, Option<String>>(6)? {
                    Some(raw) => Some(serde_json::from_str(&raw).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(6, Type::Text, Box::new(e))
                    })?),
                    None => None,
                }
            } else {
                None
            },
        })
    }
}

/// 云端变更载荷（v2 格式：含完整记录数据）
///
/// 上传/下载时使用的完整载荷，包含每条变更的实际行数据。
/// 相比旧的 `PendingChanges`（仅含 ChangeLogEntry 元数据），
/// 此格式确保下载端可以直接回放 INSERT/UPDATE 操作。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncChangesPayload {
    /// 带完整数据的变更列表
    pub changes: Vec<SyncChangeWithData>,
    /// 变更总数
    pub total_count: usize,
    /// 上传设备 ID
    pub device_id: String,
    /// 格式版本号（2 = 带完整数据）
    #[serde(default = "default_format_version")]
    pub format_version: u32,
}

fn default_format_version() -> u32 {
    2
}

/// JSON key 名，存储于云端 payload 的 data 对象中，
/// 用于在 apply_single_record 的 field_merge 阶段读取 counter delta
/// 的值（而非 COALESCE 后的绝对值）。
pub(crate) const SYNC_FIELD_DELTAS_KEY: &str = "__sync_field_deltas";

/// 待同步变更集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingChanges {
    /// 变更日志条目列表
    pub entries: Vec<ChangeLogEntry>,
    /// 按表名分组的变更数量
    pub changes_by_table: HashMap<String, usize>,
    /// 总变更数量
    pub total_count: usize,
    /// 最早的变更时间
    pub earliest_change: Option<String>,
    /// 最晚的变更时间
    pub latest_change: Option<String>,
}

impl PendingChanges {
    /// 创建空的待同步变更
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
            changes_by_table: HashMap::new(),
            total_count: 0,
            earliest_change: None,
            latest_change: None,
        }
    }

    /// 从变更日志条目列表构建
    pub fn from_entries(entries: Vec<ChangeLogEntry>) -> Self {
        let mut changes_by_table: HashMap<String, usize> = HashMap::new();
        let mut earliest: Option<String> = None;
        let mut latest: Option<String> = None;

        for entry in &entries {
            *changes_by_table
                .entry(entry.table_name.clone())
                .or_insert(0) += 1;

            let changed_at = &entry.changed_at;
            match &earliest {
                None => earliest = Some(changed_at.clone()),
                Some(e) if changed_at < e => earliest = Some(changed_at.clone()),
                _ => {}
            }
            match &latest {
                None => latest = Some(changed_at.clone()),
                Some(l) if changed_at > l => latest = Some(changed_at.clone()),
                _ => {}
            }
        }

        let total_count = entries.len();

        Self {
            entries,
            changes_by_table,
            total_count,
            earliest_change: earliest,
            latest_change: latest,
        }
    }

    /// 是否有待同步的变更
    pub fn has_changes(&self) -> bool {
        self.total_count > 0
    }

    /// 获取指定表的变更条目
    pub fn get_table_changes(&self, table_name: &str) -> Vec<&ChangeLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.table_name == table_name)
            .collect()
    }

    /// 获取所有变更记录的 ID 列表
    pub fn get_change_ids(&self) -> Vec<i64> {
        self.entries.iter().map(|e| e.id).collect()
    }
}

/// 合并应用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeApplicationResult {
    /// 是否成功
    pub success: bool,
    /// 保留本地的记录数
    pub kept_local: usize,
    /// 使用云端的记录数
    pub used_cloud: usize,
    /// 需要更新到云端的记录 ID 列表
    pub records_to_push: Vec<String>,
    /// 需要从云端拉取更新的记录 ID 列表
    pub records_to_pull: Vec<String>,
    /// 错误信息
    pub errors: Vec<String>,
}

impl MergeApplicationResult {
    /// 创建成功结果
    pub fn success(kept_local: usize, used_cloud: usize) -> Self {
        Self {
            success: true,
            kept_local,
            used_cloud,
            records_to_push: Vec::new(),
            records_to_pull: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// 创建失败结果
    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            success: false,
            kept_local: 0,
            used_cloud: 0,
            records_to_push: Vec::new(),
            records_to_pull: Vec::new(),
            errors,
        }
    }
}

/// 同步方向
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SyncDirection {
    /// 仅上传（本地 -> 云端）
    Upload,
    /// 仅下载（云端 -> 本地）
    Download,
    /// 双向同步
    Bidirectional,
}

impl SyncDirection {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "upload" => Some(Self::Upload),
            "download" => Some(Self::Download),
            "bidirectional" | "both" => Some(Self::Bidirectional),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Upload => "upload",
            Self::Download => "download",
            Self::Bidirectional => "bidirectional",
        }
    }
}

/// 同步执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncExecutionResult {
    /// 是否成功
    pub success: bool,
    /// 同步方向
    pub direction: SyncDirection,
    /// 上传的变更数量
    pub changes_uploaded: usize,
    /// 下载的变更数量
    pub changes_downloaded: usize,
    /// 检测到的冲突数量
    pub conflicts_detected: usize,
    /// 执行耗时（毫秒）
    pub duration_ms: u64,
    /// 错误信息（如果有）
    pub error_message: Option<String>,
}

impl SyncExecutionResult {
    /// 创建成功结果
    pub fn success(
        direction: SyncDirection,
        uploaded: usize,
        downloaded: usize,
        conflicts: usize,
        duration_ms: u64,
    ) -> Self {
        Self {
            success: true,
            direction,
            changes_uploaded: uploaded,
            changes_downloaded: downloaded,
            conflicts_detected: conflicts,
            duration_ms,
            error_message: None,
        }
    }

    /// 创建失败结果
    pub fn failure(direction: SyncDirection, error: String, duration_ms: u64) -> Self {
        Self {
            success: false,
            direction,
            changes_uploaded: 0,
            changes_downloaded: 0,
            conflicts_detected: 0,
            duration_ms,
            error_message: Some(error),
        }
    }
}
