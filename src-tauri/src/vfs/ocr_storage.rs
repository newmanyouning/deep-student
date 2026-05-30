//! OCR 结果存储模块
//!
//! 独立于 VFS 核心表，专门管理所有 OCR 处理结果。
//! 支持:
//! - 文件 OCR、对话 OCR、PDF OCR、剪贴板 OCR
//! - 内容哈希去重 (相同输入不重复 OCR)
//! - 来源追溯 (source_type + source_id)
//! - 导出标记 (增量导出)

use rusqlite::{params, Connection};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrRecord {
    pub id: String,
    pub source_type: String,
    pub source_id: Option<String>,
    pub source_name: Option<String>,
    pub source_page: Option<i32>,
    pub content_hash: String,
    pub input_size_bytes: Option<i32>,
    pub image_mime: Option<String>,
    pub image_width: Option<i32>,
    pub image_height: Option<i32>,
    pub ocr_text: String,
    pub ocr_engine: String,
    pub ocr_confidence: Option<f64>,
    pub ocr_duration_ms: Option<i32>,
    pub ocr_lang: Option<String>,
    pub tags: String,
    pub mistake_type: Option<String>,
    pub blocks_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub export_hash: Option<String>,
    pub exported_at: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrInsertRequest {
    pub source_type: String,
    pub source_id: Option<String>,
    pub source_name: Option<String>,
    pub source_page: Option<i32>,
    pub content_hash: String,
    pub input_size_bytes: Option<i32>,
    pub image_mime: Option<String>,
    pub image_width: Option<i32>,
    pub image_height: Option<i32>,
    pub ocr_text: String,
    pub ocr_engine: String,
    pub ocr_confidence: Option<f64>,
    pub ocr_duration_ms: Option<i32>,
    pub ocr_lang: Option<String>,
    pub tags: Option<Vec<String>>,
    pub mistake_type: Option<String>,
    pub blocks_json: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrListResponse {
    pub records: Vec<OcrRecord>,
    pub total: usize,
}

pub struct OcrStorageRepo;

impl OcrStorageRepo {
    /// 插入 OCR 结果 (自动去重: 相同 content_hash 跳过)
    pub fn insert(conn: &Connection, req: &OcrInsertRequest) -> Result<OcrRecord, rusqlite::Error> {
        let id = format!("ocr_{}", nanoid::nanoid!(12));
        let now = chrono::Utc::now().to_rfc3339();
        let tags_json =
            serde_json::to_string(&req.tags.as_deref().unwrap_or(&[])).unwrap_or_else(|_| "[]".into());

        let exists: Option<String> = conn
            .query_row(
                "SELECT id FROM ocr_results WHERE content_hash = ?1 AND source_id = ?2 AND deleted_at IS NULL LIMIT 1",
                params![req.content_hash, req.source_id],
                |row| row.get(0),
            )
            .optional()?;

        if let Some(existing_id) = exists {
            return conn.query_row(
                "SELECT * FROM ocr_results WHERE id = ?1",
                params![existing_id],
                Self::row_to_record,
            );
        }

        conn.execute(
            "INSERT INTO ocr_results (id, source_type, source_id, source_name, source_page,
             content_hash, input_size_bytes, image_mime, image_width, image_height,
             ocr_text, ocr_engine, ocr_confidence, ocr_duration_ms, ocr_lang,
             tags, mistake_type, blocks_json, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)",
            params![
                id, req.source_type, req.source_id, req.source_name, req.source_page,
                req.content_hash, req.input_size_bytes, req.image_mime, req.image_width, req.image_height,
                req.ocr_text, req.ocr_engine, req.ocr_confidence, req.ocr_duration_ms, req.ocr_lang,
                tags_json, req.mistake_type, req.blocks_json, now, now,
            ],
        )?;

        conn.query_row(
            "SELECT * FROM ocr_results WHERE id = ?1",
            params![id],
            Self::row_to_record,
        )
    }

    /// 按来源查询
    pub fn list_by_source(
        conn: &Connection,
        source_type: Option<&str>,
        source_id: Option<&str>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<OcrRecord>, rusqlite::Error> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let st = source_type.unwrap_or("");
        let sid = source_id.unwrap_or("");

        let records: Vec<OcrRecord> = match (source_type, source_id) {
            (Some(_), Some(_)) => {
                let mut stmt = conn.prepare(
                    "SELECT * FROM ocr_results WHERE deleted_at IS NULL AND source_type = ?1 AND source_id = ?2 ORDER BY created_at DESC LIMIT ?3 OFFSET ?4"
                )?;
                let rows = stmt.query_map(params![st, sid, limit as i64, offset as i64], Self::row_to_record)?;
                rows.filter_map(|r| r.ok()).collect()
            }
            (Some(_), None) => {
                let mut stmt = conn.prepare(
                    "SELECT * FROM ocr_results WHERE deleted_at IS NULL AND source_type = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3"
                )?;
                let rows = stmt.query_map(params![st, limit as i64, offset as i64], Self::row_to_record)?;
                rows.filter_map(|r| r.ok()).collect()
            }
            (None, Some(_)) => {
                let mut stmt = conn.prepare(
                    "SELECT * FROM ocr_results WHERE deleted_at IS NULL AND source_id = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3"
                )?;
                let rows = stmt.query_map(params![sid, limit as i64, offset as i64], Self::row_to_record)?;
                rows.filter_map(|r| r.ok()).collect()
            }
            (None, None) => {
                let mut stmt = conn.prepare(
                    "SELECT * FROM ocr_results WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
                )?;
                let rows = stmt.query_map(params![limit as i64, offset as i64], Self::row_to_record)?;
                rows.filter_map(|r| r.ok()).collect()
            }
        };

        Ok(records)
    }

    /// 软删除
    pub fn soft_delete(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE ocr_results SET deleted_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    /// 标记已导出
    pub fn mark_exported(
        conn: &Connection,
        ids: &[String],
        export_hash: &str,
    ) -> Result<usize, rusqlite::Error> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut count = 0;
        for id in ids {
            let updated = conn.execute(
                "UPDATE ocr_results SET export_hash = ?1, exported_at = ?2 WHERE id = ?3",
                params![export_hash, now, id],
            )?;
            count += updated;
        }
        Ok(count)
    }

    /// 获取增量导出列表 (自上次导出后新增/修改)
    pub fn list_for_export(
        conn: &Connection,
        last_export_hash: Option<&str>,
    ) -> Result<Vec<OcrRecord>, rusqlite::Error> {
        let sql = if let Some(hash) = last_export_hash {
            "SELECT * FROM ocr_results WHERE deleted_at IS NULL AND (export_hash IS NULL OR export_hash != ?1) ORDER BY created_at"
        } else {
            "SELECT * FROM ocr_results WHERE deleted_at IS NULL AND export_hash IS NULL ORDER BY created_at"
        };

        let mut stmt = conn.prepare(sql)?;
        let rows = if let Some(hash) = last_export_hash {
            stmt.query_map(params![hash], Self::row_to_record)?
        } else {
            stmt.query_map([], Self::row_to_record)?
        };

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    fn row_to_record(row: &rusqlite::Row) -> rusqlite::Result<OcrRecord> {
        Ok(OcrRecord {
            id: row.get("id")?,
            source_type: row.get("source_type")?,
            source_id: row.get("source_id")?,
            source_name: row.get("source_name")?,
            source_page: row.get("source_page")?,
            content_hash: row.get("content_hash")?,
            input_size_bytes: row.get("input_size_bytes")?,
            image_mime: row.get("image_mime")?,
            image_width: row.get("image_width")?,
            image_height: row.get("image_height")?,
            ocr_text: row.get("ocr_text")?,
            ocr_engine: row.get("ocr_engine")?,
            ocr_confidence: row.get("ocr_confidence")?,
            ocr_duration_ms: row.get("ocr_duration_ms")?,
            ocr_lang: row.get("ocr_lang")?,
            tags: row.get("tags")?,
            mistake_type: row.get("mistake_type")?,
            blocks_json: row.get("blocks_json")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            deleted_at: row.get("deleted_at")?,
            export_hash: row.get("export_hash")?,
            exported_at: row.get("exported_at")?,
        })
    }
}
