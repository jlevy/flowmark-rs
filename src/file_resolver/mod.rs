//! File discovery and filtering.
//!
//! Ported from Python: `flowmark/file_resolver/`

pub mod config;
pub(crate) mod defaults;
pub mod gitignore;
pub mod resolver;

pub use config::FileResolverConfig;
pub use resolver::FileResolver;
