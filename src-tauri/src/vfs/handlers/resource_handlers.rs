//! VFS 资源操作 Tauri 命令处理器
//!
//! 提供资源 CRUD 及引用计数管理的 Tauri 命令。
//!
//! ## 命令
//! - `vfs_create_or_reuse`: 创建或复用资源
//! - `vfs_get_resource`: 获取资源
//! - `vfs_resource_exists`: 检查资源是否存在
//! - `vfs_increment_ref`: 增加引用计数
//! - `vfs_decrement_ref`: 减少引用计数

use std::sync::Arc;

use serde::Deserialize;
use tauri::State;

use crate::utils::unicode::sanitize_unicode;
use crate::vfs::database::VfsDatabase;
use crate::vfs::error::{VfsError, VfsResult};
use crate::vfs::repos::VfsResourceRepo;
use crate::vfs::types::*;

// ============================================================================
// 前端输入类型
// ============================================================================

/// 创建资源输入参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResourceInput {
    /// 资源类型（字符串）
    #[serde(rename = "type")]
    pub resource_type: String,

    /// 内容
    pub data: String,

    /// 原始数据 ID（可选）
    #[serde(default)]
    pub source_id: Option<String>,

    /// 元数据（可选）
    #[serde(default)]
    pub metadata: Option<VfsResourceMetadata>,
}

// ============================================================================
// 共享列表查询类型
// ============================================================================

/// 列表查询输入参数
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListInput {
    /// 搜索关键词（可选）
    #[serde(default)]
    pub search: Option<String>,

    /// 限制数量
    #[serde(default = "default_limit")]
    pub limit: u32,

    /// 偏移量
    #[serde(default)]
    pub offset: u32,
}

pub fn default_limit() -> u32 {
    50
}

/// 搜索所有资源输入参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchAllInput {
    /// 搜索关键词
    pub query: String,

    /// 类型过滤（可选）
    #[serde(default)]
    pub types: Option<Vec<String>>,

    /// 限制数量
    #[serde(default = "default_limit")]
    pub limit: u32,

    /// 偏移量
    #[serde(default)]
    pub offset: u32,
}

// ============================================================================
// ID 格式验证（共享工具函数）
// ============================================================================

/// ★ 2026-01 优化：ID 格式验证辅助函数，减少重复代码
/// ★ BE-06 安全修复：添加 Unicode 规范化，防止绕过攻击
#[inline]
pub fn validate_id_format(id: &str, prefix: &str, param_name: &str) -> VfsResult<()> {
    // 先进行 Unicode 规范化
    let sanitized = sanitize_unicode(id);

    // 检查是否与原始值不同（可能有绕过尝试）
    if sanitized != id {
        return Err(VfsError::InvalidArgument {
            param: param_name.to_string(),
            reason: "ID contains invalid Unicode characters".to_string(),
        });
    }

    if !id.starts_with(prefix) {
        return Err(VfsError::InvalidArgument {
            param: param_name.to_string(),
            reason: format!("Invalid {} format: {}", param_name, id),
        });
    }
    Ok(())
}

#[inline]
pub fn validate_id_format_any(id: &str, prefixes: &[&str], param_name: &str) -> VfsResult<()> {
    // 先进行 Unicode 规范化
    let sanitized = sanitize_unicode(id);

    if sanitized != id {
        return Err(VfsError::InvalidArgument {
            param: param_name.to_string(),
            reason: "ID contains invalid Unicode characters".to_string(),
        });
    }

    if !prefixes.iter().any(|p| id.starts_with(p)) {
        return Err(VfsError::InvalidArgument {
            param: param_name.to_string(),
            reason: format!("Invalid {} format: {}", param_name, id),
        });
    }

    Ok(())
}

// ============================================================================
// 文件大小验证（共享工具函数）
// ============================================================================

/// 获取资源类型的大文件限制（字节）
pub fn get_max_size_bytes(resource_type: &VfsResourceType) -> usize {
    match resource_type {
        VfsResourceType::Image => 10 * 1024 * 1024,       // 10MB
        VfsResourceType::File => 50 * 1024 * 1024,        // 50MB
        VfsResourceType::Note => 50 * 1024 * 1024,        // 50MB
        VfsResourceType::Retrieval => 10 * 1024 * 1024,   // 10MB
        VfsResourceType::Exam => 50 * 1024 * 1024,        // 50MB
        VfsResourceType::Textbook => 50 * 1024 * 1024,    // 50MB
        VfsResourceType::Translation => 10 * 1024 * 1024, // 10MB
        VfsResourceType::Essay => 10 * 1024 * 1024,       // 10MB
        VfsResourceType::MindMap => 50 * 1024 * 1024,     // 50MB
    }
}

/// 验证大文件限制
pub fn validate_file_size(resource_type: &VfsResourceType, data: &str) -> VfsResult<()> {
    let size = data.len();
    let max_size = get_max_size_bytes(resource_type);

    if size > max_size {
        let max_mb = max_size / (1024 * 1024);
        let actual_mb = size as f64 / (1024.0 * 1024.0);
        return Err(VfsError::InvalidArgument {
            param: "data".to_string(),
            reason: format!(
                "File too large: {} type max {}MB, got {:.2}MB",
                resource_type, max_mb, actual_mb
            ),
        });
    }

    Ok(())
}

/// 计算内容的 SHA-256 哈希
pub fn compute_hash(data: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

// ============================================================================
// 资源操作命令
// ============================================================================

/// 创建或复用资源
///
/// 基于内容哈希自动去重：
/// - 如果相同哈希的资源已存在，返回已有资源的 ID
/// - 如果不存在，创建新资源
///
/// ## 参数
/// - `params`: 创建资源的参数
///
/// ## 返回
/// - `Ok(VfsCreateResourceResult)`: 资源 ID、哈希和是否新创建
/// - `Err(String)`: 验证失败或数据库错误
#[tauri::command]
pub async fn vfs_create_or_reuse(
    params: CreateResourceInput,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<VfsCreateResourceResult> {
    log::info!(
        "[VFS::handlers] vfs_create_or_reuse: type={}, data_len={}, source_id={:?}",
        params.resource_type,
        params.data.len(),
        params.source_id
    );

    // 解析资源类型
    let resource_type = VfsResourceType::from_str(&params.resource_type).ok_or_else(|| {
        VfsError::InvalidArgument {
            param: "type".to_string(),
            reason: format!("Invalid resource type: {}", params.resource_type),
        }
        .to_string()
    })?;

    // 验证大文件限制
    validate_file_size(&resource_type, &params.data)?;

    // 调用 VfsResourceRepo::create_or_reuse
    let result = VfsResourceRepo::create_or_reuse(
        &vfs_db,
        resource_type,
        &params.data,
        params.source_id.as_deref(),
        None, // source_table
        params.metadata.as_ref(),
    )?;

    log::info!(
        "[VFS::handlers] Resource {}: id={}, hash={}, is_new={}",
        if result.is_new { "created" } else { "reused" },
        result.resource_id,
        &result.hash[..16],
        result.is_new
    );

    Ok(result)
}

/// 获取资源
///
/// ## 参数
/// - `resource_id`: 资源 ID
///
/// ## 返回
/// - `Ok(Some(VfsResource))`: 找到资源
/// - `Ok(None)`: 资源不存在
/// - `Err(String)`: 数据库错误
#[tauri::command]
pub async fn vfs_get_resource(
    resource_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Option<VfsResource>> {
    log::debug!("[VFS::handlers] vfs_get_resource: id={}", resource_id);

    // 验证资源 ID 格式
    validate_id_format(&resource_id, "res_", "resource_id")?;

    // 调用 VfsResourceRepo::get_resource
    Ok(VfsResourceRepo::get_resource(&vfs_db, &resource_id)?)
}

/// 检查资源是否存在
///
/// ## 参数
/// - `resource_id`: 资源 ID
///
/// ## 返回
/// - `Ok(true)`: 资源存在
/// - `Ok(false)`: 资源不存在
/// - `Err(String)`: 数据库错误
#[tauri::command]
pub async fn vfs_resource_exists(
    resource_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<bool> {
    log::debug!("[VFS::handlers] vfs_resource_exists: id={}", resource_id);

    // 验证资源 ID 格式
    if !resource_id.starts_with("res_") {
        return Ok(false);
    }

    // 调用 VfsResourceRepo::exists
    Ok(VfsResourceRepo::exists(&vfs_db, &resource_id)?)
}

/// 增加资源引用计数
///
/// 消息保存时调用，表示该资源被消息引用。
///
/// ## 参数
/// - `resource_id`: 资源 ID
///
/// ## 返回
/// - `Ok(())`: 成功
/// - `Err(String)`: 资源不存在或数据库错误
#[tauri::command]
pub async fn vfs_increment_ref(
    resource_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<()> {
    log::info!("[VFS::handlers] vfs_increment_ref: id={}", resource_id);

    // 验证资源 ID 格式
    validate_id_format(&resource_id, "res_", "resource_id")?;

    // 调用 VfsResourceRepo::increment_ref
    VfsResourceRepo::increment_ref(&vfs_db, &resource_id)
        .map(|_| ())
}

/// 减少资源引用计数
///
/// 消息删除时调用，表示该资源不再被消息引用。
///
/// ## 参数
/// - `resource_id`: 资源 ID
///
/// ## 返回
/// - `Ok(())`: 成功
/// - `Err(String)`: 资源不存在或数据库错误
#[tauri::command]
pub async fn vfs_decrement_ref(
    resource_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<()> {
    log::info!("[VFS::handlers] vfs_decrement_ref: id={}", resource_id);

    // 验证资源 ID 格式
    validate_id_format(&resource_id, "res_", "resource_id")?;

    // 调用 VfsResourceRepo::decrement_ref
    VfsResourceRepo::decrement_ref(&vfs_db, &resource_id)
        .map(|_| ())
}
