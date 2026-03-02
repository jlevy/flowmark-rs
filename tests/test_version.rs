//! CLI version output tests.
#![cfg(feature = "cli")]

use regex::Regex;
use std::path::PathBuf;
use std::process::Command;

/// Get the path to the built binary.
fn flowmark_bin() -> PathBuf {
    let mut path = std::env::current_exe().expect("current exe");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("flowmark");
    path
}

#[test]
fn test_version_has_expected_shape() {
    let output =
        Command::new(flowmark_bin()).arg("--version").output().expect("run flowmark --version");

    assert!(output.status.success(), "--version should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version_line = stdout.trim();

    assert!(
        version_line.contains("Rust port of flowmark-py "),
        "version should include Python source version: {version_line}"
    );
    assert!(
        version_line.contains("; base "),
        "version should include release base tag: {version_line}"
    );

    assert!(
        !version_line.contains("-dev.unknown+gunknown"),
        "version should never expose dev.unknown+gunknown in release artifacts: {version_line}"
    );

    let re = Regex::new(r"^flowmark \d+\.\d+\.\d+(?:-dev\.\d+\+g[0-9a-f]+)? \(Rust port of flowmark-py \d+\.\d+\.\d+; base .+\)$").expect("valid regex");
    assert!(
        re.is_match(version_line),
        "version line does not match expected format: {version_line}"
    );
}
