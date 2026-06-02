//! Pomodoro Tauri 命令处理器（handlers 子模块）
//!
//! 本文件是 handlers/ 目录下的子模块，将功能委派给 vfs::todo_handlers 中的实际实现。
//! 所有番茄钟命令均位于 vfs::todo_handlers 模块中。

pub use crate::vfs::todo_handlers::{
    vfs_pomodoro_create_record, vfs_pomodoro_get_record, vfs_pomodoro_list_by_todo,
    vfs_pomodoro_list_today, vfs_pomodoro_today_stats, CreatePomodoroInput,
};
