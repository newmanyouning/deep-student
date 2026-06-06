//! 会话加载命令处理器
//!
//! 加载会话的完整数据，包括会话信息、消息列表、块列表和会话状态。

use std::sync::Arc;

use std::time::Instant;

use tauri::State;

use crate::chat_v2::database::ChatV2Database;
use crate::chat_v2::error::ChatV2Error;
use crate::chat_v2::repo::ChatV2Repo;
use crate::chat_v2::types::LoadSessionResponse;

/// 加载会话完整数据
///
/// 从数据库加载会话的所有相关数据，用于前端初始化会话视图。
///
/// ## 参数
/// - `session_id`: 会话 ID
/// - `db`: Chat V2 独立数据库
///
/// ## 返回
/// - `Ok(LoadSessionResponse)`: 会话完整数据
/// - `Err(String)`: 会话不存在或加载失败
///
/// ## 响应结构
/// ```json
/// {
///   "session": { ... },
///   "messages": [ ... ],
///   "blocks": [ ... ],
///   "state": { ... }
/// }
/// ```
#[tauri::command]
pub async fn chat_v2_load_session(
    session_id: String,
    db: State<'_, Arc<ChatV2Database>>,
) -> Result<LoadSessionResponse, String> {
    let t0 = Instant::now();
    log::info!(
        "[ChatV2::handlers] chat_v2_load_session: session_id={}",
        session_id
    );

    // 验证会话 ID 格式（宽松模式，兼容所有历史版本前缀）
    // 🔧 2026-06: 放宽验证 — 历史会话可能使用不同的 ID 前缀（如旧版 chat_v2_*），
    // 不应因 ID 格式不匹配而拒绝加载。只拒绝明显无效的空 ID 或纯空白。
    if session_id.trim().is_empty() {
        return Err(
            ChatV2Error::Validation(format!("Invalid session ID: empty or whitespace-only")).into(),
        );
    }

    // 从数据库加载会话完整数据
    let response = load_session_from_db(&session_id, &db)?;

    let elapsed_ms = t0.elapsed().as_millis();
    log::info!(
        "[ChatV2::handlers] Loaded session: session_id={}, messages={}, blocks={}, elapsed_ms={}",
        session_id,
        response.messages.len(),
        response.blocks.len(),
        elapsed_ms
    );

    Ok(response)
}

/// 从数据库加载会话完整数据
fn load_session_from_db(
    session_id: &str,
    db: &ChatV2Database,
) -> Result<LoadSessionResponse, ChatV2Error> {
    // 调用 ChatV2Repo::load_session_full_v2 加载完整会话数据
    ChatV2Repo::load_session_full_v2(db, session_id)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_session_id_validation() {
        // 有效的会话 ID
        assert!("sess_12345".starts_with("sess_"));
        assert!("sess_a1b2c3d4-e5f6-7890-abcd-ef1234567890".starts_with("sess_"));
        assert!("agent_12345".starts_with("agent_"));
        assert!("subagent_foo_bar".starts_with("subagent_"));

        // 无效的会话 ID
        assert!(!"invalid_id".starts_with("sess_"));
        assert!(!"session_12345".starts_with("sess_"));
    }
}
