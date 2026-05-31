//! OCR 结果存储模块
//!
//! 提供 OCR 结果的持久化存储，支持结果的增删查改和导出标记。

use rusqlite::params;
use serde::{Deserialize, Serialize};
use crate::vfs::error::{VfsError, VfsResult};

/// OCR 结果记录（数据库行）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrStorageEntry {
    pub id: String,
    pub resource_id: String,
    pub text: String,
    pub confidence: f64,
    pub source: String,
    pub exported: bool,
    pub created_at: String,
}

/// 存储 OCR 结果
pub fn store_ocr_result(
    conn: &rusqlite::Connection,
    resource_id: &str,
    text: &str,
    confidence: f64,
    source: &str,
) -> VfsResult<String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO ocr_results (id, resource_id, text, confidence, source, exported, created_at) VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
        params![id, resource_id, text, confidence, source, now],
    ).map_err(|e| VfsError::Database(e.to_string()))?;
    Ok(id)
}

/// 列出资源的所有 OCR 结果
pub fn list_ocr_results(conn: &rusqlite::Connection, resource_id: &str) -> VfsResult<Vec<OcrStorageEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, resource_id, text, confidence, source, exported, created_at FROM ocr_results WHERE resource_id = ?1 ORDER BY created_at DESC"
    ).map_err(|e| VfsError::Database(e.to_string()))?;
    let rows = stmt.query_map(params![resource_id], |row| {
        Ok(OcrStorageEntry {
            id: row.get(0)?,
            resource_id: row.get(1)?,
            text: row.get(2)?,
            confidence: row.get(3)?,
            source: row.get(4)?,
            exported: row.get(5)?,
            created_at: row.get(6)?,
        })
    }).map_err(|e| VfsError::Database(e.to_string()))?;
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|e| VfsError::Database(e.to_string()))?);
    }
    Ok(entries)
}

/// 删除 OCR 结果
pub fn delete_ocr_result(conn: &rusqlite::Connection, id: &str) -> VfsResult<()> {
    conn.execute("DELETE FROM ocr_results WHERE id = ?1", params![id])
        .map_err(|e| VfsError::Database(e.to_string()))?;
    Ok(())
}

/// 标记 OCR 结果为已导出
pub fn mark_ocr_exported(conn: &rusqlite::Connection, id: &str) -> VfsResult<()> {
    conn.execute("UPDATE ocr_results SET exported = 1 WHERE id = ?1", params![id])
        .map_err(|e| VfsError::Database(e.to_string()))?;
    Ok(())
}

/// 列出待导出的 OCR 结果
pub fn list_ocr_for_export(conn: &rusqlite::Connection) -> VfsResult<Vec<OcrStorageEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, resource_id, text, confidence, source, exported, created_at FROM ocr_results WHERE exported = 0 ORDER BY created_at ASC"
    ).map_err(|e| VfsError::Database(e.to_string()))?;
    let rows = stmt.query_map([], |row| {
        Ok(OcrStorageEntry {
            id: row.get(0)?,
            resource_id: row.get(1)?,
            text: row.get(2)?,
            confidence: row.get(3)?,
            source: row.get(4)?,
            exported: row.get(5)?,
            created_at: row.get(6)?,
        })
    }).map_err(|e| VfsError::Database(e.to_string()))?;
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|e| VfsError::Database(e.to_string()))?);
    }
    Ok(entries)
}
