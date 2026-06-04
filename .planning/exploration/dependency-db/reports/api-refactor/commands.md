# API 重构: commands.rs — 遗留命令文件

**日期**: 2026-05-29 | **命令数**: 137 | **对应诊断**: round-20~26

---

## 当前问题

这是项目的旧版命令集合，与 cmd/ 目录并存。137 个命令中许多已被 cmd/ 子模块接管（通过 pub use re-export），剩余的命令应迁移到对应子模块然后退役此文件。

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `State<AppState>` | 119 |
| `String` | 69 |
| `Window` | 9 |
| `AppHandle` | 9 |
| `Option<u32>` | 5 |
| `Option<String>` | 5 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `Result<String>` | 19 |
| `Result<()>` | 18 |
| `Result<serde_json::Value>` | 17 |
| `Result<bool>` | 9 |
| `serde_json::Value` | 3 |
| `Result<AnkiDocumentGenerationResponse>` | 3 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `analyze_query_performance` | `debug__analyze_query_performance` | — | — |
| `auto_install_package_manager` | *(保持)* | — | Result<serde_json::Value, AppError> |
| `call_llm_for_boundary` | `anki__call_llm_for_boundary` | — | — |
| `cancel_pdf_ocr_session` | `vfs__cancel_pdf_ocr_session` | — | — |
| `cancel_stream` | *(保持)* | — | — |
| `check_all_package_managers` | *(保持)* | — | Result<serde_json::Value, AppError> |
| `check_package_manager` | *(保持)* | — | Result<serde_json::Value, AppError> |
| `check_test_dependencies` | *(保持)* | — | — |
| `cleanup_old_debug_logs` | *(保持)* | — | — |
| `clear_debug_logs` | *(保持)* | — | — |
| `clear_message_embeddings` | `debug__clear_message_embeddings` | — | — |
| `copy_file` | *(保持)* | — | — |
| `create_custom_template` | *(保持)* | — | — |
| `create_performance_indexes` | `debug__create_performance_indexes` | — | — |
| `delete_custom_template` | *(保持)* | — | — |
| `ensure_debug_log_dir` | *(保持)* | — | — |
| `estimate_tokens` | `settings__estimate_tokens` | — | — |
| `export_mcp_config` | *(保持)* | — | — |
| `export_questions_csv` | `qbank__export_questions_csv` | — | — |
| `export_template` | *(保持)* | — | — |
| `export_unified_backup_data` | *(保持)* | — | — |
| `generate_anki_cards_from_document` | `anki__generate_anki_cards_from_document` | — | — |
| `generate_anki_cards_from_document_base64` | `anki__generate_anki_cards_from_document_base64` | → Input struct | — |
| `generate_anki_cards_from_document_file` | `anki__generate_anki_cards_from_document_file` | — | — |
| `get_all_custom_templates` | *(保持)* | — | — |
| `get_all_recent_cards` | *(保持)* | — | — |
| `get_api_configurations` | `settings__get_api_configurations` | — | — |
| `get_app_data_dir` | *(保持)* | — | — |
| `get_app_version` | *(保持)* | — | Result<String, AppError> |
| `get_csv_exportable_fields` | `qbank__get_csv_exportable_fields` | — | Result<Vec<(String, String)>, AppError> |
| `get_csv_preview` | `qbank__get_csv_preview` | — | — |
| `get_custom_template_by_id` | *(保持)* | — | — |
| `get_database_info` | *(保持)* | — | — |
| `get_debug_logs_info` | *(保持)* | — | — |
| `get_default_template_id` | *(保持)* | — | — |
| `get_enhanced_statistics` | *(保持)* | — | — |
| `get_exam_sheet_session_detail` | `exam__get_exam_sheet_session_detail` | — | — |
| `get_file_size` | *(保持)* | — | — |
| `get_image_as_base64` | `search__get_image_as_base64` | — | — |
| `get_injection_budget_config` | `search__get_injection_budget_config` | — | — |
| `get_learning_heatmap` | *(保持)* | — | — |
| `get_mcp_config` | *(保持)* | — | — |
| `get_model_assignments` | `settings__get_model_assignments` | — | — |
| `get_model_profiles` | `settings__get_model_profiles` | — | — |
| `get_pdf_ocr_temp_dir` | `vfs__get_pdf_ocr_temp_dir` | — | — |
| `get_recent_document_tasks` | *(保持)* | — | — |
| `get_test_logs` | *(保持)* | — | — |
| `get_vendor_configs` | `settings__get_vendor_configs` | — | — |
| `hash_file` | *(保持)* | — | — |
| `import_builtin_templates` | *(保持)* | — | — |
| `import_custom_templates_bulk` | *(保持)* | — | — |
| `import_mcp_config` | *(保持)* | — | — |
| `import_question_bank` | `qbank__import_question_bank` | — | — |
| `import_question_bank_stream` | `qbank__import_question_bank_stream` | — | — |
| `import_questions_csv` | `qbank__import_questions_csv` | → Input struct | — |
| `import_template` | *(保持)* | — | — |
| `init_pdf_ocr_session` | `vfs__init_pdf_ocr_session` | → Input struct | — |
| `inspect_pdf_text_for_qbank` | `exam__inspect_pdf_text_for_qbank` | — | — |
| `list_exam_sheet_sessions` | `exam__list_exam_sheet_sessions` | — | — |
| `list_importing_sessions` | `qbank__list_importing_sessions` | — | — |
| `load_webview_settings` | *(保持)* | — | — |
| `open_log_file` | *(保持)* | — | — |
| `open_logs_folder` | *(保持)* | — | — |
| `optimize_chat_embeddings_table` | `debug__optimize_chat_embeddings_table` | → Input struct | — |
| `optimize_lance_database` | *(保持)* | — | — |
| `parse_document_from_base64` | *(保持)* | — | Result<T, AppError> |
| `parse_document_from_path` | *(保持)* | — | Result<T, AppError> |
| `pause_pdf_ocr_session` | `vfs__pause_pdf_ocr_session` | — | — |
| `pin_images` | `qbank__pin_images` | — | — |
| `process_pdf_ocr` | `vfs__process_pdf_ocr` | — | — |
| `qbank_batch_create_questions` | `qbank__qbank_batch_create_questions` | — | — |
| `qbank_batch_delete_questions` | `qbank__qbank_batch_delete_questions` | — | — |
| `qbank_batch_update_questions` | `qbank__qbank_batch_update_questions` | — | — |
| `qbank_create_question` | `qbank__qbank_create_question` | — | — |
| `qbank_crop_source_image` | `qbank__qbank_crop_source_image` | — | — |
| `qbank_delete_question` | `qbank__qbank_delete_question` | — | — |
| `qbank_generate_mock_exam` | `qbank__qbank_generate_mock_exam` | — | — |
| `qbank_generate_paper` | `qbank__qbank_generate_paper` | — | — |
| `qbank_get_activity_heatmap` | `qbank__qbank_get_activity_heatmap` | — | — |
| `qbank_get_check_in_calendar` | `qbank__qbank_get_check_in_calendar` | — | — |
| `qbank_get_daily_practice` | `qbank__qbank_get_daily_practice` | — | — |
| `qbank_get_history` | `qbank__qbank_get_history` | — | — |
| `qbank_get_knowledge_stats` | `qbank__qbank_get_knowledge_stats` | — | — |
| `qbank_get_knowledge_stats_with_comparison` | `qbank__qbank_get_knowledge_stats_with_comparison` | — | — |
| `qbank_get_learning_trend` | `qbank__qbank_get_learning_trend` | — | — |
| `qbank_get_question` | `qbank__qbank_get_question` | — | — |
| `qbank_get_question_by_card_id` | `qbank__qbank_get_question_by_card_id` | — | — |
| `qbank_get_source_images` | `qbank__qbank_get_source_images` | — | — |
| `qbank_get_stats` | `qbank__qbank_get_stats` | — | — |
| `qbank_get_submissions` | `qbank__qbank_get_submissions` | — | — |
| `qbank_list_questions` | `qbank__qbank_list_questions` | — | — |
| `qbank_rebuild_fts_index` | `qbank__qbank_rebuild_fts_index` | — | — |
| `qbank_refresh_stats` | `qbank__qbank_refresh_stats` | — | — |
| `qbank_remove_question_image` | `qbank__qbank_remove_question_image` | — | — |
| `qbank_reset_progress` | `qbank__qbank_reset_progress` | — | — |
| `qbank_reset_questions_progress` | `qbank__qbank_reset_questions_progress` | — | — |
| `qbank_search_questions` | `qbank__qbank_search_questions` | — | — |
| `qbank_start_timed_practice` | `qbank__qbank_start_timed_practice` | — | — |
| `qbank_submit_answer` | `qbank__qbank_submit_answer` | — | — |
| `qbank_submit_mock_exam` | `qbank__qbank_submit_mock_exam` | — | — |
| `qbank_toggle_favorite` | `qbank__qbank_toggle_favorite` | — | — |
| `qbank_update_question` | `qbank__qbank_update_question` | — | — |
| `read_debug_log_file` | *(保持)* | — | — |
| `read_file_bytes` | *(保持)* | — | — |
| `read_file_text` | *(保持)* | — | — |
| `rename_exam_sheet_session` | `exam__rename_exam_sheet_session` | — | — |
| `report_frontend_log` | *(保持)* | — | — |
| `research_delete_report` | *(保持)* | — | — |
| `research_export_all_reports_zip` | *(保持)* | — | — |
| `research_get_report` | *(保持)* | — | — |
| `research_list_reports` | *(保持)* | — | — |
| `reset_test_database` | *(保持)* | — | — |
| `resume_pdf_ocr_session` | `vfs__resume_pdf_ocr_session` | — | — |
| `resume_question_import` | `qbank__resume_question_import` | — | — |
| `save_api_configurations` | `settings__save_api_configurations` | — | — |
| `save_model_assignments` | `settings__save_model_assignments` | — | — |
| `save_model_profiles` | `settings__save_model_profiles` | — | — |
| `save_pdf_to_temp` | `vfs__save_pdf_to_temp` | — | — |
| `save_template_debug_data` | *(保持)* | — | — |
| `save_test_log` | *(保持)* | → Input struct | — |
| `save_text_to_file` | *(保持)* | — | — |
| `save_vendor_configs` | `settings__save_vendor_configs` | — | — |
| `save_webview_settings` | *(保持)* | — | — |
| `seed_test_database` | *(保持)* | — | — |
| `set_default_template` | *(保持)* | — | — |
| `set_test_run_id` | *(保持)* | — | — |
| `simulate_budget_allocation` | `search__simulate_budget_allocation` | — | — |
| `skip_pdf_ocr_page` | `vfs__skip_pdf_ocr_page` | — | — |
| `start_pdf_ocr_backend` | `vfs__start_pdf_ocr_backend` | → Input struct | — |
| `switch_to_production_database` | *(保持)* | — | — |
| `switch_to_test_database` | *(保持)* | — | — |
| `test_api_connection` | `settings__test_api_connection` | → Input struct | — |
| `unpin_images` | `qbank__unpin_images` | — | — |
| `update_custom_template` | *(保持)* | — | — |
| `update_exam_sheet_cards` | `exam__update_exam_sheet_cards` | — | — |
| `upload_pdf_ocr_page` | `vfs__upload_pdf_ocr_page` | — | — |
| `write_test_report` | *(保持)* | — | — |

## 改进操作

退役计划：将剩余命令迁移到 cmd/ 子模块，commands.rs 保留为兼容 re-export 层（或直接删除）

## 统一错误类型

`AppError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/commands.json`*
