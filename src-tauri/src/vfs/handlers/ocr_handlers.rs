//! VFS OCR/文本透视 Tauri 命令处理器
//!
//! 提供 OCR 文本查看、清除、文本块查看等数据透视命令。

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::vfs::database::VfsDatabase;
use crate::vfs::error::VfsResult;
use crate::vfs::repos::VfsIndexStateRepo;

// ============================================================================
// 输出类型
// ============================================================================

/// OCR 文本查看结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceOcrInfo {
    pub resource_id: String,
    pub resource_type: String,
    pub has_ocr: bool,
    pub ocr_text: Option<String>,
    pub ocr_text_length: usize,
    pub extracted_text: Option<String>,
    pub extracted_text_length: usize,
    pub active_source: String,
    pub ocr_pages: Option<Vec<OcrPageInfo>>,
}

/// 单页 OCR 信息
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrPageInfo {
    pub page_index: usize,
    pub text: String,
    pub char_count: usize,
    pub is_failed: bool,
}

/// 文本块信息（用于数据透视）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextChunkInfo {
    pub unit_id: String,
    pub unit_index: i32,
    pub text_content: Option<String>,
    pub text_source: Option<String>,
    pub text_state: String,
    pub text_chunk_count: i32,
    pub char_count: usize,
}

// ============================================================================
// OCR 文本命令
// ============================================================================

/// 获取资源的 OCR 文本和提取文本详情
///
/// 数据透视：让用户能看到 OCR 识别了什么，与提取文本对比
#[tauri::command]
pub async fn vfs_get_resource_ocr_info(
    resource_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<ResourceOcrInfo> {
    log::info!(
        "[VFS::handlers] vfs_get_resource_ocr_info: resource_id={}",
        resource_id
    );

    let conn = vfs_db.get_conn_safe()?;

    let resource_type: String = conn
        .query_row(
            "SELECT type FROM resources WHERE id = ?1",
            rusqlite::params![resource_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Resource not found: {}", e))?;

    let ocr_text: Option<String> = conn
        .query_row(
            "SELECT ocr_text FROM resources WHERE id = ?1",
            rusqlite::params![resource_id],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    let file_info: Option<(Option<String>, Option<String>)> = conn
        .query_row(
            "SELECT extracted_text, ocr_pages_json FROM files WHERE resource_id = ?1",
            rusqlite::params![resource_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok()
        .or_else(|| {
            let source_id: Option<String> = conn
                .query_row(
                    "SELECT source_id FROM resources WHERE id = ?1",
                    rusqlite::params![resource_id],
                    |row| row.get(0),
                )
                .ok()
                .flatten();
            source_id.and_then(|sid| {
                conn.query_row(
                    "SELECT extracted_text, ocr_pages_json FROM files WHERE id = ?1",
                    rusqlite::params![sid],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .ok()
            })
        });

    let (extracted_text, ocr_pages_json) = file_info.unwrap_or((None, None));

    let ocr_text_length = ocr_text.as_ref().map(|t| t.len()).unwrap_or(0);
    let extracted_text_length = extracted_text.as_ref().map(|t| t.len()).unwrap_or(0);

    let active_source = if ocr_text_length > 0 {
        "ocr".to_string()
    } else if extracted_text_length > 0 {
        "extracted".to_string()
    } else {
        "none".to_string()
    };

    let ocr_pages = parse_ocr_pages_for_display(&ocr_pages_json);

    Ok(ResourceOcrInfo {
        resource_id,
        resource_type,
        has_ocr: ocr_text_length > 0 || ocr_pages.is_some(),
        ocr_text,
        ocr_text_length,
        extracted_text,
        extracted_text_length,
        active_source,
        ocr_pages,
    })
}

fn parse_ocr_pages_for_display(ocr_pages_json: &Option<String>) -> Option<Vec<OcrPageInfo>> {
    let json_str = ocr_pages_json.as_ref()?;
    if json_str.trim().is_empty() {
        return None;
    }

    if let Ok(pages) = serde_json::from_str::<Vec<Option<String>>>(json_str) {
        let result: Vec<OcrPageInfo> = pages
            .into_iter()
            .enumerate()
            .map(|(i, text_opt)| {
                let (text, is_failed) = match text_opt {
                    Some(ref t) if t == "[OCR_FAILED]" => (String::new(), true),
                    Some(t) => {
                        let failed = t.trim().is_empty();
                        (t, failed)
                    }
                    None => (String::new(), true),
                };
                OcrPageInfo {
                    page_index: i,
                    char_count: text.len(),
                    text,
                    is_failed,
                }
            })
            .collect();
        return Some(result);
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct OcrPagesJsonCompat {
        pages: Vec<OcrPageResultCompat>,
        #[allow(dead_code)]
        total_pages: Option<usize>,
        #[allow(dead_code)]
        completed_at: Option<String>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct OcrPageResultCompat {
        page_index: usize,
        blocks: Vec<OcrTextBlockCompat>,
    }
    #[derive(Deserialize)]
    struct OcrTextBlockCompat {
        text: String,
    }

    if let Ok(ocr_json) = serde_json::from_str::<OcrPagesJsonCompat>(json_str) {
        let result: Vec<OcrPageInfo> = ocr_json
            .pages
            .into_iter()
            .map(|page| {
                let text = page
                    .blocks
                    .iter()
                    .map(|b| b.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                OcrPageInfo {
                    page_index: page.page_index,
                    char_count: text.len(),
                    is_failed: text.trim().is_empty(),
                    text,
                }
            })
            .collect();
        return Some(result);
    }

    None
}

/// 清除资源的 OCR 数据（用于强制重新 OCR）
///
/// 清除 resources.ocr_text 和 files.ocr_pages_json，
/// 然后重置索引状态为 pending，下次索引时会重新触发 OCR
#[tauri::command]
pub async fn vfs_clear_resource_ocr(
    resource_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<bool> {
    log::info!(
        "[VFS::handlers] vfs_clear_resource_ocr: resource_id={}",
        resource_id
    );

    let conn = vfs_db.get_conn_safe()?;

    conn.execute(
        "UPDATE resources SET ocr_text = NULL, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![chrono::Utc::now().timestamp_millis(), resource_id],
    )
    .map_err(|e| format!("Failed to clear ocr_text: {}", e))?;

    let now_str = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    let files_updated = conn
        .execute(
            "UPDATE files SET ocr_pages_json = NULL, updated_at = ?1 WHERE resource_id = ?2",
            rusqlite::params![now_str, resource_id],
        )
        .map_err(|e| format!("Failed to clear ocr_pages_json: {}", e))?;

    if files_updated == 0 {
        let source_id: Option<String> = conn
            .query_row(
                "SELECT source_id FROM resources WHERE id = ?1",
                rusqlite::params![resource_id],
                |row| row.get(0),
            )
            .ok()
            .flatten();
        if let Some(sid) = source_id {
            let _ = conn.execute(
                "UPDATE files SET ocr_pages_json = NULL, updated_at = ?1 WHERE id = ?2",
                rusqlite::params![now_str, sid],
            );
        }
    }

    if let Err(e) = VfsIndexStateRepo::mark_pending(&vfs_db, &resource_id) {
        log::warn!(
            "[VFS::handlers] Failed to mark resource as pending after OCR clear: {}",
            e
        );
    }

    log::info!(
        "[VFS::handlers] Cleared OCR data for resource {} and marked as pending",
        resource_id
    );
    Ok(true)
}

/// 获取资源的文本块列表（数据透视）
///
/// 让用户能看到系统把内容切成了哪些块
#[tauri::command]
pub async fn vfs_get_resource_text_chunks(
    resource_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Vec<TextChunkInfo>> {
    log::info!(
        "[VFS::handlers] vfs_get_resource_text_chunks: resource_id={}",
        resource_id
    );

    let conn = vfs_db.get_conn_safe()?;

    let mut stmt = conn
        .prepare(
            "SELECT id, unit_index, text_content, text_source, text_state, text_chunk_count
             FROM vfs_index_units
             WHERE resource_id = ?1
             ORDER BY unit_index ASC",
        )
        .map_err(|e| format!("Prepare failed: {}", e))?;

    let chunks: Vec<TextChunkInfo> = stmt
        .query_map(rusqlite::params![resource_id], |row| {
            let text_content: Option<String> = row.get(2)?;
            let char_count = text_content.as_ref().map(|t| t.len()).unwrap_or(0);
            Ok(TextChunkInfo {
                unit_id: row.get(0)?,
                unit_index: row.get(1)?,
                text_content,
                text_source: row.get(3)?,
                text_state: row.get(4)?,
                text_chunk_count: row.get(5)?,
                char_count,
            })
        })
        .map_err(|e| format!("Query failed: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row mapping failed: {}", e))?;

    log::info!(
        "[VFS::handlers] Found {} text chunks for resource {}",
        chunks.len(),
        resource_id
    );
    Ok(chunks)
}
