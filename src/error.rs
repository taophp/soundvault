//! Error types for the SoundVault library

use thiserror::Error;

/// Custom error type for SoundVault operations
#[derive(Error, Debug)]
pub enum VaultError {
    /// Error related to the local file system
    #[error("File system error: {0}")]
    FileSystem(String),

    /// Error related to the database
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Error related to Freesound API
    #[error("Freesound API error: {0}")]
    FreesoundApi(#[from] freesound_rs::FreesoundError),

    /// Error related to JSON serialization/deserialization
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error related to IO operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Error related to configuration
    #[error("Configuration error: {0}")]
    Config(String),

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Sound not found
    #[error("Sound not found: {0}")]
    NotFound(String),
}

/// Convenience type alias for Result with VaultError
pub type Result<T> = std::result::Result<T, VaultError>;
