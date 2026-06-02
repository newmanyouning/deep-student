//! VFS 知识导图操作 Tauri 命令处理器
//!
//! 提供知识导图 CRUD 及收藏管理命令。

use std::sync::Arc;

use serde::Deserialize;
use tauri::State;

use crate::vfs::database::VfsDatabase;
use crate::vfs::error::{VfsError, VfsResult};
use crate::vfs::repos::VfsMindMapRepo;
use crate::vfs::types::*;

// ============================================================================
// 前端输入类型
// ============================================================================

/// 创建知识导图输入参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMindMapInput {
    /// 标题
    pub title: String,

    /// 描述
    #[serde(default)]
    pub description: Option<String>,

    /// 初始内容（MindMapDocument JSON）
    #[serde(default = "default_mindmap_content")]
    pub content: String,

    /// 默认视图
    #[serde(default = "default_mindmap_view")]
    pub default_view: String,

    /// 主题
    #[serde(default)]
    pub theme: Option<String>,

    /// 目标文件夹（可选）
    #[serde(default)]
    pub folder_id: Option<String>,
}

fn default_mindmap_content() -> String {
    r#"{"version":"1.0","root":{"id":"root","text":"根节点","children":[]}}"#.to_string()
}

fn default_mindmap_view() -> String {
    "mindmap".to_string()
}

/// 更新知识导图输入参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMindMapInput {
    /// 新标题
    #[serde(default)]
    pub title: Option<String>,

    /// 新描述
    #[serde(default)]
    pub description: Option<String>,

    /// 新内容（MindMapDocument JSON）
    #[serde(default)]
    pub content: Option<String>,

    /// 新默认视图
    #[serde(default)]
    pub default_view: Option<String>,

    /// 新主题
    #[serde(default)]
    pub theme: Option<String>,

    /// 新设置
    #[serde(default)]
    pub settings: Option<serde_json::Value>,

    /// 乐观并发控制：期望的 updatedAt（ISO8601）
    #[serde(default)]
    pub expected_updated_at: Option<String>,
}

// ============================================================================
// 知识导图操作命令
// ============================================================================

/// 创建知识导图
#[tauri::command]
pub async fn vfs_create_mindmap(
    params: CreateMindMapInput,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<VfsMindMap> {
    log::info!(
        "[VFS::handlers] vfs_create_mindmap: title={}, folder_id={:?}",
        params.title,
        params.folder_id
    );

    let create_params = VfsCreateMindMapParams {
        title: params.title,
        description: params.description,
        content: params.content,
        default_view: params.default_view,
        theme: params.theme,
    };

    if let Some(folder_id) = params.folder_id {
        VfsMindMapRepo::create_mindmap_in_folder(&vfs_db, create_params, Some(&folder_id))
    } else {
        VfsMindMapRepo::create_mindmap_in_folder(&vfs_db, create_params, None)
    }
}

/// 获取知识导图元数据
#[tauri::command]
pub async fn vfs_get_mindmap(
    mindmap_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Option<VfsMindMap>> {
    log::debug!("[VFS::handlers] vfs_get_mindmap: id={}", mindmap_id);

    if !mindmap_id.starts_with("mm_") {
        return Err(VfsError::InvalidArgument {
            param: "mindmap_id".to_string(),
            reason: format!("Invalid mindmap ID format: {}", mindmap_id),
        });
    }

    Ok(VfsMindMapRepo::get_mindmap(&vfs_db, &mindmap_id)?)
}

/// 获取知识导图内容
#[tauri::command]
pub async fn vfs_get_mindmap_content(
    mindmap_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Option<String>> {
    log::debug!("[VFS::handlers] vfs_get_mindmap_content: id={}", mindmap_id);

    if !mindmap_id.starts_with("mm_") {
        return Err(VfsError::InvalidArgument {
            param: "mindmap_id".to_string(),
            reason: format!("Invalid mindmap ID format: {}", mindmap_id),
        });
    }

    Ok(VfsMindMapRepo::get_mindmap_content(&vfs_db, &mindmap_id)?)
}

/// 获取思维导图的版本历史
#[tauri::command]
pub async fn vfs_get_mindmap_versions(
    mindmap_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Vec<VfsMindMapVersion>> {
    log::debug!(
        "[VFS::handlers] vfs_get_mindmap_versions: id={}",
        mindmap_id
    );

    if !mindmap_id.starts_with("mm_") {
        return Err(VfsError::InvalidArgument {
            param: "mindmap_id".to_string(),
            reason: format!("Invalid mindmap ID format: {}", mindmap_id),
        });
    }

    Ok(VfsMindMapRepo::get_versions(&vfs_db, &mindmap_id)?)
}

/// 获取指定版本的思维导图内容
#[tauri::command]
pub async fn vfs_get_mindmap_version_content(
    version_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Option<String>> {
    log::debug!(
        "[VFS::handlers] vfs_get_mindmap_version_content: id={}",
        version_id
    );

    if !version_id.starts_with("mv_") {
        return Err(VfsError::InvalidArgument {
            param: "version_id".to_string(),
            reason: format!("Invalid version ID format: {}", version_id),
        });
    }

    Ok(VfsMindMapRepo::get_version_content(&vfs_db, &version_id)?)
}

/// 获取指定版本的思维导图元数据
#[tauri::command]
pub async fn vfs_get_mindmap_version(
    version_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Option<VfsMindMapVersion>> {
    log::debug!("[VFS::handlers] vfs_get_mindmap_version: id={}", version_id);

    if !version_id.starts_with("mv_") {
        return Err(VfsError::InvalidArgument {
            param: "version_id".to_string(),
            reason: format!("Invalid version ID format: {}", version_id),
        });
    }

    Ok(VfsMindMapRepo::get_version(&vfs_db, &version_id)?)
}

/// 更新知识导图
#[tauri::command]
pub async fn vfs_update_mindmap(
    mindmap_id: String,
    params: UpdateMindMapInput,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<VfsMindMap> {
    log::info!("[VFS::handlers] vfs_update_mindmap: id={}", mindmap_id);

    if !mindmap_id.starts_with("mm_") {
        return Err(VfsError::InvalidArgument {
            param: "mindmap_id".to_string(),
            reason: format!("Invalid mindmap ID format: {}", mindmap_id),
        });
    }

    let update_params = VfsUpdateMindMapParams {
        title: params.title,
        description: params.description,
        content: params.content,
        default_view: params.default_view,
        theme: params.theme,
        settings: params.settings,
        expected_updated_at: params.expected_updated_at,
        version_source: Some("manual".to_string()),
    };

    Ok(VfsMindMapRepo::update_mindmap(&vfs_db, &mindmap_id, update_params)?)
}

/// 删除知识导图（软删除）
#[tauri::command]
pub async fn vfs_delete_mindmap(
    mindmap_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<()> {
    log::info!("[VFS::handlers] vfs_delete_mindmap: id={}", mindmap_id);

    if !mindmap_id.starts_with("mm_") {
        return Err(VfsError::InvalidArgument {
            param: "mindmap_id".to_string(),
            reason: format!("Invalid mindmap ID format: {}", mindmap_id),
        });
    }

    Ok(VfsMindMapRepo::delete_mindmap(&vfs_db, &mindmap_id)?)
}

/// 列出知识导图
#[tauri::command]
pub async fn vfs_list_mindmaps(
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Vec<VfsMindMap>> {
    log::debug!("[VFS::handlers] vfs_list_mindmaps");

    Ok(VfsMindMapRepo::list_mindmaps(&vfs_db)?)
}

/// 设置知识导图收藏状态
#[tauri::command]
pub async fn vfs_set_mindmap_favorite(
    mindmap_id: String,
    is_favorite: bool,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<()> {
    log::info!(
        "[VFS::handlers] vfs_set_mindmap_favorite: id={}, is_favorite={}",
        mindmap_id,
        is_favorite
    );

    if !mindmap_id.starts_with("mm_") {
        return Err(VfsError::InvalidArgument {
            param: "mindmap_id".to_string(),
            reason: format!("Invalid mindmap ID format: {}", mindmap_id),
        });
    }

    Ok(VfsMindMapRepo::set_favorite(&vfs_db, &mindmap_id, is_favorite)?)
}
