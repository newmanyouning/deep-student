# API 重构: Debug Commands — 调试命令

**日期**: 2026-05-29 | **命令数**: 7 | **对应诊断**: round-20~26

---

## 当前问题

7 个命令，生产代码含调试

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `State<AppState>` | 5 |
| `String` | 3 |
| `Vec<String>` | 1 |
| `State<std::sync::Arc<vfs::database::VfsDatabase>>` | 1 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `DebugDatabaseStats` | 1 |
| `Option<DebugRawMistakeRecord>` | 1 |
| `Vec<DebugRawMistakeRecord>` | 1 |
| `DebugIntegrityReport` | 1 |
| `VfsMigrationDiagnostic` | 1 |
| `Vec<DebugTextbookPageInfo>` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `debug_get_database_stats` | *(保持)* | — | — |
| `debug_get_raw_mistake` | *(保持)* | — | — |
| `debug_get_raw_mistakes_batch` | *(保持)* | — | — |
| `debug_verify_mistake_integrity` | *(保持)* | — | — |
| `debug_vfs_migration_status` | *(保持)* | — | Result<T, AppError> |
| `debug_vfs_textbook_pages` | *(保持)* | — | — |
| `log_debug_message` | *(保持)* | — | Result<T, AppError> |

## 改进操作

标记为 dev-only

## 统一错误类型

`AppError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/debug_commands.json`*
