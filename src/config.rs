//! Configuration file loading.
//!
//! Ported from Python: flowmark/config.py

use serde::Deserialize;
use std::path::Path;

use crate::formats::flowmark_markdown::ListSpacing;
use crate::linewrapping::text_filling::DEFAULT_WRAP_WIDTH;

/// Configuration for Flowmark formatting.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FlowmarkConfig {
    /// Wrap width in characters.
    pub width: usize,
    /// Use semantic line breaks (break at sentence boundaries).
    pub semantic: bool,
    /// Apply cleanups (e.g., unbold headings).
    pub cleanups: bool,
    /// Convert straight quotes to typographic quotes.
    pub smartquotes: bool,
    /// Convert `...` to proper ellipsis character.
    pub ellipses: bool,
    /// List spacing mode.
    pub list_spacing: String,
}

impl Default for FlowmarkConfig {
    fn default() -> Self {
        Self {
            width: DEFAULT_WRAP_WIDTH,
            semantic: true,
            cleanups: true,
            smartquotes: false,
            ellipses: false,
            list_spacing: "preserve".to_string(),
        }
    }
}

impl FlowmarkConfig {
    /// Parse list spacing mode from config string.
    pub fn list_spacing_mode(&self) -> ListSpacing {
        match self.list_spacing.as_str() {
            "loose" => ListSpacing::Loose,
            "tight" => ListSpacing::Tight,
            _ => ListSpacing::Preserve,
        }
    }

    /// Load configuration from a TOML file.
    pub fn from_file(path: &Path) -> Result<Self, crate::error::FlowmarkError> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| crate::error::FlowmarkError::ConfigParse(e.to_string()))
    }

    /// Search for and load a configuration file.
    ///
    /// Looks for `.flowmark.toml` or `flowmark.toml` in the current directory
    /// and parent directories.
    pub fn find_and_load() -> Option<Self> {
        let cwd = std::env::current_dir().ok()?;
        let mut dir = cwd.as_path();

        loop {
            for name in &[".flowmark.toml", "flowmark.toml"] {
                let path = dir.join(name);
                if path.exists() {
                    return Self::from_file(&path).ok();
                }
            }
            dir = dir.parent()?;
        }
    }
}
