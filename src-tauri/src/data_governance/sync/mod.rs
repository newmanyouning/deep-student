//! # Sync 模块
//!
//! 云同步管理系统。
//!
//! ## 设计原则
//!
//! 1. **版本戳机制**：每条记录有 `local_version` 和 `updated_at`
//! 2. **记录级冲突检测**：不是全库覆盖，而是按记录检测冲突
//! 3. **Tombstone 删除**：删除用 `deleted_at` 标记，而非直接删除
//! 4. **用户选择**：冲突时由用户选择合并策略
//!
//! ## 同步字段
//!
//! 所有需要同步的表应添加以下字段：
//!
//! ```sql
//! ALTER TABLE xxx ADD COLUMN device_id TEXT;
//! ALTER TABLE xxx ADD COLUMN local_version INTEGER DEFAULT 0;
//! ALTER TABLE xxx ADD COLUMN updated_at TEXT;
//! ALTER TABLE xxx ADD COLUMN deleted_at TEXT;  -- tombstone
//! ```
//!
//! ## 组件
//!
//! - `manager`: 同步管理器
//! - `conflict`: 记录级冲突检测
//! - `merge`: 合并策略
//! - `progress`: 同步进度管理
//! - `emitter`: 进度事件发射器
//!
//! ## 云存储集成
//!
//! 支持与云存储模块对接，提供以下功能：
//! - 上传/下载同步清单
//! - 上传/下载变更数据
//! - 支持增量同步
//! - 进度回调和实时状态更新

// 子模块声明
pub mod classification;
pub mod conflict_resolver;
pub mod emitter;
pub mod field_merge;
pub mod hlc;
pub mod progress;
pub mod tombstone;
pub mod retry;
pub mod manifest;
pub mod orchestrator;
pub mod changeset;

#[cfg(test)]
mod tests;

// 重新导出核心类型
pub use manifest::{
    parse_flexible_timestamp_public, ApplyChangeFailure, ApplyChangesResult, AssetDirsManifest,
    AssetFileEntry, AssetSyncOutcome, BlobEntry, BlobSyncOutcome, BlobsManifest, ChangeLogEntry,
    ChangeLogStats, ChangeOperation, ConflictDetectionResult, ConflictRecord, ConflictResolution,
    DatabaseConflict, DatabaseConflictType, DatabaseSyncState, DownloadChangesResult,
    MergeApplicationResult, MergeStrategy, PendingChanges, RecordSnapshot, ResolvedRecord,
    SyncChangeWithData, SyncChangesPayload, SyncDirection, SyncError, SyncExecutionResult,
    SyncManifest, SyncResult, SyncTransactionStatus, SYNC_FIELDS_SQL, WorkspaceEntry,
    WorkspacesManifest,
};
pub use orchestrator::SyncManager;

// 重新导出子模块常用类型
pub use conflict_resolver::{
    ConflictAwareApplyResult, ConflictOutcome, ConflictPolicy, ConflictRecordToSave,
    ConflictResolver, ConflictSide,
};
pub use emitter::{OptionalEmitter, SyncProgressCallback, SyncProgressEmitter, EVENT_NAME};
pub use hlc::{compare_hlc_strings, Hlc, HlcClock, HlcError, MAX_DRIFT_MS};
pub use progress::{ProgressTracker, SpeedCalculator, SyncPhase, SyncProgress};
pub use tombstone::{
    apply_blob_tombstones, AssetTombstoneEntry, AssetTombstones, BlobTombstoneEntry, BlobTombstones,
};
