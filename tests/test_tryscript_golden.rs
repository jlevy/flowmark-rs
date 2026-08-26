//! Run the shared upstream tryscript suite against the Cargo-built Rust CLI.
//!
//! Each script runs in a private copy of its suite so concurrent tests cannot mutate
//! the upstream submodule or race through shared fixture state. The one local script
//! covers the Rust-only incremental cache.
#![cfg(feature = "cli")]
#![cfg(not(windows))]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const TRYSCRIPT_VERSION: &str = "0.1.7";

#[derive(Clone, Copy)]
enum Suite {
    Shared,
    Local,
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn suite_root(root: &Path, suite: Suite) -> PathBuf {
    match suite {
        Suite::Shared => root.join("repos/flowmark/tests/tryscript"),
        Suite::Local => root.join("tests/tryscript"),
    }
}

fn copy_directory(source: &Path, destination: &Path) {
    fs::create_dir_all(destination)
        .unwrap_or_else(|error| panic!("cannot create {}: {error}", destination.display()));
    let mut entries: Vec<_> = fs::read_dir(source)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", source.display()))
        .map(|entry| entry.expect("cannot read tryscript directory entry"))
        .collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path)
            .unwrap_or_else(|error| panic!("cannot inspect {}: {error}", source_path.display()));
        assert!(
            !metadata.file_type().is_symlink(),
            "tryscript suite contains a symlink: {}",
            source_path.display()
        );
        if metadata.is_dir() {
            copy_directory(&source_path, &destination_path);
        } else {
            assert!(metadata.is_file(), "non-file tryscript entry: {}", source_path.display());
            fs::copy(&source_path, &destination_path).unwrap_or_else(|error| {
                panic!(
                    "cannot copy {} to {}: {error}",
                    source_path.display(),
                    destination_path.display()
                )
            });
        }
    }
}

fn assert_tryscript_version() {
    let output = Command::new("tryscript").arg("--version").output().unwrap_or_else(|error| {
        panic!(
            "tryscript {TRYSCRIPT_VERSION} is required; install it with \
                 `npm install -g tryscript@{TRYSCRIPT_VERSION}`: {error}"
        )
    });
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    assert!(output.status.success(), "cannot query tryscript version: {stderr}");
    assert_eq!(
        stdout, TRYSCRIPT_VERSION,
        "tryscript version mismatch; install tryscript@{TRYSCRIPT_VERSION}"
    );
}

fn run_tryscript(suite: Suite, file: &str) {
    assert_tryscript_version();
    let root = project_root();
    let source = suite_root(&root, suite);
    assert!(source.is_dir(), "tryscript suite not found: {}", source.display());

    let workspace = TempDir::with_prefix("flowmark-tryscript-")
        .expect("cannot create isolated tryscript workspace");
    let isolated_suite = workspace.path().join("tests/tryscript");
    copy_directory(&source, &isolated_suite);
    let script = isolated_suite.join(file);
    assert!(script.is_file(), "tryscript file not found: {}", script.display());

    let binary = PathBuf::from(env!("CARGO_BIN_EXE_flowmark"));
    let binary_directory = binary.parent().expect("Cargo binary must have a parent directory");
    let output = Command::new("tryscript")
        .args(["run", script.to_str().expect("tryscript path must be UTF-8")])
        .env("TRYSCRIPT_GIT_ROOT", workspace.path())
        .env("FLOWMARK_BIN_DIR", binary_directory)
        .current_dir(workspace.path())
        .output()
        .expect("failed to execute tryscript");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "tryscript test failed: {file}\n\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
}

macro_rules! shared_tryscript_test {
    ($test:ident, $file:literal) => {
        #[test]
        fn $test() {
            run_tryscript(Suite::Shared, $file);
        }
    };
}

shared_tryscript_test!(tryscript_auto_mode, "auto-mode.tryscript.md");
shared_tryscript_test!(tryscript_cli_golden, "cli-golden.tryscript.md");
shared_tryscript_test!(tryscript_config_interaction, "config-interaction.tryscript.md");
shared_tryscript_test!(tryscript_errors_version, "errors-version.tryscript.md");
shared_tryscript_test!(tryscript_file_discovery, "file-discovery.tryscript.md");
shared_tryscript_test!(tryscript_file_ops, "file-ops.tryscript.md");
shared_tryscript_test!(tryscript_formatting, "formatting.tryscript.md");
shared_tryscript_test!(tryscript_help, "help.tryscript.md");
shared_tryscript_test!(tryscript_list_spacing, "list-spacing.tryscript.md");
shared_tryscript_test!(tryscript_stdin, "stdin.tryscript.md");
shared_tryscript_test!(tryscript_typography_tests, "typography-tests.tryscript.md");
shared_tryscript_test!(tryscript_verbose_docs, "verbose-docs.tryscript.md");

#[test]
fn tryscript_cache_behavior() {
    run_tryscript(Suite::Local, "cache-behavior.tryscript.md");
}
