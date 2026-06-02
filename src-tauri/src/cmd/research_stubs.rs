//! Research command stubs — placeholder implementations.
//!
//! These 26 research_* commands are called from the frontend but lack
//! actual Rust implementations. Each stub returns a "not yet implemented"
//! error so that callers get a clear, typed response instead of a cryptic
//! "command not found" from Tauri's invoke handler.
//!
//! TODO: Replace each stub with a real implementation.
//!       Refer to the frontend call site in src/utils/settingsApi.ts
//!       for the exact parameter shapes and return types expected.

use tauri::State;
use crate::commands::AppState;
use crate::models::AppError;

// ──────────────────────────────────────────────
// Research session / round commands
// ──────────────────────────────────────────────

#[tauri::command]
pub async fn research_get_round(
    _session_id: String,
    _round_no: i32,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    Err(AppError::not_implemented(
        "research_get_round: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_get_round_visual_summary(
    _session_id: String,
    _round_no: i32,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    Err(AppError::not_implemented(
        "research_get_round_visual_summary: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_delete_round(
    _session_id: String,
    _round_no: i32,
    _clean_coverage: Option<bool>,
    _state: State<'_, AppState>,
) -> Result<String, AppError> {
    Err(AppError::not_implemented(
        "research_delete_round: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_generate_round_report(
    _session_id: String,
    _round_no: i32,
    _format: Option<String>,
    _opts_json: Option<String>,
    _state: State<'_, AppState>,
) -> Result<String, AppError> {
    Err(AppError::not_implemented(
        "research_generate_round_report: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_set_round_note(
    _session_id: String,
    _round_no: i32,
    _note: String,
    _tags: Option<Vec<String>>,
    _state: State<'_, AppState>,
) -> Result<String, AppError> {
    Err(AppError::not_implemented(
        "research_set_round_note: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_get_round_note(
    _session_id: String,
    _round_no: i32,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    Err(AppError::not_implemented(
        "research_get_round_note: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_get_round_notes(
    _session_id: String,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    Err(AppError::not_implemented(
        "research_get_round_notes: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_generate_session_report(
    _session_id: String,
    _format: Option<String>,
    _opts_json: Option<String>,
    _rounds: Option<Vec<i32>>,
    _state: State<'_, AppState>,
) -> Result<String, AppError> {
    Err(AppError::not_implemented(
        "research_generate_session_report: not yet implemented",
    ))
}

// ──────────────────────────────────────────────
// Document chunk commands
// ──────────────────────────────────────────────

#[tauri::command]
pub async fn research_get_chunk_text(
    _session_id: String,
    _document_id: String,
    _chunk_index: i32,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    Err(AppError::not_implemented(
        "research_get_chunk_text: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_get_chunk_context(
    _session_id: String,
    _document_id: String,
    _chunk_index: i32,
    _before: i32,
    _after: i32,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    Err(AppError::not_implemented(
        "research_get_chunk_context: not yet implemented",
    ))
}

// ──────────────────────────────────────────────
// Session management
// ──────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct UpdateSessionOptionsRequest {
    pub session_id: Option<String>,
    pub options_json: Option<String>,
}

#[tauri::command]
pub async fn research_update_session_options(
    req: UpdateSessionOptionsRequest,
    _state: State<'_, AppState>,
) -> Result<String, AppError> {
    let _ = req; // suppress unused warning
    Err(AppError::not_implemented(
        "research_update_session_options: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_delete_session(
    _session_id: String,
    _state: State<'_, AppState>,
) -> Result<String, AppError> {
    Err(AppError::not_implemented(
        "research_delete_session: not yet implemented",
    ))
}

// ──────────────────────────────────────────────
// Run / execution commands
// ──────────────────────────────────────────────

#[tauri::command]
pub async fn research_run_until(
    _session_id: String,
    _max_rounds: Option<i32>,
    _min_selected: Option<i32>,
    _silent_approval: Option<bool>,
    _state: State<'_, AppState>,
) -> Result<String, AppError> {
    Err(AppError::not_implemented(
        "research_run_until: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_run_macro(
    _session_id: String,
    _keywords: Option<Vec<String>>,
    _state: State<'_, AppState>,
) -> Result<String, AppError> {
    Err(AppError::not_implemented(
        "research_run_macro: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_run_to_full_coverage(
    _session_id: String,
    _state: State<'_, AppState>,
) -> Result<String, AppError> {
    Err(AppError::not_implemented(
        "research_run_to_full_coverage: not yet implemented",
    ))
}

// ──────────────────────────────────────────────
// QA / audit commands
// ──────────────────────────────────────────────

#[tauri::command]
pub async fn research_audit_user_questions(
    _date_range: Option<(String, String)>,
    _keywords: Option<Vec<String>>,
    _group_by: Option<String>,
    _limit: Option<i32>,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    Err(AppError::not_implemented(
        "research_audit_user_questions: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_find_similar_questions(
    _question_text: String,
    _top_k: Option<i32>,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    Err(AppError::not_implemented(
        "research_find_similar_questions: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_get_full_chat_history(
    _document_id: Option<String>,
    _message_id: Option<String>,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    Err(AppError::not_implemented(
        "research_get_full_chat_history: not yet implemented",
    ))
}

// ──────────────────────────────────────────────
// Deep read commands
// ──────────────────────────────────────────────

#[tauri::command]
pub async fn research_deep_read_by_docs(
    _session_id: String,
    _document_ids: Vec<String>,
    _context_threshold: Option<i32>,
    _state: State<'_, AppState>,
) -> Result<String, AppError> {
    Err(AppError::not_implemented(
        "research_deep_read_by_docs: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_deep_read_by_tag(
    _session_id: String,
    _tag: String,
    _context_threshold: Option<i32>,
    _state: State<'_, AppState>,
) -> Result<String, AppError> {
    Err(AppError::not_implemented(
        "research_deep_read_by_tag: not yet implemented",
    ))
}

// ──────────────────────────────────────────────
// Token / content utilities
// ──────────────────────────────────────────────

#[tauri::command]
pub async fn research_count_tokens(
    _document_ids: Vec<String>,
    _precise: Option<bool>,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    Err(AppError::not_implemented(
        "research_count_tokens: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_get_full_content(
    _document_ids: Vec<String>,
    _precise: Option<bool>,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    Err(AppError::not_implemented(
        "research_get_full_content: not yet implemented",
    ))
}

// ──────────────────────────────────────────────
// Settings CRUD
// ──────────────────────────────────────────────

#[tauri::command]
pub async fn research_get_setting(
    _key: String,
    _state: State<'_, AppState>,
) -> Result<Option<String>, AppError> {
    Err(AppError::not_implemented(
        "research_get_setting: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_set_setting(
    _key: String,
    _value: String,
    _state: State<'_, AppState>,
) -> Result<String, AppError> {
    Err(AppError::not_implemented(
        "research_set_setting: not yet implemented",
    ))
}

#[tauri::command]
pub async fn research_delete_setting(
    _key: String,
    _state: State<'_, AppState>,
) -> Result<String, AppError> {
    Err(AppError::not_implemented(
        "research_delete_setting: not yet implemented",
    ))
}

// ──────────────────────────────────────────────
// Artifacts
// ──────────────────────────────────────────────

#[tauri::command]
pub async fn research_list_artifacts(
    _session_id: String,
    _round_no: Option<i32>,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    Err(AppError::not_implemented(
        "research_list_artifacts: not yet implemented",
    ))
}
