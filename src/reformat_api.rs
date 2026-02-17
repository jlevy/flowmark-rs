//! High-level reformatting API.
//!
//! Ported from Python: flowmark/reformat_api.py

use std::path::Path;

use crate::config::FlowmarkConfig;
use crate::error::Result;
use crate::linewrapping::markdown_filling::fill_markdown;

/// Reformat a Markdown text string.
pub fn reformat_text(
    text: &str,
    config: &FlowmarkConfig,
) -> String {
    fill_markdown(
        text,
        config.semantic,
        config.width,
        config.cleanups,
        config.smartquotes,
        config.ellipses,
        None,
        config.list_spacing_mode(),
    )
}

/// Reformat a Markdown file in place.
pub fn reformat_file(path: &Path, config: &FlowmarkConfig) -> Result<bool> {
    let original = std::fs::read_to_string(path)?;
    let reformatted = reformat_text(&original, config);

    if reformatted != original {
        std::fs::write(path, &reformatted)?;
        Ok(true)
    } else {
        Ok(false)
    }
}
