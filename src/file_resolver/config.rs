//! Configuration types for file resolution.

use super::defaults::{DEFAULT_EXCLUDES, DEFAULT_INCLUDES};

/// Configuration for file discovery and filtering.
///
/// `tool_name` determines the ignore file name (e.g., `.flowmarkignore`).
/// `exclude = None` means use `DEFAULT_EXCLUDES`; providing a list replaces them entirely.
/// `files_max_size = 0` disables the size limit.
#[derive(Debug, Clone)]
pub struct FileResolverConfig {
    /// Tool name, determines ignore file name (e.g., `.flowmarkignore`).
    pub tool_name: String,
    /// Base include patterns.
    pub include: Vec<String>,
    /// Additional include patterns.
    pub extend_include: Vec<String>,
    /// Override default exclude patterns. `None` = use `DEFAULT_EXCLUDES`.
    pub exclude: Option<Vec<String>>,
    /// Additional exclude patterns.
    pub extend_exclude: Vec<String>,
    /// Whether to respect `.gitignore` files.
    pub respect_gitignore: bool,
    /// Apply exclusions to explicitly named files.
    pub force_exclude: bool,
    /// Maximum file size in bytes. 0 = no limit.
    pub files_max_size: u64,
}

impl Default for FileResolverConfig {
    fn default() -> Self {
        Self {
            tool_name: "flowmark".to_string(),
            include: DEFAULT_INCLUDES.iter().map(|s| (*s).to_string()).collect(),
            extend_include: Vec::new(),
            exclude: None,
            extend_exclude: Vec::new(),
            respect_gitignore: true,
            force_exclude: false,
            files_max_size: 1_048_576, // 1 MiB
        }
    }
}

impl FileResolverConfig {
    /// Combined include patterns: `include + extend_include`.
    pub fn effective_include(&self) -> Vec<String> {
        let mut result = self.include.clone();
        result.extend(self.extend_include.clone());
        result
    }

    /// Combined exclude patterns: defaults (or `exclude`) + `extend_exclude`.
    pub fn effective_exclude(&self) -> Vec<String> {
        let base: Vec<String> = if let Some(ref exclude) = self.exclude {
            exclude.clone()
        } else {
            DEFAULT_EXCLUDES.iter().map(|s| (*s).to_string()).collect()
        };
        let mut result = base;
        result.extend(self.extend_exclude.clone());
        result
    }
}
