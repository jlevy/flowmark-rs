//! Tier 2: Cross-binary parity tests — invoke Python flowmark and Rust flowmark
//! on the same input and assert byte-for-byte identity.
//!
//! These tests are opt-in: set `FLOWMARK_PARITY_PYTHON=1` to enable.
//! Requires: uvx (uv tool runner) with access to the pinned Python flowmark version.
//!
//! Also validates that committed golden files still match Python output (drift detection).
//!
//! The Python version is read from `PARITY_VERSION` (set by build.rs from Cargo.toml).
#![allow(clippy::unwrap_used)]
#![cfg(feature = "cli")]

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Python flowmark uvx package spec, e.g. `"flowmark@0.6.5"`.
/// Version is read at compile time from `[package.metadata.parity]` in Cargo.toml.
fn flowmark_uvx_spec() -> String {
    format!("flowmark@{}", env!("PARITY_VERSION"))
}

fn parity_python_enabled() -> bool {
    std::env::var("FLOWMARK_PARITY_PYTHON").is_ok()
}

fn python_flowmark_available() -> bool {
    let spec = flowmark_uvx_spec();
    Command::new("uvx")
        .args([spec.as_str(), "--version"])
        .output()
        .is_ok_and(|o| o.status.success())
}

fn run_python_stdin(args: &[&str], input: &str) -> String {
    let mut child = Command::new("uvx")
        .arg(flowmark_uvx_spec())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn uvx flowmark");
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "Python flowmark failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn rust_binary() -> String {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.join("target/debug/flowmark").to_string_lossy().to_string()
}

fn run_rust_stdin(args: &[&str], input: &str) -> String {
    let mut child = Command::new(rust_binary())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn Rust flowmark");
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "Rust flowmark failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

struct CrossBinaryMode {
    name: &'static str,
    args: &'static [&'static str],
    expected_file: &'static str,
}

const MODES: &[CrossBinaryMode] = &[
    CrossBinaryMode {
        name: "default",
        args: &["-w", "88", "-"],
        expected_file: "corner-cases.expected.default.md",
    },
    CrossBinaryMode {
        name: "auto",
        args: &["-w", "88", "--semantic", "--cleanups", "--smartquotes", "--ellipses", "-"],
        expected_file: "corner-cases.expected.auto.md",
    },
    CrossBinaryMode {
        name: "tight",
        args: &["-w", "88", "--semantic", "--list-spacing", "tight", "-"],
        expected_file: "corner-cases.expected.tight.md",
    },
    CrossBinaryMode {
        name: "loose",
        args: &["-w", "88", "--semantic", "--list-spacing", "loose", "-"],
        expected_file: "corner-cases.expected.loose.md",
    },
    CrossBinaryMode {
        name: "plaintext",
        args: &["-w", "88", "--plaintext", "-"],
        expected_file: "corner-cases.expected.plaintext.md",
    },
];

#[test]
fn test_cross_binary_corner_cases() {
    if !parity_python_enabled() {
        eprintln!("SKIP: Set FLOWMARK_PARITY_PYTHON=1 to enable cross-binary tests");
        return;
    }
    if !python_flowmark_available() {
        eprintln!("SKIP: uvx {} not available", flowmark_uvx_spec());
        return;
    }

    let parity_dir = Path::new("tests/parity");
    let input = std::fs::read_to_string(parity_dir.join("corner-cases.md"))
        .expect("Failed to read corner-cases.md");

    let mut failures = Vec::new();

    for mode in MODES {
        let py_out = run_python_stdin(mode.args, &input);
        let rs_out = run_rust_stdin(mode.args, &input);

        if py_out != rs_out {
            failures.push(format!(
                "{}: Python and Rust outputs differ ({} bytes vs {} bytes)",
                mode.name,
                py_out.len(),
                rs_out.len()
            ));

            // Write both outputs for debugging
            let py_file = format!("corner-cases.python.{}.md", mode.name);
            let rs_file = format!("corner-cases.rust.{}.md", mode.name);
            let _ = std::fs::write(parity_dir.join(&py_file), &py_out);
            let _ = std::fs::write(parity_dir.join(&rs_file), &rs_out);
        }
    }

    assert!(failures.is_empty(), "Cross-binary parity failures:\n{}", failures.join("\n"));
}

#[test]
fn test_golden_files_match_python() {
    if !parity_python_enabled() {
        eprintln!("SKIP: Set FLOWMARK_PARITY_PYTHON=1 to enable golden file verification");
        return;
    }
    if !python_flowmark_available() {
        eprintln!("SKIP: uvx {} not available", flowmark_uvx_spec());
        return;
    }

    let parity_dir = Path::new("tests/parity");
    let input = std::fs::read_to_string(parity_dir.join("corner-cases.md"))
        .expect("Failed to read corner-cases.md");

    let mut failures = Vec::new();

    for mode in MODES {
        let expected_path = parity_dir.join(mode.expected_file);
        if !expected_path.exists() {
            failures.push(format!("{}: expected file not found", mode.name));
            continue;
        }

        let expected =
            std::fs::read_to_string(&expected_path).expect("Failed to read expected file");
        let py_out = run_python_stdin(mode.args, &input);

        if py_out != expected {
            failures.push(format!(
                "{}: committed golden file has drifted from Python output",
                mode.name
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Golden file drift detected (regenerate with scripts/generate-parity-golden.sh):\n{}",
        failures.join("\n")
    );
}
