//! Error types for flowmark.

/// Errors that can occur during flowmark operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}

/// Result type alias for flowmark operations.
pub type Result<T> = std::result::Result<T, Error>;
