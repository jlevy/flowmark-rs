//! Known parity gaps discovered during the v0.7.0 release-readiness review (2026-05-29).
//!
//! Each case below is a CLI-level divergence from Python flowmark v0.7.0 that the
//! curated `testdoc`/`corner-cases` fixtures did not catch, but a broader real-world
//! corpus sweep did. Per the porting playbook (Principles 2 & 8), every known parity
//! gap MUST have a discriminating, CI-visible **failing** test — a red CI for a real
//! gap is correct; a green CI that hides it is a lie.
//!
//! These tests are EXPECTED TO FAIL until each gap is either fixed or explicitly moved
//! to the tolerated-variations list in `docs/port-status.md` (decide fix-vs-tolerate
//! before tagging v0.3.0). Do not weaken or `#[ignore]` them to get green — fix the
//! formatter or reclassify the variation with user approval.
//!
//! Expected outputs are the **Python reference** (`flowmark@<PARITY_VERSION>`), captured
//! into `tests/parity/known-gaps/*.expected.md` via the Python binary. The Rust side is
//! the built CLI binary, so both sides share the same (CLI) output convention.
//!
//! Tracking (epic fmr-4o6y — exact parity with Python flowmark v0.7.0):
//! - gap A  -> fmr-4cnr (class of fmr-5u8i): `--plaintext` glues adjacent `{% %}` tags
//! - gap B  -> fmr-8vy3: HTML comment with internal blank line collapsed
//! - gap C1 -> fmr-rscu: missing blank line before thematic break after a list
//! - gap C2 -> fmr-fle0: extra blank lines around code block in a tight list item
//! - gap E  -> fmr-0lz1: inline code span with backtick renders with wrong delimiter
//! - gap F  -> fmr-iblt: paragraph→blockquote (tight in source) gets an inserted blank
//!
//! (gap D, fmr-kylr, wide/irregular table reflow — needs characterization first, no
//! test here yet.)
#![allow(clippy::unwrap_used)]
#![cfg(feature = "cli")]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn rust_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/flowmark")
}

fn run_rust_stdin(args: &[&str], input: &str) -> String {
    let bin = rust_binary();
    // Fail loudly rather than silently skipping (porting playbook, Principle 4):
    // `cargo test --all-features` builds this binary, so its absence is a real error.
    assert!(
        bin.exists(),
        "Rust binary not found at {}. Build with `cargo build --all-features`.",
        bin.display()
    );
    let mut child = Command::new(&bin)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn Rust flowmark");
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "Rust flowmark exited non-zero: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// Run the Rust CLI over `<stem>.md` and assert byte-identity with the Python-reference
/// golden `<stem>.expected.md`. Diverging output (the gap) makes this fail.
fn assert_matches_python_golden(stem: &str, args: &[&str], issue: &str) {
    let dir = Path::new("tests/parity/known-gaps");
    let input = std::fs::read_to_string(dir.join(format!("{stem}.md")))
        .unwrap_or_else(|e| panic!("read {stem}.md: {e}"));
    let expected = std::fs::read_to_string(dir.join(format!("{stem}.expected.md")))
        .unwrap_or_else(|e| panic!("read {stem}.expected.md: {e}"));
    let actual = run_rust_stdin(args, &input);
    assert_eq!(
        actual, expected,
        "\nKNOWN PARITY GAP ({issue}): Rust output differs from Python v0.7.0 reference \
         for `{stem}`.\n--- Python (expected) ---\n{expected}\n--- Rust (actual) ---\n{actual}"
    );
}

#[test]
fn gap_a_plaintext_tag_wrap() {
    // fmr-4cnr (class of fmr-5u8i)
    assert_matches_python_golden(
        "gapA_plaintext_tag_wrap",
        &["-w", "88", "--plaintext", "-"],
        "fmr-4cnr",
    );
}

#[test]
fn gap_b_html_comment_blank_line() {
    // fmr-8vy3
    assert_matches_python_golden("gapB_html_comment_blank_line", &["-w", "88", "-"], "fmr-8vy3");
}

#[test]
fn gap_c1_tasklist_thematic_break() {
    // fmr-rscu
    assert_matches_python_golden("gapC1_tasklist_thematic_break", &["-w", "88", "-"], "fmr-rscu");
}

#[test]
fn gap_c2_listitem_codeblock() {
    // fmr-fle0
    assert_matches_python_golden("gapC2_listitem_codeblock", &["-w", "88", "-"], "fmr-fle0");
}

#[test]
fn gap_e_inline_code_backtick() {
    // fmr-0lz1
    assert_matches_python_golden("gapE_inline_code_backtick", &["-w", "88", "-"], "fmr-0lz1");
}

#[test]
fn gap_f_paragraph_blockquote_tight() {
    // fmr-iblt
    assert_matches_python_golden("gapF_paragraph_blockquote_tight", &["-w", "88", "-"], "fmr-iblt");
}

#[test]
fn gap_g_codeblock_paragraph_tight() {
    // fmr-h5u3
    assert_matches_python_golden("gapG_codeblock_paragraph_tight", &["-w", "88", "-"], "fmr-h5u3");
}

#[test]
fn gap_h_list_list_tight() {
    // fmr-27ba
    assert_matches_python_golden("gapH_list_list_tight", &["-w", "88", "-"], "fmr-27ba");
}

#[test]
fn gap_h_code_list_tight() {
    // fmr-27ba / fmr-5vyd
    assert_matches_python_golden("gapH_code_list_tight", &["-w", "88", "-"], "fmr-5vyd");
}

#[test]
fn gap_h_list_code_tight() {
    // fmr-27ba
    assert_matches_python_golden("gapH_list_code_tight", &["-w", "88", "-"], "fmr-27ba");
}

#[test]
fn gap_h_code_code_tight() {
    // fmr-27ba
    assert_matches_python_golden("gapH_code_code_tight", &["-w", "88", "-"], "fmr-27ba");
}

#[test]
fn gap_h_list_blockquote_tight() {
    // fmr-27ba
    assert_matches_python_golden("gapH_list_blockquote_tight", &["-w", "88", "-"], "fmr-27ba");
}
