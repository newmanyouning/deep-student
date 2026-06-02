//! VFS 附件操作 Tauri 命令处理器
//!
//! 提供附件上传、查询、删除等 Tauri 命令。
//!
//! ## 命令
//! - `vfs_upload_attachment`: 上传附件
//! - `vfs_get_attachment_config`: 获取附件配置
//! - `vfs_set_attachment_root_folder`: 设置附件根文件夹
//! - `vfs_create_attachment_root_folder`: 创建附件根文件夹
//! - `vfs_get_or_create_attachment_root_folder`: 获取或创建附件根文件夹
//! - `vfs_get_attachment_content`: 获取附件内容
//! - `vfs_get_attachment`: 获取附件元数据
//! - `vfs_delete_attachment`: 删除附件

use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD, Engine};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::vfs::attachment_config::AttachmentConfig;
use crate::vfs::database::VfsDatabase;
use crate::vfs::error::{VfsError, VfsResult};
use crate::vfs::index_service::VfsIndexService;
use crate::vfs::pdf_processing_service::{PdfProcessingService, ProcessingStage};
use crate::vfs::repos::{VfsAttachmentRepo, VfsFolderRepo};
use crate::vfs::types::*;
use crate::vfs::unit_builder::UnitBuildInput;

use super::pdf_handlers::{image_needs_compression_with_conn, pdf_preview_needs_compression};

// ============================================================================
// 前端输入/输出类型
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VfsUploadAttachmentParamsExt {
    pub name: String,
    pub mime_type: String,
    pub base64_content: String,
    #[serde(default)]
    pub attachment_type: Option<String>,
    #[serde(default)]
    pub folder_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentConfigOutput {
    pub attachment_root_folder_id: Option<String>,
    pub attachment_root_folder_title: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VfsAttachmentContentResult {
    /// Base64 编码的内容（如果找到）
    pub content: Option<String>,
    /// 是否找到附件
    pub found: bool,
    /// 可选错误信息（向后兼容：旧前端可忽略此字段）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ============================================================================
// 附件操作命令
// ============================================================================

#[tauri::command]
pub async fn vfs_upload_attachment(
    params: VfsUploadAttachmentParamsExt,
    vfs_db: State<'_, Arc<VfsDatabase>>,
    pdf_processing_service: State<'_, Arc<PdfProcessingService>>,
) -> VfsResult<VfsUploadAttachmentResult> {
    log::info!(
        "[VFS::handlers] vfs_upload_attachment: name={}, mime_type={}, folder_id={:?}",
        params.name,
        params.mime_type,
        params.folder_id
    );

    // 判断是否为 PDF 文件
    let is_pdf =
        params.mime_type == "application/pdf" || params.name.to_lowercase().ends_with(".pdf");

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

    let upload_params = VfsUploadAttachmentParams {
        name: params.name.clone(),
        mime_type: params.mime_type.clone(),
        base64_content: params.base64_content,
        attachment_type: params.attachment_type,
    };

    let result =
        VfsAttachmentRepo::upload_with_folder(&vfs_db, upload_params, target_folder_id.as_deref())?;

    log::info!(
        "[VFS::handlers] Attachment {}: source_id={}, hash={}, folder={:?}",
        if result.is_new { "uploaded" } else { "reused" },
        result.source_id,
        &result.resource_hash[..16.min(result.resource_hash.len())],
        target_folder_id
    );

    // ★ P2 修复：上传后自动同步 Units 以触发索引
    if let Some(ref resource_id) = result.attachment.resource_id {
        let index_service = VfsIndexService::new(vfs_db.inner().clone());
        let input = UnitBuildInput {
            resource_id: resource_id.clone(),
            resource_type: "attachment".to_string(),
            data: None,
            ocr_text: None,
            ocr_pages_json: None,
            blob_hash: result.attachment.blob_hash.clone(),
            page_count: result.attachment.page_count,
            extracted_text: result.attachment.extracted_text.clone(),
            preview_json: result.attachment.preview_json.clone(),
        };
        match index_service.sync_resource_units(input) {
            Ok(units) => {
                log::info!(
                    "[VFS::handlers] Auto-synced {} units for attachment {}",
                    units.len(),
                    result.source_id
                );
            }
            Err(e) => {
                log::warn!(
                    "[VFS::handlers] Failed to auto-sync units for attachment {}: {}",
                    result.source_id,
                    e
                );
            }
        }
    }

    // ★ 2026-02 修复：PDF/图片 上传后异步触发 Pipeline
    // PDF: Stage 1-2（文本提取、页面渲染）已在 upload_with_conn 中完成，从 OCR 阶段开始
    // 图片: 从压缩阶段开始
    let is_image = params.mime_type.starts_with("image/");

    // ★ v2.1 新增：查询处理状态并填充返回值
    // 对于重用的附件，需要返回实际的处理状态
    let (mut processing_status, mut processing_percent, mut ready_modes, mut needs_processing) =
        if is_pdf || is_image {
            match pdf_processing_service.get_status(&result.source_id) {
                Ok(Some(status)) => {
                    let percent = status.progress.percent;
                    let modes = status.progress.ready_modes.clone();
                    let stage = status.progress.stage.clone();
                    // ★ v2.1: 判断是否需要继续处理（未完成且非错误状态）
                    let needs_resume = stage != "completed"
                        && stage != "completed_with_issues"
                        && stage != "error";
                    (Some(stage), Some(percent), Some(modes), needs_resume)
                }
                _ => {
                    // 没有处理状态，设置初始值，需要启动处理
                    // ★ P0 架构改造：初始 ready_modes 不再包含 image
                    // image 模式必须等到压缩完成后才就绪
                    if is_pdf {
                        // PDF: text 在上传时已提取完成，image 需要等页面压缩
                        let text_ready = result
                            .attachment
                            .extracted_text
                            .as_ref()
                            .map(|t| !t.trim().is_empty())
                            .unwrap_or(false);
                        let mut modes = Vec::new();
                        if text_ready {
                            modes.push("text".to_string());
                        }
                        (
                            Some("page_compression".to_string()),
                            Some(25.0),
                            Some(modes),
                            true,
                        )
                    } else {
                        // 图片: 需要等压缩完成后 image 才就绪
                        (
                            Some("image_compression".to_string()),
                            Some(10.0),
                            Some(vec![]),
                            true,
                        )
                    }
                }
            }
        } else {
            (None, None, None, false)
        };

    // ★ P0 修复：旧数据缺失压缩结果时强制重新处理
    let mut needs_compression = false;
    if is_pdf {
        if let Some(ref preview_json) = result.attachment.preview_json {
            needs_compression = pdf_preview_needs_compression(preview_json);
        }
    } else if is_image {
        if let Ok(conn) = vfs_db.get_conn_safe() {
            needs_compression =
                image_needs_compression_with_conn(&conn, vfs_db.blobs_dir(), &result.source_id);
        }
    }

    if needs_compression && processing_status.as_deref() != Some("error") {
        needs_processing = true;
    }

    if needs_compression
        && (processing_status.as_deref() == Some("completed") || processing_status.is_none())
    {
        if is_pdf {
            let mut modes = Vec::new();
            if result
                .attachment
                .extracted_text
                .as_ref()
                .map(|t| !t.trim().is_empty())
                .unwrap_or(false)
            {
                modes.push("text".to_string());
            }
            processing_status = Some("page_compression".to_string());
            processing_percent = Some(25.0);
            ready_modes = Some(modes);
        } else if is_image {
            processing_status = Some("image_compression".to_string());
            processing_percent = Some(10.0);
            ready_modes = Some(vec![]);
        }
    }

    // ★ v2.1 修复：不仅新上传需要处理，重用但未完成的也需要继续处理
    if (result.is_new || needs_processing) && (is_pdf || is_image) {
        let file_id = result.source_id.clone();
        let media_service = pdf_processing_service.inner().clone();
        let start_stage = if is_pdf {
            Some(ProcessingStage::OcrProcessing)
        } else {
            Some(ProcessingStage::ImageCompression)
        };
        tokio::spawn(async move {
            log::info!(
                "[VFS::handlers] Starting media pipeline for attachment: {} (pdf={}, image={}, is_new={}, resume={})",
                file_id, is_pdf, is_image, result.is_new, !result.is_new
            );
            if let Err(e) = media_service.start_pipeline(&file_id, start_stage).await {
                log::error!(
                    "[VFS::handlers] Failed to start media pipeline for attachment {}: {}",
                    file_id,
                    e
                );
            }
        });
    }

    // 返回包含处理状态的结果
    Ok(VfsUploadAttachmentResult {
        source_id: result.source_id,
        resource_hash: result.resource_hash,
        is_new: result.is_new,
        attachment: result.attachment,
        processing_status,
        processing_percent,
        ready_modes,
    })
}

#[tauri::command]
pub async fn vfs_get_attachment_config(
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<AttachmentConfigOutput> {
    let config = AttachmentConfig::new(vfs_db.inner().clone());
    let root_id = config.get_root_folder_id()?;
    let root_title = config.get_root_folder_title()?;

    Ok(AttachmentConfigOutput {
        attachment_root_folder_id: root_id,
        attachment_root_folder_title: root_title,
    })
}

#[tauri::command]
pub async fn vfs_set_attachment_root_folder(
    folder_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<()> {
    if !VfsFolderRepo::folder_exists(&vfs_db, &folder_id)? {
        return Err(VfsError::Other(format!("Folder not found: {}", folder_id)));
    }

    let config = AttachmentConfig::new(vfs_db.inner().clone());
    config
        .set_root_folder_id(&folder_id)
}

#[tauri::command]
pub async fn vfs_create_attachment_root_folder(
    title: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<String> {
    let config = AttachmentConfig::new(vfs_db.inner().clone());
    Ok(config.create_root_folder(&title)?)
}

#[tauri::command]
pub async fn vfs_get_or_create_attachment_root_folder(
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<String> {
    let config = AttachmentConfig::new(vfs_db.inner().clone());
    config
        .get_or_create_root_folder()
}

/// 获取附件内容（Base64 编码）
///
/// ## 参数
/// - `attachment_id`: 附件/文件/教材 ID（att_xxx / file_xxx / tb_xxx）
///
/// ## 返回
/// - `Ok(VfsAttachmentContentResult)`: 包含 content/found（以及可选 error）字段
/// - `Err(String)`: 读取失败
///
/// ★ 2025-12-10 修复：返回结构体匹配前端 ImageContentView 期望的格式
#[tauri::command]
pub async fn vfs_get_attachment_content(
    attachment_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<VfsAttachmentContentResult> {
    log::info!(
        "[VFS::handlers] vfs_get_attachment_content: START id={}",
        attachment_id
    );

    // 验证附件 ID 格式（支持 att_、file_、tb_ 和 img_ 前缀）
    if !attachment_id.starts_with("att_")
        && !attachment_id.starts_with("file_")
        && !attachment_id.starts_with("tb_")
        && !attachment_id.starts_with("img_")
    {
        log::error!(
            "[VFS::handlers] Invalid attachment ID format: {}",
            attachment_id
        );
        return Err(VfsError::Other(format!("Invalid attachment ID format: {}", attachment_id)));
    }

    // ★ img_ 前缀：DOCX VLM 直提路径产生的图片 ID，blob hash 存在 questions.images_json 中
    if attachment_id.starts_with("img_") {
        let conn = vfs_db.get_conn_safe()?;
        // 在 questions.images_json 中搜索此 img_ ID，提取 blob hash
        let rows: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT images_json FROM questions WHERE images_json LIKE ?1 AND deleted_at IS NULL LIMIT 5")?;
            let iter = stmt
                .query_map(rusqlite::params![format!("%{}%", attachment_id)], |row| {
                    row.get::<_, String>(0)
                })?;
            iter.filter_map(|r| r.ok()).collect()
        };

        for images_json_str in &rows {
            if let Ok(images) = serde_json::from_str::<Vec<serde_json::Value>>(images_json_str) {
                for img in &images {
                    if img.get("id").and_then(|v| v.as_str()) == Some(&attachment_id) {
                        if let Some(blob_hash) = img.get("hash").and_then(|v| v.as_str()) {
                            // 从 blobs 表查 relative_path，再拼接 blobs_dir 得到绝对路径
                            let blob_path: Option<std::path::PathBuf> = conn
                                .query_row(
                                    "SELECT relative_path FROM blobs WHERE hash = ?1",
                                    rusqlite::params![blob_hash],
                                    |row| row.get::<_, String>(0),
                                )
                                .optional()
                                .ok()
                                .flatten()
                                .map(|rel| vfs_db.blobs_dir().join(rel));

                            let blob_path = match blob_path {
                                Some(p) => p,
                                None => {
                                    log::warn!(
                                        "[VFS::handlers] img_ blob not in DB: hash={}",
                                        blob_hash
                                    );
                                    continue;
                                }
                            };

                            if blob_path.exists() {
                                match std::fs::read(&blob_path) {
                                    Ok(data) => {
                                        let b64 = STANDARD.encode(&data);
                                        log::info!(
                                            "[VFS::handlers] vfs_get_attachment_content: img_ resolved via blob hash={}, size={}",
                                            blob_hash, data.len()
                                        );
                                        return Ok(VfsAttachmentContentResult {
                                            content: Some(b64),
                                            found: true,
                                            error: None,
                                        });
                                    }
                                    Err(e) => {
                                        log::warn!("[VFS::handlers] img_ blob read failed: {}", e);
                                    }
                                }
                            } else {
                                log::warn!(
                                    "[VFS::handlers] img_ blob file not found: {:?}",
                                    blob_path
                                );
                            }
                        }
                    }
                }
            }
        }

        log::warn!(
            "[VFS::handlers] vfs_get_attachment_content: img_ ID not resolved: {}",
            attachment_id
        );
        return Ok(VfsAttachmentContentResult {
            content: None,
            found: false,
            error: None,
        });
    }

    // ★ 详细日志：查询文件元数据
    {
        let conn = vfs_db.get_conn_safe()?;
        let meta: Option<(Option<String>, Option<String>, Option<String>)> = conn
            .query_row(
                "SELECT resource_id, blob_hash, original_path FROM files WHERE id = ?1",
                rusqlite::params![&attachment_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;

        log::info!(
            "[VFS::handlers] vfs_get_attachment_content: file meta for {}: resource_id={:?}, blob_hash={:?}, original_path={:?}",
            attachment_id,
            meta.as_ref().map(|m| &m.0),
            meta.as_ref().map(|m| &m.1),
            meta.as_ref().map(|m| &m.2),
        );
    }

    match VfsAttachmentRepo::get_content(&vfs_db, &attachment_id) {
        Ok(Some(content)) => {
            log::info!(
                "[VFS::handlers] vfs_get_attachment_content: SUCCESS id={}, content_len={}",
                attachment_id,
                content.len()
            );
            Ok(VfsAttachmentContentResult {
                content: Some(content),
                found: true,
                error: None,
            })
        }
        Ok(None) => {
            log::warn!(
                "[VFS::handlers] vfs_get_attachment_content: NOT FOUND id={}",
                attachment_id
            );
            Ok(VfsAttachmentContentResult {
                content: None,
                found: false,
                error: None,
            })
        }
        Err(e) => {
            let err_msg = e.to_string();
            log::error!(
                "[VFS::handlers] vfs_get_attachment_content: ERROR id={}, error={}",
                attachment_id,
                err_msg
            );
            Ok(VfsAttachmentContentResult {
                content: None,
                found: false,
                error: Some(err_msg),
            })
        }
    }
}

/// 获取附件元数据
///
/// ## 参数
/// - `attachment_id`: 附件 ID（att_xxx 或 file_xxx）
///
/// ## 返回
/// - `Ok(Some(VfsAttachment))`: 附件元数据
/// - `Ok(None)`: 附件不存在
/// - `Err(String)`: 查询失败
#[tauri::command]
pub async fn vfs_get_attachment(
    attachment_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Option<VfsAttachment>> {
    log::debug!("[VFS::handlers] vfs_get_attachment: id={}", attachment_id);

    if !attachment_id.starts_with("att_") && !attachment_id.starts_with("file_") {
        return Err(VfsError::Other(format!("Invalid attachment ID format: {}", attachment_id)));
    }

    Ok(VfsAttachmentRepo::get_by_id(&vfs_db, &attachment_id)?)
}

/// 软删除附件
///
/// 将附件标记为已删除（可恢复），同时清理相关索引。
/// 用于清理测试产生的废弃附件等场景。
#[tauri::command]
pub async fn vfs_delete_attachment(
    attachment_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<()> {
    log::info!(
        "[VFS::handlers] vfs_delete_attachment: id={}",
        attachment_id
    );

    if !attachment_id.starts_with("att_") {
        return Err(VfsError::Other(format!("Invalid attachment ID format: {}", attachment_id)));
    }

    Ok(VfsAttachmentRepo::delete_attachment(&vfs_db, &attachment_id)?)
}
