//! VFS 调试/诊断/缓存管理 Tauri 命令处理器
//!
//! 提供索引诊断、状态重置、媒体缓存管理等工具命令。

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::vfs::database::VfsDatabase;
use crate::vfs::error::VfsResult;
use crate::vfs::lance_store::VfsLanceStore;
use crate::vfs::repos::VfsBlobRepo;

// ============================================================================
// 诊断相关类型
// ============================================================================

/// 索引诊断信息（统一索引架构版本）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexDiagnosticInfo {
    /// 时间戳
    pub timestamp: String,
    /// 架构版本
    pub architecture_version: String,
    /// 资源总数
    pub total_resources: i32,
    /// 各状态数量
    pub state_counts: IndexStateCounts,
    /// 抽样资源详情（最多15条，用于快速预览）
    pub sample_resources: Vec<ResourceDiagnostic>,
    /// 所有资源详情（用于完整对比）
    pub all_resources: Vec<ResourceDiagnostic>,
    /// vfs_index_units 表统计
    pub units_stats: UnitsStats,
    /// vfs_index_segments 表统计
    pub segments_stats: SegmentsStats,
    /// vfs_embedding_dims 表统计
    pub dimensions_stats: Vec<DimensionStats>,
    /// 数据一致性检查
    pub consistency_checks: Vec<ConsistencyCheck>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStateCounts {
    pub pending: i32,
    pub indexing: i32,
    pub indexed: i32,
    pub failed: i32,
    pub disabled: i32,
    pub null_state: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDiagnostic {
    pub id: String,
    /// 资源名称（用于 UI 显示）
    pub name: Option<String>,
    pub resource_type: String,
    pub storage_mode: String,
    pub index_state: Option<String>,
    pub index_error: Option<String>,
    pub data_len: i32,
    pub has_ocr_text: bool,
    /// 该资源的 unit 数量
    pub unit_count: i32,
    /// 该资源的 segment 数量
    pub segment_count: i32,
    /// Unit 的 text_state
    pub unit_text_state: Option<String>,
    /// Unit 的 mm_state
    pub unit_mm_state: Option<String>,
    /// 文本嵌入维度
    pub text_embedding_dim: Option<i32>,
    /// 文本分块数量
    pub text_chunk_count: Option<i32>,
    pub updated_at: i64,
}

/// vfs_index_units 表统计
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitsStats {
    pub total_count: i32,
    pub distinct_resources: i32,
    pub text_pending: i32,
    pub text_indexing: i32,
    pub text_indexed: i32,
    pub text_failed: i32,
    pub text_disabled: i32,
    pub mm_pending: i32,
    pub mm_indexing: i32,
    pub mm_indexed: i32,
    pub mm_failed: i32,
    pub mm_disabled: i32,
}

/// vfs_index_segments 表统计
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentsStats {
    pub total_count: i32,
    pub distinct_units: i32,
    pub text_modality_count: i32,
    pub mm_modality_count: i32,
    pub avg_segments_per_unit: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionStats {
    pub dimension: i32,
    pub modality: String,
    /// ★ 审计修复：统一为 i64
    pub record_count: i64,
    pub actual_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsistencyCheck {
    pub check_name: String,
    pub passed: bool,
    pub details: String,
}

// ============================================================================
// 媒体缓存类型
// ============================================================================

/// 媒体缓存统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaCacheStats {
    /// PDF 页面图片缓存（blobs 中的 preview 图片）
    pub pdf_preview_count: u64,
    pub pdf_preview_size: u64,
    /// 压缩图片缓存（compressed_blob_hash 引用的 blobs）
    pub compressed_image_count: u64,
    pub compressed_image_size: u64,
    /// OCR 文本缓存（resources.ocr_text 和 files.ocr_pages_json）
    pub ocr_text_count: u64,
    /// 向量索引数量（LanceDB 中的记录数）
    pub vector_index_count: u64,
    /// 向量索引大小（LanceDB 目录大小）
    pub vector_index_size: u64,
    /// 总缓存大小
    pub total_size: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearMediaCacheParams {
    pub clear_pdf_preview: bool,
    pub clear_compressed_images: bool,
    pub clear_ocr_text: bool,
    pub clear_vector_index: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearMediaCacheResult {
    pub pdf_preview_cleared: u64,
    pub compressed_images_cleared: u64,
    pub ocr_text_cleared: u64,
    pub vector_index_cleared: u64,
    pub total_bytes_freed: u64,
    pub files_reset: u64,
}

// ============================================================================
// 索引诊断命令
// ============================================================================

/// 获取索引诊断信息
///
/// 返回数据库各表的真实状态，用于调试索引问题
#[tauri::command]
pub async fn vfs_debug_index_status(
    resource_id: Option<String>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<IndexDiagnosticInfo> {
    log::info!(
        "[VFS::handlers] vfs_debug_index_status: resource_id={:?}",
        resource_id
    );

    let conn = vfs_db.get_conn_safe()?;
    let timestamp = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S%.3f UTC")
        .to_string();

    // 1. 统计各状态数量
    let state_counts: IndexStateCounts = conn
        .query_row(
            r#"
        SELECT
            COALESCE(SUM(CASE WHEN index_state = 'pending' THEN 1 ELSE 0 END), 0) as pending,
            COALESCE(SUM(CASE WHEN index_state = 'indexing' THEN 1 ELSE 0 END), 0) as indexing,
            COALESCE(SUM(CASE WHEN index_state = 'indexed' THEN 1 ELSE 0 END), 0) as indexed,
            COALESCE(SUM(CASE WHEN index_state = 'failed' THEN 1 ELSE 0 END), 0) as failed,
            COALESCE(SUM(CASE WHEN index_state = 'disabled' THEN 1 ELSE 0 END), 0) as disabled,
            COALESCE(SUM(CASE WHEN index_state IS NULL THEN 1 ELSE 0 END), 0) as null_state
        FROM resources
        "#,
            [],
            |row| {
                Ok(IndexStateCounts {
                    pending: row.get(0)?,
                    indexing: row.get(1)?,
                    indexed: row.get(2)?,
                    failed: row.get(3)?,
                    disabled: row.get(4)?,
                    null_state: row.get(5)?,
                })
            },
        )?;

    let total_resources: i32 = conn
        .query_row("SELECT COUNT(*) FROM resources", [], |row| row.get(0))?;

    // 2. 获取所有资源详情（使用统一索引架构）
    let all_resources_query = r#"
        SELECT r.id, r.source_id, r.type, r.storage_mode, r.index_state, r.index_error,
               LENGTH(COALESCE(r.data, '')) as data_len,
               CASE WHEN r.ocr_text IS NOT NULL AND r.ocr_text != '' THEN 1 ELSE 0 END as has_ocr,
               (SELECT COUNT(*) FROM vfs_index_units WHERE resource_id = r.id) as unit_count,
               (SELECT COUNT(*) FROM vfs_index_segments s JOIN vfs_index_units u ON s.unit_id = u.id WHERE u.resource_id = r.id) as seg_count,
               (SELECT text_state FROM vfs_index_units WHERE resource_id = r.id LIMIT 1) as unit_text_state,
               (SELECT mm_state FROM vfs_index_units WHERE resource_id = r.id LIMIT 1) as unit_mm_state,
               (SELECT text_embedding_dim FROM vfs_index_units WHERE resource_id = r.id LIMIT 1) as text_embedding_dim,
               (SELECT text_chunk_count FROM vfs_index_units WHERE resource_id = r.id LIMIT 1) as text_chunk_count,
               r.updated_at
        FROM resources r
        ORDER BY r.updated_at DESC
    "#;

    let mut all_stmt = conn
        .prepare(all_resources_query)?;
    let all_resources: Vec<ResourceDiagnostic> = all_stmt
        .query_map([], |row| {
            Ok(ResourceDiagnostic {
                id: row.get(0)?,
                name: row.get(1)?,
                resource_type: row.get(2)?,
                storage_mode: row.get(3)?,
                index_state: row.get(4)?,
                index_error: row.get(5)?,
                data_len: row.get(6)?,
                has_ocr_text: row.get::<_, i32>(7)? == 1,
                unit_count: row.get(8)?,
                segment_count: row.get(9)?,
                unit_text_state: row.get(10)?,
                unit_mm_state: row.get(11)?,
                text_embedding_dim: row.get(12)?,
                text_chunk_count: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                log::warn!("[VfsHandlers] Skipping malformed row: {}", e);
                None
            }
        })
        .collect();

    // 抽样资源（最多15条，用于快速预览）
    let sample_resources: Vec<ResourceDiagnostic> =
        all_resources.iter().take(15).cloned().collect();

    // 3. vfs_index_units 表统计
    let units_stats: UnitsStats = conn
        .query_row(
            r#"
        SELECT
            COUNT(*) as total,
            COUNT(DISTINCT resource_id) as distinct_res,
            COALESCE(SUM(CASE WHEN text_state = 'pending' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN text_state = 'indexing' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN text_state = 'indexed' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN text_state = 'failed' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN text_state = 'disabled' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN mm_state = 'pending' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN mm_state = 'indexing' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN mm_state = 'indexed' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN mm_state = 'failed' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN mm_state = 'disabled' THEN 1 ELSE 0 END), 0)
        FROM vfs_index_units
        "#,
            [],
            |row| {
                Ok(UnitsStats {
                    total_count: row.get(0)?,
                    distinct_resources: row.get(1)?,
                    text_pending: row.get(2)?,
                    text_indexing: row.get(3)?,
                    text_indexed: row.get(4)?,
                    text_failed: row.get(5)?,
                    text_disabled: row.get(6)?,
                    mm_pending: row.get(7)?,
                    mm_indexing: row.get(8)?,
                    mm_indexed: row.get(9)?,
                    mm_failed: row.get(10)?,
                    mm_disabled: row.get(11)?,
                })
            },
        )?;

    // 4. vfs_index_segments 表统计
    let segments_stats: SegmentsStats = conn
        .query_row(
            r#"
        SELECT
            COUNT(*) as total,
            COUNT(DISTINCT unit_id) as distinct_units,
            COALESCE(SUM(CASE WHEN modality = 'text' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN modality = 'multimodal' THEN 1 ELSE 0 END), 0),
            CASE WHEN COUNT(DISTINCT unit_id) > 0
                 THEN CAST(COUNT(*) AS REAL) / COUNT(DISTINCT unit_id)
                 ELSE 0.0 END as avg_segs
        FROM vfs_index_segments
        "#,
            [],
            |row| {
                Ok(SegmentsStats {
                    total_count: row.get(0)?,
                    distinct_units: row.get(1)?,
                    text_modality_count: row.get(2)?,
                    mm_modality_count: row.get(3)?,
                    avg_segments_per_unit: row.get(4)?,
                })
            },
        )?;

    // 5. vfs_embedding_dims 表统计
    let mut dim_stmt = conn.prepare(
        r#"
        SELECT d.dimension, d.modality, d.record_count,
               (SELECT COUNT(*) FROM vfs_index_segments WHERE embedding_dim = d.dimension AND modality = d.modality) as actual
        FROM vfs_embedding_dims d
        "#
    )?;

    let dimensions_stats: Vec<DimensionStats> = dim_stmt
        .query_map([], |row| {
            Ok(DimensionStats {
                dimension: row.get(0)?,
                modality: row.get(1)?,
                record_count: row.get(2)?,
                actual_count: row.get(3)?,
            })
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                log::warn!("[VfsHandlers] Skipping malformed row: {}", e);
                None
            }
        })
        .collect();

    // 6. 一致性检查
    let mut consistency_checks = Vec::new();

    // 检查1: resources.index_state='indexed' 但无对应 units
    let indexed_no_units: i32 = conn
        .query_row(
            r#"
        SELECT COUNT(*) FROM resources r
        WHERE r.index_state = 'indexed'
          AND NOT EXISTS (SELECT 1 FROM vfs_index_units WHERE resource_id = r.id)
        "#,
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    consistency_checks.push(ConsistencyCheck {
        check_name: "resources_indexed_without_units".to_string(),
        passed: indexed_no_units == 0,
        details: format!("{} 个资源状态为 indexed 但无 units 记录", indexed_no_units),
    });

    // 检查2: units 存在但 resources.index_state 不是 indexed
    let units_not_indexed: i32 = conn
        .query_row(
            r#"
        SELECT COUNT(DISTINCT u.resource_id) FROM vfs_index_units u
        LEFT JOIN resources r ON u.resource_id = r.id
        WHERE r.index_state IS NULL OR r.index_state != 'indexed'
        "#,
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    consistency_checks.push(ConsistencyCheck {
        check_name: "units_exist_but_not_indexed".to_string(),
        passed: units_not_indexed == 0,
        details: format!(
            "{} 个资源有 units 但 resources.index_state 不是 indexed",
            units_not_indexed
        ),
    });

    // 检查3: unit.text_state='indexed' 但无对应 segments
    let unit_indexed_no_segments: i32 = conn.query_row(
        r#"
        SELECT COUNT(*) FROM vfs_index_units u
        WHERE u.text_state = 'indexed'
          AND NOT EXISTS (SELECT 1 FROM vfs_index_segments WHERE unit_id = u.id AND modality = 'text')
        "#,
        [],
        |row| row.get(0),
    ).unwrap_or(0);
    consistency_checks.push(ConsistencyCheck {
        check_name: "units_indexed_without_segments".to_string(),
        passed: unit_indexed_no_segments == 0,
        details: format!(
            "{} 个 unit 状态为 text_indexed 但无 text segments",
            unit_indexed_no_segments
        ),
    });

    // 检查4: segments 存在但 unit.text_state 不是 indexed
    let segments_unit_not_indexed: i32 = conn
        .query_row(
            r#"
        SELECT COUNT(DISTINCT s.unit_id) FROM vfs_index_segments s
        JOIN vfs_index_units u ON s.unit_id = u.id
        WHERE s.modality = 'text' AND u.text_state != 'indexed'
        "#,
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    consistency_checks.push(ConsistencyCheck {
        check_name: "segments_exist_but_unit_not_indexed".to_string(),
        passed: segments_unit_not_indexed == 0,
        details: format!(
            "{} 个 unit 有 segments 但 text_state 不是 indexed",
            segments_unit_not_indexed
        ),
    });

    // 检查4.1: unit.mm_state='indexed' 但无对应多模态 segments
    let unit_mm_indexed_no_segments: i32 = conn.query_row(
        r#"
        SELECT COUNT(*) FROM vfs_index_units u
        WHERE u.mm_state = 'indexed'
          AND NOT EXISTS (SELECT 1 FROM vfs_index_segments WHERE unit_id = u.id AND modality = 'multimodal')
        "#,
        [],
        |row| row.get(0),
    ).unwrap_or(0);
    consistency_checks.push(ConsistencyCheck {
        check_name: "units_mm_indexed_without_segments".to_string(),
        passed: unit_mm_indexed_no_segments == 0,
        details: format!(
            "{} 个 unit 状态为 mm_indexed 但无 multimodal segments",
            unit_mm_indexed_no_segments
        ),
    });

    // 检查4.2: multimodal segments 存在但 unit.mm_state 不是 indexed
    let mm_segments_unit_not_indexed: i32 = conn
        .query_row(
            r#"
        SELECT COUNT(DISTINCT s.unit_id) FROM vfs_index_segments s
        JOIN vfs_index_units u ON s.unit_id = u.id
        WHERE s.modality = 'multimodal' AND u.mm_state != 'indexed'
        "#,
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    consistency_checks.push(ConsistencyCheck {
        check_name: "mm_segments_exist_but_unit_not_indexed".to_string(),
        passed: mm_segments_unit_not_indexed == 0,
        details: format!(
            "{} 个 unit 有 multimodal segments 但 mm_state 不是 indexed",
            mm_segments_unit_not_indexed
        ),
    });

    // 检查5: vfs_embedding_dims.record_count 与实际 segments 数量一致性
    let mut dim_mismatch = 0;
    let mut dim_mismatch_details = Vec::new();
    for dim in &dimensions_stats {
        if dim.record_count != dim.actual_count {
            dim_mismatch += 1;
            dim_mismatch_details.push(format!(
                "dim={}:{} (recorded={}, actual={})",
                dim.dimension, dim.modality, dim.record_count, dim.actual_count
            ));
        }
    }
    consistency_checks.push(ConsistencyCheck {
        check_name: "dimension_record_count_match".to_string(),
        passed: dim_mismatch == 0,
        details: if dim_mismatch == 0 {
            "所有维度 record_count 与实际数量一致".to_string()
        } else {
            format!(
                "{} 个维度不一致: {}",
                dim_mismatch,
                dim_mismatch_details.join(", ")
            )
        },
    });

    // 检查6: 架构验证
    let architecture_valid = units_stats.total_count >= 0 && segments_stats.total_count >= 0;
    consistency_checks.push(ConsistencyCheck {
        check_name: "unified_index_architecture".to_string(),
        passed: architecture_valid,
        details: format!(
            "统一索引架构: {} resources → {} units → {} segments",
            total_resources, units_stats.total_count, segments_stats.total_count
        ),
    });

    // 检查7: pending 状态资源信息
    consistency_checks.push(ConsistencyCheck {
        check_name: "pending_resources_info".to_string(),
        passed: true,
        details: format!("{} 个资源待索引", state_counts.pending),
    });

    // 检查8: disabled 状态资源数量
    consistency_checks.push(ConsistencyCheck {
        check_name: "disabled_resources_info".to_string(),
        passed: true,
        details: format!(
            "{} 个资源被标记为 disabled（不适用）",
            state_counts.disabled
        ),
    });

    Ok(IndexDiagnosticInfo {
        timestamp,
        architecture_version: "unified_index_v1".to_string(),
        total_resources,
        state_counts,
        sample_resources,
        all_resources,
        units_stats,
        segments_stats,
        dimensions_stats,
        consistency_checks,
    })
}

/// 重置所有 disabled 资源为 pending 状态
///
/// 用于在修复后重新触发索引
#[tauri::command]
pub async fn vfs_reset_disabled_to_pending(
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<i32> {
    log::info!("[VFS::handlers] vfs_reset_disabled_to_pending");

    let conn = vfs_db.get_conn_safe()?;

    let updated = conn.execute(
        "UPDATE resources SET index_state = 'pending', index_error = NULL WHERE index_state = 'disabled'",
        [],
    )?;

    log::info!(
        "[VFS::handlers] Reset {} disabled resources to pending",
        updated
    );

    Ok(updated as i32)
}

/// 重置所有 indexed 但无 embeddings 的资源为 pending 状态
#[tauri::command]
pub async fn vfs_reset_indexed_without_embeddings(
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<i32> {
    log::info!("[VFS::handlers] vfs_reset_indexed_without_segments");

    let conn = vfs_db.get_conn_safe()?;

    // 使用统一索引架构的新表
    let updated = conn
        .execute(
            r#"
        UPDATE resources
        SET index_state = 'pending', index_error = NULL
        WHERE index_state = 'indexed'
          AND NOT EXISTS (
            SELECT 1 FROM vfs_index_units u
            JOIN vfs_index_segments s ON s.unit_id = u.id
            WHERE u.resource_id = resources.id
          )
        "#,
            [],
        )?;

    log::info!(
        "[VFS::handlers] Reset {} indexed-without-segments resources to pending",
        updated
    );

    Ok(updated as i32)
}

/// 重置所有索引状态（用于调试/重新索引）
///
/// 将所有资源的索引状态重置为 pending，并清空 segments、units、维度统计和 LanceDB 向量数据
#[tauri::command]
pub async fn vfs_reset_all_index_state(
    vfs_db: State<'_, Arc<VfsDatabase>>,
    lance_store: State<'_, Arc<VfsLanceStore>>,
) -> VfsResult<i32> {
    use crate::vfs::repos::{MODALITY_MULTIMODAL, MODALITY_TEXT};

    log::info!("[VFS::handlers] vfs_reset_all_index_state - 重置所有索引状态");

    // 清除文本向量
    let text_cleared = lance_store
        .clear_all(MODALITY_TEXT)
        .await?;
    log::info!("[VFS::handlers] 清除 {} 个文本向量表", text_cleared);

    // 清除多模态向量
    let mm_cleared = lance_store
        .clear_all(MODALITY_MULTIMODAL)
        .await?;
    log::info!("[VFS::handlers] 清除 {} 个多模态向量表", mm_cleared);

    let conn = vfs_db.get_conn_safe()?;

    // 1. 删除所有 segments
    let deleted_segments = conn
        .execute("DELETE FROM vfs_index_segments", [])?;
    log::info!("[VFS::handlers] 删除 {} 个 segments", deleted_segments);

    // 2. 删除所有 units
    let deleted_units = conn
        .execute("DELETE FROM vfs_index_units", [])?;
    log::info!("[VFS::handlers] 删除 {} 个 units", deleted_units);

    // 3. 重置维度统计
    conn.execute("UPDATE vfs_embedding_dims SET record_count = 0", [])?;

    // 4. 将所有资源状态重置为 pending（含多模态状态）
    let updated = conn
        .execute(
            r#"
        UPDATE resources
        SET index_state = 'pending',
            index_hash = NULL,
            index_error = NULL,
            index_retry_count = 0,
            mm_index_state = 'pending',
            mm_index_error = NULL,
            mm_index_retry_count = 0,
            mm_embedding_dim = NULL,
            mm_indexing_mode = NULL,
            mm_indexed_at = NULL
        "#,
            [],
        )?;

    // 5. 同步重置业务表中的多模态索引状态
    let files_reset = conn
        .execute(
            r#"
        UPDATE files
        SET mm_index_state = 'pending',
            mm_index_error = NULL,
            mm_indexed_pages_json = NULL,
            updated_at = datetime('now')
        "#,
            [],
        )?;

    let exams_reset = conn
        .execute(
            r#"
        UPDATE exam_sheets
        SET mm_index_state = 'pending',
            mm_index_error = NULL,
            mm_indexed_pages_json = NULL,
            mm_embedding_dim = NULL,
            mm_indexed_at = NULL,
            updated_at = datetime('now')
        "#,
            [],
        )?;

    log::info!(
        "[VFS::handlers] 重置 {} 个资源为 pending 状态（files={}, exam_sheets={})",
        updated,
        files_reset,
        exams_reset
    );

    Ok(updated as i32)
}

/// 诊断 LanceDB 表结构
#[tauri::command]
pub async fn vfs_diagnose_lance_schema(
    modality: Option<String>,
    _vfs_db: State<'_, Arc<VfsDatabase>>,
    lance_store: State<'_, Arc<VfsLanceStore>>,
) -> VfsResult<Vec<crate::vfs::lance_store::LanceTableDiagnostic>> {
    use crate::vfs::repos::MODALITY_TEXT;

    log::info!(
        "[VFS::handlers] vfs_diagnose_lance_schema: modality={:?}",
        modality
    );

    let modality_str = modality.as_deref().unwrap_or(MODALITY_TEXT);

    lance_store
        .diagnose_table_schema(modality_str)
        .await
}

// ============================================================================
// 媒体缓存管理命令
// ============================================================================

/// 获取媒体缓存统计信息
///
/// 统计 PDF 预览图片、压缩图片、OCR 文本和向量索引的缓存大小。
#[tauri::command]
pub async fn vfs_get_media_cache_stats(
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<MediaCacheStats> {
    log::info!("[VFS::handlers] vfs_get_media_cache_stats: Starting...");

    let conn = vfs_db.get_conn_safe()?;
    let blobs_dir = vfs_db.blobs_dir();

    // 1. 统计 PDF 预览图片（preview_json 中引用的 blobs）
    let (pdf_preview_count, pdf_preview_size) = {
        // 获取所有 preview_json 中的 blob_hash
        let mut stmt = conn
            .prepare("SELECT preview_json FROM files WHERE preview_json IS NOT NULL")?;

        let mut count = 0u64;
        let mut size = 0u64;

        let rows = stmt
            .query_map([], |row| {
                let json_str: String = row.get(0)?;
                Ok(json_str)
            })?;

        for row in rows.flatten() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&row) {
                if let Some(pages) = json.get("pages").and_then(|p| p.as_array()) {
                    for page in pages {
                        if let Some(blob_hash) = page.get("blob_hash").and_then(|h| h.as_str()) {
                            // 获取 blob 大小
                            let blob_size: i64 = conn
                                .query_row(
                                    "SELECT size FROM blobs WHERE hash = ?1",
                                    rusqlite::params![blob_hash],
                                    |r| r.get(0),
                                )
                                .unwrap_or(0);
                            count += 1;
                            size += blob_size as u64;
                        }
                    }
                }
            }
        }
        (count, size)
    };

    // 2. 统计压缩图片缓存
    let (compressed_image_count, compressed_image_size) = {
        let result: (i64, i64) = conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(b.size), 0) FROM files f
             JOIN blobs b ON f.compressed_blob_hash = b.hash
             WHERE f.compressed_blob_hash IS NOT NULL",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or((0, 0));
        (result.0 as u64, result.1 as u64)
    };

    // 3. 统计 OCR 文本缓存
    let ocr_text_count: u64 = conn
        .query_row(
            r#"
            SELECT COUNT(*)
            FROM files f
            LEFT JOIN resources r ON r.id = f.resource_id
            WHERE r.ocr_text IS NOT NULL OR f.ocr_pages_json IS NOT NULL
            "#,
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) as u64;

    // 4. 统计向量索引（LanceDB）
    let lance_dir = blobs_dir
        .parent()
        .map(|p| p.join("lance").join("vfs"))
        .unwrap_or_else(|| blobs_dir.join("lance").join("vfs"));

    let (vector_index_count, vector_index_size) = if lance_dir.exists() {
        // 计算目录大小
        let mut dir_size = 0u64;
        if let Ok(entries) = std::fs::read_dir(&lance_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        dir_size += metadata.len();
                    } else if metadata.is_dir() {
                        // 递归计算子目录大小
                        dir_size += calculate_dir_size(&entry.path()).unwrap_or(0);
                    }
                }
            }
        }
        // 向量数量从 resources.vector_indexed_at 统计
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM resources WHERE vector_indexed_at IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        (count as u64, dir_size)
    } else {
        (0, 0)
    };

    let total_size = pdf_preview_size + compressed_image_size + vector_index_size;

    let stats = MediaCacheStats {
        pdf_preview_count,
        pdf_preview_size,
        compressed_image_count,
        compressed_image_size,
        ocr_text_count,
        vector_index_count,
        vector_index_size,
        total_size,
    };

    log::info!(
        "[VFS::handlers] vfs_get_media_cache_stats: total_size={} bytes, pdf_preview={}, compressed={}, ocr={}, vector={}",
        total_size, pdf_preview_count, compressed_image_count, ocr_text_count, vector_index_count
    );

    Ok(stats)
}

/// 清理媒体缓存
#[tauri::command]
pub async fn vfs_clear_media_cache(
    params: ClearMediaCacheParams,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<ClearMediaCacheResult> {
    log::info!(
        "[VFS::handlers] vfs_clear_media_cache: pdf={}, compressed={}, ocr={}, vector={}",
        params.clear_pdf_preview,
        params.clear_compressed_images,
        params.clear_ocr_text,
        params.clear_vector_index
    );

    let conn = vfs_db.get_conn_safe()?;
    let blobs_dir = vfs_db.blobs_dir();

    let mut result = ClearMediaCacheResult {
        pdf_preview_cleared: 0,
        compressed_images_cleared: 0,
        ocr_text_cleared: 0,
        vector_index_cleared: 0,
        total_bytes_freed: 0,
        files_reset: 0,
    };

    // 1. 清理 PDF 预览图片
    if params.clear_pdf_preview {
        // 获取所有 preview_json 中的 blob_hash
        let mut stmt = conn
            .prepare("SELECT id, preview_json FROM files WHERE preview_json IS NOT NULL")?;

        let rows: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| match r {
                Ok(val) => Some(val),
                Err(e) => {
                    log::warn!("[VfsHandlers] Skipping malformed row: {}", e);
                    None
                }
            })
            .collect();

        for (file_id, json_str) in rows {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                if let Some(pages) = json.get("pages").and_then(|p| p.as_array()) {
                    for page in pages {
                        let original_hash = page
                            .get("blob_hash")
                            .or_else(|| page.get("blobHash"))
                            .and_then(|h| h.as_str());
                        let compressed_hash = page
                            .get("compressed_blob_hash")
                            .or_else(|| page.get("compressedBlobHash"))
                            .and_then(|h| h.as_str());

                        if let Some(blob_hash) = original_hash {
                            // 减少原始 blob 引用计数
                            let _ =
                                VfsBlobRepo::decrement_ref_with_conn(&conn, blobs_dir, blob_hash);
                            // 获取大小
                            let size: i64 = conn
                                .query_row(
                                    "SELECT size FROM blobs WHERE hash = ?1",
                                    rusqlite::params![blob_hash],
                                    |r| r.get(0),
                                )
                                .unwrap_or(0);
                            result.pdf_preview_cleared += 1;
                            result.total_bytes_freed += size as u64;
                        }

                        if let Some(ch) = compressed_hash {
                            let is_same = original_hash.map(|oh| oh == ch).unwrap_or(false);
                            if !is_same {
                                let _ = VfsBlobRepo::decrement_ref_with_conn(&conn, blobs_dir, ch);
                                let size: i64 = conn
                                    .query_row(
                                        "SELECT size FROM blobs WHERE hash = ?1",
                                        rusqlite::params![ch],
                                        |r| r.get(0),
                                    )
                                    .unwrap_or(0);
                                result.pdf_preview_cleared += 1;
                                result.total_bytes_freed += size as u64;
                            }
                        }
                    }
                }
            }
            // 清空 preview_json
            let _ = conn.execute(
                "UPDATE files SET preview_json = NULL WHERE id = ?1",
                rusqlite::params![file_id],
            );
            result.files_reset += 1;
        }

        // 清理无引用的 blobs
        let _ = VfsBlobRepo::cleanup_unreferenced(&vfs_db);
    }

    // 2. 清理压缩图片缓存
    if params.clear_compressed_images {
        let mut stmt = conn.prepare(
            "SELECT id, compressed_blob_hash, blob_hash FROM files WHERE compressed_blob_hash IS NOT NULL"
        )?;

        let rows: Vec<(String, String, Option<String>)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .filter_map(|r| match r {
                Ok(val) => Some(val),
                Err(e) => {
                    log::warn!("[VfsHandlers] Skipping malformed row: {}", e);
                    None
                }
            })
            .collect();

        for (file_id, compressed_hash, original_hash) in rows {
            let is_same_as_original = original_hash
                .as_ref()
                .map(|h| h == &compressed_hash)
                .unwrap_or(false);
            // 减少引用计数（仅当与原始 blob 不同）
            if !is_same_as_original {
                // 获取大小（仅当确实释放）
                let size: i64 = conn
                    .query_row(
                        "SELECT size FROM blobs WHERE hash = ?1",
                        rusqlite::params![compressed_hash],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                let _ = VfsBlobRepo::decrement_ref_with_conn(&conn, blobs_dir, &compressed_hash);
                result.total_bytes_freed += size as u64;
            }

            // 清空 compressed_blob_hash
            let _ = conn.execute(
                "UPDATE files SET compressed_blob_hash = NULL WHERE id = ?1",
                rusqlite::params![file_id],
            );

            result.compressed_images_cleared += 1;
            result.files_reset += 1;
        }

        let _ = VfsBlobRepo::cleanup_unreferenced(&vfs_db);
    }

    // 3. 清理 OCR 文本
    if params.clear_ocr_text {
        let cleared: i64 = conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM files f
                LEFT JOIN resources r ON r.id = f.resource_id
                WHERE r.ocr_text IS NOT NULL OR f.ocr_pages_json IS NOT NULL
                "#,
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        conn.execute(
            "UPDATE files SET ocr_pages_json = NULL WHERE ocr_pages_json IS NOT NULL",
            [],
        )?;

        // 同时清理 resources 表的 ocr_text
        conn.execute(
            "UPDATE resources SET ocr_text = NULL WHERE ocr_text IS NOT NULL",
            [],
        )?;

        result.ocr_text_cleared = cleared as u64;
        result.files_reset += cleared as u64;
    }

    // 4. 清理向量索引
    if params.clear_vector_index {
        let lance_dir = blobs_dir
            .parent()
            .map(|p| p.join("lance").join("vfs"))
            .unwrap_or_else(|| blobs_dir.join("lance").join("vfs"));

        if lance_dir.exists() {
            // 计算目录大小
            let dir_size = calculate_dir_size(&lance_dir).unwrap_or(0);

            // 删除 LanceDB 目录
            if let Err(e) = std::fs::remove_dir_all(&lance_dir) {
                log::warn!("[VFS::handlers] Failed to remove lance dir: {}", e);
            } else {
                result.vector_index_cleared = 1;
                result.total_bytes_freed += dir_size;
            }
        }

        // ★ P1 修复：清理 vfs_index_units 和 vfs_index_segments 表
        conn.execute("DELETE FROM vfs_index_segments", [])?;
        conn.execute("DELETE FROM vfs_index_units", [])?;

        // 重置维度统计
        conn.execute("UPDATE vfs_embedding_dims SET record_count = 0", [])?;

        // 重置 resources.vector_indexed_at
        let reset_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM resources WHERE vector_indexed_at IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        conn.execute(
            "UPDATE resources SET vector_indexed_at = NULL WHERE vector_indexed_at IS NOT NULL",
            [],
        )?;

        result.files_reset += reset_count as u64;
    }

    // 5. ★ P0 修复：根据清理的缓存类型更新 processing_progress 中的 ready_modes
    if params.clear_pdf_preview || params.clear_compressed_images || params.clear_ocr_text {
        // 查询所有有 processing_progress 的文件
        let mut stmt = conn
            .prepare(
                "SELECT id, processing_progress FROM files
             WHERE processing_progress IS NOT NULL
             AND (mime_type LIKE 'application/pdf' OR mime_type LIKE 'image/%')",
            )?;

        let files_to_update: Vec<(String, String)> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(|r| match r {
                Ok(val) => Some(val),
                Err(e) => {
                    log::warn!("[VfsHandlers] Skipping malformed row: {}", e);
                    None
                }
            })
            .collect();

        for (file_id, progress_json) in files_to_update {
            // 解析 processing_progress JSON
            if let Ok(mut progress) = serde_json::from_str::<serde_json::Value>(&progress_json) {
                let modes_key = if progress.get("readyModes").is_some() {
                    "readyModes"
                } else {
                    "ready_modes"
                };
                if let Some(ready_modes) =
                    progress.get_mut(modes_key).and_then(|v| v.as_array_mut())
                {
                    // 根据清理类型移除对应的 ready_mode
                    if params.clear_pdf_preview {
                        ready_modes.retain(|v| v.as_str() != Some("image"));
                    }
                    if params.clear_ocr_text {
                        ready_modes.retain(|v| v.as_str() != Some("ocr"));
                    }
                    // 注意：clear_compressed_images 不影响 ready_modes（压缩是优化，不是模式）
                }

                // 更新 processing_progress
                let updated_json = serde_json::to_string(&progress).unwrap_or_default();
                conn.execute(
                    "UPDATE files SET
                        processing_status = 'pending',
                        processing_progress = ?1,
                        processing_error = NULL,
                        processing_started_at = NULL,
                        processing_completed_at = NULL
                    WHERE id = ?2",
                    rusqlite::params![updated_json, file_id],
                )?;
            }
        }

        // 对于没有 processing_progress 的文件，重置为 pending
        conn.execute(
            "UPDATE files SET
                processing_status = 'pending',
                processing_error = NULL,
                processing_started_at = NULL,
                processing_completed_at = NULL
            WHERE processing_progress IS NULL
            AND (mime_type LIKE 'application/pdf' OR mime_type LIKE 'image/%')",
            [],
        )?;
    }

    log::info!(
        "[VFS::handlers] vfs_clear_media_cache: Complete! freed {} bytes, reset {} files",
        result.total_bytes_freed,
        result.files_reset
    );

    Ok(result)
}

/// 计算目录大小（递归）
fn calculate_dir_size(path: &std::path::Path) -> std::io::Result<u64> {
    let mut size = 0u64;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                size += calculate_dir_size(&path)?;
            } else {
                size += entry.metadata()?.len();
            }
        }
    }
    Ok(size)
}
