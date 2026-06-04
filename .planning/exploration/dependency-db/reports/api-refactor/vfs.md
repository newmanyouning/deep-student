# API 重构: VFS — 虚拟文件系统

**日期**: 2026-05-29 | **命令数**: 119 | **对应诊断**: round-20~26

---

## 当前问题

119 个命令混合了 5 种不同的职责域：文件CRUD、PDF处理、番茄钟(Pomodoro)、待办(Todo)、语音输入(Voice)。Pomodoro/Todo/Voice 不应属于 VFS 模块。

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `State<VfsDatabase>` | 87 |
| `String` | 66 |
| `AppHandle` | 28 |
| `State<vfs::lance_store::VfsLanceStore>` | 13 |
| `Option<String>` | 12 |
| `State<llm_manager::LLMManager>` | 8 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `()` | 15 |
| `bool` | 7 |
| `Vec<VfsTodoItem>` | 6 |
| `VfsTodoList` | 4 |
| `String` | 4 |
| `Option<String>` | 4 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `pomodoro_create_record` | *(保持)* | — | Result<T, VfsError> |
| `pomodoro_get_record` | *(保持)* | — | Result<T, VfsError> |
| `pomodoro_list_by_todo` | *(保持)* | — | Result<T, VfsError> |
| `pomodoro_list_today` | *(保持)* | — | Result<T, VfsError> |
| `pomodoro_today_stats` | *(保持)* | — | Result<T, VfsError> |
| `todo_create_item` | *(保持)* | — | Result<T, VfsError> |
| `todo_create_list` | *(保持)* | — | Result<T, VfsError> |
| `todo_delete_item` | *(保持)* | — | Result<T, VfsError> |
| `todo_delete_list` | *(保持)* | — | Result<T, VfsError> |
| `todo_ensure_inbox` | *(保持)* | — | Result<T, VfsError> |
| `todo_get_active_summary` | *(保持)* | — | Result<T, VfsError> |
| `todo_get_item` | *(保持)* | — | Result<T, VfsError> |
| `todo_get_list` | *(保持)* | — | Result<T, VfsError> |
| `todo_list_completed` | *(保持)* | — | Result<T, VfsError> |
| `todo_list_items` | *(保持)* | — | Result<T, VfsError> |
| `todo_list_lists` | *(保持)* | — | Result<T, VfsError> |
| `todo_list_overdue` | *(保持)* | — | Result<T, VfsError> |
| `todo_list_today` | *(保持)* | — | Result<T, VfsError> |
| `todo_list_upcoming` | *(保持)* | — | Result<T, VfsError> |
| `todo_reorder_items` | *(保持)* | — | Result<T, VfsError> |
| `todo_search` | *(保持)* | — | Result<T, VfsError> |
| `todo_toggle_item` | *(保持)* | — | Result<T, VfsError> |
| `todo_toggle_list_favorite` | *(保持)* | — | Result<T, VfsError> |
| `todo_update_item` | *(保持)* | — | Result<T, VfsError> |
| `todo_update_list` | *(保持)* | — | Result<T, VfsError> |
| `vfs_assign_dimension_model` | *(保持)* | → Input struct | Result<T, VfsError> |
| `vfs_batch_index_pending` | *(保持)* | → Input struct | Result<T, VfsError> |
| `vfs_cancel_pdf_processing` | *(保持)* | — | Result<T, VfsError> |
| `vfs_clear_default_embedding_dimension` | *(保持)* | — | Result<T, VfsError> |
| `vfs_clear_media_cache` | *(保持)* | — | Result<T, VfsError> |
| `vfs_clear_resource_ocr` | *(保持)* | — | Result<T, VfsError> |
| `vfs_create_attachment_root_folder` | *(保持)* | — | Result<T, VfsError> |
| `vfs_create_dimension` | *(保持)* | → Input struct | Result<T, VfsError> |
| `vfs_create_mindmap` | *(保持)* | — | Result<T, VfsError> |
| `vfs_create_note` | *(保持)* | — | Result<T, VfsError> |
| `vfs_create_or_reuse` | *(保持)* | — | Result<T, VfsError> |
| `vfs_debug_index_status` | *(保持)* | — | Result<T, VfsError> |
| `vfs_decrement_ref` | *(保持)* | — | Result<T, VfsError> |
| `vfs_delete_attachment` | *(保持)* | — | Result<T, VfsError> |
| `vfs_delete_dimension` | *(保持)* | → Input struct | Result<T, VfsError> |
| `vfs_delete_file` | *(保持)* | — | Result<T, VfsError> |
| `vfs_delete_mindmap` | *(保持)* | — | Result<T, VfsError> |
| `vfs_delete_note` | *(保持)* | — | Result<T, VfsError> |
| `vfs_delete_resource_index` | *(保持)* | — | Result<T, VfsError> |
| `vfs_diagnose_lance_schema` | *(保持)* | — | Result<T, VfsError> |
| `vfs_download_paper` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_all_index_status` | *(保持)* | → Input struct | Result<T, VfsError> |
| `vfs_get_attachment` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_attachment_config` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_attachment_content` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_blob_base64` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_default_embedding_dimension` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_dimension_range` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_embedding_stats` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_file` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_file_content` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_index_status` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_indexing_config` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_lance_stats` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_media_cache_stats` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_mindmap` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_mindmap_content` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_mindmap_version` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_mindmap_version_content` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_mindmap_versions` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_note` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_note_content` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_or_create_attachment_root_folder` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_pdf_page_image` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_pdf_processing_status` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_pending_resources` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_preset_dimensions` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_resource` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_resource_ocr_info` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_resource_path` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_resource_ref_count` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_resource_refs` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_resource_text_chunks` | *(保持)* | — | Result<T, VfsError> |
| `vfs_get_resource_units` | *(保持)* | — | Result<T, VfsError> |
| `vfs_increment_ref` | *(保持)* | — | Result<T, VfsError> |
| `vfs_list_dimensions` | *(保持)* | — | Result<T, VfsError> |
| `vfs_list_embedding_dims` | *(保持)* | — | Result<T, VfsError> |
| `vfs_list_essays` | *(保持)* | — | Result<T, VfsError> |
| `vfs_list_exam_sheets` | *(保持)* | — | Result<T, VfsError> |
| `vfs_list_files` | *(保持)* | → Input struct | Result<T, VfsError> |
| `vfs_list_mindmaps` | *(保持)* | — | Result<T, VfsError> |
| `vfs_list_notes` | *(保持)* | — | Result<T, VfsError> |
| `vfs_list_pending_pdf_processing` | *(保持)* | — | Result<T, VfsError> |
| `vfs_list_textbooks` | *(保持)* | — | Result<T, VfsError> |
| `vfs_list_translations` | *(保持)* | — | Result<T, VfsError> |
| `vfs_multimodal_delete` | *(保持)* | → Input struct | Result<T, VfsError> |
| `vfs_multimodal_index` | *(保持)* | → Input struct | Result<T, VfsError> |
| `vfs_multimodal_search` | *(保持)* | → Input struct | Result<T, VfsError> |
| `vfs_multimodal_stats` | *(保持)* | → Context struct | Result<T, VfsError> |
| `vfs_optimize_lance` | *(保持)* | — | Result<T, VfsError> |
| `vfs_rag_search` | *(保持)* | → Input struct | Result<T, VfsError> |
| `vfs_reindex_resource` | *(保持)* | → Input struct | Result<T, VfsError> |
| `vfs_reindex_unit` | *(保持)* | → Input struct | Result<T, VfsError> |
| `vfs_reset_all_index_state` | *(保持)* | — | Result<T, VfsError> |
| `vfs_reset_disabled_to_pending` | *(保持)* | — | Result<T, VfsError> |
| `vfs_reset_indexed_without_embeddings` | *(保持)* | — | Result<T, VfsError> |
| `vfs_resolve_resource_refs` | *(保持)* | — | Result<T, VfsError> |
| `vfs_resource_exists` | *(保持)* | — | Result<T, VfsError> |
| `vfs_retry_pdf_processing` | *(保持)* | — | Result<T, VfsError> |
| `vfs_search` | *(保持)* | — | Result<T, VfsError> |
| `vfs_search_all` | *(保持)* | — | Result<T, VfsError> |
| `vfs_set_attachment_root_folder` | *(保持)* | — | Result<T, VfsError> |
| `vfs_set_default_embedding_dimension` | *(保持)* | → Input struct | Result<T, VfsError> |
| `vfs_set_indexing_config` | *(保持)* | — | Result<T, VfsError> |
| `vfs_set_mindmap_favorite` | *(保持)* | — | Result<T, VfsError> |
| `vfs_start_pdf_processing` | *(保持)* | — | Result<T, VfsError> |
| `vfs_toggle_index_disabled` | *(保持)* | — | Result<T, VfsError> |
| `vfs_unified_batch_index` | *(保持)* | → Input struct | Result<T, VfsError> |
| `vfs_unified_index_status` | *(保持)* | — | Result<T, VfsError> |
| `vfs_update_mindmap` | *(保持)* | — | Result<T, VfsError> |
| `vfs_update_note` | *(保持)* | — | Result<T, VfsError> |
| `vfs_update_path_cache` | *(保持)* | — | Result<T, VfsError> |
| `vfs_upload_attachment` | *(保持)* | — | Result<T, VfsError> |
| `vfs_upload_file` | *(保持)* | → Input struct | Result<T, VfsError> |

## 改进操作

拆分为 VFS Core + Pomodoro + Todo + Voice Input 四个独立模块

## 统一错误类型

`VfsError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/vfs.json`*
