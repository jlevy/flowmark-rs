//! Tier 3: Red/green characterization tests — prove bugs existed in old v0.2.0 binary
//! and are fixed in the current version.
//!
//! RED: old binary output does NOT match expected (bug existed)
//! GREEN: new binary output MATCHES expected (bug is fixed)
//!
//! Set `FLOWMARK_OLD_BINARY` to the path of the old v0.2.0 binary to enable.
//! Default: looks for ~/.cargo/bin/flowmark-rs (if it reports v0.2.0).
#![allow(clippy::unwrap_used)]
#![cfg(feature = "cli")]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn old_binary() -> Option<PathBuf> {
    // Check env var first
    if let Ok(path) = std::env::var("FLOWMARK_OLD_BINARY") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }
    // Default: ~/.cargo/bin/flowmark-rs
    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home).join(".cargo/bin/flowmark-rs");
        if p.exists() {
            // Verify it's v0.2.0
            if let Ok(output) = Command::new(&p).arg("--version").output() {
                let version = String::from_utf8_lossy(&output.stdout);
                if version.contains("0.2.0") {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn new_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/flowmark")
}

fn run_binary_stdin(bin: &Path, args: &[&str], input: &str) -> Option<String> {
    let mut child = Command::new(bin)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    child.stdin.take().unwrap().write_all(input.as_bytes()).ok()?;
    let output = child.wait_with_output().ok()?;
    if output.status.success() { Some(String::from_utf8(output.stdout).ok()?) } else { None }
}

/// A red/green test case: old binary should produce different output (RED),
/// new binary should match expected (GREEN).
struct RedGreenCase {
    name: &'static str,
    mode_args: &'static [&'static str],
    expected_file: &'static str,
}

const CASES: &[RedGreenCase] = &[
    // Default mode covers: D12, D12b, D13, D16, D6, D10, P1, P3
    RedGreenCase {
        name: "default",
        mode_args: &["-w", "88", "-"],
        expected_file: "corner-cases.expected.default.md",
    },
    // Auto mode covers: D15 (smart quotes after inline code)
    RedGreenCase {
        name: "auto",
        mode_args: &["-w", "88", "--semantic", "--cleanups", "--smartquotes", "--ellipses", "-"],
        expected_file: "corner-cases.expected.auto.md",
    },
    // Tight mode covers: D4 (tight list spacing)
    RedGreenCase {
        name: "tight",
        mode_args: &["-w", "88", "--semantic", "--list-spacing", "tight", "-"],
        expected_file: "corner-cases.expected.tight.md",
    },
    // Loose mode covers: D5 (loose footnote lists)
    RedGreenCase {
        name: "loose",
        mode_args: &["-w", "88", "--semantic", "--list-spacing", "loose", "-"],
        expected_file: "corner-cases.expected.loose.md",
    },
    // Plaintext mode covers: D1, D2
    RedGreenCase {
        name: "plaintext",
        mode_args: &["-w", "88", "--plaintext", "-"],
        expected_file: "corner-cases.expected.plaintext.md",
    },
];

#[test]
fn test_red_green_v020_regression() {
    let Some(old_bin) = old_binary() else {
        eprintln!(
            "SKIP: Old v0.2.0 binary not found. Set FLOWMARK_OLD_BINARY \
             or install with: cargo install flowmark@0.2.0"
        );
        return;
    };

    let new_bin = new_binary();
    if !new_bin.exists() {
        eprintln!("SKIP: Current binary not built. Run: cargo build --features cli");
        return;
    }

    let parity_dir = Path::new("tests/parity");
    let input = std::fs::read_to_string(parity_dir.join("corner-cases.md"))
        .expect("Failed to read corner-cases.md");

    let mut results = Vec::new();
    let mut all_pass = true;

    for case in CASES {
        let expected_path = parity_dir.join(case.expected_file);
        let expected = std::fs::read_to_string(&expected_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", case.expected_file));

        // GREEN: new binary must match expected
        let new_out = run_binary_stdin(&new_bin, case.mode_args, &input)
            .unwrap_or_else(|| panic!("New binary failed on {} mode", case.name));

        let green_pass = new_out == expected;

        // RED: old binary should NOT match expected (bugs existed)
        let old_out = run_binary_stdin(&old_bin, case.mode_args, &input);
        let red_pass = match &old_out {
            Some(out) => out != &expected,
            None => true, // If old binary fails, that's also "different"
        };

        let status = match (red_pass, green_pass) {
            (true, true) => "PASS (red/green)",
            (false, true) => "WARN: old binary matches expected (no regression to catch)",
            (true, false) => {
                all_pass = false;
                "FAIL: new binary does NOT match expected"
            }
            (false, false) => {
                all_pass = false;
                "FAIL: both old and new binaries wrong"
            }
        };

        results.push(format!("  {}: {}", case.name, status));
    }

    eprintln!("Red/green results:");
    for r in &results {
        eprintln!("{r}");
    }

    assert!(all_pass, "Red/green test failures detected (see above)");
}
