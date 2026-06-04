# API 重构: Memory — 智能记忆

**日期**: 2026-05-29 | **命令数**: 27 | **对应诊断**: round-20~26

---

## 当前问题

27 个命令，每个命令都需要 3 个 State 参数 (VfsDatabase + VfsLanceStore + LLMManager)。应该封装为 MemoryContext。

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `State<VfsDatabase>` | 27 |
| `State<VfsLanceStore>` | 26 |
| `State<LLMManager>` | 26 |
| `String` | 18 |
| `Option<String>` | 10 |
| `Option<u32>` | 5 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `()` | 10 |
| `BatchOperationResult` | 2 |
| `String` | 2 |
| `Vec<String>` | 2 |
| `Vec<MemoryExportItem>` | 1 |
| `Vec<MemoryAuditLogItem>` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `memory_add_relation` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_batch_delete` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_batch_move` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_create_root_folder` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_delete` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_export_all` | *(保持)* | → Context struct | Result<T, MemoryError> |
| `memory_get_audit_logs` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_get_config` | *(保持)* | → Context struct | Result<T, MemoryError> |
| `memory_get_or_create_root_folder` | *(保持)* | → Context struct | Result<T, MemoryError> |
| `memory_get_profile` | *(保持)* | → Context struct | Result<T, MemoryError> |
| `memory_get_related` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_get_tags` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_get_tree` | *(保持)* | → Context struct | Result<T, MemoryError> |
| `memory_list` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_move_to_folder` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_read` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_remove_relation` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_search` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_set_auto_create_subfolders` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_set_auto_extract_frequency` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_set_default_category` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_set_privacy_mode` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_set_root_folder` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_to_anki_document` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_update_by_id` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_update_tags` | *(保持)* | → Input struct | Result<T, MemoryError> |
| `memory_write_batch` | *(保持)* | → Input struct | Result<T, MemoryError> |

## 改进操作

引入 MemoryContext 封装 3 个 State，统一错误类型为 MemoryError

## 统一错误类型

`MemoryError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/memory.json`*
