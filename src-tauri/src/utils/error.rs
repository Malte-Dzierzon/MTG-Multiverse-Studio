//! Error Handling
//! 
//! Custom error types and utilities for the application.

use serde::Serialize;

/// Application-specific error type
#[derive(Debug, thiserror::Error, Serialize)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

// ─── From impls ─────────────────────────────────

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Http(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(e: tauri::Error) -> Self {
        AppError::Unknown(e.to_string())
    }
}

/// Convenient type alias for Result with AppError
pub type Result<T> = std::result::Result<T, AppError>;
