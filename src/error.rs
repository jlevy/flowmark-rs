//! Error types for flowmark.

/// Errors that can occur during flowmark operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}

impl Error {
    /// Whether this error was caused by strict UTF-8 decoding.
    #[must_use]
    pub fn is_invalid_utf8(&self) -> bool {
        match self {
            Self::Io(source) => source
                .get_ref()
                .is_some_and(|cause| cause.downcast_ref::<std::str::Utf8Error>().is_some()),
        }
    }
}

/// Result type alias for flowmark operations.
pub type Result<T> = std::result::Result<T, Error>;
