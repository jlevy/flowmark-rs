//! Parser-independent source preservation for Markdown constructs that Flowmark
//! recognizes but does not semantically model.

mod bridge;
mod model;
mod normalization;
mod registry;
mod scanner;

pub(crate) use bridge::{InlineRewriteSegment, ProtectedSource, protect_source, restore_source};
pub(crate) use model::{NormalizedSource, PreservationError};
pub(crate) use normalization::{finalize_output, normalize_source};
pub(crate) use scanner::{
    fenced_code_info_ranges, opaque_markdown_line_flags, scan_protected_regions,
};
