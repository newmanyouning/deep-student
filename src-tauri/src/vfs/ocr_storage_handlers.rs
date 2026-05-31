//! OCR 结果存储 Tauri 命令处理器

use std::sync::Arc;
use tauri::State;
use crate::vfs::database::VfsDatabase;
use crate::vfs::error::VfsResult;
use crate::vfs::ocr_storage;

/// 存储 OCR 结果
#[tauri::command]
pub fn vfs_ocr_store_result(
    vfs_db: State<'_, Arc<VfsDatabase>>,
    resource_id: String,
    text: String,
    confidence: f64,
    source: String,
) -> VfsResult<String> {
    let conn = vfs_db.get_conn_safe().map_err(|e| crate::vfs::error::VfsError::Database(e.to_string()))?;
    ocr_storage::store_ocr_result(&conn, &resource_id, &text, confidence, &source)
}

/// 列出 OCR 结果
#[tauri::command]
pub fn vfs_ocr_list_results(
    vfs_db: State<'_, Arc<VfsDatabase>>,
    resource_id: String,
) -> VfsResult<Vec<ocr_storage::OcrStorageEntry>> {
    let conn = vfs_db.get_conn_safe().map_err(|e| crate::vfs::error::VfsError::Database(e.to_string()))?;
    ocr_storage::list_ocr_results(&conn, &resource_id)
}

/// 删除 OCR 结果
#[tauri::command]
pub fn vfs_ocr_delete_result(
    vfs_db: State<'_, Arc<VfsDatabase>>,
    id: String,
) -> VfsResult<()> {
    let conn = vfs_db.get_conn_safe().map_err(|e| crate::vfs::error::VfsError::Database(e.to_string()))?;
    ocr_storage::delete_ocr_result(&conn, &id)
}

/// 标记 OCR 结果为已导出
#[tauri::command]
pub fn vfs_ocr_mark_exported(
    vfs_db: State<'_, Arc<VfsDatabase>>,
    id: String,
) -> VfsResult<()> {
    let conn = vfs_db.get_conn_safe().map_err(|e| crate::vfs::error::VfsError::Database(e.to_string()))?;
    ocr_storage::mark_ocr_exported(&conn, &id)
}

/// 列出待导出的 OCR 结果
#[tauri::command]
pub fn vfs_ocr_list_for_export(
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Vec<ocr_storage::OcrStorageEntry>> {
    let conn = vfs_db.get_conn_safe().map_err(|e| crate::vfs::error::VfsError::Database(e.to_string()))?;
    ocr_storage::list_ocr_for_export(&conn)
}
