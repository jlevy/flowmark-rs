//! Edge case tests identified from reviewing the previous flowmark-rs-1 implementation.
//!
//! These test cases cover behaviors that the old implementation handled via ~25
//! post-processing fixup functions. The current custom AST renderer should handle
//! all of these correctly without post-processing.

use flowmark::fill_markdown;
use flowmark::config::ListSpacing;

fn fmt(input: &str) -> String {
    fill_markdown(input, true, 88, true, false, false, false, None, ListSpacing::Preserve)
}

// === Edge case 1: Code fence with indented YAML-like content ===
// The old implementation had escape_fence_list_markers/unescape_fence_list_markers
// to work around comrak incorrectly parsing fenced code blocks containing indented
// list-like content with blank lines.

#[test]
fn test_code_fence_with_indented_list_content() {
    let input = r#"```yaml
config:
    - item1
    - item2

    - item3
    - item4
```
"#;
    let result = fmt(input);
    // The code block content should be preserved exactly
    assert!(
        result.contains("    - item1"),
        "Indented list items in code block should be preserved: got {:?}",
        result
    );
    assert!(
        result.contains("    - item3"),
        "Items after blank line in code block should be preserved: got {:?}",
        result
    );
    // The code block should remain intact (not split into two)
    let fence_count = result.matches("```").count();
    assert_eq!(
        fence_count, 2,
        "Code block should have exactly one opening and one closing fence, got {fence_count} fences: {:?}",
        result
    );
}

// === Edge case 2: Inline math with LaTeX backslashes ===
// The old implementation had fix_math_escaping to handle comrak double-escaping
// backslashes inside math (e.g., \frac -> \\frac).

#[test]
fn test_inline_math_latex_backslashes() {
    let input = "The formula is $\\frac{1}{2}$ here.\n";
    let result = fmt(input);
    assert!(
        result.contains("$\\frac{1}{2}$"),
        "LaTeX backslashes in inline math should be preserved: got {:?}",
        result
    );
}

#[test]
fn test_display_math_latex() {
    let input = "$$\n\\sum_{i=1}^{n} x_i\n$$\n";
    let result = fmt(input);
    assert!(
        result.contains("\\sum_{i=1}^{n}"),
        "LaTeX in display math should be preserved: got {:?}",
        result
    );
}

// === Edge case 3: Bare dollar signs in text ===
// The old implementation had preserve_dollar_escaping that re-escaped bare $
// followed by alphanumeric characters. The current implementation should leave
// bare $ alone (not add escapes that weren't there).

#[test]
fn test_bare_dollar_in_text() {
    let input = "The cost is $420K and profits are $100M.\n";
    let result = fmt(input);
    // Bare $ should not be escaped
    assert!(
        result.contains("$420K"),
        "Bare dollar signs should not be escaped: got {:?}",
        result
    );
    assert!(
        result.contains("$100M"),
        "Bare dollar signs should not be escaped: got {:?}",
        result
    );
}

// === Edge case 4: Trailing blank lines inside code blocks ===
// The old implementation had remove_blank_lines_before_fence_close to strip
// trailing blank lines before the closing ```.

#[test]
fn test_code_block_trailing_content() {
    let input = "```python\ndef foo():\n    return 42\n```\n";
    let result = fmt(input);
    assert!(
        result.contains("```python\ndef foo():\n    return 42\n```"),
        "Code block content should be preserved exactly: got {:?}",
        result
    );
}

// === Edge case 5: Footnote handling ===
// The old implementation had complex footnote workarounds. Test that footnotes
// with references work correctly in the current implementation.

#[test]
fn test_footnote_with_reference() {
    let input = "Text with a footnote[^1] reference.\n\n[^1]: This is the footnote content.\n";
    let result = fmt(input);
    assert!(
        result.contains("[^1]"),
        "Footnote reference should be preserved: got {:?}",
        result
    );
    assert!(
        result.contains("[^1]:"),
        "Footnote definition should be preserved: got {:?}",
        result
    );
}

#[test]
fn test_multiple_footnotes() {
    let input = "First[^a] and second[^b] notes.\n\n[^a]: Note A content.\n\n[^b]: Note B content.\n";
    let result = fmt(input);
    assert!(
        result.contains("[^a]") && result.contains("[^b]"),
        "Multiple footnote references should be preserved: got {:?}",
        result
    );
    assert!(
        result.contains("[^a]:") && result.contains("[^b]:"),
        "Multiple footnote definitions should be preserved: got {:?}",
        result
    );
}
