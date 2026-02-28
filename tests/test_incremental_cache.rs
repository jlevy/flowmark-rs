//! CLI integration tests for incremental cache behavior.
#![cfg(feature = "cli")]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

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

fn run_flowmark(current_dir: &std::path::Path, args: &[&str]) -> Output {
    Command::new(flowmark_bin()).current_dir(current_dir).args(args).output().expect("run flowmark")
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

    let output = run_flowmark(
        project_dir.path(),
        &["--auto", "--cache-dir", cache_dir.path().to_str().expect("cache path to str"), "doc.md"],
    );

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

    let output = run_flowmark(
        project_dir.path(),
        &[
            "--auto",
            "--no-cache",
            "--cache-dir",
            cache_dir.path().to_str().expect("cache path to str"),
            "doc.md",
        ],
    );

    assert!(
        output.status.success(),
        "flowmark should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        cache_manifest_count(cache_dir.path()),
        0,
        "no incremental cache file should be created when --no-cache is set"
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

    let output = run_flowmark(project_dir.path(), &["--auto", "doc.md"]);

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

    let output = run_flowmark(project_dir.path(), &["--auto", "doc.md"]);

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

#[test]
fn test_show_cache_reports_usage_without_files() {
    let project_dir = tempfile::tempdir().expect("create project dir");
    let cache_parent = tempfile::tempdir().expect("create cache parent dir");
    let cache_root = cache_parent.path().join("cache-root");
    fs::write(
        project_dir.path().join("doc.md"),
        "# Title\n\nA short paragraph that will stay unchanged.\n",
    )
    .expect("write doc.md");

    let seed = run_flowmark(
        project_dir.path(),
        &["--auto", "--cache-dir", cache_root.to_str().expect("cache path to str"), "doc.md"],
    );
    assert!(
        seed.status.success(),
        "seed run should succeed, stderr: {}",
        String::from_utf8_lossy(&seed.stderr)
    );

    let output = run_flowmark(
        project_dir.path(),
        &["--show-cache", "--cache-dir", cache_root.to_str().expect("cache path to str")],
    );
    assert!(
        output.status.success(),
        "--show-cache should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&format!("Cache directory: {}", cache_root.display())),
        "show-cache output should include resolved cache directory, got: {stdout}"
    );
    assert!(stdout.contains("Cache files: "), "show-cache output should include file count");
    assert!(stdout.contains("Cache size: "), "show-cache output should include total size");
}

#[test]
fn test_clear_cache_deletes_cache_without_confirmation() {
    let project_dir = tempfile::tempdir().expect("create project dir");
    let cache_parent = tempfile::tempdir().expect("create cache parent dir");
    let cache_root = cache_parent.path().join("cache-root");
    fs::write(
        project_dir.path().join("doc.md"),
        "# Title\n\nA short paragraph that will stay unchanged.\n",
    )
    .expect("write doc.md");

    let seed = run_flowmark(
        project_dir.path(),
        &["--auto", "--cache-dir", cache_root.to_str().expect("cache path to str"), "doc.md"],
    );
    assert!(
        seed.status.success(),
        "seed run should succeed, stderr: {}",
        String::from_utf8_lossy(&seed.stderr)
    );
    assert!(cache_root.exists(), "seed run should create cache root");

    let clear = run_flowmark(
        project_dir.path(),
        &["--clear-cache", "--cache-dir", cache_root.to_str().expect("cache path to str")],
    );
    assert!(
        clear.status.success(),
        "--clear-cache should succeed, stderr: {}",
        String::from_utf8_lossy(&clear.stderr)
    );
    assert!(!cache_root.exists(), "clear-cache should remove cache root");

    let clear_again = run_flowmark(
        project_dir.path(),
        &["--clear-cache", "--cache-dir", cache_root.to_str().expect("cache path to str")],
    );
    assert!(clear_again.status.success(), "second --clear-cache should be idempotent");
    let stdout = String::from_utf8_lossy(&clear_again.stdout);
    assert!(
        stdout.contains("Cache already empty."),
        "second clear should report already empty, got: {stdout}"
    );
}
