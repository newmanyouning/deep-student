# Tauri Command Registry — Mapping Report

> Generated: 2026-06-01
> Method: Static analysis of `lib.rs` (invoke_handler), Rust source implementations, and TypeScript `invoke()` calls.

## Overview

| Metric | Count |
|--------|-------|
| Rust commands registered in `invoke_handler` | **682** |
| TypeScript `invoke()` call sites (unique command names) | **193** |
| Commands called from TS but NOT registered (NOR implemented) | **45+** |
| Commands implemented in Rust (with `#[tauri::command]`) but NOT registered | **2** |

---

## 1. All Registered Commands by Module

### 1.1 `commands` (legacy module: `src-tauri/src/commands.rs`) — 248 commands

The `commands` module serves as the main hub and re-exports from sub-modules via `pub use crate::cmd::*;`.

**PDF OCR** (`commands.rs`):
`test_pdfium_status`, `get_app_version`, `get_app_data_dir`, `process_pdf_ocr`, `init_pdf_ocr_session`, `upload_pdf_ocr_page`, `cancel_pdf_ocr_session`, `pause_pdf_ocr_session`, `resume_pdf_ocr_session`, `skip_pdf_ocr_page`, `start_pdf_ocr_backend`, `get_pdf_ocr_temp_dir`, `save_pdf_to_temp`

**Exam Sheet**:
`list_exam_sheet_sessions`, `get_exam_sheet_session_detail`, `update_exam_sheet_cards`, `rename_exam_sheet_session`, `inspect_pdf_text_for_qbank`

**Question Bank Import**:
`import_question_bank`, `import_question_bank_stream`, `resume_question_import`, `list_importing_sessions`

**Question Bank Images**:
`qbank_get_source_images`, `qbank_crop_source_image`, `qbank_remove_question_image`

**CSV Import/Export**:
`import_questions_csv`, `export_questions_csv`, `get_csv_preview`, `get_csv_exportable_fields`

**Images**:
`pin_images`, `unpin_images`

**Statistics**:
`get_enhanced_statistics`

**Settings**:
`save_setting`, `get_setting`, `delete_setting`, `get_settings_by_prefix`, `delete_settings_by_prefix`

**Voice Input** (re-exported from `voice_input`): `voice_input_transcribe`

**Debug Logs**:
`get_debug_logs_info`, `clear_debug_logs`, `cleanup_old_debug_logs`, `ensure_debug_log_dir`, `read_debug_log_file`

**Security**:
`get_security_status`, `get_cn_whitelist_config`, `detect_tool_conflicts`

**Provider/Model Config**:
`get_tools_namespace_config`, `get_provider_strategies_config`, `save_provider_strategies_config`, `get_feature_flags`, `update_feature_flag`, `is_feature_enabled`, `get_injection_budget_config`, `simulate_budget_allocation`, `test_search_engine`, `get_image_as_base64`, `get_api_configurations`, `save_api_configurations`, `get_model_assignments`, `save_model_assignments`, `get_vendor_configs`, `save_vendor_configs`, `get_model_profiles`, `save_model_profiles`, `test_api_connection`, `get_model_adapter_options`, `save_model_adapter_options`, `reset_model_adapter_options`, `estimate_tokens`

**OCR Engine Config**:
`get_ocr_engines`, `get_ocr_engine_type`, `set_ocr_engine_type`, `get_ocr_thinking_enabled`, `set_ocr_thinking_enabled`, `infer_ocr_engine_from_model`, `validate_ocr_model`, `get_ocr_prompt_template`, `get_available_ocr_models`, `save_available_ocr_models`, `test_ocr_engine`, `update_ocr_engine_priority`, `add_ocr_engine`, `remove_ocr_engine`

**Lance/Vector**:
`optimize_chat_embeddings_table`, `create_performance_indexes`, `analyze_query_performance`, `clear_message_embeddings`, `optimize_lance_database`

**Anki Card Generation**:
`generate_anki_cards_from_document`, `generate_anki_cards_from_document_file`, `generate_anki_cards_from_document_base64`, `call_llm_for_boundary`

**AnkiConnect** (re-export from `cmd::anki_connect`):
`anki_connect_check_status`, `anki_connect_get_deck_names`, `anki_connect_get_model_names`, `anki_connect_create_deck`, `anki_connect_save_cards`, `anki_connect_add_cards`, `anki_connect_import_package`, `anki_connect_export_apkg`, `anki_connect_export_apkg_with_template`, `anki_connect_export_multi_apkg`, `anki_connect_batch_export_cards`, `anki_connect_save_json_file`

**Legacy**:
`anki_get_deck_names`

**Enhanced Anki** (re-export from `cmd::enhanced_anki`):
`enhanced_anki_start_document_processing`, `enhanced_anki_pause_document_processing`, `enhanced_anki_resume_document_processing`, `enhanced_anki_get_document_processing_state`, `enhanced_anki_get_document_task_counts`, `enhanced_anki_trigger_task_processing`, `enhanced_anki_get_document_tasks`, `enhanced_anki_get_task_cards`, `enhanced_anki_update_card`, `enhanced_anki_delete_card`, `enhanced_anki_delete_document_task`, `enhanced_anki_delete_document_session`, `enhanced_anki_export_apkg_for_selection`, `enhanced_anki_get_document_cards`, `enhanced_anki_list_library_cards`, `enhanced_anki_export_cards`, `enhanced_anki_recover_stuck_tasks`, `enhanced_anki_list_document_sessions`, `enhanced_anki_get_stats`

**State Recovery**:
`get_recent_document_tasks`, `get_all_recent_cards`, `enhanced_anki_get_pending_memory_candidates`, `enhanced_anki_dismiss_pending_memory_candidates`, `enhanced_anki_mark_pending_memory_candidates_saved`

**Document Parsing**:
`parse_document_from_path`, `parse_document_from_base64`

**Translation** (re-export from `cmd::translation`):
`translate_text_stream`, `stream_chat_translation_aligned`, `stream_chat_translation_plain`

**OCR**:
`ocr_extract_text`

**File operations**:
`read_file_text`, `get_file_size`, `hash_file`, `read_file_bytes`, `copy_file`, `save_text_to_file`

**Templates**:
`get_all_custom_templates`, `get_custom_template_by_id`, `create_custom_template`, `update_custom_template`, `delete_custom_template`, `export_template`, `import_template`, `import_custom_templates_bulk`, `import_builtin_templates`, `set_default_template`, `get_default_template_id`

**Test/Log**:
`save_test_log`, `get_test_logs`, `open_log_file`, `open_logs_folder`, `report_frontend_log`, `save_template_debug_data`, `export_unified_backup_data`

**MCP** (re-export from `cmd::mcp`):
`get_mcp_status`, `get_mcp_tools`, `test_mcp_connection`, `test_mcp_websocket`, `test_mcp_sse`, `test_mcp_http`, `mcp_stdio_start`, `mcp_stdio_send`, `mcp_stdio_close`, `save_mcp_config`, `reload_mcp_client`, `get_mcp_config`, `import_mcp_config`, `export_mcp_config`, `test_all_search_engines`

**Cancel**:
`cancel_stream`

**Notes** (re-export from `cmd::notes`):
`notes_list`, `notes_list_meta`, `notes_create`, `notes_update`, `notes_set_favorite`, `notes_delete`, `notes_get`, `notes_save_asset`, `notes_list_assets`, `notes_delete_asset`, `notes_resolve_asset_path`, `notes_restore`, `notes_assets_index_scan`, `notes_assets_scan_orphans`, `notes_assets_bulk_delete`, `notes_list_advanced`, `notes_get_subject_rag_config`, `notes_update_subject_rag_config`, `notes_set_pref`, `notes_get_pref`, `notes_export`, `notes_export_single`, `notes_import`, `notes_import_markdown`, `notes_import_markdown_batch`, `notes_db_stats`, `notes_db_vacuum`, `notes_list_tags`, `notes_search`, `notes_mentions_search`, `rag_rebuild_fts_index`, `notes_rag_rebuild_fts_index`, `notes_hard_delete`, `notes_empty_trash`, `notes_list_deleted`

**Canvas**:
`canvas_note_read`, `canvas_note_append`, `canvas_note_replace`, `canvas_note_set`

**Package Manager**:
`check_package_manager`, `auto_install_package_manager`, `check_all_package_managers`

**Test Database**:
`switch_to_test_database`, `reset_test_database`, `switch_to_production_database`, `get_database_info`, `seed_test_database`, `check_test_dependencies`, `set_test_run_id`, `write_test_report`

**WebView Settings**:
`save_webview_settings`, `load_webview_settings`

**Textbooks** (re-export from `cmd::textbooks`):
`textbooks_add`, `textbooks_update_bookmarks`

**Question Bank V2**:
`qbank_list_questions`, `qbank_search_questions`, `qbank_rebuild_fts_index`, `qbank_get_question`, `qbank_get_question_by_card_id`, `qbank_create_question`, `qbank_batch_create_questions`, `qbank_update_question`, `qbank_batch_update_questions`, `qbank_delete_question`, `qbank_batch_delete_questions`, `qbank_submit_answer`, `qbank_toggle_favorite`, `qbank_get_stats`, `qbank_refresh_stats`, `qbank_get_history`, `qbank_get_submissions`, `qbank_reset_progress`, `qbank_reset_questions_progress`

**Learning Trends**:
`qbank_get_learning_trend`, `qbank_get_activity_heatmap`, `qbank_get_knowledge_stats`, `qbank_get_knowledge_stats_with_comparison`

**Practice Modes**:
`qbank_start_timed_practice`, `qbank_generate_mock_exam`, `qbank_submit_mock_exam`, `qbank_get_daily_practice`, `qbank_generate_paper`, `qbank_get_check_in_calendar`

**Heatmap**:
`get_learning_heatmap`

### 1.2 VFS (`vfs`) — 127 commands

**Resource operations** (`vfs/handlers.rs`):
`vfs_create_or_reuse`, `vfs_get_resource`, `vfs_resource_exists`, `vfs_increment_ref`, `vfs_decrement_ref`

**Note operations**:
`vfs_create_note`, `vfs_update_note`, `vfs_get_note`, `vfs_get_note_content`, `vfs_list_notes`, `vfs_delete_note`

**List operations**:
`vfs_list_textbooks`, `vfs_list_exam_sheets`, `vfs_list_translations`, `vfs_list_essays`, `vfs_search_all`

**Path cache**:
`vfs_get_resource_path`, `vfs_update_path_cache`

**Reference handlers** (`vfs/ref_handlers.rs`):
`vfs_get_resource_refs`, `vfs_resolve_resource_refs`, `vfs_get_resource_ref_count`

**Attachment operations**:
`vfs_upload_attachment`, `vfs_get_attachment_content`, `vfs_get_attachment`, `vfs_delete_attachment`, `vfs_get_attachment_config`, `vfs_set_attachment_root_folder`, `vfs_create_attachment_root_folder`, `vfs_get_or_create_attachment_root_folder`

**File operations**:
`vfs_upload_file`, `vfs_download_paper`, `vfs_get_file`, `vfs_list_files`, `vfs_delete_file`, `vfs_get_file_content`

**Blob**:
`vfs_get_blob_base64`

**PDF page image**:
`vfs_get_pdf_page_image`

**PDF processing pipeline**:
`vfs_get_pdf_processing_status`, `vfs_cancel_pdf_processing`, `vfs_retry_pdf_processing`, `vfs_start_pdf_processing`, `vfs_get_batch_pdf_processing_status`, `vfs_list_pending_pdf_processing`

**Media cache**:
`vfs_get_media_cache_stats`, `vfs_clear_media_cache`

**Unified knowledge management**:
`vfs_search`, `vfs_reindex_resource`, `vfs_get_index_status`, `vfs_toggle_index_disabled`, `vfs_get_embedding_stats`, `vfs_list_dimensions`, `vfs_assign_dimension_model`, `vfs_create_dimension`, `vfs_delete_dimension`, `vfs_get_preset_dimensions`, `vfs_get_dimension_range`, `vfs_set_default_embedding_dimension`, `vfs_get_default_embedding_dimension`, `vfs_clear_default_embedding_dimension`, `vfs_get_pending_resources`, `vfs_batch_index_pending`, `vfs_set_indexing_config`, `vfs_get_indexing_config`, `vfs_get_all_index_status`

**Index handlers** (`vfs/index_handlers.rs`):
`vfs_unified_index_status`, `vfs_get_resource_units`, `vfs_reindex_unit`, `vfs_unified_batch_index`, `vfs_sync_resource_units`, `vfs_delete_resource_index`, `vfs_list_embedding_dims`

**OCR data**:
`vfs_get_resource_ocr_info`, `vfs_clear_resource_ocr`, `vfs_get_resource_text_chunks`

**RAG**:
`vfs_rag_search`, `vfs_get_lance_stats`, `vfs_optimize_lance`

**Multimodal**:
`vfs_multimodal_index`, `vfs_multimodal_search`, `vfs_multimodal_stats`, `vfs_multimodal_delete`, `vfs_multimodal_index_resource`

**Mindmaps**:
`vfs_create_mindmap`, `vfs_get_mindmap`, `vfs_get_mindmap_content`, `vfs_get_mindmap_versions`, `vfs_get_mindmap_version_content`, `vfs_get_mindmap_version`, `vfs_update_mindmap`, `vfs_delete_mindmap`, `vfs_list_mindmaps`, `vfs_set_mindmap_favorite`

**Todo handlers** (`vfs/todo_handlers.rs`):
`vfs_todo_create_list`, `vfs_todo_get_list`, `vfs_todo_list_lists`, `vfs_todo_update_list`, `vfs_todo_delete_list`, `vfs_todo_toggle_list_favorite`, `vfs_todo_ensure_inbox`, `vfs_todo_create_item`, `vfs_todo_get_item`, `vfs_todo_list_items`, `vfs_todo_update_item`, `vfs_todo_toggle_item`, `vfs_todo_delete_item`, `vfs_todo_reorder_items`, `vfs_todo_list_today`, `vfs_todo_list_overdue`, `vfs_todo_list_upcoming`, `vfs_todo_list_completed`, `vfs_todo_search`, `vfs_todo_get_active_summary`

**Pomodoro**:
`vfs_pomodoro_create_record`, `vfs_pomodoro_get_record`, `vfs_pomodoro_list_by_todo`, `vfs_pomodoro_today_stats`, `vfs_pomodoro_list_today`

**OCR storage** (`vfs/ocr_storage_handlers.rs`):
`vfs_ocr_store_result`, `vfs_ocr_list_results`, `vfs_ocr_delete_result`, `vfs_ocr_mark_exported`, `vfs_ocr_list_for_export`

**Index diagnosis**:
`vfs_debug_index_status`, `vfs_reset_disabled_to_pending`, `vfs_reset_indexed_without_embeddings`, `vfs_reset_all_index_state`, `vfs_diagnose_lance_schema`

### 1.3 Chat V2 (`chat_v2`) — 77 commands

**Send message** (`chat_v2/handlers/send_message.rs`):
`chat_v2_send_message`, `chat_v2_cancel_stream`, `chat_v2_retry_message`, `chat_v2_edit_and_resend`, `chat_v2_continue_message`

**Load session** (`chat_v2/handlers/load_session.rs`):
`chat_v2_load_session`

**Manage session** (`chat_v2/handlers/manage_session.rs`):
`chat_v2_create_session`, `chat_v2_get_session`, `chat_v2_update_session_settings`, `chat_v2_archive_session`, `chat_v2_save_session`, `chat_v2_list_sessions`, `chat_v2_list_agent_sessions`, `chat_v2_count_sessions`, `chat_v2_session_message_count`, `chat_v2_delete_session`, `chat_v2_empty_deleted_sessions`, `chat_v2_soft_delete_session`, `chat_v2_restore_session`, `chat_v2_branch_session`

**Block actions** (`chat_v2/handlers/block_actions.rs`):
`chat_v2_delete_message`, `chat_v2_copy_block_content`, `chat_v2_update_block_content`, `chat_v2_update_block_tool_output`, `chat_v2_get_anki_cards_from_block_by_document_id`, `chat_v2_upsert_streaming_block`, `chat_v2_anki_cards_result`

**Groups** (`chat_v2/handlers/group_handlers.rs`):
`chat_v2_create_group`, `chat_v2_update_group`, `chat_v2_delete_group`, `chat_v2_get_group`, `chat_v2_list_groups`, `chat_v2_reorder_groups`, `chat_v2_move_session_to_group`

**OCR** (`chat_v2/handlers/ocr.rs`):
`chat_v2_perform_ocr`

**Variants** (`chat_v2/handlers/variant_handlers.rs`):
`chat_v2_switch_variant`, `chat_v2_delete_variant`, `chat_v2_retry_variant`, `chat_v2_retry_variants`, `chat_v2_cancel_variant`

**Approval** (`chat_v2/handlers/approval_handlers.rs`):
`chat_v2_tool_approval_respond`, `chat_v2_tool_approval_cancel`, `chat_v2_clear_approval_history`

**Ask user** (`chat_v2/handlers/ask_user_handlers.rs`):
`chat_v2_ask_user_respond`

**Canvas** (`chat_v2/handlers/canvas_handlers.rs`):
`chat_v2_canvas_edit_result`

**Migration** (`chat_v2/handlers/migration.rs`):
`chat_v2_check_migration_status`, `chat_v2_migrate_legacy_chat`, `chat_v2_rollback_migration`

**Search & Tags** (`chat_v2/handlers/search_handlers.rs`):
`chat_v2_search_content`, `chat_v2_get_session_tags`, `chat_v2_get_tags_batch`, `chat_v2_add_tag`, `chat_v2_remove_tag`, `chat_v2_list_all_tags`

**Workspace** (`chat_v2/handlers/workspace_handlers.rs`):
`chat_v2_workspace_create`, `chat_v2_workspace_get`, `chat_v2_workspace_close`, `chat_v2_workspace_delete`, `chat_v2_workspace_create_agent`, `chat_v2_workspace_list_agents`, `chat_v2_workspace_send_message`, `chat_v2_workspace_list_messages`, `chat_v2_workspace_set_context`, `chat_v2_workspace_get_context`, `chat_v2_workspace_list_documents`, `chat_v2_workspace_get_document`, `chat_v2_workspace_list_all`, `chat_v2_workspace_run_agent`, `chat_v2_workspace_cancel_agent`, `chat_v2_workspace_manual_wake`, `chat_v2_workspace_cancel_sleep`, `chat_v2_workspace_restore_executions`

**Skills** (`chat_v2/skills.rs`):
`skill_list_directories`, `skill_read_file`, `skill_create`, `skill_update`, `skill_delete`

### 1.4 DSTU (`dstu`) — 54 commands

**Core** (`dstu/handlers.rs`):
`dstu_list`, `dstu_get`, `dstu_create`, `dstu_update`, `dstu_delete`, `dstu_restore`, `dstu_purge`, `dstu_set_favorite`, `dstu_list_deleted`, `dstu_purge_all`, `dstu_move`, `dstu_rename`, `dstu_copy`, `dstu_search`, `dstu_get_content`, `dstu_set_metadata`, `dstu_watch`, `dstu_unwatch`, `dstu_delete_many`, `dstu_restore_many`, `dstu_move_many`, `dstu_search_in_folder`, `dstu_get_exam_content`

**Real path**:
`dstu_parse_path`, `dstu_build_path`, `dstu_get_resource_location`, `dstu_get_resource_by_path`, `dstu_move_to_folder`, `dstu_batch_move`, `dstu_refresh_path_cache`, `dstu_get_path_by_id`

**Export** (`dstu/export.rs`):
`dstu_export_formats`, `dstu_export`

**Folder** (`dstu/folder_handlers.rs`):
`dstu_folder_create`, `dstu_folder_get`, `dstu_folder_rename`, `dstu_folder_delete`, `dstu_folder_move`, `dstu_folder_set_expanded`, `dstu_folder_add_item`, `dstu_folder_remove_item`, `dstu_folder_move_item`, `dstu_folder_list`, `dstu_folder_get_tree`, `dstu_folder_get_items`, `dstu_folder_get_all_resources`, `dstu_folder_reorder`, `dstu_folder_reorder_items`, `dstu_folder_get_breadcrumbs`

**Trash** (`dstu/trash_handlers.rs`):
`dstu_soft_delete`, `dstu_trash_restore`, `dstu_list_trash`, `dstu_empty_trash`, `dstu_permanently_delete`

### 1.5 Data Governance (`data_governance`) — 45 commands

**Core** (`data_governance/commands.rs`):
`data_governance_get_maintenance_status`, `data_governance_get_schema_registry`, `data_governance_get_migration_status`, `data_governance_get_database_status`, `data_governance_run_health_check`, `data_governance_get_audit_logs`, `data_governance_cleanup_audit_logs`, `data_governance_get_migration_diagnostic_report`, `data_governance_run_slot_c_empty_db_test`, `data_governance_run_slot_d_clone_db_test`

**Backup** (`data_governance/commands_backup.rs`):
`data_governance_run_backup`, `data_governance_cancel_backup`, `data_governance_get_backup_job`, `data_governance_list_backup_jobs`, `data_governance_get_backup_list`, `data_governance_delete_backup`, `data_governance_check_disk_space_for_restore`, `data_governance_verify_backup`, `data_governance_auto_verify_latest_backup`, `data_governance_backup_tiered`, `data_governance_resume_backup_job`, `data_governance_list_resumable_jobs`, `data_governance_cleanup_persisted_jobs`

**ZIP** (`data_governance/commands_zip.rs`):
`data_governance_backup_and_export_zip`, `data_governance_export_zip`, `data_governance_import_zip`

**Restore** (`data_governance/commands_restore.rs`):
`data_governance_restore_backup`

**Sync** (`data_governance/commands_sync.rs`):
`data_governance_get_sync_status`, `data_governance_detect_conflicts`, `data_governance_resolve_conflicts`, `data_governance_run_sync`, `data_governance_run_sync_with_progress`, `data_governance_export_sync_data`, `data_governance_import_sync_data`, `data_governance_mark_blob_deleted`, `data_governance_mark_asset_deleted`, `data_governance_list_record_conflicts`, `data_governance_count_record_conflicts`, `data_governance_resolve_record_conflict`, `data_governance_purge_resolved_conflicts`, `data_governance_detect_prune_gap`

**Asset** (`data_governance/commands_asset.rs`):
`data_governance_scan_assets`, `data_governance_get_asset_types`, `data_governance_restore_with_assets`, `data_governance_verify_backup_with_assets`

### 1.6 Memory (`memory`) — 29 commands

**Core** (`memory/handlers.rs`):
`memory_get_config`, `memory_set_root_folder`, `memory_set_privacy_mode`, `memory_create_root_folder`, `memory_get_or_create_root_folder`, `memory_search`, `memory_read`, `memory_write`, `memory_list`, `memory_get_tree`, `memory_update_by_id`, `memory_delete`, `memory_move_to_folder`, `memory_batch_delete`, `memory_batch_move`, `memory_update_tags`, `memory_get_tags`, `memory_add_relation`, `memory_remove_relation`, `memory_get_related`, `memory_to_anki_document`, `memory_write_smart`, `memory_write_batch`, `memory_set_auto_create_subfolders`, `memory_set_default_category`, `memory_set_auto_extract_frequency`, `memory_export_all`, `memory_get_profile`, `memory_get_audit_logs`

### 1.7 Essay Grading — 20 commands

(`essay_grading/`):
`essay_grading_stream`, `essay_grading_create_session`, `essay_grading_get_session`, `essay_grading_update_session`, `essay_grading_delete_session`, `essay_grading_list_sessions`, `essay_grading_toggle_favorite`, `essay_grading_get_rounds`, `essay_grading_get_round`, `essay_grading_get_latest_round_number`, `essay_grading_get_modes`, `essay_grading_get_mode`, `essay_grading_get_models`, `essay_grading_create_custom_mode`, `essay_grading_update_custom_mode`, `essay_grading_delete_custom_mode`, `essay_grading_list_custom_modes`, `essay_grading_save_builtin_override`, `essay_grading_reset_builtin_mode`, `essay_grading_has_builtin_override`

### 1.8 Review Plan — 17 commands

(`review_plan_service.rs`):
`review_plan_create`, `review_plan_process`, `review_plan_get_due`, `review_plan_get_due_with_filter`, `review_plan_get_stats`, `review_plan_refresh_stats`, `review_plan_get_by_question`, `review_plan_get`, `review_plan_suspend`, `review_plan_resume`, `review_plan_delete`, `review_plan_get_history`, `review_plan_batch_create`, `review_plan_create_for_exam`, `review_plan_list_by_exam`, `review_plan_get_or_create`, `review_plan_get_calendar_data`

### 1.9 Cloud Storage — 14 commands

**Cloud storage ops** (`cloud_storage.rs`):
`cloud_storage_check_connection`, `cloud_storage_put`, `cloud_storage_get`, `cloud_storage_list`, `cloud_storage_delete`, `cloud_storage_stat`, `cloud_storage_exists`

**Cloud sync manager**:
`cloud_sync_get_status`, `cloud_sync_list_versions`, `cloud_sync_upload`, `cloud_sync_download`, `cloud_sync_delete_version`, `cloud_sync_get_device_id`, `cloud_storage_is_s3_enabled`

### 1.10 LLM Usage — 7 commands

(`llm_usage/handlers.rs`):
`llm_usage_get_trends`, `llm_usage_by_model`, `llm_usage_by_caller`, `llm_usage_summary`, `llm_usage_recent`, `llm_usage_daily`, `llm_usage_cleanup`

### 1.11 Question Sync — 6 commands

(`question_sync_service.rs`):
`qbank_sync_check`, `qbank_get_sync_conflicts`, `qbank_resolve_sync_conflict`, `qbank_batch_resolve_conflicts`, `qbank_set_sync_enabled`, `qbank_update_sync_config`

### 1.12 Data Space — 6 commands

(`data_space.rs`):
`get_data_space_info`, `mark_data_space_pending_switch_to_inactive`, `get_test_slot_info`, `clear_test_slots`, `get_slot_directory`, `restart_app`

### 1.13 Remaining modules

| Module | Count | Commands |
|--------|-------|----------|
| `backup_config` | 5 | `get_backup_config`, `set_backup_config`, `pick_backup_directory`, `clear_backup_directory`, `get_default_backup_directory` |
| `secure_store` | 4 | `secure_save_cloud_credentials`, `secure_get_cloud_credentials`, `secure_delete_cloud_credentials`, `secure_store_is_available` |
| `debug_commands` | 4 | `debug_get_database_stats`, `log_debug_message`, `debug_vfs_migration_status`, `debug_vfs_textbook_pages` |
| `tts` | 3 | `tts_check_available`, `tts_speak`, `tts_stop` |
| `translation` | 3 | `translate_text_stream`, `stream_chat_translation_aligned`, `stream_chat_translation_plain` |
| `qbank_grading` | 2 | `qbank_ai_grade`, `qbank_cancel_grading` |
| `config_recovery` | 2 | `restore_default_api_configs`, `check_api_config_status` |
| `voice_input` | 1 | `voice_input_transcribe` |
| `pdfium_utils` | 1 | `test_pdfium_status` |
| `debug_logger` | 1 | `write_debug_logs` |

---

## 2. Issues

### 2.1 CRITICAL: Commands Called from Frontend but NOT Implemented in Rust

These commands are invoked via `invoke('command_name', ...)` from TypeScript but have NO corresponding Rust implementation at all. Every invocation will fail at runtime with a Tauri "command not found" error.

#### `unified_*` commands (deprecated graph module) — 11 commands

Called from `src/utils/graphApi.ts` and `src/utils/chatApi.ts`.

| Command | TS file | Notes |
|---------|---------|-------|
| `unified_search_cards` | `src/utils/chatApi.ts:22` | Comment says "unified module deprecated" |
| `unified_get_card` | `src/utils/graphApi.ts:631` | |
| `unified_create_tag` | `src/utils/graphApi.ts:530` | |
| `unified_update_card` | `src/utils/graphApi.ts:1293` | |
| `unified_add_card_tag` | `src/utils/graphApi.ts:444` | |
| `unified_remove_card_tag` | `src/utils/graphApi.ts:428` | |
| `unified_outline_update_tag` | `src/utils/graphApi.ts:815,1380,1449` | Called from 3 sites |
| `unified_outline_move_tag` | `src/utils/graphApi.ts:841` | |
| `unified_fix_tag_hierarchy` | `src/utils/graphApi.ts:1468` | |
| `unified_log_metric_event` | `src/utils/graphApi.ts:894` | |
| `unified_track_card_access` | `src/utils/graphApi.ts:227` | |

**Root cause**: The unified graph module was removed from the Rust backend but the frontend code calling it (`src/utils/graphApi.ts`, `src/utils/chatApi.ts`) was not cleaned up.

#### `research_*` commands (missing implementations) — ~22 commands

Called from `src/utils/settingsApi.ts`. Only 4 of ~26 `research_*` commands have Rust implementations.

| Command | TS file | Line |
|---------|---------|------|
| `research_get_round` | `src/utils/settingsApi.ts` | 86 |
| `research_get_round_visual_summary` | `src/utils/settingsApi.ts` | 89 |
| `research_delete_round` | `src/utils/settingsApi.ts` | 92 |
| `research_generate_round_report` | `src/utils/settingsApi.ts` | 96 |
| `research_set_round_note` | `src/utils/settingsApi.ts` | 99 |
| `research_get_round_note` | `src/utils/settingsApi.ts` | 102 |
| `research_get_round_notes` | `src/utils/settingsApi.ts` | 105 |
| `research_generate_session_report` | `src/utils/settingsApi.ts` | 109 |
| `research_get_chunk_text` | `src/utils/settingsApi.ts` | 114 |
| `research_get_chunk_context` | `src/utils/settingsApi.ts` | 119 |
| `research_update_session_options` | `src/utils/settingsApi.ts` | 125 |
| `research_delete_session` | `src/utils/settingsApi.ts` | 131 |
| `research_run_until` | `src/utils/settingsApi.ts` | 140 |
| `research_run_macro` | `src/utils/settingsApi.ts` | 154 |
| `research_run_to_full_coverage` | `src/utils/settingsApi.ts` | 158 |
| `research_audit_user_questions` | `src/utils/settingsApi.ts` | 164 |
| `research_find_similar_questions` | `src/utils/settingsApi.ts` | 168 |
| `research_get_full_chat_history` | `src/utils/settingsApi.ts` | 172 |
| `research_deep_read_by_docs` | `src/utils/settingsApi.ts` | 176 |
| `research_deep_read_by_tag` | `src/utils/settingsApi.ts` | 180 |
| `research_count_tokens` | `src/utils/settingsApi.ts` | 185 |
| `research_get_full_content` | `src/utils/settingsApi.ts` | 189 |
| `research_get_setting` | `src/utils/settingsApi.ts` | 194 |
| `research_set_setting` | `src/utils/settingsApi.ts` | 197 |
| `research_delete_setting` | `src/utils/settingsApi.ts` | 200 |
| `research_list_artifacts` | `src/utils/settingsApi.ts` | 204 |

#### Other missing commands — 9 commands

| Command | TS file | Notes |
|---------|---------|-------|
| `continue_unified_chat_stream` | `src/utils/graphApi.ts:208` | Deprecated graph module |
| `generate_anki_cards_for_segment` | Unknown | No Rust implementation found |
| `search_existing_tags` | Unknown | No Rust implementation found |
| `get_detailed_tag_hierarchy` | Unknown | No Rust implementation found |
| `get_tag_mapping_history` | Unknown | No Rust implementation found |
| `graph_batch_reorder_tags` | Unknown | No Rust implementation found |
| `graph_reorder_tag` | Unknown | No Rust implementation found |
| `update_card_content` | Unknown | No Rust implementation found |
| `vfs_update_resource_hash` | Unknown | No Rust implementation found |
| `plugin` | Unknown | Possibly a grep artifact |

### 2.2 HIGH: Commands Implemented in Rust but NOT Registered in invoke_handler

These functions exist with `#[tauri::command]` and are public, but are NOT listed in the `generate_handler![]` macro in `lib.rs`. The frontend cannot invoke them.

| Command | Location | Called from TS? |
|---------|----------|-----------------|
| `preheat_mcp_tools` | `src-tauri/src/cmd/mcp.rs:349` | Yes — 3 call sites (`preheat_mcp_tools` in TS) |
| `test_rmcp_streamable_http` | `src-tauri/src/cmd/mcp.rs` | No |

### 2.3 MEDIUM: Commands Implemented but NOT Registered (research_* stubs)

These 4 commands exist with `#[tauri::command]` in `commands.rs` but are NOT registered in the `invoke_handler`:

| Command | Location in commands.rs |
|---------|----------------------|
| `research_list_reports` | line 4159 |
| `research_get_report` | line 4170 |
| `research_delete_report` | line 4182 |
| `research_export_all_reports_zip` | line 4196 |

They are called from `src/utils/chatApi.ts` and will fail at runtime. These 4 are a subset of the ~26 `research_*` commands listed above — the other ~22 have no Rust implementation at all.

### 2.4 LOW: Typo/Incorrect Command Name in Test

In `src/features/chat/dev/playground/eval/cases.ts:74`:
```typescript
await invoke('chat_v2_send', { sessionId, content: text, messageId: id });
```
The correct command name should be `chat_v2_send_message`.

### 2.5 Frontend API Wrappers (for reference)

The following API wrapper files abstract Tauri commands behind typed functions:

| API Module | File | Wrapped Commands |
|------------|------|-----------------|
| `settingsApi` | `src/api/settingsApi.ts` | `get_setting`, `save_setting`, `delete_setting`, `get_settings_by_prefix` |
| `chatV2Api` | `src/api/chatV2Api.ts` | `chat_v2_delete_session`, `chat_v2_update_session_settings`, `chat_v2_archive_session`, `chat_v2_save_session`, `chat_v2_upsert_streaming_block`, `chat_v2_update_block_tool_output`, `chat_v2_cancel_stream`, `chat_v2_add_tag`, `chat_v2_remove_tag`, `chat_v2_reorder_groups`, `chat_v2_ask_user_respond`, `chat_v2_tool_approval_respond` |
| `vfsRagApi` | `src/api/vfsRagApi.ts` | `vfs_rag_search`, `vfs_get_lance_stats`, `vfs_optimize_lance`, `vfs_debug_index_status`, `vfs_reset_disabled_to_pending`, `vfs_reset_indexed_without_embeddings`, `vfs_reset_all_index_state`, `vfs_multimodal_index`, `vfs_multimodal_search`, `vfs_multimodal_stats`, `vfs_multimodal_delete`, `vfs_multimodal_index_resource`, `vfs_get_pdf_page_image`, `vfs_diagnose_lance_schema` |
| `memoryApi` | `src/api/memoryApi.ts` | ~35 memory_* commands |
| `vfsUnifiedIndexApi` | `src/api/vfsUnifiedIndexApi.ts` | `vfs_batch_index_pending` |
| `vfsFileApi` | `src/api/vfsFileApi.ts` | `vfs_upload_file`, `vfs_get_file`, `vfs_list_files`, `vfs_delete_file`, `vfs_get_file_content` |
| `attachmentConfigApi` | `src/api/attachmentConfigApi.ts` | `vfs_get_attachment_config`, `vfs_set_attachment_root_folder`, `vfs_create_attachment_root_folder` |
| `dstu` | `src/dstu/api.ts` + sub-APIs | Various `dstu_*` commands |
| `llmUsageApi` | `src/api/llmUsageApi.ts` | Various `llm_usage_*` commands |
| `dataGovernance` | `src/api/dataGovernance.ts` | Various `data_governance_*` commands |

### 2.6 No Auto-Generated TypeScript Bindings

The project does NOT use `@tauri-apps/cli`'s `tauri::command` type generation. There is no `src/types/commands.ts` or `src/bindings/` directory. TypeScript types for command parameters and return values are manually maintained in the API wrapper files. This means:

- Type mismatches between Rust `#[tauri::command]` signatures and TypeScript calls cannot be detected automatically.
- The manual types in API files (e.g., `VfsRagSearchInput`, `MemoryConfig`) may drift from the actual Rust types.

---

## 3. Recommendations

1. **Clean up deprecated graph module calls**: Remove or guard the `unified_*` and `continue_unified_chat_stream` invocations from `src/utils/graphApi.ts` and `src/utils/chatApi.ts`. These commands no longer exist in the backend.

2. **Register or remove `research_*` commands**: Either register the 4 existing `research_*` implementations in the `invoke_handler` and implement the missing ~22 commands, or remove all `research_*` frontend calls from `src/utils/settingsApi.ts` and `src/utils/chatApi.ts`.

3. **Register `preheat_mcp_tools`**: Add `crate::commands::preheat_mcp_tools` to the `generate_handler![]` macro in `lib.rs`.

4. **Fix `chat_v2_send` typo**: In `src/features/chat/dev/playground/eval/cases.ts`, change `'chat_v2_send'` to `'chat_v2_send_message'`.

5. **Consider auto-generating TypeScript types**: Use `tauri::command` attribute's type generation capability (e.g., `tauri 2.0`'s `specta` integration or `@tauri-apps/cli` codegen) to prevent drift between Rust command signatures and TypeScript types.
