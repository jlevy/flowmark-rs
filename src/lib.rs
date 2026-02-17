//! Flowmark: Better auto-formatting for Markdown and plaintext.
//!
//! Ported from Python: <https://github.com/jlevy/flowmark>
//!
//! Flowmark provides enhanced text wrapping capabilities with special handling for
//! Markdown content. It can:
//!
//! - Format Markdown with proper line wrapping while preserving structure
//!   and normalizing Markdown formatting
//! - Optionally break lines at sentence boundaries for better diff readability
//! - Process plaintext with HTML-aware word splitting

pub mod config;
pub mod error;
pub mod formats;
pub mod linewrapping;
pub mod reformat_api;
pub mod transforms;
pub mod typography;

// Re-exports for the public API
pub use formats::flowmark_markdown::flowmark_markdown;
pub use formats::frontmatter::split_frontmatter;
pub use linewrapping::line_wrappers::{line_wrap_by_sentence, line_wrap_to_width};
pub use linewrapping::markdown_filling::fill_markdown;
pub use linewrapping::sentence_split_regex::{
    first_sentence, first_sentences, split_sentences_regex,
};
pub use linewrapping::text_filling::{fill_text, Wrap};
pub use linewrapping::text_wrapping::{
    get_html_md_word_splitter, simple_word_splitter, wrap_paragraph, wrap_paragraph_lines,
};
pub use reformat_api::{reformat_file, reformat_text};
