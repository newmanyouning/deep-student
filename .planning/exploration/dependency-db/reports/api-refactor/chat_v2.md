# API 重构: Chat V2 — 对话引擎

**日期**: 2026-05-29 | **命令数**: 78 | **对应诊断**: round-20~26

---

## 当前问题

78 个命令，返回类型用 Result<T, String> 而非统一的错误类型。部分命令（anki_cards_result, canvas_edit_result）通过 AppHandle 发事件而非返回值。

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `String` | 92 |
| `State<ChatV2Database>` | 51 |
| `State<WorkspaceCoordinator>` | 18 |
| `Option<String>` | 12 |
| `State<ChatV2State>` | 10 |
| `Window` | 7 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `()` | 23 |
| `ChatSession` | 4 |
| `u32` | 3 |
| `bool` | 3 |
| `ChatV2Result<SkillFileContent>` | 3 |
| `String` | 2 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `chat_v2_add_tag` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_anki_cards_result` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_archive_session` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_ask_user_respond` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `chat_v2_branch_session` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `chat_v2_cancel_stream` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `chat_v2_cancel_variant` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `chat_v2_canvas_edit_result` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_check_migration_status` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_clear_approval_history` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_copy_block_content` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_count_sessions` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_create_group` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_create_session` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `chat_v2_delete_group` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_delete_message` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `chat_v2_delete_session` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `chat_v2_delete_variant` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `chat_v2_empty_deleted_sessions` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_get_anki_cards_from_block_by_document_id` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_get_group` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_get_session` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_get_session_tags` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_get_tags_batch` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_list_agent_sessions` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_list_all_tags` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_list_groups` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_list_sessions` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `chat_v2_load_session` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_migrate_legacy_chat` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_move_session_to_group` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_perform_ocr` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_remove_tag` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_reorder_groups` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_restore_session` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_rollback_migration` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_save_session` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_search_content` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_send_message` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `chat_v2_session_message_count` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_soft_delete_session` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_switch_variant` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `chat_v2_tool_approval_cancel` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_update_block_content` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `chat_v2_update_block_tool_output` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_update_group` | *(保持)* | — | Result<T, ChatV2Error> |
| `chat_v2_update_session_settings` | *(保持)* | — | Result<T, ChatV2Error> |
| `resource_create_or_reuse` | *(保持)* | — | Result<T, ChatV2Error> |
| `resource_decrement_ref` | *(保持)* | — | Result<T, ChatV2Error> |
| `resource_exists` | *(保持)* | — | Result<T, ChatV2Error> |
| `resource_get` | *(保持)* | — | Result<T, ChatV2Error> |
| `resource_get_content_from_vfs` | *(保持)* | — | Result<T, ChatV2Error> |
| `resource_get_latest` | *(保持)* | — | Result<T, ChatV2Error> |
| `resource_get_versions_by_source` | *(保持)* | — | Result<T, ChatV2Error> |
| `resource_increment_ref` | *(保持)* | — | Result<T, ChatV2Error> |
| `skill_create` | *(保持)* | — | — |
| `skill_delete` | *(保持)* | — | — |
| `skill_list_directories` | *(保持)* | — | — |
| `skill_read_file` | *(保持)* | — | — |
| `skill_update` | *(保持)* | — | — |
| `workspace_cancel_agent` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `workspace_cancel_sleep` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `workspace_close` | *(保持)* | — | Result<T, ChatV2Error> |
| `workspace_create` | *(保持)* | — | Result<T, ChatV2Error> |
| `workspace_create_agent` | *(保持)* | — | Result<T, ChatV2Error> |
| `workspace_delete` | *(保持)* | — | Result<T, ChatV2Error> |
| `workspace_get` | *(保持)* | — | Result<T, ChatV2Error> |
| `workspace_get_context` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `workspace_get_document` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `workspace_list_agents` | *(保持)* | — | Result<T, ChatV2Error> |
| `workspace_list_all` | *(保持)* | — | Result<T, ChatV2Error> |
| `workspace_list_documents` | *(保持)* | — | Result<T, ChatV2Error> |
| `workspace_list_messages` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `workspace_manual_wake` | *(保持)* | — | Result<T, ChatV2Error> |
| `workspace_restore_executions` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `workspace_run_agent` | *(保持)* | → Input struct | Result<T, ChatV2Error> |
| `workspace_send_message` | *(保持)* | — | Result<T, ChatV2Error> |
| `workspace_set_context` | *(保持)* | → Input struct | Result<T, ChatV2Error> |

## 改进操作

统一错误类型为 ChatV2Error，将事件驱动命令改为返回值模式

## 统一错误类型

`ChatV2Error` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/chat_v2.json`*
