//! VFS 索引/搜索/检索 Tauri 命令处理器（handlers 子模块）
//!
//! 提供搜索、索引管理、维度配置、RAG 检索等命令。
//!
//! ## 命令分类
//! - **搜索**: vfs_search, vfs_search_all, vfs_rag_search
//! - **索引管理**: vfs_reindex_resource, vfs_get_index_status, vfs_toggle_index_disabled
//! - **批量索引**: vfs_batch_index_pending, vfs_set_indexing_config, vfs_get_indexing_config
//! - **嵌入维度**: vfs_get_embedding_stats, vfs_list_dimensions, vfs_create_dimension, vfs_delete_dimension
//! - **默认维度**: vfs_set_default_embedding_dimension, vfs_get_default_embedding_dimension, vfs_clear_default_embedding_dimension
//! - **索引状态**: vfs_get_all_index_status, vfs_get_pending_resources
//! - **列表**: vfs_list_textbooks, vfs_list_exam_sheets, vfs_list_translations, vfs_list_essays
//! - **LanceDB**: vfs_get_lance_stats, vfs_optimize_lance

use std::sync::Arc;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::database::Database;
use crate::llm_manager::LLMManager;
use crate::vfs::database::VfsDatabase;
use crate::vfs::embedding_service::EmbeddingProgressCallback;
use crate::vfs::error::{VfsError, VfsResult};
use crate::vfs::indexing::{
    VfsEmbeddingStats, VfsFullIndexingService, VfsIndexingService, VfsSearchParams,
    VfsSearchResult, VfsSearchService,
};
use crate::vfs::lance_store::VfsLanceStore;
use crate::vfs::repos::{
    embedding_dim_repo, VfsEssayRepo, VfsExamRepo, VfsIndexStateRepo, VfsIndexingConfigRepo,
    VfsTextbookRepo, VfsTranslationRepo, INDEX_STATE_DISABLED, INDEX_STATE_PENDING,
    MODALITY_TEXT,
};
use crate::vfs::types::*;

use super::resource_handlers::{ListInput, SearchAllInput};

// ============================================================================
// 工具函数
// ============================================================================

/// 解析 ISO 8601 时间字符串为毫秒时间戳
fn parse_timestamp(s: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.timestamp_millis())
        .unwrap_or(0)
}

/// 检查 resources 表是否有 index_state 列
fn has_index_state_column(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('resources') WHERE name = 'index_state'",
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|c| c > 0)
    .unwrap_or(false)
}

/// 检查 vfs_index_units 表是否存在（统一索引架构）
fn has_vfs_index_units_table(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='vfs_index_units'",
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|c| c > 0)
    .unwrap_or(false)
}

// ============================================================================
// 列表操作命令（供 Learning Hub 调用）
// ============================================================================

/// 列出教材
#[tauri::command]
pub async fn vfs_list_textbooks(
    params: Option<ListInput>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Vec<VfsTextbook>> {
    let params = params.unwrap_or_default();
    log::debug!(
        "[VFS::handlers] vfs_list_textbooks: search={:?}, limit={}, offset={}",
        params.search,
        params.limit,
        params.offset
    );

    if let Some(search) = params
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        VfsTextbookRepo::search_textbooks(&vfs_db, search, params.limit, params.offset)
    } else {
        VfsTextbookRepo::list_textbooks(&vfs_db, params.limit, params.offset)
    }
}

/// 列出题目集识别
#[tauri::command]
pub async fn vfs_list_exam_sheets(
    params: Option<ListInput>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Vec<VfsExamSheet>> {
    let params = params.unwrap_or_default();
    log::debug!(
        "[VFS::handlers] vfs_list_exam_sheets: search={:?}, limit={}, offset={}",
        params.search,
        params.limit,
        params.offset
    );

    VfsExamRepo::list_exam_sheets(
        &vfs_db,
        params.search.as_deref(),
        params.limit,
        params.offset,
    )
}

/// 列出翻译
#[tauri::command]
pub async fn vfs_list_translations(
    params: Option<ListInput>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Vec<VfsTranslation>> {
    let params = params.unwrap_or_default();
    log::debug!(
        "[VFS::handlers] vfs_list_translations: search={:?}, limit={}, offset={}",
        params.search,
        params.limit,
        params.offset
    );

    VfsTranslationRepo::list_translations(
        &vfs_db,
        params.search.as_deref(),
        params.limit,
        params.offset,
    )
}

/// 列出作文
#[tauri::command]
pub async fn vfs_list_essays(
    params: Option<ListInput>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Vec<VfsEssay>> {
    let params = params.unwrap_or_default();
    log::debug!(
        "[VFS::handlers] vfs_list_essays: search={:?}, limit={}, offset={}",
        params.search,
        params.limit,
        params.offset
    );

    VfsEssayRepo::list_essays(
        &vfs_db,
        params.search.as_deref(),
        params.limit,
        params.offset,
    )
}

/// 搜索所有资源
///
/// 跨类型全文搜索。
#[tauri::command]
pub async fn vfs_search_all(
    params: SearchAllInput,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Vec<VfsListItem>> {
    log::debug!(
        "[VFS::handlers] vfs_search_all: query={}, types={:?}, limit={}, offset={}",
        params.query,
        params.types,
        params.limit,
        params.offset
    );

    // 验证查询词
    if params.query.trim().is_empty() {
        return Err(VfsError::InvalidArgument {
            param: "query".to_string(),
            reason: "Search query cannot be empty".to_string(),
        });
    }

    let types = params.types.as_ref();
    let search_limit = params.limit.min(50); // 每种类型最多搜索 50 条

    // 根据 types 过滤要搜索的类型
    let search_notes = types.is_none() || types.unwrap().iter().any(|t| t == "note");
    let search_exams = types.is_none() || types.unwrap().iter().any(|t| t == "exam");
    let search_translations = types.is_none() || types.unwrap().iter().any(|t| t == "translation");
    let search_essays = types.is_none() || types.unwrap().iter().any(|t| t == "essay");

    // ★ 2026-01 优化：并行搜索多种类型，提升响应速度
    let vfs_db_clone = Arc::clone(&vfs_db);
    let query_clone = params.query.clone();

    // 使用 tokio::task::spawn_blocking 并行执行同步搜索
    let notes_handle = if search_notes {
        let db = Arc::clone(&vfs_db_clone);
        let q = query_clone.clone();
        Some(tokio::task::spawn_blocking(move || {
            crate::vfs::repos::VfsNoteRepo::list_notes(&db, Some(&q), search_limit, 0)
                .map_err(|e| {
                    tracing::warn!(
                        "[VFS::handlers] Note search failed for query '{}': {}",
                        q,
                        e
                    );
                    e
                })
                .ok()
        }))
    } else {
        None
    };

    let exams_handle = if search_exams {
        let db = Arc::clone(&vfs_db_clone);
        let q = query_clone.clone();
        Some(tokio::task::spawn_blocking(move || {
            VfsExamRepo::search_exam_sheets(&db, &q, search_limit)
                .map_err(|e| {
                    tracing::warn!(
                        "[VFS::handlers] Exam search failed for query '{}': {}",
                        q,
                        e
                    );
                    e
                })
                .ok()
        }))
    } else {
        None
    };

    let translations_handle = if search_translations {
        let db = Arc::clone(&vfs_db_clone);
        let q = query_clone.clone();
        Some(tokio::task::spawn_blocking(move || {
            VfsTranslationRepo::search_translations(&db, &q, search_limit)
                .map_err(|e| {
                    tracing::warn!(
                        "[VFS::handlers] Translation search failed for query '{}': {}",
                        q,
                        e
                    );
                    e
                })
                .ok()
        }))
    } else {
        None
    };

    let essays_handle = if search_essays {
        let db = Arc::clone(&vfs_db_clone);
        let q = query_clone.clone();
        Some(tokio::task::spawn_blocking(move || {
            VfsEssayRepo::search_essays(&db, &q, search_limit)
                .map_err(|e| {
                    tracing::warn!(
                        "[VFS::handlers] Essay search failed for query '{}': {}",
                        q,
                        e
                    );
                    e
                })
                .ok()
        }))
    } else {
        None
    };

    // 收集结果
    let mut results: Vec<VfsListItem> = Vec::new();

    // 笔记结果
    if let Some(handle) = notes_handle {
        if let Ok(Some(notes)) = handle.await {
            results.extend(notes.into_iter().map(|n| VfsListItem {
                id: n.id,
                resource_id: n.resource_id,
                resource_type: VfsResourceType::Note,
                title: n.title,
                preview_type: PreviewType::Markdown,
                created_at: parse_timestamp(&n.created_at),
                updated_at: Some(parse_timestamp(&n.updated_at)),
                metadata: None,
            }));
        }
    }

    // 题目集结果
    if let Some(handle) = exams_handle {
        if let Ok(Some(exams)) = handle.await {
            results.extend(exams.into_iter().map(|e| VfsListItem {
                id: e.id,
                resource_id: e.resource_id.unwrap_or_default(),
                resource_type: VfsResourceType::Exam,
                title: e.exam_name.unwrap_or_else(|| "未命名题目集".to_string()),
                preview_type: PreviewType::Card,
                created_at: parse_timestamp(&e.created_at),
                updated_at: Some(parse_timestamp(&e.updated_at)),
                metadata: None,
            }));
        }
    }

    // 翻译结果
    if let Some(handle) = translations_handle {
        if let Ok(Some(translations)) = handle.await {
            results.extend(translations.into_iter().map(|t| VfsListItem {
                id: t.id,
                resource_id: t.resource_id,
                resource_type: VfsResourceType::Translation,
                title: format!("翻译 ({}→{})", t.src_lang, t.tgt_lang),
                preview_type: PreviewType::Card,
                created_at: parse_timestamp(&t.created_at),
                updated_at: None,
                metadata: None,
            }));
        }
    }

    // 作文结果
    if let Some(handle) = essays_handle {
        if let Ok(Some(essays)) = handle.await {
            results.extend(essays.into_iter().map(|e| VfsListItem {
                id: e.id,
                resource_id: e.resource_id,
                resource_type: VfsResourceType::Essay,
                title: e.title.unwrap_or_else(|| "未命名作文".to_string()),
                preview_type: PreviewType::Markdown,
                created_at: parse_timestamp(&e.created_at),
                updated_at: Some(parse_timestamp(&e.updated_at)),
                metadata: None,
            }));
        }
    }

    // 按 updated_at 排序（降序），优先显示最近更新的
    results.sort_by(|a, b| {
        let a_time = a.updated_at.unwrap_or(a.created_at);
        let b_time = b.updated_at.unwrap_or(b.created_at);
        b_time.cmp(&a_time)
    });

    // 应用全局分页语义（先 offset 后 limit）
    let offset = params.offset as usize;
    if offset >= results.len() {
        results.clear();
    } else if offset > 0 {
        results = results.into_iter().skip(offset).collect();
    }
    results.truncate(params.limit as usize);

    log::info!(
        "[VFS::handlers] vfs_search_all: found {} results for query '{}'",
        results.len(),
        params.query
    );

    Ok(results)
}

// ============================================================================
// 索引搜索命令
// ============================================================================

#[tauri::command]
pub async fn vfs_search(
    params: VfsSearchParams,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Vec<VfsSearchResult>> {
    log::info!(
        "[VFS::handlers] vfs_search: query={}, top_k={}",
        params.query,
        params.top_k
    );

    if params.query.trim().is_empty() {
        return Err(VfsError::Other("Query cannot be empty".to_string()));
    }

    let search_service = VfsSearchService::new(Arc::clone(&vfs_db));
    search_service
        .search_fts(&params.query, params.top_k)
}

// ============================================================================
// 索引管理命令
// ============================================================================

#[tauri::command]
pub async fn vfs_reindex_resource(
    resource_id: String,
    embedding_dim: Option<i32>,
    app_handle: AppHandle,
    llm_manager: State<'_, Arc<LLMManager>>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
    lance_store: State<'_, Arc<VfsLanceStore>>,
) -> VfsResult<usize> {
    log::info!("[VFS::handlers] vfs_reindex_resource: id={}", resource_id);

    if !resource_id.starts_with("res_") {
        return Err(VfsError::Other(format!("Invalid resource ID format: {}", resource_id)));
    }

    // ★ 2026-02 修复：并发防护 - 检查资源是否正在索引中，避免重复执行
    {
        let conn = vfs_db.get_conn_safe()?;
        let current_state: Option<String> = conn
            .query_row(
                "SELECT index_state FROM resources WHERE id = ?1",
                rusqlite::params![resource_id],
                |row| row.get(0),
            )
            .ok();
        if current_state.as_deref() == Some("indexing") {
            log::warn!(
                "[VFS::handlers] vfs_reindex_resource: resource {} is already indexing, skipping",
                resource_id
            );
            return Err(VfsError::Other("资源正在索引中，请等待完成后再试".to_string()));
        }
    }

    // 发送开始事件
    if let Err(e) = app_handle.emit(
        "vfs-index-progress",
        serde_json::json!({
            "type": "started",
            "resourceId": resource_id,
            "message": "开始索引资源..."
        }),
    ) {
        log::warn!("[VFS::handlers] Failed to emit index progress: {}", e);
    }

    if embedding_dim.is_some() {
        log::warn!(
            "[VFS::handlers] vfs_reindex_resource: embedding_dim ignored (full indexing uses model config)"
        );
    }

    let mut indexing_service = VfsFullIndexingService::new(
        Arc::clone(&vfs_db),
        Arc::clone(&llm_manager),
        Arc::clone(lance_store.inner()),
    )?;
    // ★ 2026-02-19：传递 AppHandle，使 try_auto_ocr 能发送细粒度进度事件
    indexing_service.set_app_handle(app_handle.clone());

    // ★ 构造嵌入进度回调，上报单资源索引的嵌入批次进度
    let cb_handle = app_handle.clone();
    let cb_resource_id = resource_id.clone();
    let progress_callback: Option<EmbeddingProgressCallback> =
        Some(Box::new(move |chunks_done: usize, chunks_total: usize| {
            let progress = if chunks_total > 0 {
                ((chunks_done as f64 / chunks_total as f64) * 100.0).min(99.0) as u32
            } else {
                0
            };
            if let Err(e) = cb_handle.emit(
                "vfs-index-progress",
                serde_json::json!({
                    "type": "embedding_progress",
                    "resourceId": cb_resource_id,
                    "chunksProcessed": chunks_done,
                    "chunksTotal": chunks_total,
                    "progress": progress,
                    "message": format!("正在生成嵌入 {}/{}", chunks_done, chunks_total)
                }),
            ) {
                log::warn!("[VFS::handlers] Failed to emit embedding progress: {}", e);
            }
        }));

    match indexing_service
        .reindex_resource(&resource_id, None, progress_callback)
        .await
    {
        Ok((chunk_count, _)) => {
            // 发送完成事件
            if let Err(e) = app_handle.emit(
                "vfs-index-progress",
                serde_json::json!({
                    "type": "completed",
                    "resourceId": resource_id,
                    "chunkCount": chunk_count,
                    "message": format!("索引完成，共 {} 个块", chunk_count)
                }),
            ) {
                log::warn!("[VFS::handlers] Failed to emit index progress: {}", e);
            }
            Ok(chunk_count)
        }
        Err(e) => {
            // 发送失败事件
            if let Err(emit_err) = app_handle.emit(
                "vfs-index-progress",
                serde_json::json!({
                    "type": "failed",
                    "resourceId": resource_id,
                    "error": e.to_string(),
                    "message": format!("索引失败: {}", e)
                }),
            ) {
                log::warn!("[VFS::handlers] Failed to emit failed event: {}", emit_err);
            }
            Err(VfsError::from(e.to_string()))
        }
    }
}

#[tauri::command]
pub async fn vfs_get_index_status(
    resource_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Option<crate::vfs::repos::IndexState>> {
    log::debug!("[VFS::handlers] vfs_get_index_status: id={}", resource_id);
    Ok(VfsIndexStateRepo::get_index_state(&vfs_db, &resource_id)?)
}

/// 切换资源的索引禁用状态
///
/// - 如果当前是 disabled，则恢复为 pending
/// - 如果当前不是 disabled，则设置为 disabled
#[tauri::command]
pub async fn vfs_toggle_index_disabled(
    resource_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<String> {
    log::info!(
        "[VFS::handlers] vfs_toggle_index_disabled: id={}",
        resource_id
    );

    // 获取当前状态
    let current_state =
        VfsIndexStateRepo::get_index_state(&vfs_db, &resource_id)?;

    let current = current_state
        .map(|s| s.state)
        .unwrap_or_else(|| INDEX_STATE_PENDING.to_string());

    let new_state = if current == INDEX_STATE_DISABLED {
        // 恢复为 pending
        VfsIndexStateRepo::mark_pending(&vfs_db, &resource_id)?;
        INDEX_STATE_PENDING
    } else {
        // 禁用索引
        VfsIndexStateRepo::mark_disabled(&vfs_db, &resource_id)?;
        INDEX_STATE_DISABLED
    };

    log::info!(
        "[VFS::handlers] vfs_toggle_index_disabled: {} -> {}",
        current,
        new_state
    );
    Ok(new_state.to_string())
}

#[tauri::command]
pub async fn vfs_get_embedding_stats(
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<VfsEmbeddingStats> {
    log::debug!("[VFS::handlers] vfs_get_embedding_stats");
    let search_service = VfsSearchService::new(Arc::clone(&vfs_db));
    search_service
        .get_embedding_stats()
}

#[tauri::command]
pub async fn vfs_list_dimensions(
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Vec<crate::vfs::repos::VfsEmbeddingDimension>> {
    log::debug!("[VFS::handlers] vfs_list_dimensions");
    let conn = vfs_db.get_conn()?;
    let dims = embedding_dim_repo::list_all(&conn)?;
    Ok(dims
        .into_iter()
        .map(|d| crate::vfs::repos::VfsEmbeddingDimension {
            dimension: d.dimension,
            modality: d.modality,
            record_count: d.record_count,
            lance_table_name: d.lance_table_name,
            created_at: d.created_at,
            last_used_at: d.last_used_at,
            model_config_id: d.model_config_id,
            model_name: d.model_name,
        })
        .collect())
}

#[tauri::command]
pub async fn vfs_get_pending_resources(
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Vec<String>> {
    log::debug!("[VFS::handlers] vfs_get_pending_resources");
    let indexing_service = VfsIndexingService::new(Arc::clone(&vfs_db));
    indexing_service
        .get_pending_resources()
}

// ============================================================================
// 嵌入维度管理命令
// ============================================================================

/// 为维度分配模型（用于跨维度检索）
///
/// 模型分配是配置项，不是数据绑定。用户可以随时更改维度使用的模型。
/// 更改后，跨维度检索时会使用新分配的模型生成查询向量。
///
/// 如果该维度是当前的默认嵌入维度，会同步更新 settings 中的模型配置ID
#[tauri::command]
pub async fn vfs_assign_dimension_model(
    dimension: i32,
    modality: String,
    model_config_id: String,
    model_name: String,
    database: State<'_, Arc<Database>>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<bool> {
    log::info!(
        "[VFS::handlers] vfs_assign_dimension_model: dim={}, modality={}, model={}",
        dimension,
        modality,
        model_config_id
    );

    // ★ 审计修复：统一使用 embedding_dim_repo（替代已废弃的 VfsDimensionRepo）
    let conn = vfs_db.get_conn()?;
    let existing = embedding_dim_repo::get_by_key(&conn, dimension, &modality)?;
    if existing.is_none() {
        return Err(VfsError::Other(format!("维度 {}:{} 不存在", dimension, modality)));
    }
    embedding_dim_repo::register_with_model(
        &conn,
        dimension,
        &modality,
        Some(&model_config_id),
        Some(&model_name),
    )?;
    drop(conn);

    // 检查该维度是否是当前的默认嵌入维度，如果是则同步更新 settings 中的模型配置ID
    let (dim_key, model_key) = match modality.as_str() {
        "text" => (
            "embedding.default_text_dimension",
            "embedding.default_text_model_config_id",
        ),
        "multimodal" => (
            "embedding.default_multimodal_dimension",
            "embedding.default_multimodal_model_config_id",
        ),
        _ => return Ok(true), // 未知模态，跳过默认设置检查
    };

    // 读取当前默认维度
    if let Ok(Some(default_dim_str)) = database.get_setting(dim_key) {
        if let Ok(default_dim) = default_dim_str.parse::<i32>() {
            if default_dim == dimension {
                // 该维度是默认维度，同步更新 settings 中的模型配置ID
                database
                    .save_setting(model_key, &model_config_id)?;
                log::info!(
                    "[VFS::handlers] 已同步更新默认 {} 嵌入模型: {}",
                    modality,
                    model_config_id
                );
            }
        }
    }

    Ok(true)
}

#[tauri::command]
pub async fn vfs_create_dimension(
    dimension: i32,
    modality: String,
    model_config_id: Option<String>,
    model_name: Option<String>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<embedding_dim_repo::VfsEmbeddingDim> {
    log::info!(
        "[VFS::handlers] vfs_create_dimension: dim={}, modality={}, model={:?}",
        dimension,
        modality,
        model_config_id
    );

    let conn = vfs_db.get_conn()?;
    embedding_dim_repo::create_dimension(
        &conn,
        dimension,
        &modality,
        model_config_id.as_deref(),
        model_name.as_deref(),
    )
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDimensionResult {
    pub deleted_segments: usize,
    pub dimension: i32,
    pub modality: String,
}

#[tauri::command]
pub async fn vfs_delete_dimension(
    dimension: i32,
    modality: String,
    database: State<'_, Arc<Database>>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
    lance_store: State<'_, Arc<VfsLanceStore>>,
) -> VfsResult<DeleteDimensionResult> {
    log::info!(
        "[VFS::handlers] vfs_delete_dimension: dim={}, modality={}",
        dimension,
        modality
    );

    // S8 fix: 检查是否有正在索引的 units 使用了该维度
    let conn = vfs_db.get_conn()?;
    let has_indexing = embedding_dim_repo::has_indexing_units_for_dimension(
        &conn, dimension, &modality,
    )?;
    if has_indexing {
        return Err(format!(
            "维度 {}:{} 有正在进行的索引任务，请等待索引完成后再删除",
            dimension, modality
        )
        .into());
    }

    // 检查是否正在删除默认维度，如果是则清除默认设置
    let (dim_key, model_key) = match modality.as_str() {
        "text" => (
            "embedding.default_text_dimension",
            "embedding.default_text_model_config_id",
        ),
        "multimodal" => (
            "embedding.default_multimodal_dimension",
            "embedding.default_multimodal_model_config_id",
        ),
        _ => ("", ""),
    };

    if !dim_key.is_empty() {
        if let Ok(Some(default_dim_str)) = database.get_setting(dim_key) {
            if let Ok(default_dim) = default_dim_str.parse::<i32>() {
                if default_dim == dimension {
                    // 正在删除默认维度，清除默认设置
                    let _ = database.delete_setting(dim_key);
                    let _ = database.delete_setting(model_key);
                    log::info!(
                        "[VFS::handlers] 已清除默认 {} 嵌入维度设置（因维度被删除）",
                        modality
                    );
                }
            }
        }
    }

    // S2 fix: 优先使用数据库中记录的 LanceDB 表名，避免遗留命名不一致
    let lance_table_name =
        embedding_dim_repo::get_by_key(&conn, dimension, &modality)?
            .map(|d| d.lance_table_name)
            .unwrap_or_else(|| {
                embedding_dim_repo::generate_lance_table_name(
                    &modality, dimension,
                )
            });

    let deleted_segments = embedding_dim_repo::delete_dimension_cascade(
        &conn, dimension, &modality,
    )?;
    drop(conn);

    // S2 fix: 删除对应的 LanceDB 表，清理磁盘向量数据
    if let Err(e) = lance_store.drop_table(&lance_table_name).await {
        log::warn!(
            "[VFS::handlers] LanceDB table {} cleanup failed (non-fatal): {}",
            lance_table_name,
            e
        );
    }

    Ok(DeleteDimensionResult {
        deleted_segments,
        dimension,
        modality,
    })
}

#[tauri::command]
pub async fn vfs_get_preset_dimensions() -> VfsResult<Vec<i32>> {
    Ok(embedding_dim_repo::PRESET_DIMENSIONS.to_vec())
}

#[tauri::command]
pub async fn vfs_get_dimension_range() -> VfsResult<(i32, i32)> {
    Ok((
        embedding_dim_repo::MIN_DIMENSION,
        embedding_dim_repo::MAX_DIMENSION,
    ))
}

// ============================================================================
// 默认嵌入维度管理 API
// ============================================================================

/// 设置默认嵌入维度
///
/// 将指定维度设为该模态的默认嵌入维度。
/// - modality: "text" | "multimodal"
///
/// 同时保存维度值和绑定的模型配置ID，供 LLMManager 直接读取
#[tauri::command]
pub async fn vfs_set_default_embedding_dimension(
    dimension: i32,
    modality: String,
    database: State<'_, Arc<Database>>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<bool> {
    log::info!(
        "[VFS::handlers] vfs_set_default_embedding_dimension: dim={}, modality={}",
        dimension,
        modality
    );

    // 验证维度存在并获取绑定的模型
    let conn = vfs_db.get_conn()?;
    let dim_info = embedding_dim_repo::get_by_key(&conn, dimension, &modality)?
        .ok_or_else(|| format!("维度 {}:{} 不存在", dimension, modality))?;

    // ★ 审计修复：后端也校验模型绑定，与前端保持一致
    if dim_info.model_config_id.is_none() {
        log::warn!(
            "[VFS::handlers] Dimension {}:{} has no model binding, allowing set_default but clearing model config",
            dimension, modality
        );
    }

    // 保存维度值和模型配置ID到 settings
    let (dim_key, model_key) = match modality.as_str() {
        "text" => (
            "embedding.default_text_dimension",
            "embedding.default_text_model_config_id",
        ),
        "multimodal" => (
            "embedding.default_multimodal_dimension",
            "embedding.default_multimodal_model_config_id",
        ),
        _ => return Err(format!("无效的模态类型: {}", modality).into()),
    };

    database
        .save_setting(dim_key, &dimension.to_string())?;

    // 如果维度有绑定模型，同时保存模型配置ID
    if let Some(model_config_id) = &dim_info.model_config_id {
        database
            .save_setting(model_key, model_config_id)?;
        log::info!(
            "[VFS::handlers] 已设置默认 {} 嵌入模型: {}",
            modality,
            model_config_id
        );
    } else {
        // 如果维度没有绑定模型，清除旧的模型配置
        let _ = database.delete_setting(model_key);
        log::warn!(
            "[VFS::handlers] 维度 {}:{} 未绑定模型，已清除默认模型配置",
            dimension,
            modality
        );
    }

    Ok(true)
}

/// 获取默认嵌入维度信息
///
/// 返回指定模态的默认维度完整信息（包括绑定的模型）
#[tauri::command]
pub async fn vfs_get_default_embedding_dimension(
    modality: String,
    database: State<'_, Arc<Database>>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Option<embedding_dim_repo::VfsEmbeddingDim>> {
    log::debug!(
        "[VFS::handlers] vfs_get_default_embedding_dimension: modality={}",
        modality
    );

    let key = match modality.as_str() {
        "text" => "embedding.default_text_dimension",
        "multimodal" => "embedding.default_multimodal_dimension",
        _ => return Err(format!("无效的模态类型: {}", modality).into()),
    };

    // 从 settings 获取默认维度值
    let dim_str = match database.get_setting(key) {
        Ok(Some(s)) => s,
        Ok(None) => return Ok(None),
        Err(e) => return Err(VfsError::from(e.to_string())),
    };

    let dimension: i32 = dim_str
        .parse()
        .map_err(|_| format!("无效的维度值: {}", dim_str))?;

    // M3 fix: 从 vfs_embedding_dims 获取完整信息，如果维度已不存在则自动清除设置
    let conn = vfs_db.get_conn()?;
    let dim_info = embedding_dim_repo::get_by_key(&conn, dimension, &modality)?;

    if dim_info.is_none() {
        // 维度记录不存在（可能被删除或数据库恢复导致），自动清除 settings
        log::warn!(
            "[VFS::handlers] Default dimension {}:{} no longer exists in VFS DB, auto-clearing setting",
            dimension, modality
        );
        let _ = database.delete_setting(key);
        let model_key = match modality.as_str() {
            "text" => "embedding.default_text_model_config_id",
            "multimodal" => "embedding.default_multimodal_model_config_id",
            _ => "",
        };
        if !model_key.is_empty() {
            let _ = database.delete_setting(model_key);
        }
    }

    Ok(dim_info)
}

/// 清除默认嵌入维度设置
#[tauri::command]
pub async fn vfs_clear_default_embedding_dimension(
    modality: String,
    database: State<'_, Arc<Database>>,
) -> VfsResult<bool> {
    log::info!(
        "[VFS::handlers] vfs_clear_default_embedding_dimension: modality={}",
        modality
    );

    let (dim_key, model_key) = match modality.as_str() {
        "text" => (
            "embedding.default_text_dimension",
            "embedding.default_text_model_config_id",
        ),
        "multimodal" => (
            "embedding.default_multimodal_dimension",
            "embedding.default_multimodal_model_config_id",
        ),
        _ => return Err(format!("无效的模态类型: {}", modality).into()),
    };

    // 同时清除维度和模型配置
    let _ = database.delete_setting(dim_key);
    let _ = database.delete_setting(model_key);

    Ok(true)
}

// ============================================================================
// 批量索引
// ============================================================================

/// 批量索引结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchIndexResult {
    /// 成功数
    pub success_count: usize,
    /// 失败数
    pub fail_count: usize,
    /// 总数
    pub total: usize,
}

/// 批量索引待处理资源（带进度事件）
#[tauri::command]
pub async fn vfs_batch_index_pending(
    batch_size: Option<u32>,
    app_handle: AppHandle,
    llm_manager: State<'_, Arc<LLMManager>>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
    lance_store: State<'_, Arc<VfsLanceStore>>,
) -> VfsResult<BatchIndexResult> {
    let batch_size = batch_size.unwrap_or(10);
    log::info!(
        "[VFS::handlers] vfs_batch_index_pending: batch_size={}",
        batch_size
    );

    let indexing_service = VfsIndexingService::new(Arc::clone(&vfs_db));
    log::info!("[VFS::handlers] vfs_batch_index_pending: 获取索引配置...");
    let config = indexing_service
        .get_indexing_config()?;
    // ★ 2026-02 修复：使用 claim_pending_resources 原子抢占，避免并发重复索引
    log::info!("[VFS::handlers] vfs_batch_index_pending: 原子抢占待处理资源...");
    let pending =
        VfsIndexStateRepo::claim_pending_resources(&vfs_db, batch_size, config.max_retries)?;
    log::info!(
        "[VFS::handlers] vfs_batch_index_pending: 原子抢占 {} 个待处理资源",
        pending.len()
    );

    if pending.is_empty() {
        return Ok(BatchIndexResult {
            success_count: 0,
            fail_count: 0,
            total: 0,
        });
    }

    let total = pending.len();

    // 发送批量索引开始事件
    if let Err(e) = app_handle.emit(
        "vfs-index-progress",
        serde_json::json!({
            "type": "batch_started",
            "total": total,
            "message": format!("开始批量索引 {} 个资源...", total)
        }),
    ) {
        log::warn!("[VFS::handlers] Failed to emit batch_started: {}", e);
    }

    let mut success_count = 0usize;
    let mut fail_count = 0usize;

    let full_indexing_service = match VfsFullIndexingService::new(
        Arc::clone(&vfs_db),
        Arc::clone(&llm_manager),
        Arc::clone(lance_store.inner()),
    ) {
        Ok(mut svc) => {
            svc.set_app_handle(app_handle.clone());
            svc
        }
        Err(e) => {
            log::error!(
                "[VFS::handlers] vfs_batch_index_pending: IndexingService 初始化失败，回退 {} 个已 claim 的资源",
                pending.len()
            );
            for resource_id in &pending {
                let _ = VfsIndexStateRepo::mark_pending(&vfs_db, resource_id);
            }
            return Err(VfsError::Other(e.to_string()));
        }
    };

    for (index, resource_id) in pending.iter().enumerate() {
        // ★ P1-2 修复: 将 "processing" 改为 "resource_started" 以匹配前端期望
        if let Err(e) = app_handle.emit(
            "vfs-index-progress",
            serde_json::json!({
                "type": "resource_started",
                "resourceId": resource_id,
                "current": index + 1,
                "total": total,
                "progress": ((index as f64 / total as f64) * 100.0) as u32,
                "message": format!("正在索引资源 {}/{}", index + 1, total)
            }),
        ) {
            log::warn!("[VFS::handlers] Failed to emit resource_started: {}", e);
        }

        // ★ 构造嵌入进度回调，按 embedding batch (每16块) 粒度上报细粒度进度
        let cb_handle = app_handle.clone();
        let cb_resource_id = resource_id.clone();
        let cb_index = index;
        let cb_total = total;
        let progress_callback: Option<EmbeddingProgressCallback> =
            Some(Box::new(move |chunks_done: usize, chunks_total: usize| {
                // 整体进度 = 当前资源基准 + 当前资源内嵌入子进度
                let base = cb_index as f64 / cb_total as f64;
                let sub = if chunks_total > 0 {
                    chunks_done as f64 / chunks_total as f64 / cb_total as f64
                } else {
                    0.0
                };
                let progress = ((base + sub) * 100.0).min(99.0) as u32;
                if let Err(e) = cb_handle.emit(
                    "vfs-index-progress",
                    serde_json::json!({
                        "type": "embedding_progress",
                        "resourceId": cb_resource_id,
                        "current": cb_index + 1,
                        "total": cb_total,
                        "chunksProcessed": chunks_done,
                        "chunksTotal": chunks_total,
                        "progress": progress,
                        "message": format!("正在索引资源 {}/{} (嵌入 {}/{})",
                            cb_index + 1, cb_total, chunks_done, chunks_total)
                    }),
                ) {
                    log::warn!("[VFS::handlers] Failed to emit embedding progress: {}", e);
                }
            }));

        match full_indexing_service
            .index_resource(resource_id, None, progress_callback)
            .await
        {
            Ok((chunk_count, _)) => {
                success_count += 1;
                if let Err(e) = app_handle.emit(
                    "vfs-index-progress",
                    serde_json::json!({
                        "type": "resource_completed",
                        "resourceId": resource_id,
                        "chunkCount": chunk_count,
                        "current": index + 1,
                        "total": total,
                        "progress": (((index + 1) as f64 / total as f64) * 100.0) as u32,
                        "message": format!("资源索引完成: {} 个块", chunk_count)
                    }),
                ) {
                    log::warn!("[VFS::handlers] Failed to emit resource_completed: {}", e);
                }
            }
            Err(e) => {
                fail_count += 1;
                log::warn!("[VFS::handlers] Failed to index {}: {}", resource_id, e);
                if let Err(emit_err) = app_handle.emit(
                    "vfs-index-progress",
                    serde_json::json!({
                        "type": "resource_failed",
                        "resourceId": resource_id,
                        "error": e.to_string(),
                        "current": index + 1,
                        "total": total,
                        "progress": (((index + 1) as f64 / total as f64) * 100.0) as u32,
                        "message": format!("索引失败 ({}/{}): {}", index + 1, total, e)
                    }),
                ) {
                    log::warn!("[VFS::handlers] Failed to emit resource_failed: {}", emit_err);
                }
            }
        }
    }

    // 发送批量索引完成事件
    if let Err(e) = app_handle.emit(
        "vfs-index-progress",
        serde_json::json!({
            "type": "batch_completed",
            "successCount": success_count,
            "failCount": fail_count,
            "total": total,
            "progress": 100,
            "message": format!("批量索引完成: {} 成功, {} 失败", success_count, fail_count)
        }),
    ) {
        log::warn!("[VFS::handlers] Failed to emit batch_completed: {}", e);
    }

    Ok(BatchIndexResult {
        success_count,
        fail_count,
        total,
    })
}

#[tauri::command]
pub async fn vfs_set_indexing_config(
    key: String,
    value: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<()> {
    log::info!("[VFS::handlers] vfs_set_indexing_config: {}={}", key, value);
    Ok(VfsIndexingConfigRepo::set_config(&vfs_db, &key, &value)?)
}

#[tauri::command]
pub async fn vfs_get_indexing_config(
    key: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Option<String>> {
    log::debug!("[VFS::handlers] vfs_get_indexing_config: {}", key);
    Ok(VfsIndexingConfigRepo::get_config(&vfs_db, &key)?)
}

// ============================================================================
// 向量化状态视图命令
// ============================================================================

/// 单个资源的向量化状态
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceIndexStatus {
    /// 资源 ID
    pub resource_id: String,
    /// 业务来源 ID（如 textbook_xxx, exam_xxx）用于多模态索引
    pub source_id: Option<String>,
    /// 资源类型
    pub resource_type: String,
    /// 资源名称
    pub name: String,

    // ========== OCR 状态 ==========
    /// 是否有 OCR 数据
    pub has_ocr: bool,
    /// OCR 页数（教材）或字符数（图片）
    pub ocr_count: i32,

    // ========== 文本索引状态 ==========
    /// 文本索引状态
    pub text_index_state: String,
    /// 文本索引时间
    pub text_indexed_at: Option<i64>,
    /// 文本索引错误
    pub text_index_error: Option<String>,
    /// 文本块数量
    pub text_chunk_count: i32,
    /// 提取文本块数量（text_source = 'native'）
    pub native_text_chunk_count: i32,
    /// OCR 文本块数量（text_source = 'ocr'）
    pub ocr_text_chunk_count: i32,
    /// 文本向量维度
    pub text_embedding_dim: Option<i32>,
    /// 文本索引来源（sqlite = 仅FTS，lance = 向量化完成）
    pub text_index_source: Option<String>,

    // ========== 多模态索引状态 ==========
    /// 多模态索引状态（pending, indexing, indexed, failed, disabled）
    pub mm_index_state: String,
    /// 多模态索引页数
    pub mm_indexed_pages: i32,
    /// 多模态向量维度
    pub mm_embedding_dim: Option<i32>,
    /// 多模态索引模式
    pub mm_indexing_mode: Option<String>,
    /// 多模态索引错误
    pub mm_index_error: Option<String>,

    // ========== 通用 ==========
    /// 模态类型（text, multimodal 等）- 保留向后兼容
    pub modality: Option<String>,
    /// 向量维度 - 保留向后兼容
    pub embedding_dim: Option<i32>,
    /// 更新时间
    pub updated_at: i64,
    /// 索引是否过时
    pub is_stale: bool,
}

/// 向量化状态统计
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStatusSummary {
    /// 总资源数
    pub total_resources: i32,
    /// 已索引数
    pub indexed_count: i32,
    /// 待索引数
    pub pending_count: i32,
    /// 索引中数
    pub indexing_count: i32,
    /// 失败数
    pub failed_count: i32,
    /// 禁用数
    pub disabled_count: i32,
    /// 索引过时数（内容已更新但索引未更新）
    pub stale_count: i32,
    // ========== 多模态索引统计 ==========
    /// 多模态总资源数（教材/附件/题目集/图片）
    pub mm_total_resources: i32,
    /// 多模态已索引数
    pub mm_indexed_count: i32,
    /// 多模态待索引数
    pub mm_pending_count: i32,
    /// 多模态索引中数
    pub mm_indexing_count: i32,
    /// 多模态失败数
    pub mm_failed_count: i32,
    /// 多模态禁用数
    pub mm_disabled_count: i32,
    /// 资源状态列表
    pub resources: Vec<ResourceIndexStatus>,
}

/// 获取所有资源的向量化状态
#[tauri::command]
pub async fn vfs_get_all_index_status(
    folder_id: Option<String>,
    resource_type: Option<String>,
    state_filter: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<IndexStatusSummary> {
    log::info!(
        "[VFS::handlers] vfs_get_all_index_status: folder={:?}, type={:?}, state={:?}",
        folder_id,
        resource_type,
        state_filter
    );

    let conn = vfs_db.get_conn_safe()?;

    // 检查必要的列和表是否存在
    let has_index_state = has_index_state_column(&conn);
    let has_index_tables = has_vfs_index_units_table(&conn);

    if !has_index_state {
        log::warn!(
            "[VFS::handlers] vfs_get_all_index_status: index_state column not found, migration may not have run"
        );
        return Ok(IndexStatusSummary {
            total_resources: 0,
            indexed_count: 0,
            pending_count: 0,
            indexing_count: 0,
            failed_count: 0,
            disabled_count: 0,
            stale_count: 0,
            mm_total_resources: 0,
            mm_indexed_count: 0,
            mm_pending_count: 0,
            mm_indexing_count: 0,
            mm_failed_count: 0,
            mm_disabled_count: 0,
            resources: vec![],
        });
    }

    // ========== 构建查询条件 ==========
    let mut stats_conditions = vec!["r.deleted_at IS NULL".to_string()];
    let mut stats_params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    let mut list_conditions = vec!["r.deleted_at IS NULL".to_string()];
    let mut list_params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    if let Some(ref rt) = resource_type {
        stats_conditions.push("r.type = ?".to_string());
        stats_params.push(Box::new(rt.clone()));
        list_conditions.push("r.type = ?".to_string());
        list_params.push(Box::new(rt.clone()));
    }

    if let Some(ref sf) = state_filter {
        list_conditions.push("COALESCE(r.index_state, 'pending') = ?".to_string());
        list_params.push(Box::new(sf.clone()));
    }

    let folder_join = if let Some(ref fid) = folder_id {
        stats_conditions.push("fi.folder_id = ?".to_string());
        stats_params.push(Box::new(fid.clone()));
        list_conditions.push("fi.folder_id = ?".to_string());
        list_params.push(Box::new(fid.clone()));
        "JOIN folder_items fi ON r.id = fi.item_id AND fi.deleted_at IS NULL"
    } else {
        ""
    };

    let stats_where_clause = stats_conditions.join(" AND ");
    let list_where_clause = list_conditions.join(" AND ");
    let limit_val = limit.unwrap_or(100) as i64;
    let offset_val = offset.unwrap_or(0) as i64;

    let index_ctes = if has_index_tables {
        r#"
        unit_agg AS (
            SELECT resource_id,
                COALESCE(SUM(CASE WHEN text_source = 'native' THEN text_chunk_count ELSE 0 END), 0) as native_chunks,
                COALESCE(SUM(CASE WHEN text_source = 'ocr' THEN text_chunk_count ELSE 0 END), 0) as ocr_chunks
            FROM vfs_index_units
            GROUP BY resource_id
        ),
        seg_agg AS (
            SELECT u.resource_id,
                COUNT(*) as segment_count,
                MIN(s.embedding_dim) as first_embedding_dim,
                MAX(CASE WHEN s.modality = 'text' THEN 1 ELSE 0 END) as has_text_seg,
                MAX(CASE WHEN s.modality = 'multimodal' THEN 1 ELSE 0 END) as has_mm_seg
            FROM vfs_index_segments s
            JOIN vfs_index_units u ON s.unit_id = u.id
            GROUP BY u.resource_id
        ),
        "#
    } else {
        ""
    };

    let (chunk_count_col, native_chunks_col, ocr_chunks_col, embedding_dim_col, modality_col) =
        if has_index_tables {
            (
                "COALESCE(sa.segment_count, 0)",
                "COALESCE(ua.native_chunks, 0)",
                "COALESCE(ua.ocr_chunks, 0)",
                "sa.first_embedding_dim",
                r#"CASE
                    WHEN COALESCE(sa.has_text_seg, 0) = 1 AND COALESCE(sa.has_mm_seg, 0) = 1 THEN 'both'
                    WHEN COALESCE(sa.has_text_seg, 0) = 1 THEN 'text'
                    WHEN COALESCE(sa.has_mm_seg, 0) = 1 THEN 'multimodal'
                    ELSE NULL
                END"#,
            )
        } else {
            ("0", "0", "0", "NULL", "NULL")
        };

    let index_joins = if has_index_tables {
        "LEFT JOIN unit_agg ua ON ua.resource_id = r.id\n        LEFT JOIN seg_agg sa ON sa.resource_id = r.id"
    } else {
        ""
    };

    let query = format!(
        r#"
        WITH
        file_by_res AS (
            SELECT resource_id, file_name, name, ocr_pages_json, extracted_text,
                   mm_index_state, mm_indexed_pages_json, mm_index_error
            FROM files
            WHERE resource_id IS NOT NULL AND status = 'active'
            GROUP BY resource_id
        ),
        exam_by_res AS (
            SELECT resource_id, exam_name, preview_json,
                   mm_index_state, mm_indexed_pages_json, mm_index_error
            FROM exam_sheets
            WHERE resource_id IS NOT NULL
            GROUP BY resource_id
        ),
        {index_ctes}
        note_names AS (
            SELECT resource_id, title FROM notes GROUP BY resource_id
        ),
        tr_names AS (
            SELECT resource_id, title FROM translations GROUP BY resource_id
        ),
        essay_names AS (
            SELECT resource_id, title FROM essays GROUP BY resource_id
        ),
        mm_names AS (
            SELECT resource_id, title FROM mindmaps GROUP BY resource_id
        )
        SELECT
            r.id,
            r.source_id,
            r.type,
            COALESCE(
                nn.title,
                COALESCE(fr.file_name, fs.file_name),
                COALESCE(ei.exam_name, es_src.exam_name),
                tn.title,
                en.title,
                mn.title,
                COALESCE(fs.name, fr.name),
                r.id
            ) as name,
            CASE
                WHEN r.type = 'textbook' THEN (COALESCE(fr.ocr_pages_json, fs.ocr_pages_json) IS NOT NULL)
                WHEN r.type = 'image' THEN (r.ocr_text IS NOT NULL AND r.ocr_text != '')
                WHEN r.type = 'file' THEN (
                    (COALESCE(fr.extracted_text, fs.extracted_text) IS NOT NULL AND COALESCE(fr.extracted_text, fs.extracted_text) != '')
                    OR (COALESCE(fr.ocr_pages_json, fs.ocr_pages_json) IS NOT NULL AND COALESCE(fr.ocr_pages_json, fs.ocr_pages_json) != '')
                    OR (r.ocr_text IS NOT NULL AND r.ocr_text != '')
                )
                WHEN r.type = 'exam' THEN (ei.preview_json IS NOT NULL)
                ELSE 0
            END as has_ocr,
            CASE
                WHEN r.type = 'textbook' THEN COALESCE(
                    CASE
                        WHEN json_type(COALESCE(fr.ocr_pages_json, fs.ocr_pages_json)) = 'array'
                            THEN json_array_length(COALESCE(fr.ocr_pages_json, fs.ocr_pages_json))
                        WHEN json_type(COALESCE(fr.ocr_pages_json, fs.ocr_pages_json), '$.pages') = 'array'
                            THEN json_array_length(json_extract(COALESCE(fr.ocr_pages_json, fs.ocr_pages_json), '$.pages'))
                        ELSE 0
                    END, 0)
                WHEN r.type = 'image' THEN COALESCE(LENGTH(r.ocr_text), 0)
                WHEN r.type = 'file' THEN COALESCE(
                    COALESCE(LENGTH(COALESCE(fr.extracted_text, fs.extracted_text)), 0)
                    + COALESCE(LENGTH(r.ocr_text), 0), 0)
                WHEN r.type = 'exam' THEN COALESCE(LENGTH(ei.preview_json), 0)
                ELSE 0
            END as ocr_count,
            COALESCE(r.index_state, 'pending') as index_state,
            r.indexed_at,
            r.index_error,
            {chunk_count} as chunk_count,
            {native_chunks} as native_chunk_count,
            {ocr_chunks} as ocr_chunk_count,
            {embedding_dim} as text_embedding_dim,
            CASE
                WHEN COALESCE(r.index_state, 'pending') = 'indexed' AND {chunk_count} > 0 THEN 'lance'
                WHEN COALESCE(r.index_state, 'pending') = 'indexed' THEN 'sqlite'
                ELSE NULL
            END as text_index_source,
            CASE
                WHEN r.type IN ('textbook', 'file', 'image') THEN COALESCE(
                    fr.mm_index_state, fs.mm_index_state, 'pending')
                WHEN r.type = 'exam' THEN COALESCE(ei.mm_index_state, 'pending')
                ELSE 'disabled'
            END as mm_index_state,
            CASE
                WHEN r.type IN ('textbook', 'file', 'image') THEN COALESCE(
                    json_array_length(COALESCE(fr.mm_indexed_pages_json, fs.mm_indexed_pages_json)), 0)
                WHEN r.type = 'exam' THEN COALESCE(json_array_length(ei.mm_indexed_pages_json), 0)
                ELSE CASE WHEN r.mm_embedding_dim IS NOT NULL THEN 1 ELSE 0 END
            END as mm_indexed_pages,
            CASE
                WHEN r.type IN ('textbook', 'file', 'image') THEN
                    json_extract(COALESCE(fr.mm_indexed_pages_json, fs.mm_indexed_pages_json), '$[0].embedding_dim')
                WHEN r.type = 'exam' THEN
                    json_extract(ei.mm_indexed_pages_json, '$[0].embedding_dim')
                ELSE r.mm_embedding_dim
            END as mm_embedding_dim,
            CASE
                WHEN r.type IN ('textbook', 'file', 'image') THEN
                    json_extract(COALESCE(fr.mm_indexed_pages_json, fs.mm_indexed_pages_json), '$[0].indexing_mode')
                WHEN r.type = 'exam' THEN
                    json_extract(ei.mm_indexed_pages_json, '$[0].indexing_mode')
                ELSE r.mm_indexing_mode
            END as mm_indexing_mode,
            CASE
                WHEN r.type IN ('textbook', 'file', 'image') THEN COALESCE(fr.mm_index_error, fs.mm_index_error)
                WHEN r.type = 'exam' THEN ei.mm_index_error
                ELSE r.mm_index_error
            END as mm_index_error,
            {modality} as modality,
            r.updated_at,
            CASE
                WHEN COALESCE(r.index_state, 'pending') = 'indexed'
                     AND r.index_hash IS NOT NULL
                     AND r.index_hash != r.hash
                THEN 1
                ELSE 0
            END as is_stale
        FROM resources r
        LEFT JOIN note_names nn ON nn.resource_id = r.id
        LEFT JOIN file_by_res fr ON fr.resource_id = r.id
        LEFT JOIN files fs ON fs.id = r.source_id AND fs.status = 'active'
        LEFT JOIN exam_by_res ei ON ei.resource_id = r.id
        LEFT JOIN exam_sheets es_src ON es_src.id = r.source_id
        LEFT JOIN tr_names tn ON tn.resource_id = r.id
        LEFT JOIN essay_names en ON en.resource_id = r.id
        LEFT JOIN mm_names mn ON mn.resource_id = r.id
        {index_joins}
        {folder_join}
        WHERE {list_where}
            AND (
                nn.resource_id IS NOT NULL
                OR fr.resource_id IS NOT NULL
                OR fs.id IS NOT NULL
                OR ei.resource_id IS NOT NULL
                OR es_src.id IS NOT NULL
                OR tn.resource_id IS NOT NULL
                OR en.resource_id IS NOT NULL
                OR mn.resource_id IS NOT NULL
            )
        ORDER BY r.updated_at DESC
        LIMIT ? OFFSET ?
        "#,
        index_ctes = index_ctes,
        chunk_count = chunk_count_col,
        native_chunks = native_chunks_col,
        ocr_chunks = ocr_chunks_col,
        embedding_dim = embedding_dim_col,
        modality = modality_col,
        index_joins = index_joins,
        folder_join = folder_join,
        list_where = list_where_clause,
    );

    let mut stmt = conn.prepare(&query).map_err(|e| {
        log::error!(
            "[VFS::handlers] vfs_get_all_index_status: prepare error: {}",
            e
        );
        e.to_string()
    })?;

    let mut all_params: Vec<&dyn rusqlite::ToSql> =
        list_params.iter().map(|p| p.as_ref()).collect();
    all_params.push(&limit_val);
    all_params.push(&offset_val);

    let query_result = stmt.query_map(rusqlite::params_from_iter(all_params.iter()), |row| {
        let updated_at: i64 = match row.get::<_, i64>(20) {
            Ok(v) => v,
            Err(_) => {
                let text_val: String = row.get(20)?;
                chrono::DateTime::parse_from_rfc3339(&text_val)
                    .map(|dt| dt.timestamp_millis())
                    .or_else(|_| {
                        chrono::NaiveDateTime::parse_from_str(&text_val, "%Y-%m-%dT%H:%M:%S%.f")
                            .or_else(|_| {
                                chrono::NaiveDateTime::parse_from_str(
                                    &text_val,
                                    "%Y-%m-%d %H:%M:%S",
                                )
                            })
                            .map(|dt| dt.and_utc().timestamp_millis())
                    })
                    .unwrap_or(0)
            }
        };

        let text_embedding_dim: Option<i32> = row.get(12)?;

        Ok(ResourceIndexStatus {
            resource_id: row.get(0)?,
            source_id: row.get(1)?,
            resource_type: row.get(2)?,
            name: row.get(3)?,
            has_ocr: row.get::<_, i32>(4).unwrap_or(0) == 1,
            ocr_count: row.get(5).unwrap_or(0),
            text_index_state: row.get(6)?,
            text_indexed_at: row.get(7)?,
            text_index_error: row.get(8)?,
            text_chunk_count: row.get(9).unwrap_or(0),
            native_text_chunk_count: row.get(10).unwrap_or(0),
            ocr_text_chunk_count: row.get(11).unwrap_or(0),
            text_embedding_dim,
            text_index_source: row.get(13)?,
            mm_index_state: row
                .get::<_, String>(14)
                .unwrap_or_else(|_| "pending".to_string()),
            mm_indexed_pages: row.get(15).unwrap_or(0),
            mm_embedding_dim: row.get(16)?,
            mm_indexing_mode: row.get(17)?,
            mm_index_error: row.get(18)?,
            modality: row.get(19)?,
            embedding_dim: text_embedding_dim,
            updated_at,
            is_stale: row.get::<_, i32>(21).unwrap_or(0) == 1,
        })
    });

    let resources: Vec<ResourceIndexStatus> = match query_result {
        Ok(rows) => {
            let mut resources = Vec::new();
            let mut error_count = 0;
            for (idx, row) in rows.enumerate() {
                match row {
                    Ok(r) => resources.push(r),
                    Err(e) => {
                        error_count += 1;
                        log::warn!(
                            "[VFS::handlers] vfs_get_all_index_status: row {} parse error: {}",
                            idx,
                            e
                        );
                    }
                }
            }
            if error_count > 0 {
                log::warn!(
                    "[VFS::handlers] vfs_get_all_index_status: {} rows had parse errors",
                    error_count
                );
            }
            log::info!(
                "[VFS::handlers] vfs_get_all_index_status: 资源列表查询完成, 返回 {} 条记录",
                resources.len()
            );
            resources
        }
        Err(e) => {
            log::error!(
                "[VFS::handlers] vfs_get_all_index_status: query error: {}",
                e
            );
            return Err(VfsError::Other(e.to_string()));
        }
    };

    // ========== 统计查询 ==========
    let stats_query = format!(
        r#"
        WITH file_mm AS (
            SELECT resource_id, mm_index_state
            FROM files
            WHERE resource_id IS NOT NULL AND status = 'active'
            GROUP BY resource_id
        ),
        exam_mm AS (
            SELECT resource_id, mm_index_state
            FROM exam_sheets
            WHERE resource_id IS NOT NULL
            GROUP BY resource_id
        )
        SELECT
            COUNT(*) as total,
            COALESCE(SUM(CASE WHEN COALESCE(r.index_state, 'pending') = 'indexed' THEN 1 ELSE 0 END), 0) as indexed,
            COALESCE(SUM(CASE WHEN COALESCE(r.index_state, 'pending') = 'pending' THEN 1 ELSE 0 END), 0) as pending,
            COALESCE(SUM(CASE WHEN r.index_state = 'indexing' THEN 1 ELSE 0 END), 0) as indexing,
            COALESCE(SUM(CASE WHEN r.index_state = 'failed' THEN 1 ELSE 0 END), 0) as failed,
            COALESCE(SUM(CASE WHEN r.index_state = 'disabled' THEN 1 ELSE 0 END), 0) as disabled,
            COALESCE(SUM(CASE
                WHEN COALESCE(r.index_state, 'pending') = 'indexed'
                     AND r.index_hash IS NOT NULL AND r.index_hash != r.hash
                THEN 1 ELSE 0 END), 0) as stale
            ,COALESCE(SUM(CASE WHEN r.type IN ('textbook', 'file', 'exam', 'image') THEN 1 ELSE 0 END), 0) as mm_total
            ,COALESCE(SUM(CASE WHEN r.type IN ('textbook', 'file', 'exam', 'image')
                AND COALESCE(
                    CASE WHEN r.type IN ('textbook', 'file', 'image') THEN COALESCE(fm.mm_index_state, fs_mm.mm_index_state) END,
                    CASE WHEN r.type = 'exam' THEN COALESCE(em.mm_index_state, es_mm.mm_index_state) END,
                    COALESCE(r.mm_index_state, 'pending')
                ) = 'indexed' THEN 1 ELSE 0 END), 0) as mm_indexed
            ,COALESCE(SUM(CASE WHEN r.type IN ('textbook', 'file', 'exam', 'image')
                AND COALESCE(
                    CASE WHEN r.type IN ('textbook', 'file', 'image') THEN COALESCE(fm.mm_index_state, fs_mm.mm_index_state) END,
                    CASE WHEN r.type = 'exam' THEN COALESCE(em.mm_index_state, es_mm.mm_index_state) END,
                    COALESCE(r.mm_index_state, 'pending')
                ) = 'pending' THEN 1 ELSE 0 END), 0) as mm_pending
            ,COALESCE(SUM(CASE WHEN r.type IN ('textbook', 'file', 'exam', 'image')
                AND COALESCE(
                    CASE WHEN r.type IN ('textbook', 'file', 'image') THEN COALESCE(fm.mm_index_state, fs_mm.mm_index_state) END,
                    CASE WHEN r.type = 'exam' THEN COALESCE(em.mm_index_state, es_mm.mm_index_state) END,
                    COALESCE(r.mm_index_state, 'pending')
                ) = 'indexing' THEN 1 ELSE 0 END), 0) as mm_indexing
            ,COALESCE(SUM(CASE WHEN r.type IN ('textbook', 'file', 'exam', 'image')
                AND COALESCE(
                    CASE WHEN r.type IN ('textbook', 'file', 'image') THEN COALESCE(fm.mm_index_state, fs_mm.mm_index_state) END,
                    CASE WHEN r.type = 'exam' THEN COALESCE(em.mm_index_state, es_mm.mm_index_state) END,
                    COALESCE(r.mm_index_state, 'pending')
                ) = 'failed' THEN 1 ELSE 0 END), 0) as mm_failed
            ,COALESCE(SUM(CASE WHEN r.type IN ('textbook', 'file', 'exam', 'image')
                AND COALESCE(
                    CASE WHEN r.type IN ('textbook', 'file', 'image') THEN COALESCE(fm.mm_index_state, fs_mm.mm_index_state) END,
                    CASE WHEN r.type = 'exam' THEN COALESCE(em.mm_index_state, es_mm.mm_index_state) END,
                    COALESCE(r.mm_index_state, 'pending')
                ) = 'disabled' THEN 1 ELSE 0 END), 0) as mm_disabled
        FROM resources r
        LEFT JOIN file_mm fm ON fm.resource_id = r.id
        LEFT JOIN files fs_mm ON fs_mm.id = r.source_id
        LEFT JOIN exam_mm em ON em.resource_id = r.id
        LEFT JOIN exam_sheets es_mm ON es_mm.id = r.source_id
        {0}
        WHERE {1}
            AND (
                fm.resource_id IS NOT NULL
                OR fs_mm.id IS NOT NULL
                OR em.resource_id IS NOT NULL
                OR es_mm.id IS NOT NULL
                OR EXISTS (SELECT 1 FROM notes WHERE resource_id = r.id)
                OR EXISTS (SELECT 1 FROM translations WHERE resource_id = r.id)
                OR EXISTS (SELECT 1 FROM essays WHERE resource_id = r.id)
                OR EXISTS (SELECT 1 FROM mindmaps WHERE resource_id = r.id)
            )
        "#,
        folder_join, stats_where_clause
    );

    let stats_query_params: Vec<&dyn rusqlite::ToSql> =
        stats_params.iter().map(|p| p.as_ref()).collect();

    let (
        total,
        indexed,
        pending,
        indexing,
        failed,
        disabled,
        stale,
        mm_total,
        mm_indexed,
        mm_pending,
        mm_indexing,
        mm_failed,
        mm_disabled,
    ): (
        i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32,
    ) = conn
        .query_row(
            &stats_query,
            rusqlite::params_from_iter(stats_query_params.iter()),
            |row| {
                Ok((
                    row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?,
                    row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?,
                    row.get(8)?, row.get(9)?, row.get(10)?, row.get(11)?,
                    row.get(12)?,
                ))
            },
        )?;

    log::info!(
        "[VFS::handlers] vfs_get_all_index_status: 返回结果 total={}, indexed={}, pending={}, resources_len={}, state_filter={:?}",
        total, indexed, pending, resources.len(), state_filter
    );

    Ok(IndexStatusSummary {
        total_resources: total,
        indexed_count: indexed,
        pending_count: pending,
        indexing_count: indexing,
        failed_count: failed,
        disabled_count: disabled,
        stale_count: stale,
        mm_total_resources: mm_total,
        mm_indexed_count: mm_indexed,
        mm_pending_count: mm_pending,
        mm_indexing_count: mm_indexing,
        mm_failed_count: mm_failed,
        mm_disabled_count: mm_disabled,
        resources,
    })
}

// ============================================================================
// VFS RAG 向量检索命令
// ============================================================================

/// VFS RAG 向量检索输入参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VfsRagSearchInput {
    /// 查询文本
    pub query: String,

    /// 文件夹 ID 列表（可选，用于范围过滤）
    #[serde(default)]
    pub folder_ids: Option<Vec<String>>,

    /// 资源类型列表（可选，如 ["note", "textbook"]）
    #[serde(default)]
    pub resource_types: Option<Vec<String>>,

    /// 返回结果数量
    #[serde(default = "default_rag_top_k")]
    pub top_k: u32,

    /// 是否启用重排序
    #[serde(default = "default_enable_reranking")]
    pub enable_reranking: bool,

    /// ★ P2-1: 模态类型（"text" 或 "multimodal"，默认 "text"）
    #[serde(default = "default_modality")]
    pub modality: String,

    /// 是否启用跨维度搜索（聚合所有已分配模型的维度，默认启用）
    #[serde(default = "default_enable_cross_dimension")]
    pub enable_cross_dimension: bool,
}

fn default_modality() -> String {
    "text".to_string()
}

fn default_rag_top_k() -> u32 {
    10
}

fn default_enable_reranking() -> bool {
    true
}

fn default_enable_cross_dimension() -> bool {
    true
}

/// VFS RAG 检索结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VfsRagSearchOutput {
    /// 检索结果列表
    pub results: Vec<VfsSearchResult>,
    /// 结果数量
    pub count: usize,
    /// 检索耗时（毫秒）
    pub elapsed_ms: u64,
}

/// VFS RAG 向量检索命令
///
/// 使用 VFS 统一知识管理架构进行 RAG 检索。
#[tauri::command]
pub async fn vfs_rag_search(
    input: VfsRagSearchInput,
    vfs_db: State<'_, Arc<VfsDatabase>>,
    llm_manager: State<'_, Arc<LLMManager>>,
    lance_store: State<'_, Arc<VfsLanceStore>>,
) -> VfsResult<VfsRagSearchOutput> {
    use crate::vfs::indexing::{VfsFullSearchService, VfsSearchParams};

    let start = std::time::Instant::now();

    log::info!(
        "[VFS::handlers] vfs_rag_search: query='{}', folders={:?}, types={:?}, top_k={}",
        input.query,
        input.folder_ids,
        input.resource_types,
        input.top_k
    );

    if input.query.trim().is_empty() {
        return Err(VfsError::Other("查询文本不能为空".to_string()));
    }

    let lance_store = Arc::clone(lance_store.inner());

    let search_service =
        VfsFullSearchService::new(Arc::clone(&vfs_db), lance_store, Arc::clone(&llm_manager));

    let normalized_modality = input.modality.trim().to_lowercase();
    let modality = match normalized_modality.as_str() {
        "" | "text" => MODALITY_TEXT.to_string(),
        "multimodal" | "mm" => crate::vfs::repos::MODALITY_MULTIMODAL.to_string(),
        _ => {
            return Err(VfsError::Other("modality 仅支持 'text' 或 'multimodal'".to_string()));
        }
    };

    let params = VfsSearchParams {
        query: input.query.clone(),
        folder_ids: input.folder_ids,
        resource_ids: None,
        resource_types: input.resource_types,
        modality,
        top_k: input.top_k,
    };

    let results = if input.enable_cross_dimension {
        search_service
            .search_cross_dimension_with_resource_info(
                &input.query,
                &params,
                input.enable_reranking,
            )
            .await?
    } else {
        search_service
            .search_with_resource_info(&input.query, &params, input.enable_reranking)
            .await?
    };

    let elapsed = start.elapsed();
    let count = results.len();

    log::info!(
        "[VFS::handlers] vfs_rag_search completed: {} results in {}ms",
        count,
        elapsed.as_millis()
    );

    Ok(VfsRagSearchOutput {
        results,
        count,
        elapsed_ms: elapsed.as_millis() as u64,
    })
}

/// VFS 获取 Lance 统计信息命令
#[tauri::command]
pub async fn vfs_get_lance_stats(
    modality: Option<String>,
    _vfs_db: State<'_, Arc<VfsDatabase>>,
    lance_store: State<'_, Arc<VfsLanceStore>>,
) -> VfsResult<Vec<(String, usize)>> {
    log::debug!("[VFS::handlers] vfs_get_lance_stats");

    let modality_str = modality.as_deref().unwrap_or(MODALITY_TEXT);

    lance_store
        .get_table_stats(modality_str)
        .await
}

/// VFS 优化 Lance 表命令
#[tauri::command]
pub async fn vfs_optimize_lance(
    modality: Option<String>,
    _vfs_db: State<'_, Arc<VfsDatabase>>,
    lance_store: State<'_, Arc<VfsLanceStore>>,
) -> VfsResult<usize> {
    log::info!("[VFS::handlers] vfs_optimize_lance");

    let modality_str = modality.as_deref().unwrap_or(MODALITY_TEXT);

    lance_store
        .optimize_all(modality_str)
        .await
}
