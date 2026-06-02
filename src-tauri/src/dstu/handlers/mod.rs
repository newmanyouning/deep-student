//! DSTU 命令处理器（拆分后）
//!
//! 本模块将原来的 handlers.rs 拆分为按资源类型组织的子模块。
//! 路由逻辑在 common.rs 中，类型特定实现在各自文件中。

pub mod common;
pub mod note_handlers;
pub mod textbook_handlers;
pub mod exam_handlers;
pub mod translation_handlers;
pub mod essay_handlers;
pub mod image_handlers;
pub mod file_handlers;
pub mod mindmap_handlers;
pub mod search_handlers;

// 旧 handlers.rs 中的辅助函数和常量
pub use common::{
    is_memory_system_hidden_name, log_and_skip_err, MAX_BATCH_SIZE, MAX_CONTENT_SIZE,
    MAX_METADATA_SIZE, MAX_NAME_LENGTH,
};

// 重导出 Tauri 命令（向后兼容）
pub use common::{
    dstu_batch_move, dstu_build_path, dstu_copy, dstu_create, dstu_delete, dstu_delete_many,
    dstu_get, dstu_get_content, dstu_get_exam_content, dstu_get_path_by_id,
    dstu_get_resource_by_path, dstu_get_resource_location, dstu_list, dstu_list_deleted, dstu_move,
    dstu_move_many, dstu_move_to_folder, dstu_parse_path, dstu_purge, dstu_purge_all,
    dstu_refresh_path_cache, dstu_rename, dstu_restore, dstu_restore_many, dstu_set_favorite,
    dstu_set_metadata, dstu_update, dstu_unwatch, dstu_watch,
};
pub use search_handlers::{dstu_search, dstu_search_in_folder};
