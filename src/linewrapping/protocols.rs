//! Protocol definitions for the linewrapping module.
//!
//! Ported from Python: flowmark/linewrapping/protocols.py

/// A line wrapper takes text and indents, returns wrapped text.
///
/// In Python this is a Protocol class; in Rust we use a trait alias via
/// a boxed function type.
pub type LineWrapper = Box<dyn Fn(&str, &str, &str) -> String>;
