//! OCR 结果存储 Tauri 命令处理器

use std::sync::Arc;
use tauri::State;

use super::database::VfsDatabase;
use super::error::VfsResult;
use super::ocr_storage::{OcrInsertRequest, OcrListResponse, OcrStorageRepo};

/// 存储 OCR 结果
///
/// 自动去重: 相同 content_hash + source_id 只保存一次。
/// 返回已有记录 (不重复插入)。
#[tauri::command]
pub async fn ocr_store_result(
    request: OcrInsertRequest,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<super::ocr_storage::OcrRecord> {
    let conn = vfs_db.get_conn()?;
    Ok(OcrStorageRepo::insert(&conn, &request)?)
}

/// 查询 OCR 结果列表
#[tauri::command]
pub async fn ocr_list_results(
    source_type: Option<String>,
    source_id: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<OcrListResponse> {
    let conn = vfs_db.get_conn()?;
    let records = OcrStorageRepo::list_by_source(
        &conn,
        source_type.as_deref(),
        source_id.as_deref(),
        limit.or(Some(50)),
        offset.or(Some(0)),
    )?;
    Ok(OcrListResponse {
        total: records.len(),
        records,
    })
}

/// 删除 OCR 结果 (软删除)
#[tauri::command]
pub async fn ocr_delete_result(
    id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<()> {
    let conn = vfs_db.get_conn()?;
    Ok(OcrStorageRepo::soft_delete(&conn, &id)?)
}

/// 批量标记 OCR 结果为已导出
#[tauri::command]
pub async fn ocr_mark_exported(
    ids: Vec<String>,
    export_hash: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<usize> {
    let conn = vfs_db.get_conn()?;
    Ok(OcrStorageRepo::mark_exported(&conn, &ids, &export_hash)?)
}

/// 获取增量导出列表 (自上次导出后新增/修改的 OCR 结果)
#[tauri::command]
pub async fn ocr_list_for_export(
    last_export_hash: Option<String>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<Vec<super::ocr_storage::OcrRecord>> {
    let conn = vfs_db.get_conn()?;
    Ok(OcrStorageRepo::list_for_export(&conn, last_export_hash.as_deref())?)
}
