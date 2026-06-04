# API 重构: Question Sync — 题目同步

**日期**: 2026-05-29 | **命令数**: 6 | **对应诊断**: round-20~26

---

## 当前问题

6 个命令

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `String` | 8 |
| `State<AppState>` | 6 |
| `bool` | 1 |
| `SyncConfig` | 1 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `()` | 2 |
| `Vec<crate::vfs::repos::question_repo::Question>` | 1 |
| `Vec<SyncConflict>` | 1 |
| `crate::vfs::repos::question_repo::Question` | 1 |
| `SyncStatusResult` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `qbank_batch_resolve_conflicts` | *(保持)* | — | Result<T, SyncError> |
| `qbank_get_sync_conflicts` | *(保持)* | — | Result<T, SyncError> |
| `qbank_resolve_sync_conflict` | *(保持)* | — | Result<T, SyncError> |
| `qbank_set_sync_enabled` | *(保持)* | — | Result<T, SyncError> |
| `qbank_sync_check` | *(保持)* | — | Result<T, SyncError> |
| `qbank_update_sync_config` | *(保持)* | — | Result<T, SyncError> |

## 改进操作

统一错误类型

## 统一错误类型

`SyncError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/question_sync_service.json`*
