//! Todo Tauri 命令处理器（handlers 子模块）
//!
//! 本文件是 handlers/ 目录下的子模块，将功能委派给 vfs::todo_handlers 中的实际实现。
//! 所有待办列表和待办项的 CRUD 命令均位于 vfs::todo_handlers 模块中。

pub use crate::vfs::todo_handlers::{
    vfs_todo_create_item, vfs_todo_create_list, vfs_todo_delete_item, vfs_todo_delete_list,
    vfs_todo_ensure_inbox, vfs_todo_get_active_summary, vfs_todo_get_item, vfs_todo_get_list,
    vfs_todo_list_completed, vfs_todo_list_items, vfs_todo_list_lists, vfs_todo_list_overdue,
    vfs_todo_list_today, vfs_todo_list_upcoming, vfs_todo_reorder_items, vfs_todo_search,
    vfs_todo_toggle_item, vfs_todo_toggle_list_favorite, vfs_todo_update_item,
    vfs_todo_update_list, CreateTodoItemInput, CreateTodoListInput, ReorderItemsInput,
    UpdateTodoItemInput, UpdateTodoListInput,
};
