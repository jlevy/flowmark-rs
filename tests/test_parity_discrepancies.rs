//! Parity discrepancy tests: Python flowmark v0.6.4 vs Rust flowmark.
//!
//! Each test documents a specific discrepancy found during senior review (2026-02-18).
//! Expected values are the Python v0.6.4 output. Tests are marked `#[ignore]` when
//! the fix has not yet been implemented — remove `#[ignore]` as each fix lands.
//!
//! See: docs/project/specs/active/plan-2026-02-18-parity-discrepancies.md
#![allow(clippy::unwrap_used)]

use flowmark::Wrap;
use flowmark::config::ListSpacing;
use flowmark::fill_markdown;
use flowmark::fill_text;

fn fmt(input: &str) -> String {
    fill_markdown(input, true, 88, false, false, false, false, None, ListSpacing::Preserve)
}

fn fmt_semantic(input: &str) -> String {
    fill_markdown(input, true, 88, true, false, false, false, None, ListSpacing::Preserve)
}

fn fmt_width(input: &str, width: usize) -> String {
    fill_markdown(input, true, width, false, false, false, false, None, ListSpacing::Preserve)
}

fn _fmt_semantic_width(input: &str, width: usize) -> String {
    fill_markdown(input, true, width, true, false, false, false, None, ListSpacing::Preserve)
}

fn fmt_tight(input: &str) -> String {
    fill_markdown(input, true, 88, true, false, false, false, None, ListSpacing::Tight)
}

fn fmt_loose(input: &str) -> String {
    fill_markdown(input, true, 88, true, false, false, false, None, ListSpacing::Loose)
}

fn fmt_plaintext(input: &str) -> String {
    // Match Python: Wrap::Wrap (replace_whitespace=false) with html_md_word_split (default).
    // Python's use of html_md_word_splitter in plaintext mode is likely a bug (fmr-5u8i).
    fill_text(input, Wrap::Wrap, 88, "", "", 0, None)
}

// =============================================================================
// D1: Plaintext mode collapses code blocks (fmr-n69j)
// Python's plaintext mode preserves code fence structure.
// =============================================================================

#[test]
fn test_d1_plaintext_preserves_code_fences() {
    let input =
        "Some text.\n\n```javascript\n// This is a code block\nvar x = 5;\n```\n\nMore text.\n";
    let result = fmt_plaintext(input);
    assert!(
        result.contains("```javascript\n// This is a code block\nvar x = 5;\n```"),
        "D1: Plaintext mode should preserve code fence structure, got:\n{result}"
    );
}

#[test]
fn test_d1_plaintext_preserves_empty_code_block() {
    let input = "Before.\n\n```\nThis is\nanother.\n```\n\nAfter.\n";
    let result = fmt_plaintext(input);
    assert!(
        result.contains("```\nThis is\nanother.\n```"),
        "D1: Plaintext mode should preserve unfenced code block, got:\n{result}"
    );
}

// =============================================================================
// D2: Plaintext mode word splitting (fmr-fzth)
// Python's plaintext mode uses html_md_word_splitter which treats markdown
// links as atomic constructs. This is likely a bug in Python (fmr-5u8i) —
// plaintext mode should use simple_word_splitter — but we match the behavior
// for parity.
// =============================================================================

#[test]
fn test_d2_plaintext_treats_markdown_links_as_atomic() {
    // Python's plaintext mode uses html_md_word_splitter, so markdown links
    // are treated as indivisible tokens. The link wraps onto its own line.
    let input = "The school is [St. John's Beaumont School](https://en.wikipedia.org/wiki/St_John%27s_Beaumont_School) in the area.\n";
    let result = fmt_plaintext(input);
    // The link should be kept as one atomic token (matching Python)
    assert!(
        result.contains("[St. John's Beaumont School](https://en.wikipedia.org/wiki/St_John%27s_Beaumont_School)"),
        "D2: Plaintext mode should treat markdown links as atomic (matching Python), got:\n{result}"
    );
    // Verify the text wraps to multiple lines
    assert!(
        result.lines().count() >= 2,
        "D2: Long plaintext with link should wrap to multiple lines, got:\n{result}"
    );
}

// =============================================================================
// D3: Narrow width wraps differently around <sup> tags (fmr-bzra)
// Original discrepancy found at width 60 with 4-space list item indent
// (effective width 56). Test uses width 56 without indent to match
// the effective wrapping behavior.
// =============================================================================

#[test]
fn test_d3_sup_tag_wrapping_at_width_56() {
    // Text without 4-space indent (dedent_input=true would strip it anyway).
    // Width 56 = effective width inside a list item at width 60.
    let input = "wb\\+ mode (binary read/write), automatically deleted when closed or on process termination.<sup>19</sup> While convenient, POSIX notes potential permission issues and recommends mkstemp followed by fdopen for multithreaded apps to avoid leaking file descriptors.<sup>59</sup>\n";
    // Python output (indent stripped, at effective width 56): 6 lines
    let python_output = "wb\\+ mode (binary read/write), automatically\ndeleted when closed or on process\ntermination.<sup>19</sup> While convenient, POSIX\nnotes potential permission issues and recommends\nmkstemp followed by fdopen for multithreaded apps\nto avoid leaking file descriptors.<sup>59</sup>\n";
    let result = fmt_width(input, 56);
    // Verify same number of lines and that <sup> tags are preserved
    let result_lines: Vec<&str> = result.trim_end().lines().collect();
    let python_lines: Vec<&str> = python_output.trim_end().lines().collect();
    assert!(
        result.contains("<sup>19</sup>") && result.contains("<sup>59</sup>"),
        "D3: <sup> tags should be preserved in output, got:\n{result}"
    );
    assert_eq!(
        result_lines.len(),
        python_lines.len(),
        "D3: Width 56 wrapping should produce same number of lines.\nRust ({} lines):\n{}\nPython ({} lines):\n{}",
        result_lines.len(),
        result,
        python_lines.len(),
        python_output,
    );
}

// =============================================================================
// D4: Tight list spacing with nested sublists (fmr-r9k6)
// Python's tight mode adds blank lines between items when any item has sublists,
// but keeps sublists themselves tight. Within-item spacing is loose only when
// the item's sublist has deeper nesting.
// =============================================================================

#[test]
fn test_d4_tight_nested_lists_match_python() {
    let input = "- Level 1a\n  - Level 2a\n    - Level 3a\n- Level 1b\n  - Level 2b\n";
    let result = fmt_tight(input);
    // Python: blank after "Level 1a" (item has complex sublist with deeper nesting),
    // tight between "Level 2a" and "Level 3a", blank before "Level 1b",
    // tight between "Level 1b" and "Level 2b" (item has flat sublist).
    let python_output =
        "- Level 1a\n\n  - Level 2a\n    - Level 3a\n\n- Level 1b\n  - Level 2b\n";
    assert_eq!(
        result, python_output,
        "D4: Tight nested lists should match Python behavior.\nGot:\n{result}"
    );
}

#[test]
fn test_d4_tight_simple_sublists() {
    // Simple sublists (no deeper nesting) — Python adds blanks between items
    // but keeps within-item spacing tight.
    let input = "- A\n  - B\n- C\n  - D\n";
    let result = fmt_tight(input);
    let python_output = "- A\n  - B\n\n- C\n  - D\n";
    assert_eq!(
        result, python_output,
        "D4: Tight simple sublists should match Python.\nGot:\n{result}"
    );
}

#[test]
fn test_d4_tight_ordered_sublists() {
    let input = "1. Ordered 1\n   1. Sub 1\n   2. Sub 2\n2. Ordered 2\n";
    let result = fmt_tight(input);
    // Python: tight within item (Ordered 1 → Sub 1), blank between items
    let python_output = "1. Ordered 1\n   1. Sub 1\n   2. Sub 2\n\n2. Ordered 2\n";
    assert_eq!(
        result, python_output,
        "D4: Tight ordered sublists should match Python.\nGot:\n{result}"
    );
}

// =============================================================================
// D5: Loose list spacing missing blank lines in footnote embedded lists (fmr-vpg4)
// Python adds blank lines after footnote list items that Rust omits.
// =============================================================================

#[test]
fn test_d5_loose_footnote_list_items() {
    let input = "[^217]: Testing - : Is Ketamine Contraindicated?\n    - REBEL EM - more words,\n      <https://rebelem.com/test>\n\n[^multiline]: Another footnote.\n";
    let result = fmt_loose(input);
    // Python adds blank line after the footnote list item
    assert!(
        result.contains("<https://rebelem.com/test>\n\n[^multiline]:"),
        "D5: Loose mode should add blank line after footnote list items, got:\n{result}"
    );
}

// =============================================================================
// D6: Nested blockquotes get extra blank separator lines (fmr-3i50)
// Rust inserts "> " blank lines between nested blockquote levels.
// =============================================================================

#[test]
fn test_d6_nested_blockquotes_no_extra_blanks() {
    let input = "> Level 1\n> > Level 2\n> > > Level 3\n";
    let python_output = "> Level 1\n> > Level 2\n> > > Level 3\n";
    let result = fmt(input);
    assert_eq!(result, python_output, "D6: Nested blockquotes should not have extra blank lines");
}

#[test]
fn test_d6_two_level_blockquote() {
    let input = "> Outer\n> > Inner\n";
    let result = fmt(input);
    assert!(
        !result.contains(">\n>"),
        "D6: Should not have blank '> ' line between blockquote levels, got:\n{result}"
    );
}

#[test]
fn test_d6_nested_blockquote_preserves_blank_separator() {
    // When source has a blank `>` line between outer and inner blockquote,
    // Python preserves it. Rust must do the same.
    let input = "> Outer quote.\n>\n> > Inner quote.\n";
    let python_output = "> Outer quote.\n> \n> > Inner quote.\n";
    let result = fmt(input);
    assert_eq!(
        result, python_output,
        "D6: Blank `>` separator between blockquote levels should be preserved.\nGot:\n{result}"
    );
}

// =============================================================================
// D7: Footnote body continuation list items collapsed onto one line (fmr-81j7)
// =============================================================================

#[test]
fn test_d7_footnote_with_list_items() {
    let input = "[^3]: Footnote with a list:\n    - Item 1\n    - Item 2\n    - Item 3\n";
    let result = fmt_semantic(input);
    // Python preserves list items on separate lines
    assert!(
        result.contains("- Item 1\n"),
        "D7: Footnote list items should be on separate lines, got:\n{result}"
    );
    assert!(
        result.contains("- Item 2\n"),
        "D7: Footnote list items should be on separate lines, got:\n{result}"
    );
}

#[test]
fn test_d7_footnote_preamble_then_list() {
    let input = "[^3]: Footnote with a list:\n    - Item 1\n    - Item 2\n";
    let result = fmt(input);
    // Preamble and list items should be separate
    assert!(
        !result.contains("list: - Item"),
        "D7: Footnote preamble should not collapse with list items, got:\n{result}"
    );
}

// =============================================================================
// D8: Footnote body blockquote continuation collapsed onto first line (fmr-xcr9)
// =============================================================================

#[test]
fn test_d8_footnote_with_blockquote() {
    let input = "[^4]: Footnote with blockquote:\n    > This is quoted inside footnote.\n";
    let result = fmt(input);
    // Python preserves blockquote on separate line
    assert!(
        result.contains("> This is quoted"),
        "D8: Footnote blockquote should be on its own line, got:\n{result}"
    );
    assert!(
        !result.contains("blockquote: > This"),
        "D8: Footnote blockquote should not be collapsed onto preamble, got:\n{result}"
    );
}

// =============================================================================
// D9: Empty/whitespace input produces no output (fmr-dihn)
// Python always outputs at least a trailing newline.
// =============================================================================

#[test]
fn test_d9_empty_input_outputs_newline() {
    let result = fmt("");
    assert_eq!(result, "\n", "D9: Empty input should produce a trailing newline");
}

#[test]
fn test_d9_whitespace_input_outputs_newline() {
    let result = fmt("   \n  \n");
    assert_eq!(result, "\n", "D9: Whitespace-only input should produce a trailing newline");
}

#[test]
fn test_d9_single_newline_input() {
    let result = fmt("\n");
    assert_eq!(result, "\n", "D9: Single newline input should produce a trailing newline");
}

// =============================================================================
// D10: HTML entities decoded instead of preserved (fmr-gocw)
// Comrak decodes &amp; to &, &lt; to <, etc. Python preserves them.
// =============================================================================

#[test]
fn test_d10_html_entities_preserved() {
    let input = "&amp; &lt; &gt; &quot;\n";
    let result = fmt(input);
    assert_eq!(result, "&amp; &lt; &gt; &quot;\n", "D10: HTML entities should be preserved as-is");
}

#[test]
fn test_d10_html_entity_in_paragraph() {
    let input = "The value is &gt; 5 and &lt; 10.\n";
    let result = fmt(input);
    assert!(
        result.contains("&gt;") && result.contains("&lt;"),
        "D10: HTML entities should be preserved in paragraphs, got:\n{result}"
    );
}

// =============================================================================
// D11: CLI error handling parity (fmr-8ixa)
// Verify that Rust CLI error messages match Python's error messages.
// Requires the Python flowmark binary to be available at the expected path.
// =============================================================================

#[cfg(feature = "cli")]
/// Run a CLI binary with args and capture stderr + exit code.
fn run_cli(bin: &str, args: &[&str]) -> (String, i32) {
    let output = std::process::Command::new(bin)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("Failed to run {bin}: {e}"));
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);
    (stderr.trim_end().to_string(), code)
}

#[cfg(feature = "cli")]
fn run_cli_stdin(bin: &str, args: &[&str], stdin: &str) -> (String, i32) {
    use std::io::Write;
    let mut child = std::process::Command::new(bin)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("Failed to run {bin}: {e}"));
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);
    (stderr.trim_end().to_string(), code)
}

#[cfg(feature = "cli")]
fn python_flowmark() -> &'static str {
    "flowmark"
}

#[cfg(feature = "cli")]
fn rust_flowmark() -> String {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.join("target/debug/flowmark").to_string_lossy().to_string()
}

#[test]
#[cfg(feature = "cli")]
fn test_d11_no_args_error_matches_python() {
    let (py_err, py_code) = run_cli(python_flowmark(), &[]);
    let (rs_err, rs_code) = run_cli(&rust_flowmark(), &[]);
    assert_eq!(rs_err, py_err, "D11: No-args error message should match Python");
    assert_eq!(rs_code, py_code, "D11: No-args exit code should match Python");
}

#[test]
#[cfg(feature = "cli")]
fn test_d11_auto_no_args_error_matches_python() {
    let (py_err, py_code) = run_cli(python_flowmark(), &["--auto"]);
    let (rs_err, rs_code) = run_cli(&rust_flowmark(), &["--auto"]);
    assert_eq!(rs_err, py_err, "D11: --auto no-args error should match Python");
    assert_eq!(rs_code, py_code, "D11: --auto no-args exit code should match Python");
}

#[test]
#[cfg(feature = "cli")]
fn test_d11_inplace_stdin_error_matches_python() {
    let (py_err, py_code) = run_cli_stdin(python_flowmark(), &["--inplace", "-"], "hello\n");
    let (rs_err, rs_code) = run_cli_stdin(&rust_flowmark(), &["--inplace", "-"], "hello\n");
    assert_eq!(rs_err, py_err, "D11: --inplace stdin error should match Python");
    assert_eq!(rs_code, py_code, "D11: --inplace stdin exit code should match Python");
}

#[test]
#[cfg(feature = "cli")]
fn test_d11_output_multiple_files_error_matches_python() {
    let (py_err, py_code) = run_cli(python_flowmark(), &["-o", "out.md", "/dev/null", "/dev/null"]);
    let (rs_err, rs_code) = run_cli(&rust_flowmark(), &["-o", "out.md", "/dev/null", "/dev/null"]);
    assert_eq!(rs_err, py_err, "D11: multi-file output error should match Python");
    assert_eq!(rs_code, py_code, "D11: multi-file output exit code should match Python");
}

#[test]
#[cfg(feature = "cli")]
fn test_d11_nonexistent_file_error_format() {
    let (py_err, _py_code) = run_cli(python_flowmark(), &["nonexistent.md"]);
    let (rs_err, _rs_code) = run_cli(&rust_flowmark(), &["nonexistent.md"]);
    // Python: "Error: [Errno 2] No such file or directory: 'nonexistent.md'" (exit 2)
    // Rust:   "Error: Path not found: nonexistent.md" (exit 1)
    // Exact byte-for-byte match isn't possible ([Errno 2] is a Python-ism),
    // but both must: start with "Error:", mention the filename.
    assert!(
        rs_err.starts_with("Error:"),
        "D11: Rust nonexistent file error should start with 'Error:', got: {rs_err}"
    );
    assert!(
        rs_err.contains("nonexistent.md"),
        "D11: Rust error should mention the filename, got: {rs_err}"
    );
    assert!(
        py_err.starts_with("Error:"),
        "D11: Python nonexistent file error should start with 'Error:', got: {py_err}"
    );
    assert!(
        py_err.contains("nonexistent.md"),
        "D11: Python error should mention the filename, got: {py_err}"
    );
}

// =============================================================================
// D12: Paragraph before code fence — extra blank line inserted (P6)
// Python preserves tight paragraph→code fence transition (no blank line).
// Rust inserts a blank line between the paragraph and the opening code fence.
// Root cause: render_block_children() suppress_for_tight doesn't handle
// paragraph→CodeBlock transitions.
// Discovered: Real-world corpus test (ai-trade-arena docs/, 454 instances).
// =============================================================================

#[test]
fn test_d12_paragraph_before_code_fence_tight() {
    // Python keeps paragraph tight against code fence (no blank line).
    let input = "**Configuration Options**:\n```typescript\n{\n  minTime: number,\n}\n```\n";
    let python_output = "**Configuration Options**:\n```typescript\n{\n  minTime: number,\n}\n```\n";
    let result = fmt(input);
    assert_eq!(
        result, python_output,
        "D12/P6: Should not insert blank line before code fence when source is tight.\nGot:\n{result}"
    );
}

#[test]
fn test_d12_inline_code_paragraph_before_code_fence() {
    // Paragraph ending with inline code, tight against a code fence.
    let input = "Add to root `package.json`:\n```json\n{\n  \"scripts\": {}\n}\n```\n";
    let python_output = "Add to root `package.json`:\n```json\n{\n  \"scripts\": {}\n}\n```\n";
    let result = fmt(input);
    assert_eq!(
        result, python_output,
        "D12/P6: Inline-code paragraph should stay tight before code fence.\nGot:\n{result}"
    );
}

#[test]
fn test_d12_multiple_tight_code_fences() {
    // Multiple tight paragraph→code fence transitions in one document.
    let input = "First block:\n```bash\necho hello\n```\n\nSecond block:\n```python\nprint(\"hi\")\n```\n";
    let python_output = "First block:\n```bash\necho hello\n```\n\nSecond block:\n```python\nprint(\"hi\")\n```\n";
    let result = fmt(input);
    assert_eq!(
        result, python_output,
        "D12/P6: Multiple tight code fences should not get extra blank lines.\nGot:\n{result}"
    );
}

// =============================================================================
// D13: Blockquote blank continuation line loses `>` prefix (P7)
// Python preserves blockquote-prefixed blank lines (e.g., ">    " with trailing spaces).
// Rust strips the `>` prefix, outputting a bare empty line.
// Discovered: Real-world corpus test (ai-trade-arena docs/, 9 instances).
// =============================================================================

#[test]
fn test_d13_blockquote_blank_continuation_preserves_prefix() {
    // Blockquote with a blank continuation line between ordered list and unordered list.
    let input = "> 2. **Review the previous** for context:\n>    \n>    - Check the section\n";
    let result = fmt(input);
    // Python preserves the blockquote prefix on blank lines.
    // The blank line should have a `>` prefix (possibly with trailing spaces).
    assert!(
        !result.contains("\n\n>"),
        "D13/P7: Blockquote blank continuation should keep `>` prefix, not become bare empty line.\nGot:\n{result}"
    );
}

#[test]
fn test_d13_blockquote_list_with_blank_continuation() {
    // Blockquote with rules list and blank continuation line.
    let input = "> - Rules:\n>   \n>   1. Look for duplicated code\n>\n>   2. Look for dead code\n";
    let result = fmt(input);
    // Blank continuation line inside blockquote list should keep `>` prefix.
    let lines: Vec<&str> = result.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() && i > 0 && i < lines.len() - 1 {
            // Every blank line within the blockquote should have a `>` prefix
            assert!(
                line.starts_with('>'),
                "D13/P7: Blank line {i} inside blockquote should keep `>` prefix.\nFull output:\n{result}"
            );
        }
    }
}

// =============================================================================
// D14: Escaped backtick stripped in table inline code (P8)
// Python preserves \` inside inline code in table cells.
// Rust strips the trailing \` escape.
// Related to P3 (missing ESCAPE_CHARS) but specific to backtick in inline code.
// Discovered: Real-world corpus test (ai-trade-arena docs/, 1 instance).
// =============================================================================

#[test]
fn test_d14_escaped_backtick_in_table_inline_code() {
    let input = "| Col1 | Col2 |\n| --- | --- |\n| swallowing | `throw new CLIError(\\`${msg}: ${error.message}\\`)` |\n";
    let result = fmt(input);
    // Python preserves both \` escapes in inline code within the table cell.
    assert!(
        result.contains("\\`)` |") || result.contains("\\`)`|"),
        "D14/P8: Escaped backtick at end of inline code in table should be preserved.\nGot:\n{result}"
    );
}

// =============================================================================
// D15: Smart quote conversion after inline code backtick (P9)
// Python does NOT convert apostrophes immediately after inline code to smart quotes.
// Rust converts them. This is a smartquotes engine difference.
// Discovered: Real-world corpus test (ai-trade-arena docs/, 1 instance).
// =============================================================================

fn fmt_auto(input: &str) -> String {
    // Match --auto mode: semantic + cleanups + smartquotes (no ellipses)
    fill_markdown(input, true, 88, true, true, true, false, None, ListSpacing::Preserve)
}

#[test]
fn test_d15_no_smart_quote_after_inline_code() {
    let input = "Call `foo()`'s result.\n";
    let result = fmt_auto(input);
    // Python keeps the straight apostrophe after inline code backtick.
    assert!(
        result.contains("`'s"),
        "D15/P9: Apostrophe after inline code should NOT be converted to smart quote.\nGot:\n{result}"
    );
}

// =============================================================================
// Regression: Autolink false positive for relative paths (already fixed)
// =============================================================================

#[test]
fn test_relative_path_link_preserved() {
    let input = "See [docs/port-sync-playbook.md](docs/port-sync-playbook.md) for details.\n";
    let result = fmt_semantic(input);
    assert!(
        result.contains("[docs/port-sync-playbook.md](docs/port-sync-playbook.md)"),
        "Relative path link where text==URL should be preserved as explicit link, got:\n{result}"
    );
}

#[test]
fn test_absolute_url_autolink_still_works() {
    let input = "Visit https://example.com for info.\n";
    let result = fmt(input);
    // Bare URL should remain as bare text (not wrapped in [text](url))
    assert!(
        !result.contains("[https://example.com](https://example.com)"),
        "Absolute URL autolink should render as bare text, got:\n{result}"
    );
}
