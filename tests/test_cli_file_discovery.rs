//! CLI integration tests for file discovery.
//!
//! Ported from Python: `test_cli_file_discovery.py` (19 tests)
//!
//! These tests require the `cli` feature (they run the flowmark binary).
#![cfg(feature = "cli")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Get the path to the built binary.
fn flowmark_bin() -> PathBuf {
    // cargo test builds the binary in target/debug/
    let mut path = std::env::current_exe().expect("current exe");
    // Go up from deps/ to debug/
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("flowmark");
    path
}

/// Create a minimal project directory tree for testing.
fn make_tree(root: &Path) {
    fs::write(root.join("README.md"), "# Root\n").expect("write README.md");

    let docs = root.join("docs");
    fs::create_dir_all(&docs).expect("create docs dir");
    fs::write(docs.join("guide.md"), "# Guide\n").expect("write guide.md");
    fs::write(docs.join("api.md"), "# API\n").expect("write api.md");

    fs::write(root.join("code.py"), "print('hello')\n").expect("write code.py");

    let nm = root.join("node_modules").join("pkg");
    fs::create_dir_all(&nm).expect("create node_modules dir");
    fs::write(nm.join("README.md"), "# Should be excluded\n").expect("write nm README");

    let venv = root.join(".venv").join("lib");
    fs::create_dir_all(&venv).expect("create .venv dir");
    fs::write(venv.join("README.md"), "# Should be excluded\n").expect("write venv README");
}

// --- File discovery via --list-files (7 tests) ---

#[test]
fn test_list_files_directory() {
    let dir = tempfile::tempdir().expect("create temp dir");
    make_tree(dir.path());

    let output = Command::new(flowmark_bin())
        .args(["--list-files", "."])
        .current_dir(dir.path())
        .output()
        .expect("run flowmark");

    assert!(output.status.success(), "exit code should be 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut names: Vec<String> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| Path::new(line).file_name().and_then(|n| n.to_str()).map(String::from))
        .collect();
    names.sort();
    assert_eq!(names, vec!["README.md", "api.md", "guide.md"]);
}

#[test]
fn test_list_files_skips_excluded_dirs() {
    let dir = tempfile::tempdir().expect("create temp dir");
    make_tree(dir.path());

    let output = Command::new(flowmark_bin())
        .args(["--list-files", "."])
        .current_dir(dir.path())
        .output()
        .expect("run flowmark");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("node_modules"), "node_modules should be excluded");
    assert!(!stdout.contains(".venv"), ".venv should be excluded");
}

#[test]
fn test_list_files_extend_include() {
    let dir = tempfile::tempdir().expect("create temp dir");
    make_tree(dir.path());
    fs::write(dir.path().join("page.mdx"), "# MDX page\n").expect("write page.mdx");

    let output = Command::new(flowmark_bin())
        .args(["--list-files", "--extend-include", "*.mdx", "."])
        .current_dir(dir.path())
        .output()
        .expect("run flowmark");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("page.mdx"), "should find .mdx files with --extend-include");
}

#[test]
fn test_list_files_extend_exclude() {
    let dir = tempfile::tempdir().expect("create temp dir");
    make_tree(dir.path());
    let drafts = dir.path().join("drafts");
    fs::create_dir_all(&drafts).expect("create drafts dir");
    fs::write(drafts.join("wip.md"), "# WIP\n").expect("write wip.md");

    let output = Command::new(flowmark_bin())
        .args(["--list-files", "--extend-exclude", "drafts/", "."])
        .current_dir(dir.path())
        .output()
        .expect("run flowmark");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("drafts"), "drafts/ should be excluded");
    assert!(stdout.contains("README.md"), "README.md should still be included");
}

#[test]
fn test_list_files_no_respect_gitignore() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::write(dir.path().join("keep.md"), "# Keep\n").expect("write keep.md");
    fs::write(dir.path().join(".gitignore"), "ignored/\n").expect("write .gitignore");
    let ignored = dir.path().join("ignored");
    fs::create_dir_all(&ignored).expect("create ignored dir");
    fs::write(ignored.join("found.md"), "# Found\n").expect("write found.md");

    let output = Command::new(flowmark_bin())
        .args(["--list-files", "--no-respect-gitignore", "."])
        .current_dir(dir.path())
        .output()
        .expect("run flowmark");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("found.md"), "--no-respect-gitignore should include gitignored files");
}

#[test]
fn test_list_files_force_exclude() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let nm = dir.path().join("node_modules");
    fs::create_dir_all(&nm).expect("create node_modules dir");
    fs::write(nm.join("README.md"), "# Excluded\n").expect("write README.md");

    let explicit_path = nm.join("README.md");
    let output = Command::new(flowmark_bin())
        .args(["--list-files", "--force-exclude", explicit_path.to_str().expect("path to str")])
        .current_dir(dir.path())
        .output()
        .expect("run flowmark");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().is_empty(),
        "--force-exclude should filter explicit node_modules/README.md"
    );
}

#[test]
fn test_list_files_max_size() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::write(dir.path().join("small.md"), "# Small\n").expect("write small.md");
    fs::write(dir.path().join("large.md"), "x".repeat(2_000_000)).expect("write large.md");

    let output = Command::new(flowmark_bin())
        .args(["--list-files", "--files-max-size", "100", "."])
        .current_dir(dir.path())
        .output()
        .expect("run flowmark");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("small.md"), "small file should be included");
    assert!(!stdout.contains("large.md"), "large file should be excluded by --files-max-size");
}

// --- Error cases (4 tests) ---

#[test]
fn test_auto_no_args_errors() {
    let output = Command::new(flowmark_bin()).args(["--auto"]).output().expect("run flowmark");

    assert!(!output.status.success(), "should exit with error");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--auto requires at least one file or directory argument"),
        "stderr should contain expected error message, got: {stderr}"
    );
}

#[test]
fn test_list_files_no_args_errors() {
    let output =
        Command::new(flowmark_bin()).args(["--list-files"]).output().expect("run flowmark");

    assert!(!output.status.success(), "should exit with error");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--list-files requires at least one file or directory argument"),
        "stderr should contain expected error message, got: {stderr}"
    );
}

#[test]
fn test_no_args_errors() {
    let output = Command::new(flowmark_bin()).output().expect("run flowmark");

    assert!(!output.status.success(), "should exit with error");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No input specified"), "should contain 'No input specified'");
    assert!(stderr.contains("'-' for stdin"), "should mention stdin");
    assert!(stderr.contains("'.' for current directory"), "should mention current directory");
    assert!(stderr.contains("--help"), "should mention --help");
}

#[test]
fn test_auto_list_files_no_args_errors() {
    let output = Command::new(flowmark_bin())
        .args(["--auto", "--list-files"])
        .output()
        .expect("run flowmark");

    assert!(!output.status.success(), "should exit with error");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--auto requires at least one file or directory argument"),
        "--auto message should take priority over --list-files message"
    );
}

// --- Formatting integration (4 tests) ---

#[test]
fn test_auto_with_dot_formats_cwd() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::write(dir.path().join("test.md"), "# Test\n\nSome text here.\n").expect("write test.md");

    let output = Command::new(flowmark_bin())
        .args(["--auto", "."])
        .current_dir(dir.path())
        .output()
        .expect("run flowmark");

    assert!(output.status.success(), "exit code should be 0");
    let content = fs::read_to_string(dir.path().join("test.md")).expect("read test.md");
    assert!(content.contains("# Test"), "file should still contain header");
}

#[test]
fn test_explicit_file_still_works() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file = dir.path().join("test.md");
    fs::write(&file, "# Hello World\n").expect("write test.md");

    let output = Command::new(flowmark_bin())
        .arg(file.to_str().expect("path to str"))
        .output()
        .expect("run flowmark");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# Hello World"), "stdout should contain file content");
}

#[test]
fn test_stdin_still_works() {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new(flowmark_bin())
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn flowmark");

    child.stdin.take().expect("get stdin").write_all(b"# From stdin\n").expect("write to stdin");

    let output = child.wait_with_output().expect("wait for flowmark");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# From stdin"), "stdout should contain stdin content");
}

#[test]
fn test_auto_with_explicit_file() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file = dir.path().join("README.md");
    fs::write(&file, "# Test\n\nSome text.\n").expect("write README.md");

    let output = Command::new(flowmark_bin())
        .args(["--auto", file.to_str().expect("path to str")])
        .output()
        .expect("run flowmark");

    assert!(output.status.success(), "exit code should be 0");
    let content = fs::read_to_string(&file).expect("read README.md");
    assert!(content.contains("# Test"), "file should still contain header after --auto");
}

// --- Tool ignore (1 test) ---

#[test]
fn test_flowmarkignore() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::write(dir.path().join("keep.md"), "# Keep\n").expect("write keep.md");
    let skip = dir.path().join("skip");
    fs::create_dir_all(&skip).expect("create skip dir");
    fs::write(skip.join("nope.md"), "# Nope\n").expect("write nope.md");
    fs::write(dir.path().join(".flowmarkignore"), "skip/\n").expect("write .flowmarkignore");

    let output = Command::new(flowmark_bin())
        .args(["--list-files", "."])
        .current_dir(dir.path())
        .output()
        .expect("run flowmark");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("keep.md"), "keep.md should be in output");
    assert!(!stdout.contains("skip"), "skip/ should be excluded by .flowmarkignore");
}

// --- Permission preservation (C3 regression) ---

#[cfg(unix)]
#[test]
fn test_auto_preserves_permissions_644() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().expect("create temp dir");
    let file = dir.path().join("test.md");
    fs::write(&file, "# Hello\n\nSome text.\n").expect("write test.md");
    fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).expect("set permissions");

    let output = Command::new(flowmark_bin())
        .args(["--auto", file.to_str().expect("path to str")])
        .output()
        .expect("run flowmark");

    assert!(output.status.success(), "exit code should be 0");
    let mode = file.metadata().expect("file metadata").permissions().mode() & 0o777;
    assert_eq!(mode, 0o644, "permissions should be preserved as 0o644, got 0o{mode:o}");
}

#[cfg(unix)]
#[test]
fn test_auto_preserves_permissions_755() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().expect("create temp dir");
    let file = dir.path().join("test.md");
    fs::write(&file, "# Hello\n\nSome text.\n").expect("write test.md");
    fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).expect("set permissions");

    let output = Command::new(flowmark_bin())
        .args(["--auto", file.to_str().expect("path to str")])
        .output()
        .expect("run flowmark");

    assert!(output.status.success(), "exit code should be 0");
    let mode = file.metadata().expect("file metadata").permissions().mode() & 0o777;
    assert_eq!(mode, 0o755, "permissions should be preserved as 0o755, got 0o{mode:o}");
}

// --- Edge cases (3 tests) ---

#[test]
fn test_list_files_stdin_does_not_crash() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::write(dir.path().join("README.md"), "# Root\n").expect("write README.md");

    let output = Command::new(flowmark_bin())
        .args(["--list-files", "-", dir.path().to_str().expect("path to str")])
        .current_dir(dir.path())
        .output()
        .expect("run flowmark");

    assert!(output.status.success(), "should not crash with stdin + directory");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("README.md"), "should list files from directory");
}

#[test]
fn test_stdin_explicit_dash() {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new(flowmark_bin())
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn flowmark");

    child.stdin.take().expect("get stdin").write_all(b"# Via dash\n").expect("write to stdin");

    let output = child.wait_with_output().expect("wait for flowmark");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# Via dash"), "explicit '-' should read from stdin");
}

#[test]
fn test_explicit_flag_detection_with_default_value() {
    // Passing --width 88 (the default value) should still be detected as explicit.
    // This matters for config merge precedence: explicit CLI flags override config.
    // We test this by creating a config file with width=72, then passing --width 88.
    // If explicit flag detection works, the config width should NOT override CLI.
    let dir = tempfile::tempdir().expect("create temp dir");
    let file = dir.path().join("test.md");

    // Write a long line that wraps differently at width 72 vs 88
    let long_text = "# Test\n\nThis is a sentence that is long enough to demonstrate \
                     wrapping differences between width 72 and width 88 characters in \
                     the output file.";
    fs::write(&file, long_text).expect("write test.md");

    // Create config file with width=72
    fs::write(dir.path().join("flowmark.toml"), "width = 72\n").expect("write config");

    // Run with explicit --width 88 (the default) — should override config
    let output_with_explicit = Command::new(flowmark_bin())
        .args(["--width", "88", file.to_str().expect("path to str")])
        .current_dir(dir.path())
        .output()
        .expect("run flowmark with explicit width");

    // Run without --width — config should set width to 72
    let output_with_config = Command::new(flowmark_bin())
        .arg(file.to_str().expect("path to str"))
        .current_dir(dir.path())
        .output()
        .expect("run flowmark without width");

    assert!(output_with_explicit.status.success());
    assert!(output_with_config.status.success());

    let explicit_out = String::from_utf8_lossy(&output_with_explicit.stdout);
    let config_out = String::from_utf8_lossy(&output_with_config.stdout);

    // The outputs should differ because explicit --width 88 overrides config width=72
    assert_ne!(
        explicit_out, config_out,
        "explicit --width 88 should override config width=72, producing different output"
    );
}
