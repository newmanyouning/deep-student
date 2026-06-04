# API 重构: Data Governance — 数据治理

**日期**: 2026-05-29 | **命令数**: 43 | **对应诊断**: round-20~26

---

## 当前问题

43 个命令，覆盖备份/恢复/同步/审计/迁移 5 个子域。返回类型用 String。

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `AppHandle` | 36 |
| `String` | 24 |
| `Option<String>` | 13 |
| `State<BackupJobManagerState>` | 10 |
| `State<RwLock<SchemaRegistry>>` | 5 |
| `Window` | 4 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `BackupJobStartResponse` | 5 |
| `()` | 3 |
| `bool` | 2 |
| `u64` | 2 |
| `SlotMigrationTestResponse` | 2 |
| `SyncExecutionResponse` | 2 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `data_governance_auto_verify_latest_backup` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_cancel_backup` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_check_disk_space_for_restore` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_cleanup_audit_logs` | *(保持)* | → Input struct | Result<T, DataGovernanceError> |
| `data_governance_cleanup_persisted_jobs` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_count_record_conflicts` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_delete_backup` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_detect_conflicts` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_detect_prune_gap` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_export_sync_data` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_export_zip` | *(保持)* | → Input struct | Result<T, DataGovernanceError> |
| `data_governance_get_asset_types` | *(保持)* | — | Result<Vec<AssetTypeInfo>, DataGovernanceError> |
| `data_governance_get_audit_logs` | *(保持)* | → Input struct | Result<T, DataGovernanceError> |
| `data_governance_get_backup_job` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_get_backup_list` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_get_database_status` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_get_maintenance_status` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_get_migration_diagnostic_report` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_get_migration_status` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_get_schema_registry` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_get_sync_status` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_import_sync_data` | *(保持)* | → Input struct | Result<T, DataGovernanceError> |
| `data_governance_import_zip` | *(保持)* | → Input struct | Result<T, DataGovernanceError> |
| `data_governance_list_backup_jobs` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_list_record_conflicts` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_list_resumable_jobs` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_mark_asset_deleted` | *(保持)* | → Input struct | Result<T, DataGovernanceError> |
| `data_governance_mark_blob_deleted` | *(保持)* | → Input struct | Result<T, DataGovernanceError> |
| `data_governance_purge_resolved_conflicts` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_resolve_conflicts` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_resolve_record_conflict` | *(保持)* | → Input struct | Result<T, DataGovernanceError> |
| `data_governance_restore_backup` | *(保持)* | → Input struct | Result<T, DataGovernanceError> |
| `data_governance_restore_with_assets` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_resume_backup_job` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_run_backup` | *(保持)* | → Input struct | Result<T, DataGovernanceError> |
| `data_governance_run_health_check` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_run_slot_c_empty_db_test` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_run_slot_d_clone_db_test` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_run_sync` | *(保持)* | → Input struct | Result<T, DataGovernanceError> |
| `data_governance_run_sync_with_progress` | *(保持)* | → Input struct | Result<T, DataGovernanceError> |
| `data_governance_scan_assets` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_verify_backup` | *(保持)* | — | Result<T, DataGovernanceError> |
| `data_governance_verify_backup_with_assets` | *(保持)* | — | Result<T, DataGovernanceError> |

## 改进操作

统一错误类型为 DataGovernanceError，按子域组织命令

## 统一错误类型

`DataGovernanceError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/data_governance.json`*
