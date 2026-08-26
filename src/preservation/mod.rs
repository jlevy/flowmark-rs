//! Parser-independent source preservation for Markdown constructs that Flowmark
//! recognizes but does not semantically model.

mod bridge;
mod model;
mod normalization;
mod registry;
mod scanner;

pub(crate) use bridge::{ProtectedSource, protect_source, restore_source};
pub(crate) use normalization::{finalize_output, normalize_source};
pub(crate) use scanner::scan_protected_regions;
