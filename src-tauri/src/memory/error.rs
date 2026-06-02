//! Memory 模块统一错误类型

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum MemoryError {
    Database(String),
    Validation(String),
    NotFound(String),
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

impl std::error::Error for MemoryError {}

impl From<crate::vfs::error::VfsError> for MemoryError {
    fn from(e: crate::vfs::error::VfsError) -> Self {
        MemoryError::Database(e.to_string())
    }
}

impl From<anyhow::Error> for MemoryError {
    fn from(e: anyhow::Error) -> Self {
        MemoryError::Other(format!("{:#}", e))
    }
}

impl From<rusqlite::Error> for MemoryError {
    fn from(e: rusqlite::Error) -> Self {
        MemoryError::Database(e.to_string())
    }
}

impl From<String> for MemoryError {
    fn from(s: String) -> Self {
        MemoryError::Other(s)
    }
}

pub type MemoryResult<T> = std::result::Result<T, MemoryError>;

impl From<MemoryError> for String {
    fn from(e: MemoryError) -> Self {
        e.to_string()
    }
}
