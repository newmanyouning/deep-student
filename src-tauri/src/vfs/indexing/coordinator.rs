//! VFS 索引协调器
//!
//! 协调器模式：打破 indexing <-> pdf_processing_service 之间的循环依赖。
//!
//! ## 职责
//! - 提供完整的流水线编排 API（压缩 → OCR → 向量索引）
//! - 持有 PdfProcessingService 的引用，协调其阶段调用
//! - 负责向量索引阶段（使用 VfsFullIndexingService）
//!
//! ## 循环依赖解决
//! pdf_processing_service 中的 `stage_vector_indexing` 原本直接创建
//! VfsFullIndexingService，形成 indexing ↔ pdf_processing_service 双向依赖。
//! 通过依赖注入，PdfProcessingService 存储一个由协调器注入的回调，
//! 在运行时由协调器提供 VfsFullIndexingService 的创建逻辑。

use std::sync::Arc;

use crate::llm_manager::LLMManager;
use crate::vfs::database::VfsDatabase;
use crate::vfs::error::VfsResult;
use crate::vfs::lance_store::VfsLanceStore;
use crate::vfs::pdf_processing_service::VectorIndexCallback;

use super::VfsFullIndexingService;

// ============================================================================
// VfsIndexCoordinator
// ============================================================================

/// VFS 索引协调器
///
/// 提供向量索引的编排能力，负责创建 VfsFullIndexingService 并执行索引。
/// 通过回调注入防止 pdf_processing_service 直接依赖 indexing 模块。
pub struct VfsIndexCoordinator {
    db: Arc<VfsDatabase>,
    llm_manager: Arc<LLMManager>,
}

impl VfsIndexCoordinator {
    /// 创建新的索引协调器
    pub fn new(db: Arc<VfsDatabase>, llm_manager: Arc<LLMManager>) -> Self {
        Self { db, llm_manager }
    }

    /// 创建向量索引回调，供 PdfProcessingService 注入使用
    ///
    /// 返回的回调封装了 VfsFullIndexingService 的创建和调用逻辑，
    /// PdfProcessingService 无需直接导入 VfsFullIndexingService。
    pub fn create_vector_index_callback(&self) -> VectorIndexCallback {
        let db = self.db.clone();
        let llm_manager = self.llm_manager.clone();

        Arc::new(move |resource_id: String| {
            let db = db.clone();
            let llm_manager = llm_manager.clone();
            Box::pin(async move {
                let lance_store = Arc::new(VfsLanceStore::new(db.clone())?);
                let indexing_service =
                    VfsFullIndexingService::new(db.clone(), llm_manager, lance_store)?;
                indexing_service
                    .index_resource(&resource_id, None, None)
                    .await?;
                Ok::<_, crate::vfs::error::VfsError>(())
            })
        })
    }

    /// 直接执行向量索引（供外部直接调用）
    pub async fn vector_index_resource(
        &self,
        resource_id: &str,
    ) -> VfsResult<()> {
        let callback = self.create_vector_index_callback();
        callback(resource_id.to_string()).await
    }
}
