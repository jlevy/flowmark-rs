//! Error types for Flowmark.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlowmarkError {
    #[error("Cannot use `inplace` with stdin")]
    InplaceWithStdin,

    #[error("Cannot specify output file when processing multiple files (use --inplace instead)")]
    OutputWithMultipleFiles,

    #[error("No input specified")]
    NoInput,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config parse error: {0}")]
    ConfigParse(String),
}

pub type Result<T> = std::result::Result<T, FlowmarkError>;
