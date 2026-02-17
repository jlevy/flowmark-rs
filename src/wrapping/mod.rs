//! Line wrapping, sentence splitting, and text processing.

pub mod atomic_patterns;
pub mod block_heuristics;
pub mod line_wrappers;
pub mod sentence;
pub mod tag_handling;
pub mod text_filling;
pub mod text_wrapping;

/// A function that wraps text with indentation.
///
/// Takes text, an initial indent for the first line, and a subsequent indent
/// for continuation lines. Returns the wrapped text.
pub type LineWrapper = Box<dyn Fn(&str, &str, &str) -> String + Send + Sync>;
