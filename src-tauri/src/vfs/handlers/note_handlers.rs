//! VFS 笔记操作 Tauri 命令处理器
//!
//! 提供笔记 CRUD 的 Tauri 命令。
//!
//! ## 命令
//! - `vfs_create_note`: 创建笔记
//! - `vfs_update_note`: 更新笔记
//! - `vfs_get_note`: 获取笔记
//! - `vfs_get_note_content`: 获取笔记内容
//! - `vfs_list_notes`: 列出笔记
//! - `vfs_delete_note`: 删除笔记

use std::sync::Arc;

use serde::Deserialize;
use tauri::State;

use crate::vfs::database::VfsDatabase;
use crate::vfs::error::{VfsError, VfsResult};
use crate::vfs::repos::VfsNoteRepo;
use crate::vfs::types::*;

use super::resource_handlers::{validate_id_format, ListInput};

// ============================================================================
// 前端输入类型
// ============================================================================

/// 创建笔记输入参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNoteInput {
    /// 标题
    pub title: String,

    /// 内容
    pub content: String,

    /// 标签
    #[serde(default)]
    pub tags: Vec<String>,
}

/// 更新笔记输入参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNoteInput {
    /// 新内容
    pub content: String,

    /// 新标题（可选）
    #[serde(default)]
    pub title: Option<String>,

    /// 新标签（可选）
    #[serde(default)]
    pub tags: Option<Vec<String>>,

    /// 乐观锁：调用方上次读取时的 `updated_at` 值（可选）
    ///
    /// ★ S-002 修复：传入后启用并发冲突检测，不传则向后兼容。
    #[serde(default)]
    pub expected_updated_at: Option<String>,
}

// ============================================================================
// 笔记操作命令
// ============================================================================

/// 创建笔记
///
/// 自动创建 resource 存储内容。
///
/// ## 参数
/// - `params`: 创建笔记的参数
///
/// ## 返回
/// - `Ok(VfsNote)`: 创建的笔记
/// - `Err(String)`: 数据库错误
#[tauri::command]
pub async fn vfs_create_note(
    params: CreateNoteInput,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<VfsNote> {
    log::info!("[VFS::handlers] vfs_create_note: title={}", params.title);

    // M-010: 校验内容长度，防止超大内容造成 DB 膨胀
    const MAX_NOTE_SIZE: usize = 5 * 1024 * 1024; // 5MB
    if params.content.len() > MAX_NOTE_SIZE {
        // M-015: 使用结构化错误码，让前端 toVfsError 能正确识别为 VALIDATION 错误
        return Err(VfsError::InvalidArgument {
            param: "content".to_string(),
            reason: format!(
                "笔记内容大小超出限制（最大 {}MB）",
                MAX_NOTE_SIZE / 1024 / 1024
            ),
        });
    }

    // 验证标题
    if params.title.trim().is_empty() {
        return Err(VfsError::InvalidArgument {
            param: "title".to_string(),
            reason: "Title cannot be empty".to_string(),
        });
    }

    // 调用 VfsNoteRepo::create_note
    let create_params = VfsCreateNoteParams {
        title: params.title,
        content: params.content,
        tags: params.tags,
    };
    let note = VfsNoteRepo::create_note(&vfs_db, create_params)?;

    log::info!("[VFS::handlers] Note created: id={}", note.id);
    Ok(note)
}

/// 更新笔记
///
/// 自动处理资源管理：
/// 1. 计算新内容的哈希
/// 2. 若 hash 不同，创建新 resource
/// 3. 更新 notes.resource_id
///
/// ## 参数
/// - `id`: 笔记 ID
/// - `params`: 更新参数
///
/// ## 返回
/// - `Ok(VfsNote)`: 更新后的笔记
/// - `Err(String)`: 笔记不存在或数据库错误
#[tauri::command]
pub async fn vfs_update_note(
    id: String,
    params: UpdateNoteInput,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<VfsNote> {
    log::info!(
        "[VFS::handlers] vfs_update_note: id={}, content_len={}",
        id,
        params.content.len()
    );

    // M-010: 校验内容长度，防止超大内容造成 DB 膨胀
    const MAX_NOTE_SIZE: usize = 5 * 1024 * 1024; // 5MB
    if params.content.len() > MAX_NOTE_SIZE {
        // M-015: 使用结构化错误码，让前端 toVfsError 能正确识别为 VALIDATION 错误
        return Err(VfsError::InvalidArgument {
            param: "content".to_string(),
            reason: format!(
                "笔记内容大小超出限制（最大 {}MB）",
                MAX_NOTE_SIZE / 1024 / 1024
            ),
        });
    }

    // 验证笔记 ID 格式
    validate_id_format(&id, "note_", "id")?;

    // 调用 VfsNoteRepo::update_note
    let update_params = VfsUpdateNoteParams {
        content: Some(params.content),
        title: params.title,
        tags: params.tags,
        expected_updated_at: params.expected_updated_at,
    };
    let note = VfsNoteRepo::update_note(&vfs_db, &id, update_params)?;

    log::info!("[VFS::handlers] Note updated: id={}", note.id);
    Ok(note)
}

/// 获取笔记
///
/// ## 参数
/// - `id`: 笔记 ID
///
/// ## 返回
/// - `Ok(Some(VfsNote))`: 找到笔记
/// - `Ok(None)`: 笔记不存在
/// - `Err(String)`: 数据库错误
#[tauri::command]
pub async fn vfs_get_note(
    id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Option<VfsNote>> {
    log::debug!("[VFS::handlers] vfs_get_note: id={}", id);

    // 验证笔记 ID 格式
    validate_id_format(&id, "note_", "id")?;

    // 调用 VfsNoteRepo::get_note
    Ok(VfsNoteRepo::get_note(&vfs_db, &id)?)
}

/// 获取笔记内容
///
/// 从 resources.data 获取笔记内容。
///
/// ## 参数
/// - `id`: 笔记 ID
///
/// ## 返回
/// - `Ok(Some(String))`: 笔记内容
/// - `Ok(None)`: 笔记不存在
/// - `Err(String)`: 数据库错误
#[tauri::command]
pub async fn vfs_get_note_content(
    id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Option<String>> {
    log::debug!("[VFS::handlers] vfs_get_note_content: id={}", id);

    // 验证笔记 ID 格式
    validate_id_format(&id, "note_", "id")?;

    // 调用 VfsNoteRepo::get_note_content
    Ok(VfsNoteRepo::get_note_content(&vfs_db, &id)?)
}

/// 列出笔记
///
/// ## 参数
/// - `params`: 列表参数
///
/// ## 返回
/// - `Ok(Vec<VfsNote>)`: 笔记列表
/// - `Err(String)`: 数据库错误
#[tauri::command]
pub async fn vfs_list_notes(
    params: Option<ListInput>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Vec<VfsNote>> {
    let params = params.unwrap_or_default();
    log::debug!(
        "[VFS::handlers] vfs_list_notes: search={:?}, limit={}, offset={}",
        params.search,
        params.limit,
        params.offset
    );

    VfsNoteRepo::list_notes(
        &vfs_db,
        params.search.as_deref(),
        params.limit,
        params.offset,
    )
}

/// 删除笔记
///
/// 软删除：设置 deleted_at 字段。
///
/// ## 参数
/// - `id`: 笔记 ID
///
/// ## 返回
/// - `Ok(())`: 成功
/// - `Err(String)`: 笔记不存在或数据库错误
#[tauri::command]
pub async fn vfs_delete_note(
    id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<()> {
    log::info!("[VFS::handlers] vfs_delete_note: id={}", id);

    // 验证笔记 ID 格式
    validate_id_format(&id, "note_", "id")?;

    // 保持 notes 与 folder_items 软删除一致
    Ok(VfsNoteRepo::delete_note_with_folder_item(&vfs_db, &id)?)
}
