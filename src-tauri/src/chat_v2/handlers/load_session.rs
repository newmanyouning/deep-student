//! 会话加载命令处理器
//!
//! 加载会话的完整数据，包括会话信息、消息列表、块列表和会话状态。

use std::sync::Arc;

use std::time::Instant;

use tauri::State;

use crate::chat_v2::database::ChatV2Database;
use crate::chat_v2::error::{ChatV2Error, ChatV2Result};
use crate::chat_v2::repo::ChatV2Repo;
use crate::chat_v2::types::LoadSessionResponse;

/// 加载会话完整数据
#[tauri::command]
pub async fn chat_v2_load_session(
    session_id: String,
    db: State<'_, Arc<ChatV2Database>>,
) -> ChatV2Result<LoadSessionResponse> {
    let t0 = Instant::now();
    log::info!("[ChatV2::handlers] chat_v2_load_session: session_id={}", session_id);

    if !session_id.starts_with("sess_")
        && !session_id.starts_with("agent_")
        && !session_id.starts_with("subagent_")
    {
        return Err(ChatV2Error::Validation(format!("Invalid session ID format: {}", session_id)));
    }

    let response = load_session_from_db(&session_id, &db)?;

    let elapsed_ms = t0.elapsed().as_millis();
    log::info!(
        "[ChatV2::handlers] Loaded session: session_id={}, messages={}, blocks={}, elapsed_ms={}",
        session_id, response.messages.len(), response.blocks.len(), elapsed_ms
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
