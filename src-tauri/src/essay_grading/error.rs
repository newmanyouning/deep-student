//! Essay Grading 模块统一错误类型

use serde::Serialize;

/// Essay Grading 模块统一错误类型
#[derive(Debug, Serialize)]
pub enum EssayGradingError {
    /// 数据库/VFS 错误
    Database(String),
    /// 内部错误
    Internal(String),
    /// 验证错误
    Validation(String),
    /// 未找到
    NotFound(String),
}

impl std::fmt::Display for EssayGradingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EssayGradingError::Database(msg) => write!(f, "Grading database error: {}", msg),
            EssayGradingError::Internal(msg) => write!(f, "Grading internal error: {}", msg),
            EssayGradingError::Validation(msg) => write!(f, "Grading validation error: {}", msg),
            EssayGradingError::NotFound(msg) => write!(f, "Grading not found: {}", msg),
        }
    }
}

impl From<crate::models::AppError> for EssayGradingError {
    fn from(e: crate::models::AppError) -> Self {
        use crate::models::AppErrorType;
        match e.error_type {
            AppErrorType::Database => EssayGradingError::Database(e.message),
            AppErrorType::Validation => EssayGradingError::Validation(e.message),
            AppErrorType::NotFound => EssayGradingError::NotFound(e.message),
            _ => EssayGradingError::Internal(e.message),
        }
    }
}

impl From<String> for EssayGradingError {
    fn from(e: String) -> Self {
        EssayGradingError::Internal(e)
    }
}

impl From<crate::vfs::error::VfsError> for EssayGradingError {
    fn from(e: crate::vfs::error::VfsError) -> Self {
        EssayGradingError::Database(e.to_string())
    }
}

/// Essay Grading 模块 Result 别名
pub type EssayGradingResult<T> = Result<T, EssayGradingError>;
