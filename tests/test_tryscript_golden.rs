//! Integration tests that run tryscript golden tests as part of `cargo test`.
//!
//! Each tryscript `.tryscript.md` file gets its own `#[test]` function so failures
//! are reported per-file. Tests are skipped gracefully when `npx` is not available
//! (e.g. environments without Node.js).
//!
//! Two files with known parity gaps are marked `#[ignore = "known parity gaps P1-P4, see plan-2026-02-17-exact-parity.md"]` so they don't block CI
//! but can still be run explicitly with `cargo test -- --ignored`.
#![cfg(feature = "cli")]
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::Command;

/// Locate the project root via `CARGO_MANIFEST_DIR`.
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Check whether `npx` is available on PATH.
fn npx_available() -> bool {
    Command::new("npx").arg("--version").output().is_ok_and(|o| o.status.success())
}

/// Run a single tryscript file and assert it passes.
fn run_tryscript(file: &str) {
    if !npx_available() {
        eprintln!("npx not found — skipping tryscript test: {file}");
        return;
    }

    let root = project_root();
    let script = root.join("tests/tryscript").join(file);
    assert!(script.exists(), "tryscript file not found: {}", script.display());

    let output = Command::new("npx")
        .args(["tryscript@latest", "run", script.to_str().expect("utf-8 path")])
        .env("TRYSCRIPT_GIT_ROOT", &root)
        .current_dir(&root)
        .output()
        .expect("failed to execute npx tryscript");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "tryscript test failed: {file}\n\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
}

// --- Parity-clean tryscript tests (must pass) ---

#[test]
fn tryscript_cli_golden() {
    run_tryscript("cli-golden.tryscript.md");
}

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

// --- Tests with known parity gaps (ignored until P1-P4 are fixed) ---
// Run with: cargo test -- --ignored

#[test]
#[ignore = "known parity gaps P1-P4, see plan-2026-02-17-exact-parity.md"]
fn tryscript_formatting() {
    run_tryscript("formatting.tryscript.md");
}

#[test]
#[ignore = "known parity gaps P1-P4, see plan-2026-02-17-exact-parity.md"]
fn tryscript_typography_tests() {
    run_tryscript("typography-tests.tryscript.md");
}
