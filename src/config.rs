//! Configuration types and loading.
//!
//! Ported from Python: `flowmark/config.py` and `flowmark/formats/flowmark_markdown.py`

use std::fmt;
use std::str::FromStr;

/// Controls how list item spacing is handled during Markdown normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum ListSpacing {
    /// Keep lists tight or loose as authored (default).
    #[default]
    Preserve,
    /// Convert all lists to loose format (blank lines between items).
    Loose,
    /// Convert all lists to tight format where possible.
    Tight,
}

impl fmt::Display for ListSpacing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Preserve => write!(f, "preserve"),
            Self::Loose => write!(f, "loose"),
            Self::Tight => write!(f, "tight"),
        }
    }
}

impl FromStr for ListSpacing {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "preserve" => Ok(Self::Preserve),
            "loose" => Ok(Self::Loose),
            "tight" => Ok(Self::Tight),
            _ => Err(format!("invalid list spacing: {s}")),
        }
    }
}

/// Default wrap width (88 characters).
pub const DEFAULT_WRAP_WIDTH: usize = 88;

/// Default minimum line length for sentence breaking.
pub const DEFAULT_MIN_LINE_LEN: usize = 20;

/// Options controlling Markdown formatting behavior.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct FormatOptions {
    /// Line width (0 to disable wrapping).
    pub width: usize,
    /// Treat input as plaintext (no Markdown parsing).
    pub plaintext: bool,
    /// Use semantic (sentence-based) line breaks.
    pub semantic: bool,
    /// Apply safe cleanups (e.g., unbold headings).
    pub cleanups: bool,
    /// Convert straight quotes to curly quotes.
    pub smartquotes: bool,
    /// Convert `...` to ellipsis character.
    pub ellipses: bool,
    /// Control list item spacing.
    pub list_spacing: ListSpacing,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            width: DEFAULT_WRAP_WIDTH,
            plaintext: false,
            semantic: false,
            cleanups: false,
            smartquotes: false,
            ellipses: false,
            list_spacing: ListSpacing::default(),
        }
    }
}
