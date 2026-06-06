# Database Schema -- Entity Relationship Diagrams

> This document describes all SQLite tables used by Deep Student across its multiple database files.
> Table definitions and relationships are extracted from `src-tauri/src/database/`, `src-tauri/src/vfs/`, `migrations/`, and `src-tauri/src/chat_v2/`.

---

## Database Architecture Overview

Deep Student uses **5 separate SQLite database files** plus OS filesystem storage:

| Database File | Module | Purpose | Migrations |
|---------------|--------|---------|------------|
| `mistakes.db` | `database` | Legacy chat, mistakes, settings, document_tasks, anki_cards | `database/manager.rs` + `migrations/mistakes/` |
| `vfs.db` | `vfs` | Unified resources, notes, files, folders, questions, exams, review plans | `migrations/vfs/` (Refinery) |
| `chat_v2.db` | `chat_v2` | Chat sessions, messages, blocks, workspace agents | `migrations/chat_v2/` (Refinery) |
| `llm_usage.db` | `llm_usage` | Token usage statistics | `migrations/llm_usage/` (Refinery) |
| `audit.db` | `data_governance` | Data governance audit logs | `data_governance` module |

---

## 1. mistakes.db (Main Database)

### Core: mistakes & chat_messages

```mermaid
erDiagram
  schema_version {
    int version PK
  }

  mistakes {
    text id PK
    text created_at
    text question_images
    text analysis_images
    text user_question
    text ocr_text
    text ocr_note
    text tags
    text mistake_type
    text status
    text chat_category
    text updated_at
    text last_accessed_at
    text chat_metadata
    text exam_sheet
    text autosave_signature
    text mistake_summary
    text user_error_analysis
    text irec_card_id
    int irec_status
  }

  chat_messages {
    int id PK
    text mistake_id FK
    text role
    text content
    text timestamp
    text thinking_content
    text rag_sources
    text memory_sources
    text graph_sources
    text web_search_sources
    text image_paths
    text image_base64
    text doc_attachments
    text tool_call
    text tool_result
    text overrides
    text relations
    text stable_id
    text turn_id
    int turn_seq
    int reply_to_msg_id
    text message_kind
    text lifecycle
    text metadata
  }

  mistakes ||--o{ chat_messages : "mistake_id"
```

### Review & Sessions

```mermaid
erDiagram
  review_analyses {
    text id PK
    text name
    text created_at
    text updated_at
    text mistake_ids
    text consolidated_input
    text user_question
    text status
    text tags
    text analysis_type
    text temp_session_data
    int session_sequence
  }

  review_chat_messages {
    int id PK
    text review_analysis_id FK
    text role
    text content
    text timestamp
    text thinking_content
    text rag_sources
    text memory_sources
    text graph_sources
    text web_search_sources
    text image_paths
    text image_base64
    text doc_attachments
    text tool_call
    text tool_result
    text overrides
    text relations
    text metadata
  }

  review_sessions {
    text id PK
    text title
    text start_date
    text end_date
    text created_at
  }

  review_session_mistakes {
    text session_id PK,FK
    text mistake_id PK,FK
    text added_at
  }

  temp_sessions {
    text temp_id PK
    text session_data
    text stream_state
    text created_at
    text updated_at
    text last_error
  }

  review_analyses ||--o{ review_chat_messages : "review_analysis_id"
  review_sessions ||--o{ review_session_mistakes : "session_id"
  mistakes ||--o{ review_session_mistakes : "mistake_id"
```

### Document Tasks & Anki Cards

```mermaid
erDiagram
  document_tasks {
    text id PK
    text document_id
    text original_document_name
    int segment_index
    text content_segment
    text status
    text created_at
    text updated_at
    text error_message
    text anki_generation_options_json
  }

  anki_cards {
    text id PK
    text task_id FK
    text front
    text back
    text tags_json
    text images_json
    int is_error_card
    text error_content
    int card_order_in_task
    text extra_fields_json
    text template_id
    text source_type
    text source_id
    text text
  }

  document_control_states {
    text document_id PK
    text state
    text pending_tasks_json
    text running_tasks_json
    text completed_tasks_json
    text failed_tasks_json
  }

  document_tasks ||--o{ anki_cards : "task_id"
```

### Settings & Templates

```mermaid
erDiagram
  settings {
    text key PK
    text value
    text updated_at
  }

  custom_anki_templates {
    text id PK
    text name
    text description
    text author
    text version
    text preview_front
    text preview_back
    text note_type
    text fields_json
    text generation_prompt
    text front_template
    text back_template
    text css_style
    text field_extraction_rules_json
    int is_active
    int is_built_in
  }

  rag_sub_libraries {
    text id PK
    text name
    text description
    text created_at
  }

  rag_configurations {
    text id PK
    text sub_library_id FK
    text config_json
  }

  rag_sub_libraries ||--o{ rag_configurations : "sub_library_id"
```

### Supporting Tables

```mermaid
erDiagram
  vectorized_data {
    text id PK
    text mistake_id FK
    text text_content
    text embedding_json
  }

  search_logs {
    text id PK
    text search_type
    text query
    int result_count
    text execution_time_ms
    text mistake_ids_json
    text error_message
    text user_feedback
  }

  migration_progress {
    text category PK
    text status
    text last_cursor
    int total_processed
    text last_error
  }

  research_reports {
    text id PK
    text created_at
    int segments
    int context_window
    text report
    text metadata
  }

  rag_query_logs {
    int id PK
    text query_text
    text sub_library_id
    int results_count
    int processing_time_ms
  }

  pending_memory_candidates {
    int id PK
    text conversation_id
    text content
    text category
    text origin
    int user_edited
    text status
    text expires_at
  }

  exam_sheet_sessions {
    text id PK
    text exam_name
    text temp_id
    text status
    text metadata_json
    text preview_json
    text linked_mistake_ids
  }

  embedding_dimension_registry {
    int dimension PK
    text model_config_id
    text model_name
    text table_prefix
    int is_multimodal
  }

  mistakes ||--o{ vectorized_data : "mistake_id"
```

---

## 2. vfs.db (VFS Database -- 27 tables)

### Core Resource Tables

```mermaid
erDiagram
  resources {
    text id PK
    text hash UK
    text type
    text source_id
    text source_table
    text storage_mode
    text data
    text external_hash
    text metadata_json
    int ref_count
    int created_at
    int updated_at
    int deleted_at
    text deleted_reason
    text index_state
    text index_hash
    text index_error
    int indexed_at
    int index_retry_count
    text ocr_text
  }

  blobs {
    text hash PK
    text relative_path
    int size
    text mime_type
    int ref_count
    int created_at
  }

  files {
    text id PK
    text resource_id FK
    text blob_hash FK
    text sha256 UK
    text file_name
    text original_path
    int size
    int page_count
    text tags_json
    int is_favorite
    text last_opened_at
    int last_page
    text bookmarks_json
    text cover_key
    text status
    text type
    text name
    text content_hash
    text description
    text mime_type
    text preview_json
    text extracted_text
    text ocr_pages_json
  }

  resources ||--o{ files : "resource_id"
  blobs ||--o{ files : "blob_hash"
```

### Notes & Versions

```mermaid
erDiagram
  notes {
    text id PK
    text resource_id FK
    text title
    text tags
    int is_favorite
    text created_at
    text updated_at
    text deleted_at
  }

  notes_versions {
    text version_id PK
    text note_id FK
    text resource_id FK
    text title
    text tags
    text label
  }

  resources ||--o{ notes : "resource_id"
  notes ||--o{ notes_versions : "note_id"
  resources ||--o{ notes_versions : "resource_id"
```

### Folder Hierarchy

```mermaid
erDiagram
  folders {
    text id PK
    text parent_id FK
    text title
    text icon
    text color
    int is_expanded
    int sort_order
    int created_at
    int updated_at
    text deleted_at
    int is_favorite
  }

  folder_items {
    text id PK
    text folder_id FK
    text item_type
    text item_id
    int sort_order
    int created_at
    int updated_at
    text cached_path
    text deleted_at
  }

  path_cache {
    text item_type PK
    text item_id PK
    text full_path
    text folder_path
    text updated_at
  }

  folders ||--o{ folders : "parent_id"
  folders ||--o{ folder_items : "folder_id"
```

### Exam Sheets, Questions & Review Plans

```mermaid
erDiagram
  exam_sheets {
    text id PK
    text resource_id FK
    text exam_name
    text status
    text temp_id
    text metadata_json
    text preview_json
    text linked_mistake_ids
    int is_favorite
    text ocr_pages_json
    int sync_enabled
  }

  questions {
    text id PK
    text exam_id FK
    text card_id
    text question_label
    text content
    text options_json
    text answer
    text explanation
    text question_type
    text difficulty
    text tags
    text status
    text user_answer
    int is_correct
    int attempt_count
    int correct_count
    text last_attempt_at
    int is_favorite
    int is_bookmarked
    text parent_id FK
    text sync_status
  }

  question_bank_stats {
    text exam_id PK,FK
    int total_count
    int new_count
    int in_progress_count
    int mastered_count
    int review_count
    float correct_rate
  }

  question_history {
    text id PK
    text question_id FK
    text field_name
    text old_value
    text new_value
    text operator
    text reason
  }

  question_sync_conflicts {
    text id PK
    text question_id FK
    text exam_id FK
    text conflict_type
    text local_snapshot
    text remote_snapshot
    text status
    text resolved_strategy
  }

  question_sync_logs {
    text id PK
    text exam_id FK
    text direction
    text sync_type
    text result
    int synced_count
    int conflict_count
  }

  exam_sheets ||--o{ questions : "exam_id"
  exam_sheets ||--o{ question_bank_stats : "exam_id"
  questions ||--o{ question_history : "question_id"
  questions ||--o{ question_sync_conflicts : "question_id"
  exam_sheets ||--o{ question_sync_conflicts : "exam_id"
  exam_sheets ||--o{ question_sync_logs : "exam_id"
  questions ||--o{ questions : "parent_id (self-ref)"
```

### SM-2 Review System

```mermaid
erDiagram
  review_plans {
    text id PK
    text question_id UK,FK
    text exam_id FK
    float ease_factor
    int interval_days
    int repetitions
    text next_review_date
    text status
    int total_reviews
    int total_correct
    int consecutive_failures
    int is_difficult
  }

  review_history {
    text id PK
    text plan_id FK
    text question_id FK
    int quality
    int passed
    float ease_factor_before
    float ease_factor_after
    int interval_before
    int interval_after
    int repetitions_before
    int repetitions_after
    text user_answer
    int time_spent_seconds
  }

  review_stats {
    text exam_id PK,FK
    int total_plans
    int new_count
    int learning_count
    int reviewing_count
    int graduated_count
    int due_today
    int overdue_count
    float avg_correct_rate
    float avg_ease_factor
  }

  questions ||--|| review_plans : "question_id"
  review_plans ||--o{ review_history : "plan_id"
  questions ||--o{ review_history : "question_id"
  exam_sheets ||--o{ review_plans : "exam_id"
  exam_sheets ||--o{ review_stats : "exam_id"
```

### Essays, Translations & Mindmaps

```mermaid
erDiagram
  essays {
    text id PK
    text resource_id FK
    text title
    text essay_type
    text grading_result_json
    int score
    text session_id
    int round_number
    text grade_level
    text custom_prompt
    text dimension_scores_json
    int is_favorite
  }

  essay_sessions {
    text id PK
    text title
    text essay_type
    text grade_level
    text custom_prompt
    text subject
    int total_rounds
    int latest_score
    int is_favorite
  }

  translations {
    text id PK
    text resource_id FK
    text src_lang
    text tgt_lang
    text engine
    text model
    int is_favorite
    text title
    text subject
  }

  mindmaps {
    text id PK
    text resource_id FK
    text title
    text description
    int is_favorite
    text default_view
    text theme
    text settings
  }

  resources ||--o{ essays : "resource_id"
  resources ||--o{ translations : "resource_id"
  resources ||--o{ mindmaps : "resource_id"
```

### Indexing System

```mermaid
erDiagram
  vfs_index_units {
    text id PK
    text resource_id
    int unit_index
    text image_blob_hash
    text image_mime_type
    text text_content
    text text_source
    int text_required
    text text_state
    int text_embedding_dim
    int mm_required
    text mm_state
    int mm_embedding_dim
  }

  vfs_index_segments {
    text id PK
    text unit_id FK
    int segment_index
    text modality
    int embedding_dim
    text lance_row_id
    text content_text
    text content_hash
    int start_pos
    int end_pos
  }

  vfs_embedding_dims {
    int dimension PK
    text modality PK
    text lance_table_name
    int record_count
    text model_config_id
    text model_name
  }

  vfs_indexing_config {
    text key PK
    text value
  }

  vfs_index_units ||--o{ vfs_index_segments : "unit_id"
```

### Todo, Pomodoro & Memory Config

```mermaid
erDiagram
  todo_lists_new {
    text id PK
    text title
    text description
    text icon
    text color
    int sort_order
    int is_default
    int is_favorite
  }

  todo_items {
    text id PK
    text todo_list_id FK
    text title
    text description
    text status
    text priority
    text due_date
    text due_time
    text reminder
    text tags_json
    text recurrence_json
  }

  pomodoro_records {
    text id PK
    text todo_item_id FK
    text start_time
    text end_time
    int duration
    int actual_duration
    text type
    text status
  }

  memory_config {
    text key PK
    text value
  }

  todo_lists_new ||--o{ todo_items : "todo_list_id"
  todo_items ||--o{ pomodoro_records : "todo_item_id"
```

---

## 3. chat_v2.db (Chat V2 Database)

### Core Chat Tables

```mermaid
erDiagram
  chat_v2_sessions {
    text id PK
    text mode
    text title
    text persist_status
    text created_at
    text updated_at
    text metadata_json
    text description
    text summary_hash
    text workspace_id
  }

  chat_v2_messages {
    text id PK
    text session_id FK
    text role
    text block_ids_json
    int timestamp
    text persistent_stable_id
    text parent_id
    text supersedes
    text meta_json
    text attachments_json
    text active_variant_id
    text variants_json
    text shared_context_json
  }

  chat_v2_blocks {
    text id PK
    text message_id FK
    text block_type
    text status
    int block_index
    text content
    text tool_name
    text tool_input_json
    text tool_output_json
    text citations_json
    text error
    text variant_id
    int first_chunk_at
  }

  chat_v2_attachments {
    text id PK
    text message_id FK
    text name
    text type
    text mime_type
    int size
    text status
    text preview_url
    text storage_path
    text content_hash
    text block_id FK
  }

  chat_v2_session_state {
    text session_id PK,FK
    text chat_params_json
    text features_json
    text mode_state_json
    text input_value
    text panel_states_json
    text model_id
    float temperature
    int context_limit
    int max_tokens
    int enable_thinking
    int disable_tools
    int rag_enabled
    int web_search_enabled
    int anki_enabled
    text loaded_skill_ids_json
    text active_skill_id
  }

  chat_v2_session_mistakes {
    text session_id PK,FK
    text mistake_id PK
    text relation_type
  }

  chat_v2_sessions ||--o{ chat_v2_messages : "session_id"
  chat_v2_sessions ||--|| chat_v2_session_state : "session_id"
  chat_v2_sessions ||--o{ chat_v2_session_mistakes : "session_id"
  chat_v2_messages ||--o{ chat_v2_blocks : "message_id"
  chat_v2_messages ||--o{ chat_v2_attachments : "message_id"
  chat_v2_blocks ||--o{ chat_v2_attachments : "block_id"
```

### Resources & Todo Lists

```mermaid
erDiagram
  resources {
    text id PK
    text hash UK
    text type
    text source_id
    text data
    text metadata_json
    int ref_count
    int created_at
  }

  chat_v2_todo_lists {
    text session_id PK,FK
    text message_id
    text variant_id
    text todo_list_id
    text title
    text steps_json
    int is_all_done
  }

  chat_v2_messages ||--o{ chat_v2_todo_lists : "session_id"
```

### Workspace & Agent System

```mermaid
erDiagram
  workspace {
    text id PK
    text name
    text status
    text creator_session_id
    text metadata_json
  }

  agent {
    text session_id PK
    text workspace_id FK
    text role
    text skill_id
    text status
    text metadata_json
  }

  message {
    text id PK
    text workspace_id FK
    text sender_session_id
    text target_session_id
    text message_type
    text content
    text status
    text metadata_json
  }

  inbox {
    int id PK
    text session_id
    text message_id FK
    int priority
    text status
  }

  document {
    text id PK
    text workspace_id FK
    text doc_type
    text title
    text content
    int version
    text updated_by
  }

  context {
    text workspace_id PK,FK
    text key PK
    text value_json
    text updated_by
  }

  sleep_block {
    text id PK
    text workspace_id FK
    text coordinator_session_id
    text awaiting_agents
    text status
    text timeout_at
  }

  subagent_task {
    text id PK
    text workspace_id FK
    text agent_session_id
    text skill_id
    text initial_task
    text status
  }

  workspace ||--o{ agent : "workspace_id"
  workspace ||--o{ message : "workspace_id"
  workspace ||--o{ document : "workspace_id"
  workspace ||--|| context : "workspace_id"
  workspace ||--o{ sleep_block : "workspace_id"
  workspace ||--o{ subagent_task : "workspace_id"
  message ||--o{ inbox : "message_id"
```

---

## 4. llm_usage.db (LLM Usage Statistics)

```mermaid
erDiagram
  llm_usage_logs {
    text id PK
    text timestamp
    text provider
    text model
    text adapter
    text api_config_id
    int prompt_tokens
    int completion_tokens
    int total_tokens
    int reasoning_tokens
    int cached_tokens
    text token_source
    int duration_ms
    int request_bytes
    int response_bytes
    int first_token_ms
    text caller_type
    text caller_id
    text session_id
    text stream_id
    int success
    text error_type
    float cost_usd
    text currency
  }
```

---

## 5. audit.db (Data Governance Audit)

```mermaid
erDiagram
  audit_logs {
    int id PK
    text timestamp
    text action
    text actor
    text resource_type
    text resource_id
    text details_json
    text result
  }

  schema_registry {
    text db_name PK
    int global_version
    text last_migrated_at
    text current_schema_hash
  }
```

---

## Legend

| Notation | Meaning |
|----------|---------|
| `PK` | Primary Key |
| `FK` | Foreign Key |
| `UK` | Unique Key |
| `||--o{` | One-to-many relationship |
| `||--||` | One-to-one relationship |
| `}o--||` | Many-to-one relationship |
| `+` in `<>` | See VFS migration line references |
| `text` | `TEXT` column (SQLite) |
| `int` | `INTEGER` column (SQLite) |
| `float` | `REAL` column (SQLite) |

---

## Key Source References

| Database | Key File | Lines |
|----------|----------|-------|
| Main DB schema | `src-tauri/src/database/manager.rs` | 220-425 (init), 640-820 (compat) |
| Main DB schema (v13+) | `src-tauri/src/database/mod.rs` | 710-845, 4960-5056 |
| Mistakes migrations | `src-tauri/migrations/mistakes/V20260130__init.sql` | 1-300+ |
| VFS complete schema | `src-tauri/migrations/vfs/V20260130__init.sql` | 1-800+ |
| VFS todo/pomodoro | `src-tauri/migrations/vfs/V20260308__add_todo_tables.sql` | 1-50+ |
| VFS pomodoro | `src-tauri/migrations/vfs/V20260310__add_pomodoro.sql` | 1-30+ |
| VFS decouple todo | `src-tauri/migrations/vfs/V20260309__decouple_todo_from_vfs.sql` | 1-30+ |
| Chat V2 schema | `src-tauri/migrations/chat_v2/V20260130__init.sql` | 1-230+ |
| Workspace schema | `src-tauri/src/chat_v2/workspace/database.rs` | 14-117 |
| LLM Usage schema | `src-tauri/migrations/llm_usage/V20260130__init.sql` | 1-60+ |
| Memory intake tables | `src-tauri/src/database/manager.rs` | 383-411 |
