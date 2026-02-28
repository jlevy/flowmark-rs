//! Tests for the config loading module.
//!
//! Ported from Python: `test_config.py` (20 tests)

use std::fs;

use flowmark::config::{
    ConfigValue, FlowmarkConfig, find_config_file, load_config, merge_cli_with_config,
};

// --- Config file discovery (6) ---

#[test]
fn test_find_config_flowmark_toml() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::write(dir.path().join("flowmark.toml"), "width = 80\n").expect("write config");

    let result = find_config_file(dir.path());
    assert!(result.is_some());
    let path = result.expect("should find config");
    assert!(path.ends_with("flowmark.toml"));
}

#[test]
fn test_find_config_dot_flowmark_toml_takes_precedence() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::write(dir.path().join(".flowmark.toml"), "width = 72\n").expect("write dot config");
    fs::write(dir.path().join("flowmark.toml"), "width = 80\n").expect("write config");

    let result = find_config_file(dir.path());
    assert!(result.is_some());
    let path = result.expect("should find config");
    assert!(
        path.ends_with(".flowmark.toml"),
        ".flowmark.toml should take precedence over flowmark.toml"
    );
}

#[test]
fn test_find_config_pyproject_toml() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::write(dir.path().join("pyproject.toml"), "[tool.flowmark]\nwidth = 80\n")
        .expect("write pyproject.toml");

    let result = find_config_file(dir.path());
    assert!(result.is_some());
    let path = result.expect("should find config");
    assert!(path.ends_with("pyproject.toml"));
}

#[test]
fn test_find_config_pyproject_without_section_skipped() {
    let dir = tempfile::tempdir().expect("create temp dir");
    // pyproject.toml without [tool.flowmark] section
    fs::write(dir.path().join("pyproject.toml"), "[tool.black]\nline_length = 88\n")
        .expect("write pyproject.toml");

    let result = find_config_file(dir.path());
    assert!(result.is_none(), "pyproject.toml without [tool.flowmark] should be skipped");
}

#[test]
fn test_find_config_walks_up() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let sub = dir.path().join("sub");
    fs::create_dir_all(&sub).expect("create sub dir");
    // Config in parent dir
    fs::write(dir.path().join(".flowmark.toml"), "width = 72\n").expect("write config in parent");

    let result = find_config_file(&sub);
    assert!(result.is_some(), "should find config in parent directory");
    let path = result.expect("should find config");
    assert!(path.ends_with(".flowmark.toml"));
}

#[test]
fn test_find_config_none_when_missing() {
    let dir = tempfile::tempdir().expect("create temp dir");

    let result = find_config_file(dir.path());
    assert!(result.is_none(), "should return None when no config exists");
}

// --- Config loading (5) ---

#[test]
fn test_load_config_flowmark_toml() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("flowmark.toml");
    fs::write(
        &config_path,
        r"
width = 72
semantic = true
cleanups = false
",
    )
    .expect("write config");

    let config = load_config(&config_path);
    assert_eq!(config.width, Some(72));
    assert_eq!(config.semantic, Some(true));
    assert_eq!(config.cleanups, Some(false));
    // Unset fields should be None
    assert_eq!(config.smartquotes, None);
    assert_eq!(config.ellipses, None);
    assert_eq!(config.list_spacing, None);
}

#[test]
fn test_load_config_pyproject_toml() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("pyproject.toml");
    fs::write(
        &config_path,
        r"
[tool.flowmark]
width = 72
semantic = true
",
    )
    .expect("write config");

    let config = load_config(&config_path);
    assert_eq!(config.width, Some(72));
    assert_eq!(config.semantic, Some(true));
}

#[test]
fn test_load_config_kebab_case() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("flowmark.toml");
    fs::write(
        &config_path,
        r#"
list-spacing = "loose"
extend-include = ["*.mdx"]
extend-exclude = ["drafts/"]
files-max-size = 2097152
respect-gitignore = false
force-exclude = true
"#,
    )
    .expect("write config");

    let config = load_config(&config_path);
    assert_eq!(config.list_spacing, Some("loose".to_string()));
    assert_eq!(config.extend_include, Some(vec!["*.mdx".to_string()]));
    assert_eq!(config.extend_exclude, Some(vec!["drafts/".to_string()]));
    assert_eq!(config.files_max_size, Some(2_097_152));
    assert_eq!(config.respect_gitignore, Some(false));
    assert_eq!(config.force_exclude, Some(true));
}

#[test]
fn test_load_config_file_discovery_section() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("flowmark.toml");
    fs::write(
        &config_path,
        r#"
[formatting]
width = 72
semantic = true

[file-discovery]
extend-include = ["*.mdx"]
respect-gitignore = false
"#,
    )
    .expect("write config");

    let config = load_config(&config_path);
    assert_eq!(config.width, Some(72));
    assert_eq!(config.semantic, Some(true));
    assert_eq!(config.extend_include, Some(vec!["*.mdx".to_string()]));
    assert_eq!(config.respect_gitignore, Some(false));
}

#[test]
fn test_load_config_partial() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("flowmark.toml");
    fs::write(&config_path, "width = 72\n").expect("write config");

    let config = load_config(&config_path);
    assert_eq!(config.width, Some(72));
    // All other fields should be None
    assert_eq!(config.semantic, None);
    assert_eq!(config.cleanups, None);
    assert_eq!(config.smartquotes, None);
    assert_eq!(config.ellipses, None);
    assert_eq!(config.list_spacing, None);
    assert_eq!(config.include, None);
    assert_eq!(config.extend_include, None);
    assert_eq!(config.exclude, None);
    assert_eq!(config.extend_exclude, None);
    assert_eq!(config.files_max_size, None);
    assert_eq!(config.respect_gitignore, None);
    assert_eq!(config.force_exclude, None);
    assert_eq!(config.incremental, None);
    assert_eq!(config.incremental_cache_dir, None);
}

#[test]
fn test_load_config_performance_section_cache_settings() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("flowmark.toml");
    fs::write(
        &config_path,
        r#"
[performance]
cache = false
cache-dir = "/tmp/flowmark-cache-test"
"#,
    )
    .expect("write config");

    let config = load_config(&config_path);
    assert_eq!(config.incremental, Some(false));
    assert_eq!(config.incremental_cache_dir, Some("/tmp/flowmark-cache-test".to_string()));
}

// --- Config merge (7) ---

#[test]
fn test_merge_no_config() {
    let mut width: usize = 88;
    merge_cli_with_config(None, false, &[], |_name, _value| {
        width = 0; // Should not be called
    });
    assert_eq!(width, 88, "None config should not modify anything");
}

#[test]
fn test_merge_config_overrides_defaults() {
    let config = FlowmarkConfig { width: Some(72), ..FlowmarkConfig::default() };

    let mut applied_width = None;
    merge_cli_with_config(Some(&config), false, &[], |name, value| {
        if name == "width" {
            if let ConfigValue::Usize(v) = value {
                applied_width = Some(*v);
            }
        }
    });
    assert_eq!(applied_width, Some(72));
}

#[test]
fn test_merge_explicit_cli_overrides_config() {
    let config = FlowmarkConfig { width: Some(72), ..FlowmarkConfig::default() };

    let mut applied_width = None;
    merge_cli_with_config(Some(&config), false, &["width"], |name, value| {
        if name == "width" {
            if let ConfigValue::Usize(v) = value {
                applied_width = Some(*v);
            }
        }
    });
    assert!(applied_width.is_none(), "explicit CLI flag should override config");
}

#[test]
fn test_merge_auto_mode_overrides_formatting() {
    let config = FlowmarkConfig {
        semantic: Some(false),
        cleanups: Some(false),
        ..FlowmarkConfig::default()
    };

    let mut applied: Vec<String> = Vec::new();
    merge_cli_with_config(Some(&config), true, &[], |name, _value| {
        applied.push(name.to_string());
    });
    assert!(!applied.contains(&"semantic".to_string()), "--auto should lock semantic");
    assert!(!applied.contains(&"cleanups".to_string()), "--auto should lock cleanups");
}

#[test]
fn test_merge_auto_mode_width_from_config() {
    let config = FlowmarkConfig { width: Some(72), ..FlowmarkConfig::default() };

    let mut applied_width = None;
    merge_cli_with_config(Some(&config), true, &[], |name, value| {
        if name == "width" {
            if let ConfigValue::Usize(v) = value {
                applied_width = Some(*v);
            }
        }
    });
    assert_eq!(applied_width, Some(72), "width should come from config even in auto mode");
}

#[test]
fn test_merge_file_discovery_from_config() {
    let config = FlowmarkConfig {
        respect_gitignore: Some(false),
        force_exclude: Some(true),
        ..FlowmarkConfig::default()
    };

    let mut applied: Vec<(String, bool)> = Vec::new();
    merge_cli_with_config(Some(&config), false, &[], |name, value| {
        if let ConfigValue::Bool(v) = value {
            applied.push((name.to_string(), *v));
        }
    });
    assert!(applied.contains(&("respect_gitignore".to_string(), false)));
    assert!(applied.contains(&("force_exclude".to_string(), true)));
}

#[test]
fn test_merge_extend_include_from_config() {
    let config = FlowmarkConfig {
        extend_include: Some(vec!["*.mdx".to_string()]),
        ..FlowmarkConfig::default()
    };

    let mut applied_extend = None;
    merge_cli_with_config(Some(&config), false, &[], |name, value| {
        if name == "extend_include" {
            if let ConfigValue::StringList(v) = value {
                applied_extend = Some(v.clone());
            }
        }
    });
    assert_eq!(applied_extend, Some(vec!["*.mdx".to_string()]));
}

#[test]
fn test_merge_incremental_settings_from_config() {
    let config = FlowmarkConfig {
        incremental: Some(false),
        incremental_cache_dir: Some("/tmp/flowmark-cache-test".to_string()),
        ..FlowmarkConfig::default()
    };

    let mut applied_incremental = None;
    let mut applied_cache_dir = None;
    merge_cli_with_config(Some(&config), false, &[], |name, value| match (name, value) {
        ("incremental", ConfigValue::Bool(v)) => applied_incremental = Some(*v),
        ("incremental_cache_dir", ConfigValue::String(v)) => applied_cache_dir = Some(v.clone()),
        _ => {}
    });
    assert_eq!(applied_incremental, Some(false));
    assert_eq!(applied_cache_dir, Some("/tmp/flowmark-cache-test".to_string()));
}

// --- Error handling (2) ---

#[test]
fn test_load_config_malformed_toml() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("flowmark.toml");
    fs::write(&config_path, "this is not valid toml [[[").expect("write bad config");

    let config = load_config(&config_path);
    assert_eq!(config, FlowmarkConfig::default(), "malformed TOML should return empty config");
}

#[test]
fn test_parse_config_warns_unknown_keys() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("flowmark.toml");
    fs::write(
        &config_path,
        r#"
width = 72
unknown_key = "value"
"#,
    )
    .expect("write config");

    // This should not panic but should warn to stderr
    let config = load_config(&config_path);
    assert_eq!(config.width, Some(72));
    // Unknown key should not be set on config (just warning to stderr)
}
