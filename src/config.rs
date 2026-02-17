//! Configuration types and loading.
//!
//! Ported from Python: flowmark/config.py and flowmark/formats/flowmark_markdown.py

use std::fmt;
use std::str::FromStr;

/// Controls how list item spacing is handled during Markdown normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// Default wrap width. Same as Black (88 characters).
pub const DEFAULT_WRAP_WIDTH: usize = 88;

/// Default minimum line length for sentence breaking.
pub const DEFAULT_MIN_LINE_LEN: usize = 20;
