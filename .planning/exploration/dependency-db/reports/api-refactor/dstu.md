# API 重构: DSTU — 资源协议

**日期**: 2026-05-29 | **命令数**: 54 | **对应诊断**: round-20~26

---

## 当前问题

54 个命令，命名一致性最好（全部 dstu_ 前缀）。但错误类型用 String。

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `String` | 56 |
| `State<VfsDatabase>` | 51 |
| `Window` | 20 |
| `Option<String>` | 12 |
| `Vec<String>` | 5 |
| `bool` | 5 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `()` | 17 |
| `DstuNode` | 6 |
| `usize` | 6 |
| `Vec<DstuNode>` | 5 |
| `String` | 3 |
| `Option<DstuNode>` | 2 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `dstu_batch_move` | *(保持)* | — | Result<T, DstuError> |
| `dstu_build_path` | *(保持)* | — | Result<T, DstuError> |
| `dstu_copy` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_create` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_delete` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_delete_many` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_empty_trash` | *(保持)* | — | — |
| `dstu_export` | *(保持)* | — | Result<T, DstuError> |
| `dstu_export_formats` | *(保持)* | — | Result<T, DstuError> |
| `dstu_folder_add_item` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_folder_create` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_folder_delete` | *(保持)* | — | Result<T, DstuError> |
| `dstu_folder_get` | *(保持)* | — | Result<T, DstuError> |
| `dstu_folder_get_all_resources` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_folder_get_breadcrumbs` | *(保持)* | — | Result<T, DstuError> |
| `dstu_folder_get_items` | *(保持)* | — | Result<T, DstuError> |
| `dstu_folder_get_tree` | *(保持)* | — | Result<T, DstuError> |
| `dstu_folder_list` | *(保持)* | — | Result<T, DstuError> |
| `dstu_folder_move` | *(保持)* | — | Result<T, DstuError> |
| `dstu_folder_move_item` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_folder_remove_item` | *(保持)* | — | Result<T, DstuError> |
| `dstu_folder_rename` | *(保持)* | — | Result<T, DstuError> |
| `dstu_folder_reorder` | *(保持)* | — | Result<T, DstuError> |
| `dstu_folder_reorder_items` | *(保持)* | — | Result<T, DstuError> |
| `dstu_folder_set_expanded` | *(保持)* | — | Result<T, DstuError> |
| `dstu_get` | *(保持)* | — | Result<T, DstuError> |
| `dstu_get_content` | *(保持)* | — | Result<T, DstuError> |
| `dstu_get_exam_content` | *(保持)* | — | Result<T, DstuError> |
| `dstu_get_path_by_id` | *(保持)* | — | Result<T, DstuError> |
| `dstu_get_resource_by_path` | *(保持)* | — | Result<T, DstuError> |
| `dstu_get_resource_location` | *(保持)* | — | Result<T, DstuError> |
| `dstu_list` | *(保持)* | — | Result<T, DstuError> |
| `dstu_list_deleted` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_list_trash` | *(保持)* | — | — |
| `dstu_move` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_move_many` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_move_to_folder` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_parse_path` | *(保持)* | — | Result<T, DstuError> |
| `dstu_permanently_delete` | *(保持)* | → Input struct | — |
| `dstu_purge` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_purge_all` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_refresh_path_cache` | *(保持)* | — | Result<T, DstuError> |
| `dstu_rename` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_restore` | *(保持)* | — | Result<T, DstuError> |
| `dstu_restore_many` | *(保持)* | — | Result<T, DstuError> |
| `dstu_search` | *(保持)* | — | Result<T, DstuError> |
| `dstu_search_in_folder` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_set_favorite` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_set_metadata` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_soft_delete` | *(保持)* | → Input struct | — |
| `dstu_trash_restore` | *(保持)* | → Input struct | — |
| `dstu_unwatch` | *(保持)* | — | Result<T, DstuError> |
| `dstu_update` | *(保持)* | → Input struct | Result<T, DstuError> |
| `dstu_watch` | *(保持)* | — | Result<T, DstuError> |

## 改进操作

统一错误类型为 DstuError，优化参数封装

## 统一错误类型

`DstuError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/dstu.json`*
