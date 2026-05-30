//! Review Plan 模块统一错误类型
//!
//! 按原重构计划 (2026-05-29) 创建, 17 个 Tauri 命令全部迁移到此类型。

use serde::Serialize;

/// Review Plan 模块统一错误类型
#[derive(Debug, Serialize)]
pub enum ReviewPlanError {
    /// 数据库/VFS 错误
    Database(String),
    /// 验证错误
    Validation(String),
    /// 未找到
    NotFound(String),
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for ReviewPlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewPlanError::Database(msg) => write!(f, "ReviewPlan database error: {}", msg),
            ReviewPlanError::Validation(msg) => write!(f, "ReviewPlan validation error: {}", msg),
            ReviewPlanError::NotFound(msg) => write!(f, "ReviewPlan not found: {}", msg),
            ReviewPlanError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<anyhow::Error> for ReviewPlanError {
    fn from(e: anyhow::Error) -> Self {
        ReviewPlanError::Database(format!("{:#}", e))
    }
}

impl From<crate::vfs::error::VfsError> for ReviewPlanError {
    fn from(e: crate::vfs::error::VfsError) -> Self {
        ReviewPlanError::Database(e.to_string())
    }
}

impl From<String> for ReviewPlanError {
    fn from(e: String) -> Self {
        ReviewPlanError::Other(e)
    }
}

/// Review Plan 模块 Result 别名
pub type ReviewPlanResult<T> = Result<T, ReviewPlanError>;
