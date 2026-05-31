//! Essay Grading 模块统一错误类型

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum EssayGradingError {
    Database(String),
    Validation(String),
    NotFound(String),
    Internal(String),
    Other(String),
}

impl std::fmt::Display for EssayGradingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EssayGradingError::Database(msg) => write!(f, "EssayGrading database error: {}", msg),
            EssayGradingError::Validation(msg) => write!(f, "EssayGrading validation error: {}", msg),
            EssayGradingError::NotFound(msg) => write!(f, "EssayGrading not found: {}", msg),
            EssayGradingError::Internal(msg) => write!(f, "EssayGrading internal error: {}", msg),
            EssayGradingError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for EssayGradingError {}

impl From<crate::models::AppError> for EssayGradingError {
    fn from(e: crate::models::AppError) -> Self {
        match e.error_type {
            crate::models::AppErrorType::Database => EssayGradingError::Database(e.message),
            crate::models::AppErrorType::Validation => EssayGradingError::Validation(e.message),
            crate::models::AppErrorType::NotFound => EssayGradingError::NotFound(e.message),
            _ => EssayGradingError::Internal(format!("{}: {}", e.error_type, e.message)),
        }
    }
}

impl From<crate::vfs::error::VfsError> for EssayGradingError {
    fn from(e: crate::vfs::error::VfsError) -> Self {
        EssayGradingError::Database(e.to_string())
    }
}

impl From<anyhow::Error> for EssayGradingError {
    fn from(e: anyhow::Error) -> Self {
        EssayGradingError::Other(format!("{:#}", e))
    }
}

pub type EssayGradingResult<T> = std::result::Result<T, EssayGradingError>;

impl From<EssayGradingError> for String {
    fn from(e: EssayGradingError) -> Self {
        e.to_string()
    }
}
