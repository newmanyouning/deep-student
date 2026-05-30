//! Memory 模块统一错误类型

use serde::Serialize;

/// Memory 模块统一错误类型
#[derive(Debug, Serialize)]
pub enum MemoryError {
    /// 数据库/VFS 错误
    Database(String),
    /// 验证错误
    Validation(String),
    /// 未找到
    NotFound(String),
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryError::Database(msg) => write!(f, "Memory database error: {}", msg),
            MemoryError::Validation(msg) => write!(f, "Memory validation error: {}", msg),
            MemoryError::NotFound(msg) => write!(f, "Memory not found: {}", msg),
            MemoryError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<crate::vfs::error::VfsError> for MemoryError {
    fn from(e: crate::vfs::error::VfsError) -> Self {
        MemoryError::Database(e.to_string())
    }
}

/// Memory 模块 Result 别名
pub type MemoryResult<T> = Result<T, MemoryError>;
