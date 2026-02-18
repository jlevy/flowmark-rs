//! Error types for flowmark.

/// Errors that can occur during flowmark operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("{0}")]
    Other(String),
}

/// Result type alias for flowmark operations.
pub type Result<T> = std::result::Result<T, Error>;
