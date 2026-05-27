//! CLI integration tests for `--help` text and footer guidance.
//!
//! Ported from Python: `tests/test_cli_help.py` (v0.6.5, commit `9fda859`).
//!
//! Python's `_render_help` uses `capsys` to capture stdout from `main(["--help"])`.
//! In Rust we invoke the built binary directly because clap renders help via
//! its internal printer, which is harder to capture via library calls.
#![cfg(feature = "cli")]

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

fn render_help() -> String {
    let output = Command::new(flowmark_bin()).arg("--help").output().expect("run flowmark --help");
    assert!(output.status.success(), "flowmark --help should exit 0");
    String::from_utf8(output.stdout).expect("help output is utf-8")
}

#[test]
fn test_help_includes_tagline() {
    let out = render_help();
    assert!(
        out.contains("Flowmark: Better auto-formatting for Markdown and plaintext"),
        "missing tagline; got:\n{out}"
    );
}

#[test]
fn test_help_includes_brief_common_usage() {
    let out = render_help();
    assert!(out.contains("Common usage:"), "missing 'Common usage:'; got:\n{out}");
    for snippet in [
        "flowmark --auto README.md",
        "flowmark --auto docs/",
        "flowmark --auto .",
        "flowmark --list-files .",
    ] {
        assert!(out.contains(snippet), "missing {snippet:?}; got:\n{out}");
    }
}

#[test]
fn test_help_includes_agent_guidance() {
    let out = render_help();
    for snippet in [
        "Agent usage:",
        "flowmark --skill",
        "Agents should run `flowmark --skill` for full Flowmark usage guidance.",
        "Use `flowmark --docs` for full documentation.",
    ] {
        assert!(out.contains(snippet), "missing {snippet:?}; got:\n{out}");
    }
}

#[test]
fn test_help_omits_old_long_epilog() {
    let out = render_help();
    for snippet in
        ["Command-line usage examples:", "Flowmark provides enhanced text wrapping capabilities"]
    {
        assert!(!out.contains(snippet), "should not contain {snippet:?}; got:\n{out}");
    }
}
