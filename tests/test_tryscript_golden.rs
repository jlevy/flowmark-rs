//! Integration tests that run tryscript golden tests as part of `cargo test`.
//!
//! Each tryscript `.tryscript.md` file gets its own `#[test]` function so failures
//! are reported per-file. Requires `npx` (Node.js) to be installed — tests will
//! fail if it is not available.
//!
//! All tryscript files are tested as part of the normal test suite.
//! Skipped on Windows: tryscript tests use bash and Unix paths.
#![cfg(feature = "cli")]
#![cfg(not(windows))]
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::Command;

/// Locate the project root via `CARGO_MANIFEST_DIR`.
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Run a single tryscript file and assert it passes.
/// Prefers globally installed `tryscript` over `npx` to avoid npm install
/// race conditions in parallel test execution.
fn run_tryscript(file: &str) {
    let root = project_root();
    let script = root.join("tests/tryscript").join(file);
    assert!(script.exists(), "tryscript file not found: {}", script.display());

    let script_path = script.to_str().expect("utf-8 path");

    // Prefer globally installed tryscript (avoids npx install races in parallel tests).
    let has_global =
        Command::new("tryscript").arg("--version").output().is_ok_and(|o| o.status.success());

    let output = if has_global {
        Command::new("tryscript")
            .args(["run", script_path])
            .env("TRYSCRIPT_GIT_ROOT", &root)
            .current_dir(&root)
            .output()
            .expect("failed to execute tryscript")
    } else {
        Command::new("npx")
            .args(["tryscript@latest", "run", script_path])
            .env("TRYSCRIPT_GIT_ROOT", &root)
            .current_dir(&root)
            .output()
            .expect("failed to execute npx tryscript — is Node.js installed?")
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "tryscript test failed: {file}\n\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
}

// --- Parity-clean tryscript tests (must pass) ---

#[test]
fn tryscript_auto_mode() {
    run_tryscript("auto-mode.tryscript.md");
}

#[test]
fn tryscript_config_interaction() {
    run_tryscript("config-interaction.tryscript.md");
}

#[test]
fn tryscript_errors_version() {
    run_tryscript("errors-version.tryscript.md");
}

#[test]
fn tryscript_file_discovery() {
    run_tryscript("file-discovery.tryscript.md");
}

#[test]
fn tryscript_file_ops() {
    run_tryscript("file-ops.tryscript.md");
}

#[test]
fn tryscript_list_spacing() {
    run_tryscript("list-spacing.tryscript.md");
}

#[test]
fn tryscript_stdin() {
    run_tryscript("stdin.tryscript.md");
}

#[test]
fn tryscript_verbose_docs() {
    run_tryscript("verbose-docs.tryscript.md");
}

#[test]
fn tryscript_formatting() {
    run_tryscript("formatting.tryscript.md");
}

#[test]
fn tryscript_typography_tests() {
    run_tryscript("typography-tests.tryscript.md");
}

#[test]
fn tryscript_help() {
    run_tryscript("help.tryscript.md");
}

#[test]
fn tryscript_cache_behavior() {
    run_tryscript("cache-behavior.tryscript.md");
}
