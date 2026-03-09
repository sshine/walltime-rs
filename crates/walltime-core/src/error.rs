use thiserror::Error;

/// A convenient result type alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Top-level error type for the project.
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}
