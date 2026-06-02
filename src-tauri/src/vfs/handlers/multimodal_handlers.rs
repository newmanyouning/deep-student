//! VFS 多模态索引 Tauri 命令处理器
//!
//! 提供多模态索引、检索、统计等命令。

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

use crate::llm_manager::LLMManager;
use crate::vfs::database::VfsDatabase;
use crate::vfs::error::VfsResult;
use crate::vfs::lance_store::VfsLanceStore;

// ============================================================================
// 前端输入/输出类型
// ============================================================================

/// 多模态索引页面输入参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VfsMultimodalIndexPageInput {
    /// 页面索引（0-based）
    pub page_index: i32,
    /// 图片 Base64 数据
    pub image_base64: Option<String>,
    /// 图片 MIME 类型
    pub image_mime: Option<String>,
    /// OCR 文本或 VLM 摘要
    pub text_content: Option<String>,
    /// 图片 Blob 哈希
    pub blob_hash: Option<String>,
}

/// 多模态索引输入参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VfsMultimodalIndexInput {
    /// 资源 ID
    pub resource_id: String,
    /// 资源类型
    pub resource_type: String,
    /// 文件夹 ID（可选）
    pub folder_id: Option<String>,
    /// 待索引的页面列表
    pub pages: Vec<VfsMultimodalIndexPageInput>,
}

/// 多模态索引结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VfsMultimodalIndexOutput {
    /// 成功索引的页面数
    pub indexed_pages: usize,
    /// 向量维度
    pub dimension: usize,
    /// 失败的页面索引列表
    pub failed_pages: Vec<i32>,
}

/// 多模态检索输入参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VfsMultimodalSearchInput {
    /// 查询文本
    pub query: String,
    /// 返回的最大结果数
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    /// 文件夹 ID 过滤
    pub folder_ids: Option<Vec<String>>,
    /// 资源类型过滤
    pub resource_types: Option<Vec<String>>,
}

fn default_top_k() -> usize {
    10
}

/// 多模态检索结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VfsMultimodalSearchOutput {
    /// 资源 ID
    pub resource_id: String,
    /// 资源类型
    pub resource_type: String,
    /// 页面索引
    pub page_index: i32,
    /// 文本内容
    pub text_content: Option<String>,
    /// 图片 Blob 哈希
    pub blob_hash: Option<String>,
    /// 相关度分数
    pub score: f32,
    /// 文件夹 ID
    pub folder_id: Option<String>,
}

// ============================================================================
// 多模态索引命令
// ============================================================================

/// 索引资源的多模态页面
///
/// ★ 2026-01: VFS 统一多模态索引
///
/// ## 参数
/// - `params`: 多模态索引输入参数
///
/// ## 返回
/// - `Ok(VfsMultimodalIndexOutput)`: 索引结果
/// - `Err(String)`: 索引失败
#[tauri::command]
pub async fn vfs_multimodal_index(
    params: VfsMultimodalIndexInput,
    app_handle: tauri::AppHandle,
    vfs_db: State<'_, Arc<VfsDatabase>>,
    llm_manager: State<'_, Arc<LLMManager>>,
    lance_store: State<'_, Arc<VfsLanceStore>>,
) -> VfsResult<VfsMultimodalIndexOutput> {
    use crate::multimodal::types::IndexProgressEvent;
    use crate::vfs::multimodal_service::{VfsMultimodalPage, VfsMultimodalService};
    use tokio::sync::mpsc;

    let lance_store = Arc::clone(lance_store.inner());

    // 创建多模态服务
    let service =
        VfsMultimodalService::new(Arc::clone(&vfs_db), Arc::clone(&llm_manager), lance_store);

    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<IndexProgressEvent>();
    let app_handle_clone = app_handle.clone();
    tokio::spawn(async move {
        while let Some(event) = progress_rx.recv().await {
            if let Ok(payload) = serde_json::to_value(&event) {
                if let Err(e) = app_handle_clone.emit("mm_index_progress", payload) {
                    log::warn!("Failed to emit mm_index_progress: {}", e);
                }
            }
        }
    });

    // 转换页面数据
    let pages: Vec<VfsMultimodalPage> = params
        .pages
        .into_iter()
        .map(|p| VfsMultimodalPage {
            page_index: p.page_index,
            image_base64: p.image_base64,
            image_mime: p.image_mime,
            text_content: p.text_content,
            blob_hash: p.blob_hash,
        })
        .collect();

    // 执行索引
    let result = service
        .index_resource_pages_with_progress(
            &params.resource_id,
            &params.resource_type,
            params.folder_id.as_deref(),
            pages,
            Some(progress_tx),
        )
        .await?;

    Ok(VfsMultimodalIndexOutput {
        indexed_pages: result.indexed_pages,
        dimension: result.dimension,
        failed_pages: result.failed_pages,
    })
}

/// 多模态向量检索
///
/// ★ 2026-01: VFS 统一多模态检索
///
/// ## 参数
/// - `params`: 多模态检索输入参数
///
/// ## 返回
/// - `Ok(Vec<VfsMultimodalSearchOutput>)`: 检索结果
/// - `Err(String)`: 检索失败
#[tauri::command]
pub async fn vfs_multimodal_search(
    params: VfsMultimodalSearchInput,
    vfs_db: State<'_, Arc<VfsDatabase>>,
    llm_manager: State<'_, Arc<LLMManager>>,
    lance_store: State<'_, Arc<VfsLanceStore>>,
) -> VfsResult<Vec<VfsMultimodalSearchOutput>> {
    use crate::vfs::multimodal_service::VfsMultimodalService;

    let lance_store = Arc::clone(lance_store.inner());

    // 创建多模态服务
    let service =
        VfsMultimodalService::new(Arc::clone(&vfs_db), Arc::clone(&llm_manager), lance_store);

    // 执行检索
    let results = service
        .search(
            &params.query,
            params.top_k,
            params.folder_ids.as_deref(),
            params.resource_types.as_deref(),
        )
        .await?;

    Ok(results
        .into_iter()
        .map(|r| VfsMultimodalSearchOutput {
            resource_id: r.resource_id,
            resource_type: r.resource_type,
            page_index: r.page_index,
            text_content: r.text_content,
            blob_hash: r.blob_hash,
            score: r.score,
            folder_id: r.folder_id,
        })
        .collect())
}

/// 获取 VFS 多模态索引统计
#[tauri::command]
pub async fn vfs_multimodal_stats(
    vfs_db: State<'_, Arc<VfsDatabase>>,
    llm_manager: State<'_, Arc<LLMManager>>,
    lance_store: State<'_, Arc<VfsLanceStore>>,
) -> VfsResult<serde_json::Value> {
    use crate::vfs::multimodal_service::VfsMultimodalService;

    let lance_store = Arc::clone(lance_store.inner());

    // 创建多模态服务
    let service =
        VfsMultimodalService::new(Arc::clone(&vfs_db), Arc::clone(&llm_manager), lance_store);

    let stats = service.get_stats().await?;

    Ok(serde_json::json!({
        "totalRecords": stats.total_records,
        "dimensions": stats.dimensions,
    }))
}

/// 删除资源的多模态索引
#[tauri::command]
pub async fn vfs_multimodal_delete(
    resource_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
    llm_manager: State<'_, Arc<LLMManager>>,
    lance_store: State<'_, Arc<VfsLanceStore>>,
) -> VfsResult<()> {
    use crate::vfs::multimodal_service::VfsMultimodalService;

    let lance_store = Arc::clone(lance_store.inner());

    let service =
        VfsMultimodalService::new(Arc::clone(&vfs_db), Arc::clone(&llm_manager), lance_store);

    service
        .delete_resource_index(&resource_id)
        .await
}

/// VFS 多模态索引资源（兼容旧 API）
///
/// ★ 2026-01: 兼容 mm_index_resource 的 VFS 版本
/// ★ P1-3 修复: 添加 mm_index_progress 事件发送
#[tauri::command]
pub async fn vfs_multimodal_index_resource(
    source_type: String,
    source_id: String,
    folder_id: Option<String>,
    force_rebuild: Option<bool>,
    app_handle: tauri::AppHandle,
    database: State<'_, Arc<crate::database::Database>>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
    llm_manager: State<'_, Arc<LLMManager>>,
    lance_store: State<'_, Arc<VfsLanceStore>>,
) -> VfsResult<serde_json::Value> {
    use crate::multimodal::types::IndexProgressEvent;
    use crate::vfs::multimodal_service::VfsMultimodalService;
    use tokio::sync::mpsc;

    let lance_store = Arc::clone(lance_store.inner());

    let service =
        VfsMultimodalService::new(Arc::clone(&vfs_db), Arc::clone(&llm_manager), lance_store);

    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<IndexProgressEvent>();
    let app_handle_clone = app_handle.clone();
    tokio::spawn(async move {
        while let Some(event) = progress_rx.recv().await {
            if let Ok(payload) = serde_json::to_value(&event) {
                if let Err(e) = app_handle_clone.emit("mm_index_progress", payload) {
                    log::warn!("Failed to emit mm_index_progress: {}", e);
                }
            }
        }
    });

    let result = service
        .index_resource_by_source_with_progress(
            Arc::clone(&database),
            &source_type,
            &source_id,
            folder_id.as_deref(),
            force_rebuild.unwrap_or(false),
            Some(progress_tx),
        )
        .await;

    let result = result?;

    Ok(serde_json::json!({
        "indexedPages": result.indexed_pages,
        "dimension": result.dimension,
        "failedPages": result.failed_pages,
    }))
}
