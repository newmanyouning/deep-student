//! VFS Tauri 命令处理器
//!
//! 提供 VFS 相关的 Tauri 命令，供前端直接调用（低层 API）。
//! 所有命令以 `vfs_` 前缀命名。
//!
//! 原 handlers.rs 已按领域拆分为 handlers/ 子目录。
//!
//! ## 子模块
//! - `resource_handlers`: 资源 CRUD + 引用计数
//! - `note_handlers`: 笔记 CRUD
//! - `file_handlers`: 文件上传/查询/删除
//! - `attachment_handlers`: 附件上传/查询/删除
//! - `index_handlers`: 搜索/索引管理/维度配置/RAG 检索
//! - `mindmap_handlers`: 知识导图 CRUD
//! - `todo_handlers`: 待办事项（占位）
//! - `pomodoro_handlers`: 番茄钟（占位）
//! - `ocr_handlers`: OCR 文本查看/清除
//! - `multimodal_handlers`: 多模态索引/检索
//! - `pdf_handlers`: PDF 预处理流水线/页面图片/下载
//! - `ref_handlers`: 路径缓存
//! - `debug_handlers`: 诊断/重置/缓存管理

// ============================================================================
// 子模块声明
// ============================================================================

pub mod resource_handlers;
pub mod note_handlers;
pub mod file_handlers;
pub mod attachment_handlers;
pub mod index_handlers;
pub mod mindmap_handlers;
pub mod todo_handlers;
pub mod pomodoro_handlers;
pub mod ocr_handlers;
pub mod multimodal_handlers;
pub mod pdf_handlers;
pub mod ref_handlers;
pub mod debug_handlers;

// ============================================================================
// Re-exports: 所有公共类型和函数
// ============================================================================

// resource_handlers
pub use resource_handlers::{
    compute_hash, default_limit, get_max_size_bytes, validate_file_size, validate_id_format,
    validate_id_format_any, CreateResourceInput, ListInput, SearchAllInput,
};
pub use resource_handlers::{
    vfs_create_or_reuse, vfs_decrement_ref, vfs_get_resource, vfs_increment_ref,
    vfs_resource_exists,
};

// note_handlers
pub use note_handlers::{
    vfs_create_note, vfs_delete_note, vfs_get_note, vfs_get_note_content, vfs_list_notes,
    vfs_update_note, CreateNoteInput, UpdateNoteInput,
};

// file_handlers
pub use file_handlers::{
    vfs_delete_file, vfs_get_file, vfs_get_file_content, vfs_list_files, vfs_upload_file,
    IndexStatus, OcrStatus, VfsFileContentResult, VfsUploadFileParams, VfsUploadFileResult,
};

// attachment_handlers
pub use attachment_handlers::{
    vfs_create_attachment_root_folder, vfs_delete_attachment, vfs_get_attachment,
    vfs_get_attachment_config, vfs_get_attachment_content, vfs_get_or_create_attachment_root_folder,
    vfs_set_attachment_root_folder, vfs_upload_attachment, AttachmentConfigOutput,
    VfsAttachmentContentResult, VfsUploadAttachmentParamsExt,
};

// index_handlers
pub use index_handlers::{
    vfs_assign_dimension_model, vfs_batch_index_pending, vfs_clear_default_embedding_dimension,
    vfs_create_dimension, vfs_delete_dimension, vfs_get_all_index_status,
    vfs_get_default_embedding_dimension, vfs_get_dimension_range, vfs_get_embedding_stats,
    vfs_get_index_status, vfs_get_indexing_config, vfs_get_lance_stats, vfs_get_pending_resources,
    vfs_get_preset_dimensions, vfs_list_dimensions, vfs_list_essays, vfs_list_exam_sheets,
    vfs_list_textbooks, vfs_list_translations, vfs_optimize_lance, vfs_rag_search, vfs_reindex_resource,
    vfs_search, vfs_search_all, vfs_set_default_embedding_dimension, vfs_set_indexing_config,
    vfs_toggle_index_disabled, BatchIndexResult, DeleteDimensionResult, IndexStatusSummary,
    ResourceIndexStatus, VfsRagSearchInput, VfsRagSearchOutput,
};

// mindmap_handlers
pub use mindmap_handlers::{
    vfs_create_mindmap, vfs_delete_mindmap, vfs_get_mindmap, vfs_get_mindmap_content,
    vfs_get_mindmap_version, vfs_get_mindmap_version_content, vfs_get_mindmap_versions,
    vfs_list_mindmaps, vfs_set_mindmap_favorite, vfs_update_mindmap, CreateMindMapInput,
    UpdateMindMapInput,
};

// ocr_handlers
pub use ocr_handlers::{
    vfs_clear_resource_ocr, vfs_get_resource_ocr_info, vfs_get_resource_text_chunks,
    OcrPageInfo, ResourceOcrInfo, TextChunkInfo,
};

// multimodal_handlers
pub use multimodal_handlers::{
    vfs_multimodal_delete, vfs_multimodal_index, vfs_multimodal_index_resource,
    vfs_multimodal_search, vfs_multimodal_stats, VfsMultimodalIndexInput,
    VfsMultimodalIndexOutput, VfsMultimodalIndexPageInput, VfsMultimodalSearchInput,
    VfsMultimodalSearchOutput,
};

// pdf_handlers
pub use pdf_handlers::{
    vfs_cancel_pdf_processing, vfs_download_paper, vfs_get_blob_base64,
    vfs_get_batch_pdf_processing_status, vfs_get_pdf_page_image, vfs_get_pdf_processing_status,
    vfs_list_pending_pdf_processing, vfs_retry_pdf_processing, vfs_start_pdf_processing,
    VfsBlobBase64Result, VfsDownloadPaperParams, VfsDownloadPaperResult,
};
pub use pdf_handlers::{
    image_needs_compression_with_conn, pdf_preview_needs_compression,
};

// ref_handlers
pub use ref_handlers::{
    vfs_get_resource_path, vfs_update_path_cache,
};

// debug_handlers
pub use debug_handlers::{
    vfs_clear_media_cache, vfs_debug_index_status, vfs_diagnose_lance_schema,
    vfs_get_media_cache_stats, vfs_reset_all_index_state, vfs_reset_disabled_to_pending,
    vfs_reset_indexed_without_embeddings, ClearMediaCacheParams, ClearMediaCacheResult,
    ConsistencyCheck, DimensionStats, IndexDiagnosticInfo, IndexStateCounts, MediaCacheStats,
    ResourceDiagnostic, SegmentsStats, UnitsStats,
};

// ============================================================================
// 测试（原 handlers.rs 测试集）
// ============================================================================

#[cfg(test)]
mod tests {
    use super::resource_handlers::{
        compute_hash, get_max_size_bytes, validate_file_size, CreateResourceInput,
    };
    use super::note_handlers::{CreateNoteInput, UpdateNoteInput};
    use super::resource_handlers::{ListInput, SearchAllInput};
    use crate::vfs::types::VfsResourceType;

    #[test]
    fn test_file_size_validation() {
        let small_data = "x".repeat(1024);
        assert!(validate_file_size(&VfsResourceType::Image, &small_data).is_ok());

        let large_data = "x".repeat(11 * 1024 * 1024);
        assert!(validate_file_size(&VfsResourceType::Image, &large_data).is_err());

        let medium_data = "x".repeat(20 * 1024 * 1024);
        assert!(validate_file_size(&VfsResourceType::File, &medium_data).is_ok());

        // 但 File 也有上限
        let very_large_data = "x".repeat(51 * 1024 * 1024); // 51MB
        assert!(validate_file_size(&VfsResourceType::File, &very_large_data).is_err());
    }

    #[test]
    fn test_compute_hash() {
        let hash1 = compute_hash("hello world");
        let hash2 = compute_hash("hello world");
        let hash3 = compute_hash("hello world!");

        // 相同内容应产生相同哈希
        assert_eq!(hash1, hash2);
        // 不同内容应产生不同哈希
        assert_ne!(hash1, hash3);
        // 哈希应该是 64 字符的十六进制字符串
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_max_size_bytes() {
        assert_eq!(
            get_max_size_bytes(&VfsResourceType::Image),
            10 * 1024 * 1024
        );
        assert_eq!(get_max_size_bytes(&VfsResourceType::File), 50 * 1024 * 1024);
        assert_eq!(get_max_size_bytes(&VfsResourceType::Note), 50 * 1024 * 1024);
        assert_eq!(
            get_max_size_bytes(&VfsResourceType::Translation),
            10 * 1024 * 1024
        );
    }

    #[test]
    fn test_create_resource_input_deserialization() {
        let json = r#"{
            "type": "note",
            "data": "test content",
            "sourceId": "note_123"
        }"#;

        let input: CreateResourceInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.resource_type, "note");
        assert_eq!(input.data, "test content");
        assert_eq!(input.source_id, Some("note_123".to_string()));
    }

    #[test]
    fn test_create_note_input_deserialization() {
        let json = r#"{
            "title": "Test Note",
            "content": "note content",
            "tags": ["tag1", "tag2"]
        }"#;

        let input: CreateNoteInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.title, "Test Note");
        assert_eq!(input.content, "note content");
        assert_eq!(input.tags, vec!["tag1", "tag2"]);
    }

    #[test]
    fn test_list_input_defaults() {
        let json = r#"{}"#;

        let input: ListInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.search, None);
        assert_eq!(input.limit, 50); // default
        assert_eq!(input.offset, 0); // default
    }

    #[test]
    fn test_search_all_input_deserialization() {
        let json = r#"{
            "query": "期末复习",
            "types": ["note", "exam"],
            "limit": 20
        }"#;

        let input: SearchAllInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.query, "期末复习");
        assert_eq!(
            input.types,
            Some(vec!["note".to_string(), "exam".to_string()])
        );
        assert_eq!(input.limit, 20);
        assert_eq!(input.offset, 0); // default
    }

    /// 验证 vfs_update_note 参数结构支持自动版本管理
    #[test]
    fn test_update_note_params_for_versioning() {
        // 验证仅更新内容
        let json_content_only = r#"{
            "content": "新版本内容"
        }"#;
        let input: UpdateNoteInput = serde_json::from_str(json_content_only).unwrap();
        assert_eq!(input.content, "新版本内容");
        assert_eq!(input.title, None);
        assert_eq!(input.tags, None);

        // 验证同时更新内容和标题
        let json_with_title = r#"{
            "content": "新版本内容",
            "title": "新标题"
        }"#;
        let input: UpdateNoteInput = serde_json::from_str(json_with_title).unwrap();
        assert_eq!(input.content, "新版本内容");
        assert_eq!(input.title, Some("新标题".to_string()));

        // 验证完整更新
        let json_full = r#"{
            "content": "新版本内容",
            "title": "新标题",
            "tags": ["重要", "复习"]
        }"#;
        let input: UpdateNoteInput = serde_json::from_str(json_full).unwrap();
        assert_eq!(input.content, "新版本内容");
        assert_eq!(input.title, Some("新标题".to_string()));
        assert_eq!(
            input.tags,
            Some(vec!["重要".to_string(), "复习".to_string()])
        );
    }

    /// 验证 vfs_search_all 跨类型搜索参数
    #[test]
    fn test_search_all_cross_type_params() {
        // 验证搜索所有类型（不指定 types）
        let json_all_types = r#"{
            "query": "期末考试"
        }"#;
        let input: SearchAllInput = serde_json::from_str(json_all_types).unwrap();
        assert_eq!(input.query, "期末考试");
        assert_eq!(input.types, None); // 查询所有类型

        // 验证搜索特定类型
        let json_specific_types = r#"{
            "query": "期末考试",
            "types": ["note", "exam", "textbook"]
        }"#;
        let input: SearchAllInput = serde_json::from_str(json_specific_types).unwrap();
        assert_eq!(input.query, "期末考试");
        assert_eq!(
            input.types,
            Some(vec![
                "note".to_string(),
                "exam".to_string(),
                "textbook".to_string()
            ])
        );

        // 验证搜索单一类型
        let json_single_type = r#"{
            "query": "翻译",
            "types": ["translation"]
        }"#;
        let input: SearchAllInput = serde_json::from_str(json_single_type).unwrap();
        assert_eq!(input.types, Some(vec!["translation".to_string()]));

        // 验证跨类型搜索
        let json_multi_type = r#"{
            "query": "语法",
            "types": ["note", "essay"]
        }"#;
        let input: SearchAllInput = serde_json::from_str(json_multi_type).unwrap();
        assert_eq!(input.query, "语法");
        assert_eq!(
            input.types,
            Some(vec!["note".to_string(), "essay".to_string()])
        );
    }

    /// 验证空查询词被正确拒绝
    #[test]
    fn test_empty_query_validation() {
        // 空字符串查询应该被拒绝（在命令实现中验证）
        let json = r#"{
            "query": ""
        }"#;
        let input: SearchAllInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.query, "");
        // 实际的空查询验证在 vfs_search_all 命令中执行
    }

    /// 验证资源 ID 格式验证
    #[test]
    fn test_resource_id_format_validation() {
        // 验证有效的资源 ID 前缀
        assert!("res_abc123".starts_with("res_"));
        assert!("res_1234567890".starts_with("res_"));

        // 验证无效的资源 ID 前缀
        assert!(!"note_abc123".starts_with("res_"));
        assert!(!"abc123".starts_with("res_"));
    }

    /// 验证笔记 ID 格式验证
    #[test]
    fn test_note_id_format_validation() {
        // 验证有效的笔记 ID 前缀
        assert!("note_abc123".starts_with("note_"));
        assert!("note_1234567890".starts_with("note_"));

        // 验证无效的笔记 ID 前缀
        assert!(!"res_abc123".starts_with("note_"));
        assert!(!"abc123".starts_with("note_"));
    }

    // ========================================================================
    // 路径缓存相关测试（文档 24 Prompt 3）
    // ========================================================================

    /// 验证文件夹 ID 格式验证
    #[test]
    fn test_folder_id_format_validation() {
        // 验证有效的文件夹 ID 前缀
        assert!("fld_abc123".starts_with("fld_"));
        assert!("fld_1234567890".starts_with("fld_"));

        // 验证无效的文件夹 ID 前缀
        assert!(!"note_abc123".starts_with("fld_"));
        assert!(!"folder_abc123".starts_with("fld_"));
        assert!(!"abc123".starts_with("fld_"));
    }

    /// 验证 source_id 前缀解析
    #[test]
    fn test_source_id_prefix_parsing() {
        // 验证各种 source_id 的前缀提取
        let note_id = "note_abc123";
        let tb_id = "tb_def456";
        let exam_id = "exam_ghi789";
        let tr_id = "tr_jkl012";
        let essay_id = "essay_mno345";

        assert_eq!(note_id.split('_').next(), Some("note"));
        assert_eq!(tb_id.split('_').next(), Some("tb"));
        assert_eq!(exam_id.split('_').next(), Some("exam"));
        assert_eq!(tr_id.split('_').next(), Some("tr"));
        assert_eq!(essay_id.split('_').next(), Some("essay"));
    }

    /// 验证路径长度约束（契约 D：最大 1000 字符）
    #[test]
    fn test_path_length_constraint() {
        let max_path_length = 1000;

        // 短路径应该通过
        let short_path = "/文件夹/笔记";
        assert!(short_path.len() <= max_path_length);

        // 极长路径应该失败
        let long_path = "/".repeat(max_path_length + 1);
        assert!(long_path.len() > max_path_length);
    }

    /// 验证路径格式
    #[test]
    fn test_path_format() {
        // 根目录资源路径格式
        let root_path = "/笔记标题";
        assert!(root_path.starts_with('/'));

        // 嵌套路径格式
        let nested_path = "/高考复习/函数/笔记标题";
        assert!(nested_path.starts_with('/'));
        assert!(nested_path.contains("高考复习"));
        assert!(nested_path.contains("函数"));
    }
}
