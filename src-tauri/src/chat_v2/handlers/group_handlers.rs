//! 会话分组命令处理器
//!
//! 提供会话分组的 CRUD、排序、会话移动等功能。

use std::sync::Arc;

use tauri::State;

use crate::chat_v2::database::ChatV2Database;
use crate::chat_v2::error::{ChatV2Error, ChatV2Result};
use crate::chat_v2::repo::ChatV2Repo;
use crate::chat_v2::types::{CreateGroupRequest, PersistStatus, SessionGroup, UpdateGroupRequest};

/// 创建分组
#[tauri::command]
pub async fn chat_v2_create_group(
    request: CreateGroupRequest,
    db: State<'_, Arc<ChatV2Database>>,
) -> ChatV2Result<SessionGroup> {
    let conn = db.get_conn_safe()?;

    let existing =
        ChatV2Repo::list_groups_with_conn(&conn, Some("active"), request.workspace_id.as_deref())?;
    let next_sort = existing.iter().map(|g| g.sort_order).max().unwrap_or(0) + 1;

    let now = chrono::Utc::now();
    let group = SessionGroup {
        id: SessionGroup::generate_id(),
        name: request.name,
        description: request.description,
        icon: request.icon,
        color: request.color,
        system_prompt: request.system_prompt,
        default_skill_ids: request.default_skill_ids.unwrap_or_default(),
        pinned_resource_ids: request.pinned_resource_ids.unwrap_or_default(),
        workspace_id: request.workspace_id,
        sort_order: next_sort,
        persist_status: PersistStatus::Active,
        created_at: now,
        updated_at: now,
    };

    ChatV2Repo::create_group_with_conn(&conn, &group)?;
    Ok(group)
}

/// 更新分组
#[tauri::command]
pub async fn chat_v2_update_group(
    group_id: String,
    request: UpdateGroupRequest,
    db: State<'_, Arc<ChatV2Database>>,
) -> ChatV2Result<SessionGroup> {
    let conn = db.get_conn_safe()?;
    let existing = ChatV2Repo::get_group_with_conn(&conn, &group_id)?
        .ok_or_else(|| ChatV2Error::GroupNotFound(group_id.clone()))?;

    let now = chrono::Utc::now();

    fn merge_optional_string(
        request_val: Option<String>,
        existing_val: Option<String>,
    ) -> Option<String> {
        match request_val {
            None => existing_val,
            Some(s) if s.trim().is_empty() => None,
            Some(s) => Some(s),
        }
    }

    let updated = SessionGroup {
        id: existing.id,
        name: request.name.unwrap_or(existing.name),
        description: merge_optional_string(request.description, existing.description),
        icon: merge_optional_string(request.icon, existing.icon),
        color: merge_optional_string(request.color, existing.color),
        system_prompt: merge_optional_string(request.system_prompt, existing.system_prompt),
        default_skill_ids: request.default_skill_ids.unwrap_or(existing.default_skill_ids),
        pinned_resource_ids: request.pinned_resource_ids.unwrap_or(existing.pinned_resource_ids),
        workspace_id: merge_optional_string(request.workspace_id, existing.workspace_id),
        sort_order: request.sort_order.unwrap_or(existing.sort_order),
        persist_status: request.persist_status.unwrap_or(existing.persist_status),
        created_at: existing.created_at,
        updated_at: now,
    };

    ChatV2Repo::update_group_with_conn(&conn, &updated)?;
    Ok(updated)
}

/// 删除分组（软删除）
#[tauri::command]
pub async fn chat_v2_delete_group(
    group_id: String,
    db: State<'_, Arc<ChatV2Database>>,
) -> ChatV2Result<()> {
    let mut conn = db.get_conn_safe()?;
    ChatV2Repo::soft_delete_group_with_conn(&mut conn, &group_id)?;
    Ok(())
}

/// 获取分组详情
#[tauri::command]
pub async fn chat_v2_get_group(
    group_id: String,
    db: State<'_, Arc<ChatV2Database>>,
) -> ChatV2Result<Option<SessionGroup>> {
    let conn = db.get_conn_safe()?;
    let group = ChatV2Repo::get_group_with_conn(&conn, &group_id)?;
    Ok(group)
}

/// 列出分组
#[tauri::command]
pub async fn chat_v2_list_groups(
    status: Option<String>,
    workspace_id: Option<String>,
    db: State<'_, Arc<ChatV2Database>>,
) -> ChatV2Result<Vec<SessionGroup>> {
    let conn = db.get_conn_safe()?;
    let groups =
        ChatV2Repo::list_groups_with_conn(&conn, status.as_deref(), workspace_id.as_deref())?;
    Ok(groups)
}

/// 批量更新分组排序
#[tauri::command]
pub async fn chat_v2_reorder_groups(
    group_ids: Vec<String>,
    db: State<'_, Arc<ChatV2Database>>,
) -> ChatV2Result<()> {
    let mut conn = db.get_conn_safe()?;
    ChatV2Repo::reorder_groups_with_conn(&mut conn, &group_ids)?;
    Ok(())
}

/// 移动会话到分组
#[tauri::command]
pub async fn chat_v2_move_session_to_group(
    session_id: String,
    group_id: Option<String>,
    db: State<'_, Arc<ChatV2Database>>,
) -> ChatV2Result<()> {
    let conn = db.get_conn_safe()?;
    let normalized_group_id =
        group_id.and_then(|g| if g.trim().is_empty() { None } else { Some(g) });

    if let Some(ref gid) = normalized_group_id {
        let group = ChatV2Repo::get_group_with_conn(&conn, gid)?;
        match group {
            Some(g) if g.persist_status != PersistStatus::Active => {
                return Err(ChatV2Error::GroupNotFound(gid.clone()));
            }
            None => {
                return Err(ChatV2Error::GroupNotFound(gid.clone()));
            }
            _ => {}
        }
    }

    ChatV2Repo::update_session_group_with_conn(&conn, &session_id, normalized_group_id.as_deref())?;
    Ok(())
}
