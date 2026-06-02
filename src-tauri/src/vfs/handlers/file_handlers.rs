//! VFS 文件操作 Tauri 命令处理器
//!
//! 提供文件上传、查询、删除等 Tauri 命令。
//!
//! ## 命令
//! - `vfs_upload_file`: 上传文件
//! - `vfs_get_file`: 获取文件元数据
//! - `vfs_list_files`: 列出文件
//! - `vfs_delete_file`: 删除文件（软删除）
//! - `vfs_get_file_content`: 获取文件内容

use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::State;

use crate::document_parser::DocumentParser;
use crate::llm_manager::LLMManager;
use crate::vfs::attachment_config::AttachmentConfig;
use crate::vfs::database::VfsDatabase;
use crate::vfs::error::{VfsError, VfsResult};
use crate::vfs::index_service::VfsIndexService;
use crate::vfs::pdf_processing_service::{PdfProcessingService, ProcessingStage};
use crate::vfs::repos::pdf_preview::{render_pdf_preview, PdfPreviewConfig};
use crate::vfs::repos::{VfsBlobRepo, VfsFileRepo, VfsResourceRepo};
use crate::vfs::types::*;
use crate::vfs::unit_builder::UnitBuildInput;

use super::pdf_handlers::{image_needs_compression_with_conn, pdf_preview_needs_compression};

// ============================================================================
// 前端输入/输出类型
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VfsUploadFileParams {
    pub name: String,
    pub mime_type: String,
    pub base64_content: String,
    #[serde(default)]
    pub file_type: Option<String>,
    #[serde(default)]
    pub folder_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VfsUploadFileResult {
    pub file: VfsFile,
    pub source_id: String,
    pub resource_hash: String,
    pub is_new: bool,
    /// ★ 2026-01 新增：OCR 处理状态
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocr_status: Option<OcrStatus>,
    /// ★ 2026-01 新增：索引状态
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_status: Option<IndexStatus>,
}

/// ★ 2026-01 新增：索引处理状态结构体
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStatus {
    /// 是否已加入索引队列
    pub queued: bool,
    /// 创建的索引单元数量
    pub units_created: u32,
    /// 用户可见的状态消息
    pub message: String,
}

/// ★ 2026-01 新增：OCR 处理状态结构体
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrStatus {
    /// OCR 是否被执行
    pub performed: bool,
    /// 跳过原因（如果跳过）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
    /// 成功的页数（PDF）
    pub success_count: u32,
    /// 失败的页数（PDF）
    pub failed_count: u32,
    /// Blob 缺失的页数（PDF）
    pub blob_missing_count: u32,
    /// 总页数（PDF）
    pub total_pages: u32,
    /// 是否全部成功
    pub all_success: bool,
    /// 用户可见的状态消息
    pub message: String,
}

/// PDF 文本提取阈值默认值（字符数）
/// 如果提取的文本少于此阈值，则认为是扫描版 PDF，需要触发 OCR
const DEFAULT_PDF_TEXT_THRESHOLD: usize = 100;

/// OCR 策略配置
#[derive(Debug, Clone)]
struct OcrStrategyConfig {
    /// 是否启用自动 OCR
    pub enabled: bool,
    /// 多模态模型跳过 OCR（多模态模型可直接理解图片）
    pub skip_for_multimodal: bool,
    /// PDF 文本阈值（字符数，低于此值触发 OCR）
    pub pdf_text_threshold: usize,
    /// 是否对图片启用 OCR
    pub ocr_images: bool,
    /// 是否对扫描版 PDF 启用 OCR
    pub ocr_scanned_pdf: bool,
}

impl Default for OcrStrategyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            // ★ 2026-01 修复：默认不跳过 OCR，确保文本索引有内容
            // 即使使用多模态模型，OCR 文本对于 RAG 检索和文本模型注入仍然必要
            skip_for_multimodal: false,
            pdf_text_threshold: DEFAULT_PDF_TEXT_THRESHOLD,
            ocr_images: true,
            ocr_scanned_pdf: true,
        }
    }
}

impl OcrStrategyConfig {
    /// 从数据库设置加载配置
    fn load_from_db(db: &crate::database::Database) -> Self {
        let mut config = Self::default();

        if let Ok(Some(v)) = db.get_setting("ocr.enabled") {
            config.enabled = v.to_lowercase() == "true";
        }
        if let Ok(Some(v)) = db.get_setting("ocr.skip_for_multimodal") {
            config.skip_for_multimodal = v.to_lowercase() == "true";
        }
        if let Ok(Some(v)) = db.get_setting("ocr.pdf_text_threshold") {
            if let Ok(n) = v.parse::<usize>() {
                config.pdf_text_threshold = n;
            }
        }
        if let Ok(Some(v)) = db.get_setting("ocr.images") {
            config.ocr_images = v.to_lowercase() == "true";
        }
        if let Ok(Some(v)) = db.get_setting("ocr.scanned_pdf") {
            config.ocr_scanned_pdf = v.to_lowercase() == "true";
        }

        config
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VfsFileContentResult {
    pub content: Option<String>,
    pub found: bool,
}

// ============================================================================
// 文件操作命令
// ============================================================================

#[tauri::command]
pub async fn vfs_upload_file(
    params: VfsUploadFileParams,
    vfs_db: State<'_, Arc<VfsDatabase>>,
    llm_manager: State<'_, Arc<LLMManager>>,
    database: State<'_, crate::database::Database>,
    pdf_processing_service: State<'_, Arc<PdfProcessingService>>,
) -> VfsResult<VfsUploadFileResult> {
    // ★ 2026-02 重构：llm_manager 参数保留用于未来图片 OCR 支持
    // 当前 PDF OCR 由 Pipeline 异步处理，图片 OCR 暂不支持
    let _ = &llm_manager;

    // 加载 OCR 策略配置
    let ocr_config = OcrStrategyConfig::load_from_db(&database);

    log::info!(
        "[VFS::handlers] vfs_upload_file: name={}, mime_type={}, folder_id={:?}",
        params.name,
        params.mime_type,
        params.folder_id
    );

    let content = BASE64
        .decode(&params.base64_content)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    if !crate::vfs::repos::VfsAttachmentRepo::is_supported_upload_type(&params.name, &params.mime_type) {
        return Err(VfsError::InvalidArgument {
            param: "mime_type".to_string(),
            reason: format!(
                "Unsupported mime type or file extension: {} ({})",
                params.mime_type, params.name
            ),
        });
    }

    let max_size = crate::vfs::repos::VfsAttachmentRepo::max_upload_size_bytes(&params.mime_type);
    if content.len() > max_size {
        let max_mb = max_size / (1024 * 1024);
        let actual_mb = content.len() as f64 / (1024.0 * 1024.0);
        return Err(VfsError::InvalidArgument {
            param: "base64_content".to_string(),
            reason: format!("File too large: max {}MB, got {:.2}MB", max_mb, actual_mb),
        });
    }

    let mut hasher = Sha256::new();
    hasher.update(&content);
    let sha256 = format!("{:x}", hasher.finalize());

    let file_type = params
        .file_type
        .unwrap_or_else(|| VfsFile::infer_file_type(&params.mime_type).to_string());

    let size = content.len() as i64;

    let conn = vfs_db.get_conn_safe()?;
    let blobs_dir = vfs_db.blobs_dir();
    let is_image = params.mime_type.starts_with("image/");

    let existing =
        VfsFileRepo::get_by_sha256_with_conn(&conn, &sha256)?;

    if let Some(file) = existing {
        if file.status == "active" {
            log::info!("[VFS::handlers] File reused: {}", file.id);
            let is_pdf = file.mime_type.as_deref() == Some("application/pdf")
                || file.file_name.to_lowercase().ends_with(".pdf");
            let is_image = file
                .mime_type
                .as_deref()
                .map(|m| m.starts_with("image/"))
                .unwrap_or(false);
            let mut needs_processing = false;

            if is_pdf {
                if let Some(ref preview_json) = file.preview_json {
                    needs_processing = pdf_preview_needs_compression(preview_json);
                }
            } else if is_image {
                needs_processing = image_needs_compression_with_conn(&conn, &blobs_dir, &file.id);
            }

            if needs_processing {
                let file_id = file.id.clone();
                let media_service = pdf_processing_service.inner().clone();
                let start_stage = if is_pdf {
                    Some(ProcessingStage::OcrProcessing)
                } else {
                    Some(ProcessingStage::ImageCompression)
                };
                tokio::spawn(async move {
                    log::info!(
                        "[VFS::handlers] Starting media pipeline for reused file: {} (pdf={}, image={})",
                        file_id, is_pdf, is_image
                    );
                    if let Err(e) = media_service.start_pipeline(&file_id, start_stage).await {
                        log::error!(
                            "[VFS::handlers] Failed to start media pipeline for reused file {}: {}",
                            file_id,
                            e
                        );
                    }
                });
            }
            return Ok(VfsUploadFileResult {
                source_id: file.id.clone(),
                resource_hash: sha256,
                is_new: false,
                file,
                // 已有文件不需要 OCR/索引状态
                ocr_status: None,
                index_status: None,
            });
        }
    }

    // TODO(transaction): 以下多步操作（store_blob → create_file_with_doc_data_in_folder → sync_resource_units）
    // 目前缺少 handler 级别的事务保护。create_file_with_doc_data_in_folder 已有内部 SAVEPOINT，
    // 但 store_blob 涉及文件系统写入（无法被数据库事务回滚），sync_resource_units 使用独立的
    // VfsIndexService（可能获取独立连接）。若 create_file_with_doc_data_in_folder 失败，
    // 已写入的 blob 文件和 DB 记录会成为孤儿数据（因去重设计影响较小，但仍应清理）。
    // 考虑方案：1) 用 SAVEPOINT 包裹 store_blob_db + create_file 的 DB 部分；
    //          2) 失败时补偿删除 blob 文件；3) 后台定期清理孤儿 blob。
    let blob_hash = if is_image || size >= 1024 * 1024 {
        let blob = VfsBlobRepo::store_blob_with_conn(
            &conn,
            &blobs_dir,
            &content,
            Some(&params.mime_type),
            None,
        )?;
        Some(blob.hash)
    } else {
        None
    };

    // ★ P2-1 修复：添加文档处理逻辑（与 vfs_upload_attachment 保持一致）
    let is_pdf =
        params.mime_type == "application/pdf" || params.name.to_lowercase().ends_with(".pdf");

    let (preview_json, extracted_text, page_count): (Option<String>, Option<String>, Option<i32>) =
        if is_pdf {
            log::info!(
                "[VFS::handlers] PDF detected, triggering preview render: {}",
                params.name
            );

            {
                let vfs_db_clone = vfs_db.inner().clone();
                let blobs_dir_clone = blobs_dir.to_path_buf();
                let content_clone = content.clone();
                match tokio::task::spawn_blocking(move || {
                    let conn = vfs_db_clone.get_conn_safe()?;
                    render_pdf_preview(
                        &conn,
                        &blobs_dir_clone,
                        &content_clone,
                        &PdfPreviewConfig::default(),
                    )
                })
                .await
                {
                    Ok(Ok(result)) => {
                        let preview_str = result
                            .preview_json
                            .as_ref()
                            .and_then(|p| serde_json::to_string(p).ok());
                        log::info!(
                            "[VFS::handlers] PDF preview rendered: {} pages, text_len={}, has_preview={}",
                            result.page_count,
                            result.extracted_text.as_ref().map(|t| t.len()).unwrap_or(0),
                            preview_str.is_some()
                        );
                        (
                            preview_str,
                            result.extracted_text,
                            Some(result.page_count as i32),
                        )
                    }
                    Ok(Err(e)) => {
                        log::warn!("[VFS::handlers] PDF preview failed: {}", e);
                        (None, None, None)
                    }
                    Err(e) => {
                        log::warn!("[VFS::handlers] PDF render task panicked: {}", e);
                        (None, None, None)
                    }
                }
            }
        } else {
            // 非 PDF 文件：尝试解析文本内容
            let extension = std::path::Path::new(&params.name)
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s.to_lowercase());

            let supported_extensions = [
                "docx", "xlsx", "xls", "xlsb", "ods", "pptx", "epub", "rtf", "txt", "md", "html",
                "htm", "csv", "json", "xml",
            ];

            if let Some(ref ext) = extension {
                if supported_extensions.contains(&ext.as_str()) {
                    let parser = DocumentParser::new();
                    match parser.extract_text_from_bytes(&params.name, content.clone()) {
                        Ok(text) => {
                            if !text.trim().is_empty() {
                                log::info!(
                                    "[VFS::handlers] Extracted text from {}: {} chars",
                                    params.name,
                                    text.len()
                                );
                                (None, Some(text), None)
                            } else {
                                (None, None, None)
                            }
                        }
                        Err(e) => {
                            log::warn!(
                                "[VFS::handlers] Failed to extract text from {}: {}",
                                params.name,
                                e
                            );
                            (None, None, None)
                        }
                    }
                } else {
                    (None, None, None)
                }
            } else {
                (None, None, None)
            }
        };

    let target_folder_id = match params.folder_id {
        Some(ref id) if !id.is_empty() => Some(id.clone()),
        _ => {
            let config = AttachmentConfig::new(vfs_db.inner().clone());
            Some(
                config
                    .get_or_create_root_folder()
                    ?,
            )
        }
    };

    let file = match VfsFileRepo::create_file_with_doc_data_in_folder(
        &conn,
        &sha256,
        &params.name,
        size,
        &file_type,
        Some(&params.mime_type),
        blob_hash.as_deref(),
        None,
        target_folder_id.as_deref(),
        preview_json.as_deref(),
        extracted_text.as_deref(),
        page_count,
    ) {
        Ok(file) => file,
        Err(e) => {
            if let Some(ref hash) = blob_hash {
                log::warn!(
                    "[VFS::handlers] 文件记录创建失败，补偿清理 blob: hash={}…",
                    &hash[..hash.len().min(16)]
                );
                if let Err(cleanup_err) =
                    VfsBlobRepo::cleanup_blob_with_conn(&conn, &blobs_dir, hash)
                {
                    log::error!(
                        "[VFS::handlers] 补偿清理 blob 失败（将由后台清理）: {}",
                        cleanup_err
                    );
                }
            }
            return Err(VfsError::Other(e.to_string()));
        }
    };

    log::info!(
        "[VFS::handlers] File uploaded: {} (type={}, folder={:?}, has_text={})",
        file.id,
        file_type,
        target_folder_id,
        extracted_text.is_some()
    );

    // ★ 2026-02 重构：移除旧的同步 OCR 逻辑，改由 Pipeline 异步处理
    // 参考：docs/design/pdf-preprocessing-pipeline.md
    //
    // 旧逻辑问题：
    // 1. 同步 OCR 阻塞上传，用户体验差
    // 2. 与新 Pipeline 异步 OCR 重复执行
    //
    // 新逻辑：
    // 1. 上传时只做文本提取和页面渲染（Stage 1-2）
    // 2. OCR 和向量索引由 Pipeline 异步处理（Stage 3-4）
    // 3. 前端通过事件监听处理进度

    // OCR 相关变量设为 None，由 Pipeline 异步填充
    let ocr_text: Option<String> = None;
    let ocr_pages_json: Option<String> = None;

    // 判断是否需要触发 Pipeline OCR（用于状态返回）
    let needs_image_ocr = is_image && ocr_config.enabled && ocr_config.ocr_images;
    let needs_pdf_ocr = is_pdf
        && ocr_config.enabled
        && ocr_config.ocr_scanned_pdf
        && extracted_text.as_ref().map(|t| t.len()).unwrap_or(0) < ocr_config.pdf_text_threshold;
    let needs_ocr = needs_image_ocr || needs_pdf_ocr;

    log::debug!(
        "[VFS::handlers] OCR config: enabled={}, threshold={}, images={}, pdf={}, needs_ocr={}",
        ocr_config.enabled,
        ocr_config.pdf_text_threshold,
        ocr_config.ocr_images,
        ocr_config.ocr_scanned_pdf,
        needs_ocr
    );

    // ★ P2-1 修复：上传后自动同步 Units 以触发索引
    // ★ 2026-01 新增：收集索引状态用于返回
    let mut index_units_created: u32 = 0;
    let mut index_queued = false;
    let mut index_error: Option<String> = None;

    if let Some(ref resource_id) = file.resource_id {
        let index_service = VfsIndexService::new(vfs_db.inner().clone());
        let input = UnitBuildInput {
            resource_id: resource_id.clone(),
            resource_type: "file".to_string(),
            data: None,
            ocr_text: ocr_text.clone(),
            ocr_pages_json: ocr_pages_json.clone(),
            blob_hash: file.blob_hash.clone(),
            page_count: file.page_count,
            extracted_text: file.extracted_text.clone(),
            preview_json: file.preview_json.clone(),
        };
        match index_service.sync_resource_units(input) {
            Ok(units) => {
                index_units_created = units.len() as u32;
                index_queued = true;
                log::info!(
                    "[VFS::handlers] Auto-synced {} units for file {}",
                    units.len(),
                    file.id
                );
            }
            Err(e) => {
                index_error = Some(e.to_string());
                log::warn!(
                    "[VFS::handlers] Failed to auto-sync units for file {}: {}",
                    file.id,
                    e
                );
            }
        }
    }

    // ★ 2026-02 重构：OCR 状态改为由 Pipeline 异步处理
    // 上传时返回"等待处理"状态，前端通过事件监听实际进度
    let ocr_status = if needs_ocr {
        let message = if !ocr_config.enabled {
            "OCR 已在设置中禁用".to_string()
        } else if is_pdf {
            "OCR 将由 Pipeline 异步处理".to_string()
        } else if is_image {
            "图片 OCR 将由 Pipeline 处理".to_string()
        } else {
            "等待 OCR 处理".to_string()
        };

        Some(OcrStatus {
            performed: false, // 上传时未执行，由 Pipeline 异步执行
            skip_reason: if !ocr_config.enabled {
                Some("OCR 已在设置中禁用".to_string())
            } else {
                None
            },
            success_count: 0,
            failed_count: 0,
            blob_missing_count: 0,
            total_pages: page_count.unwrap_or(0) as u32,
            all_success: false, // 上传时尚未完成
            message,
        })
    } else {
        None
    };

    // ★ 2026-01 新增：构建索引状态
    let index_status = if file.resource_id.is_some() {
        let message = if let Some(ref err) = index_error {
            format!("索引失败: {}", err)
        } else if index_queued && index_units_created > 0 {
            format!("已加入索引队列（{} 个单元）", index_units_created)
        } else if index_queued {
            "已加入索引队列".to_string()
        } else {
            "未创建索引".to_string()
        };

        Some(IndexStatus {
            queued: index_queued,
            units_created: index_units_created,
            message,
        })
    } else {
        None
    };

    // ★ 2026-02 修复：PDF/图片 上传后异步触发 Pipeline
    // PDF: Stage 1-2（文本提取、页面渲染）已在 create_file_with_doc_data_in_folder 中完成，从 OCR 阶段开始
    // 图片: 从压缩阶段开始
    let is_image = params.mime_type.starts_with("image/");
    if is_pdf || is_image {
        let file_id = file.id.clone();
        let media_service = pdf_processing_service.inner().clone();
        let start_stage = if is_pdf {
            Some(ProcessingStage::OcrProcessing)
        } else {
            Some(ProcessingStage::ImageCompression)
        };
        tokio::spawn(async move {
            log::info!(
                "[VFS::handlers] Starting media pipeline for file: {} (pdf={}, image={})",
                file_id,
                is_pdf,
                is_image
            );
            if let Err(e) = media_service.start_pipeline(&file_id, start_stage).await {
                log::error!(
                    "[VFS::handlers] Failed to start media pipeline for file {}: {}",
                    file_id,
                    e
                );
            }
        });
    }

    Ok(VfsUploadFileResult {
        source_id: file.id.clone(),
        resource_hash: sha256,
        is_new: true,
        file,
        ocr_status,
        index_status,
    })
}

#[tauri::command]
pub async fn vfs_get_file(
    file_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Option<VfsFile>> {
    if !file_id.starts_with("file_") && !file_id.starts_with("tb_") {
        return Err(VfsError::Other(format!("Invalid file ID format: {}", file_id)));
    }

    Ok(VfsFileRepo::get_file(&vfs_db, &file_id)?)
}

#[tauri::command]
pub async fn vfs_list_files(
    file_type: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Vec<VfsFile>> {
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    match file_type {
        Some(ft) => VfsFileRepo::list_files_by_type(&vfs_db, &ft, limit, offset),
        None => VfsFileRepo::list_files(&vfs_db, limit, offset),
    }
}

/// ★ M-12 修复：软删除文件时同步清理向量索引
///
/// 确保被删除的文件不会在 RAG 检索中被错误返回。
#[tauri::command]
pub async fn vfs_delete_file(
    file_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
    lance_store: State<'_, Arc<crate::vfs::lance_store::VfsLanceStore>>,
) -> VfsResult<()> {
    use crate::vfs::index_service::VfsIndexService;

    if !file_id.starts_with("file_") {
        return Err(VfsError::Other(format!("Invalid file ID format: {}", file_id)));
    }

    let index_service = VfsIndexService::new(Arc::clone(&vfs_db));

    VfsFileRepo::delete_file_with_index_cleanup(
        &vfs_db,
        &file_id,
        &index_service,
        lance_store.as_ref(),
    )
    .await
}

#[tauri::command]
pub async fn vfs_get_file_content(
    file_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<VfsFileContentResult> {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

    // 支持 file_ 与 tb_（教材）
    if !file_id.starts_with("file_") && !file_id.starts_with("tb_") {
        return Err(VfsError::Other(format!("Invalid file ID format: {}", file_id)));
    }

    let conn = vfs_db.get_conn_safe()?;

    let file = match VfsFileRepo::get_file_with_conn(&conn, &file_id)? {
        Some(f) => f,
        None => {
            return Ok(VfsFileContentResult {
                content: None,
                found: false,
            })
        }
    };

    if let Some(ref blob_hash) = file.blob_hash {
        let blobs_dir = vfs_db.blobs_dir();
        if let Some(path) = VfsBlobRepo::get_blob_path_with_conn(&conn, &blobs_dir, blob_hash)?
        {
            let data = std::fs::read(&path)?;
            let base64 = BASE64.encode(&data);
            return Ok(VfsFileContentResult {
                content: Some(base64),
                found: true,
            });
        }
    }

    if let Some(ref resource_id) = file.resource_id {
        if let Some(resource) = VfsResourceRepo::get_resource_with_conn(&conn, resource_id)?
        {
            if let Some(data) = resource.data {
                return Ok(VfsFileContentResult {
                    content: Some(data),
                    found: true,
                });
            }
        }
    }

    Ok(VfsFileContentResult {
        content: None,
        found: false,
    })
}
