//! 共享类型定义
//!
//! 将原本分布在 indexing.rs 和 pdf_processing_service.rs 中的共享类型提取到此文件，
//! 打破三者之间的循环依赖：
//!   embedding_service → indexing (TextChunk) → 改为 → embedding_service → types
//!   indexing → pdf_processing_service (OcrPageResult, OcrPagesJson) → 改为 → indexing → types
//!   pdf_processing_service → indexing (VfsFullIndexingService) → 改为 → Coordinator 模式

use serde::{Deserialize, Serialize};

// ============================================================================
// 分块和索引配置（原 indexing.rs）
// ============================================================================

/// 文本分块配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkingConfig {
    pub strategy: String,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub min_chunk_size: usize,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            strategy: "fixed_size".to_string(),
            chunk_size: 512,
            chunk_overlap: 50,
            min_chunk_size: 20,
        }
    }
}

/// 索引配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexingConfig {
    pub enabled: bool,
    pub batch_size: u32,
    pub interval_secs: u32,
    pub max_concurrent: u32,
    pub retry_delay_secs: u32,
    pub max_retries: i32,
}

impl Default for IndexingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            batch_size: 10,
            interval_secs: 5,
            max_concurrent: 2,
            retry_delay_secs: 60,
            max_retries: 3,
        }
    }
}

/// 搜索配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchConfig {
    pub default_top_k: u32,
    pub enable_hybrid: bool,
    pub enable_reranking: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            default_top_k: 10,
            enable_hybrid: true,
            enable_reranking: false,
        }
    }
}

// ============================================================================
// 文本块类型（原 indexing.rs）
// ============================================================================

/// 文本块
#[derive(Debug, Clone)]
pub struct TextChunk {
    pub index: i32,
    pub text: String,
    pub start_pos: i32,
    pub end_pos: i32,
    /// 页面索引（用于 PDF/教材定位，0-indexed）
    pub page_index: Option<i32>,
    /// 来源 ID（如 textbook_xxx, att_xxx）
    pub source_id: Option<String>,
}

/// 按页的文本内容
#[derive(Debug, Clone)]
pub struct PageText {
    pub page_index: i32,
    pub text: String,
    pub source_id: Option<String>,
}

// ============================================================================
// OCR 结果类型（原 pdf_processing_service.rs）
// ============================================================================

use crate::models::PdfOcrTextBlock;

/// 单页 OCR 结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrPageResult {
    /// 页码（0-indexed）
    pub page_index: usize,
    /// OCR 识别的文本块
    pub blocks: Vec<PdfOcrTextBlock>,
}

/// OCR 结果 JSON（存储在 ocr_pages_json 字段）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrPagesJson {
    /// 总页数
    pub total_pages: usize,
    /// 每页的 OCR 结果
    pub pages: Vec<OcrPageResult>,
    /// OCR 完成时间
    pub completed_at: String,
}
