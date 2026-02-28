//! Configuration types and TOML-based config file loading.
//!
//! Ported from Python: `flowmark/config.py` and `flowmark/formats/flowmark_markdown.py`
//!
//! Supports loading from `.flowmark.toml`, `flowmark.toml`, or
//! `pyproject.toml [tool.flowmark]`, walking up from the current directory.
//! Config values are merged with CLI flags using three-way precedence:
//! explicit CLI flags > config file > built-in defaults.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
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

// --- TOML config file loading ---

/// Parsed config from a TOML file. Fields are `None` when not set in the config,
/// allowing the merge logic to distinguish "not configured" from "explicitly set
/// to default value".
#[derive(Debug, Clone, Default, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct FlowmarkConfig {
    // Formatting
    /// Line width.
    pub width: Option<usize>,
    /// Whether to use semantic line breaks.
    pub semantic: Option<bool>,
    /// Whether to apply safe cleanups.
    pub cleanups: Option<bool>,
    /// Whether to convert straight quotes to curly.
    pub smartquotes: Option<bool>,
    /// Whether to convert `...` to ellipsis.
    pub ellipses: Option<bool>,
    /// List spacing mode.
    pub list_spacing: Option<String>,
    // File discovery
    /// Include patterns.
    pub include: Option<Vec<String>>,
    /// Additional include patterns.
    pub extend_include: Option<Vec<String>>,
    /// Exclude patterns (replaces defaults).
    pub exclude: Option<Vec<String>>,
    /// Additional exclude patterns.
    pub extend_exclude: Option<Vec<String>>,
    /// Maximum file size in bytes.
    pub files_max_size: Option<u64>,
    /// Whether to respect `.gitignore`.
    pub respect_gitignore: Option<bool>,
    /// Whether to apply exclusions to explicit files.
    pub force_exclude: Option<bool>,
    // Performance/cache
    /// Whether incremental cache is enabled.
    pub incremental: Option<bool>,
    /// Override incremental cache directory.
    pub incremental_cache_dir: Option<String>,
}

/// Config file search order (first match wins within each directory level).
const CONFIG_FILENAMES: &[&str] = &[".flowmark.toml", "flowmark.toml", "pyproject.toml"];

/// Mapping from TOML kebab-case keys to `snake_case` field names.
fn kebab_to_snake() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("list-spacing", "list_spacing");
    m.insert("extend-include", "extend_include");
    m.insert("extend-exclude", "extend_exclude");
    m.insert("files-max-size", "files_max_size");
    m.insert("respect-gitignore", "respect_gitignore");
    m.insert("force-exclude", "force_exclude");
    m.insert("cache", "incremental");
    m.insert("cache-dir", "incremental_cache_dir");
    m.insert("incremental-cache-dir", "incremental_cache_dir");
    m
}

/// All valid field names in `FlowmarkConfig`.
const VALID_FIELDS: &[&str] = &[
    "width",
    "semantic",
    "cleanups",
    "smartquotes",
    "ellipses",
    "list_spacing",
    "include",
    "extend_include",
    "exclude",
    "extend_exclude",
    "files_max_size",
    "respect_gitignore",
    "force_exclude",
    "incremental",
    "incremental_cache_dir",
];

/// Walk up from `start_dir` looking for a config file. Returns the first
/// found, or `None`. Search order per directory: `.flowmark.toml` >
/// `flowmark.toml` > `pyproject.toml` (only if it has `[tool.flowmark]`).
pub fn find_config_file(start_dir: &Path) -> Option<PathBuf> {
    let Ok(mut current) = start_dir.canonicalize() else {
        return None;
    };
    loop {
        for filename in CONFIG_FILENAMES {
            let candidate = current.join(filename);
            if candidate.is_file() {
                if *filename == "pyproject.toml" {
                    if pyproject_has_flowmark_section(&candidate) {
                        return Some(candidate);
                    }
                } else {
                    return Some(candidate);
                }
            }
        }
        let Some(parent) = current.parent() else {
            break;
        };
        if parent == current {
            break;
        }
        current = parent.to_path_buf();
    }
    None
}

/// Check if a `pyproject.toml` has a `[tool.flowmark]` section.
fn pyproject_has_flowmark_section(path: &Path) -> bool {
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(data) = toml::from_str::<toml::Value>(&text) else {
        return false;
    };
    data.get("tool").and_then(|t| t.get("flowmark")).is_some()
}

/// Load a `FlowmarkConfig` from a TOML file. Supports both standalone
/// `flowmark.toml` / `.flowmark.toml` and `pyproject.toml` (extracts
/// `[tool.flowmark]`). TOML kebab-case keys are mapped to `snake_case`.
///
/// Returns a default (empty) config if the file cannot be read or parsed.
pub fn load_config(config_path: &Path) -> FlowmarkConfig {
    let Ok(text) = std::fs::read_to_string(config_path) else {
        eprintln!("Warning: could not parse config file {}", config_path.display());
        return FlowmarkConfig::default();
    };

    let Ok(data) = toml::from_str::<toml::Value>(&text) else {
        eprintln!("Warning: could not parse config file {}", config_path.display());
        return FlowmarkConfig::default();
    };

    let section = if config_path
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n == "pyproject.toml")
    {
        data.get("tool")
            .and_then(|t| t.get("flowmark"))
            .cloned()
            .unwrap_or(toml::Value::Table(toml::map::Map::new()))
    } else {
        data
    };

    parse_config_data(&section)
}

/// Parse a flat or sectioned TOML value into `FlowmarkConfig`.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn parse_config_data(data: &toml::Value) -> FlowmarkConfig {
    let Some(table) = data.as_table() else {
        return FlowmarkConfig::default();
    };

    // Flatten sections: [formatting] and [file-discovery] merge into top level
    let mut flat: Vec<(String, toml::Value)> = Vec::new();
    for (key, value) in table {
        if let Some(sub_table) = value.as_table() {
            for (sub_key, sub_value) in sub_table {
                flat.push((sub_key.clone(), sub_value.clone()));
            }
        } else {
            flat.push((key.clone(), value.clone()));
        }
    }

    // Map kebab-case to snake_case
    let kebab_map = kebab_to_snake();
    let mut config = FlowmarkConfig::default();

    for (key, value) in &flat {
        let snake_key = kebab_map.get(key.as_str()).copied().unwrap_or(key.as_str());

        // Also handle fallback replacement
        let snake_key_owned = key.replace('-', "_");
        let effective_key =
            if VALID_FIELDS.contains(&snake_key) { snake_key } else { &snake_key_owned };

        if !VALID_FIELDS.contains(&effective_key) {
            eprintln!("Warning: unrecognized config key '{key}'");
            continue;
        }

        set_config_field(&mut config, effective_key, value);
    }

    config
}

/// Set a single field on `FlowmarkConfig` from a TOML value.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn set_config_field(config: &mut FlowmarkConfig, key: &str, value: &toml::Value) {
    match key {
        "width" => {
            if let Some(v) = value.as_integer() {
                config.width = Some(v as usize);
            }
        }
        "semantic" => config.semantic = value.as_bool(),
        "cleanups" => config.cleanups = value.as_bool(),
        "smartquotes" => config.smartquotes = value.as_bool(),
        "ellipses" => config.ellipses = value.as_bool(),
        "list_spacing" => {
            if let Some(v) = value.as_str() {
                config.list_spacing = Some(v.to_string());
            }
        }
        "include" => config.include = extract_string_array(value),
        "extend_include" => config.extend_include = extract_string_array(value),
        "exclude" => config.exclude = extract_string_array(value),
        "extend_exclude" => config.extend_exclude = extract_string_array(value),
        "files_max_size" => {
            if let Some(v) = value.as_integer() {
                config.files_max_size = Some(v as u64);
            }
        }
        "respect_gitignore" => config.respect_gitignore = value.as_bool(),
        "force_exclude" => config.force_exclude = value.as_bool(),
        "incremental" => config.incremental = value.as_bool(),
        "incremental_cache_dir" => {
            if let Some(v) = value.as_str() {
                config.incremental_cache_dir = Some(v.to_string());
            }
        }
        _ => {}
    }
}

/// Extract a `Vec<String>` from a TOML array value.
fn extract_string_array(value: &toml::Value) -> Option<Vec<String>> {
    value.as_array().map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
}

/// Fields that `--auto` locks (these come from the preset, not config).
const AUTO_LOCKED: &[&str] =
    &["semantic", "cleanups", "smartquotes", "ellipses", "inplace", "nobackup"];

/// Merge CLI options with config file settings.
///
/// Precedence: explicit CLI flags > config file > built-in defaults.
/// In `--auto` mode, formatting settings are fixed by the preset;
/// only `width` and file discovery settings come from config.
///
/// The `apply_field` callback is called for each config field that should be
/// applied. It receives the field name and value.
pub fn merge_cli_with_config<F>(
    config: Option<&FlowmarkConfig>,
    is_auto: bool,
    explicit_flags: &[&str],
    mut apply_field: F,
) where
    F: FnMut(&str, &ConfigValue),
{
    let Some(config) = config else { return };

    let fields: Vec<(&str, Option<ConfigValue>)> = vec![
        ("width", config.width.map(ConfigValue::Usize)),
        ("semantic", config.semantic.map(ConfigValue::Bool)),
        ("cleanups", config.cleanups.map(ConfigValue::Bool)),
        ("smartquotes", config.smartquotes.map(ConfigValue::Bool)),
        ("ellipses", config.ellipses.map(ConfigValue::Bool)),
        ("list_spacing", config.list_spacing.clone().map(ConfigValue::String)),
        ("include", config.include.clone().map(ConfigValue::StringList)),
        ("extend_include", config.extend_include.clone().map(ConfigValue::StringList)),
        ("exclude", config.exclude.clone().map(ConfigValue::StringList)),
        ("extend_exclude", config.extend_exclude.clone().map(ConfigValue::StringList)),
        ("files_max_size", config.files_max_size.map(ConfigValue::U64)),
        ("respect_gitignore", config.respect_gitignore.map(ConfigValue::Bool)),
        ("force_exclude", config.force_exclude.map(ConfigValue::Bool)),
        ("incremental", config.incremental.map(ConfigValue::Bool)),
        ("incremental_cache_dir", config.incremental_cache_dir.clone().map(ConfigValue::String)),
    ];

    for (name, value) in fields {
        let Some(value) = value else { continue };

        if explicit_flags.contains(&name) {
            continue;
        }

        if is_auto && AUTO_LOCKED.contains(&name) {
            continue;
        }

        apply_field(name, &value);
    }
}

/// A typed config value for use in the merge callback.
#[derive(Debug, Clone)]
pub enum ConfigValue {
    /// Boolean value.
    Bool(bool),
    /// `usize` value.
    Usize(usize),
    /// `u64` value.
    U64(u64),
    /// String value.
    String(String),
    /// List of strings.
    StringList(Vec<String>),
}
