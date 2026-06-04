# API 重构: cmd::notes — 笔记命令

**日期**: 2026-05-29 | **命令数**: 39 | **对应诊断**: round-20~26

---

## 当前问题

39 个命令，部分用 canvas_note_ 前缀（旧白板功能），部分用 notes_ 前缀。

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `String` | 39 |
| `State<AppState>` | 39 |
| `Option<String>` | 9 |
| `Window` | 8 |
| `Option<i64>` | 3 |
| `State<VfsLanceStore>` | 2 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `Result<bool>` | 7 |
| `Result<usize>` | 5 |
| `Result<crate::notes_manager::NoteItem>` | 4 |
| `Result<()>` | 2 |
| `Result<String>` | 2 |
| `Result<Vec<String>>` | 2 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `canvas_note_append` | *(保持)* | → Input struct | — |
| `canvas_note_read` | *(保持)* | — | — |
| `canvas_note_replace` | *(保持)* | → Input struct | — |
| `canvas_note_set` | *(保持)* | — | — |
| `notes_assets_bulk_delete` | *(保持)* | — | — |
| `notes_assets_index_scan` | *(保持)* | — | — |
| `notes_assets_scan_orphans` | *(保持)* | — | — |
| `notes_create` | *(保持)* | → Input struct | — |
| `notes_db_stats` | *(保持)* | — | — |
| `notes_db_vacuum` | *(保持)* | — | — |
| `notes_delete` | *(保持)* | — | — |
| `notes_delete_asset` | *(保持)* | — | — |
| `notes_empty_trash` | *(保持)* | — | — |
| `notes_export` | *(保持)* | — | — |
| `notes_export_single` | *(保持)* | — | — |
| `notes_get` | *(保持)* | — | — |
| `notes_get_pref` | *(保持)* | — | — |
| `notes_get_subject_rag_config` | *(保持)* | — | — |
| `notes_hard_delete` | *(保持)* | → Input struct | — |
| `notes_import` | *(保持)* | — | — |
| `notes_import_markdown` | *(保持)* | — | — |
| `notes_import_markdown_batch` | *(保持)* | — | — |
| `notes_list` | *(保持)* | — | — |
| `notes_list_advanced` | *(保持)* | — | — |
| `notes_list_assets` | *(保持)* | — | — |
| `notes_list_deleted` | *(保持)* | → Input struct | — |
| `notes_list_meta` | *(保持)* | — | — |
| `notes_list_tags` | *(保持)* | — | — |
| `notes_mentions_search` | *(保持)* | → Input struct | — |
| `notes_rag_rebuild_fts_index` | *(保持)* | — | — |
| `notes_resolve_asset_path` | *(保持)* | — | — |
| `notes_restore` | *(保持)* | → Input struct | — |
| `notes_save_asset` | *(保持)* | → Input struct | — |
| `notes_search` | *(保持)* | → Input struct | — |
| `notes_set_favorite` | *(保持)* | → Input struct | — |
| `notes_set_pref` | *(保持)* | — | — |
| `notes_update` | *(保持)* | → Input struct | — |
| `notes_update_subject_rag_config` | *(保持)* | — | — |
| `rag_rebuild_fts_index` | *(保持)* | — | — |

## 改进操作

统一为 notes_ 前缀，移除废弃的 canvas_note_ 命令

## 统一错误类型

`NotesError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/cmd__notes.json`*
