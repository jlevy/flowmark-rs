//! CLI integration tests for incremental cache behavior.
#![cfg(feature = "cli")]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn flowmark_bin() -> PathBuf {
    let mut path = std::env::current_exe().expect("current exe");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("flowmark");
    path
}

fn cache_manifest_count(cache_root: &std::path::Path) -> usize {
    let incremental_dir = cache_root.join("incremental");
    if !incremental_dir.exists() {
        return 0;
    }
    fs::read_dir(incremental_dir)
        .expect("read incremental dir")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .count()
}

fn toml_string_literal(path: &std::path::Path) -> String {
    format!("{:?}", path.to_string_lossy())
}

#[test]
fn test_incremental_cache_writes_manifest_when_enabled() {
    let project_dir = tempfile::tempdir().expect("create project dir");
    let cache_dir = tempfile::tempdir().expect("create cache dir");
    fs::write(
        project_dir.path().join("doc.md"),
        "# Title\n\nA short paragraph that will stay unchanged.\n",
    )
    .expect("write doc.md");

    let output = Command::new(flowmark_bin())
        .current_dir(project_dir.path())
        .args([
            "--auto",
            "--incremental-cache-dir",
            cache_dir.path().to_str().expect("cache path to str"),
            "doc.md",
        ])
        .output()
        .expect("run flowmark");

    assert!(
        output.status.success(),
        "flowmark should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        cache_manifest_count(cache_dir.path()) > 0,
        "expected incremental manifest file when incremental cache is enabled"
    );
}

#[test]
fn test_no_incremental_disables_cache_manifest_creation() {
    let project_dir = tempfile::tempdir().expect("create project dir");
    let cache_dir = tempfile::tempdir().expect("create cache dir");
    fs::write(
        project_dir.path().join("doc.md"),
        "# Title\n\nA short paragraph that will stay unchanged.\n",
    )
    .expect("write doc.md");

    let output = Command::new(flowmark_bin())
        .current_dir(project_dir.path())
        .args([
            "--auto",
            "--no-incremental",
            "--incremental-cache-dir",
            cache_dir.path().to_str().expect("cache path to str"),
            "doc.md",
        ])
        .output()
        .expect("run flowmark");

    assert!(
        output.status.success(),
        "flowmark should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        cache_manifest_count(cache_dir.path()),
        0,
        "no incremental cache file should be created when --no-incremental is set"
    );
}

#[test]
fn test_config_incremental_false_disables_cache_manifest_creation() {
    let project_dir = tempfile::tempdir().expect("create project dir");
    let cache_dir = tempfile::tempdir().expect("create cache dir");
    fs::write(
        project_dir.path().join("flowmark.toml"),
        format!(
            "incremental = false\nincremental-cache-dir = {}\n",
            toml_string_literal(cache_dir.path())
        ),
    )
    .expect("write flowmark.toml");
    fs::write(
        project_dir.path().join("doc.md"),
        "# Title\n\nA short paragraph that will stay unchanged.\n",
    )
    .expect("write doc.md");

    let output = Command::new(flowmark_bin())
        .current_dir(project_dir.path())
        .args(["--auto", "doc.md"])
        .output()
        .expect("run flowmark");

    assert!(
        output.status.success(),
        "flowmark should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        cache_manifest_count(cache_dir.path()),
        0,
        "no incremental cache file should be created when config sets incremental = false"
    );
}

#[test]
fn test_config_incremental_cache_dir_is_applied() {
    let project_dir = tempfile::tempdir().expect("create project dir");
    let cache_dir = tempfile::tempdir().expect("create cache dir");
    fs::write(
        project_dir.path().join("flowmark.toml"),
        format!("incremental-cache-dir = {}\n", toml_string_literal(cache_dir.path())),
    )
    .expect("write flowmark.toml");
    fs::write(
        project_dir.path().join("doc.md"),
        "# Title\n\nA short paragraph that will stay unchanged.\n",
    )
    .expect("write doc.md");

    let output = Command::new(flowmark_bin())
        .current_dir(project_dir.path())
        .args(["--auto", "doc.md"])
        .output()
        .expect("run flowmark");

    assert!(
        output.status.success(),
        "flowmark should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        cache_manifest_count(cache_dir.path()) > 0,
        "expected incremental manifest in cache dir configured by flowmark.toml"
    );
}
