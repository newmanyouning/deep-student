//! Data Governance 统一错误类型

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum DataGovernanceError {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Backup error: {0}")]
    Backup(String),

    #[error("Restore error: {0}")]
    Restore(String),

    #[error("Sync error: {0}")]
    Sync(String),
}

pub type DataGovernanceResult<T> = Result<T, DataGovernanceError>;

impl From<String> for DataGovernanceError {
    fn from(s: String) -> Self {
        DataGovernanceError::Internal(s)
    }
}

impl From<rusqlite::Error> for DataGovernanceError {
    fn from(e: rusqlite::Error) -> Self {
        DataGovernanceError::Database(format!("{:#}", e))
    }
}

impl From<serde_json::Error> for DataGovernanceError {
    fn from(e: serde_json::Error) -> Self {
        DataGovernanceError::Serialization(e.to_string())
    }
}

impl From<std::io::Error> for DataGovernanceError {
    fn from(e: std::io::Error) -> Self {
        DataGovernanceError::Io(e.to_string())
    }
}
