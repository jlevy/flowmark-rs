//! Edge case tests identified from reviewing the previous flowmark-rs-1 implementation.
//!
//! These test cases cover behaviors that the old implementation handled via ~25
//! post-processing fixup functions. The current custom AST renderer should handle
//! all of these correctly without post-processing.

use flowmark::config::ListSpacing;
use flowmark::fill_markdown;

fn fmt(input: &str) -> String {
    fill_markdown(input, true, 88, true, false, false, false, None, ListSpacing::Preserve)
}

// === Edge case 1: Code fence with indented YAML-like content ===
// The old implementation had escape_fence_list_markers/unescape_fence_list_markers
// to work around comrak incorrectly parsing fenced code blocks containing indented
// list-like content with blank lines.

#[test]
fn test_code_fence_with_indented_list_content() {
    let input = r"```yaml
config:
    - item1
    - item2

    - item3
    - item4
```
";
    let result = fmt(input);
    // The code block content should be preserved exactly
    assert!(
        result.contains("    - item1"),
        "Indented list items in code block should be preserved: got {result:?}"
    );
    assert!(
        result.contains("    - item3"),
        "Items after blank line in code block should be preserved: got {result:?}"
    );
    // The code block should remain intact (not split into two)
    let fence_count = result.matches("```").count();
    assert_eq!(
        fence_count, 2,
        "Code block should have exactly one opening and one closing fence, got {fence_count} fences: {result:?}"
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
        "LaTeX backslashes in inline math should be preserved: got {result:?}"
    );
}

#[test]
fn test_display_math_latex() {
    let input = "$$\n\\sum_{i=1}^{n} x_i\n$$\n";
    let result = fmt(input);
    assert!(
        result.contains("\\sum_{i=1}^{n}"),
        "LaTeX in display math should be preserved: got {result:?}"
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
    assert!(result.contains("$420K"), "Bare dollar signs should not be escaped: got {result:?}");
    assert!(result.contains("$100M"), "Bare dollar signs should not be escaped: got {result:?}");
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
        "Code block content should be preserved exactly: got {result:?}"
    );
}

// === Edge case 5: Footnote handling ===
// The old implementation had complex footnote workarounds. Test that footnotes
// with references work correctly in the current implementation.

#[test]
fn test_footnote_with_reference() {
    let input = "Text with a footnote[^1] reference.\n\n[^1]: This is the footnote content.\n";
    let result = fmt(input);
    assert!(result.contains("[^1]"), "Footnote reference should be preserved: got {result:?}");
    assert!(result.contains("[^1]:"), "Footnote definition should be preserved: got {result:?}");
}

#[test]
fn test_multiple_footnotes() {
    let input =
        "First[^a] and second[^b] notes.\n\n[^a]: Note A content.\n\n[^b]: Note B content.\n";
    let result = fmt(input);
    assert!(
        result.contains("[^a]") && result.contains("[^b]"),
        "Multiple footnote references should be preserved: got {result:?}"
    );
    assert!(
        result.contains("[^a]:") && result.contains("[^b]:"),
        "Multiple footnote definitions should be preserved: got {result:?}"
    );
}

#[test]
fn test_footnote_autolink_blank_lines() {
    let input = "[^2]: <https://example.com/path>\n\n[^3]: <https://example.com/other>\n";
    let result = fmt(input);
    assert!(
        result.contains("\n\n[^3]:"),
        "Blank line between footnote defs with autolinks should be preserved: got {result:?}"
    );
}

// === GAP3: Angle-bracket autolinks preserved ===

#[test]
fn test_angle_bracket_autolink_preserved() {
    let input = "Visit <https://example.com> for details.\n";
    let result = fmt(input);
    assert!(
        result.contains("<https://example.com>"),
        "Angle-bracket autolink should be preserved: got {result:?}"
    );
    assert!(
        !result.contains("[https://example.com](https://example.com)"),
        "Should NOT be converted to inline link: got {result:?}"
    );
}

#[test]
fn test_angle_bracket_autolink_in_footnote() {
    let input = "[^1]: <https://example.com/article>\n";
    let result = fmt(input);
    assert!(
        result.contains("<https://example.com/article>"),
        "Angle-bracket autolink in footnote should be preserved: got {result:?}"
    );
}

// === GAP4: Bare URLs not converted to markdown links ===

#[test]
fn test_bare_url_not_linkified() {
    let input = "See https://www.google.com/ for more info.\n";
    let result = fmt(input);
    assert!(
        result.contains("https://www.google.com/"),
        "Bare URL should be present: got {result:?}"
    );
    assert!(
        !result.contains("[https://www.google.com/](https://www.google.com/)"),
        "Bare URL should NOT be converted to markdown link: got {result:?}"
    );
}

// === GAP5: Email addresses not linkified ===

#[test]
fn test_email_not_linkified() {
    let input = "Contact user@example.com for details.\n";
    let result = fmt(input);
    assert!(
        result.contains("user@example.com"),
        "Email should be present: got {result:?}"
    );
    assert!(
        !result.contains("[user@example.com](mailto:user@example.com)"),
        "Email should NOT be converted to mailto link: got {result:?}"
    );
}

#[test]
fn test_angle_bracket_email_preserved() {
    let input = "Email us at <user@example.com> for help.\n";
    let result = fmt(input);
    assert!(
        result.contains("<user@example.com>"),
        "Angle-bracket email should be preserved: got {result:?}"
    );
    assert!(
        !result.contains("[user@example.com](mailto:user@example.com)"),
        "Should NOT be converted to mailto link: got {result:?}"
    );
}

// === GAP9: Extra blank lines between HTML comment blocks and adjacent text ===

#[test]
fn test_html_comment_no_extra_blank_line_tight() {
    // When original has no blank line between HTML comment and text,
    // output should also have no blank line.
    let input = "<!-- comment -->\nText after comment.\n";
    let result = fmt(input);
    assert!(
        result.contains("<!-- comment -->\nText after comment."),
        "No blank line should be inserted after tight HTML comment: got {result:?}"
    );
}

#[test]
fn test_html_comment_preserves_blank_line_when_present() {
    // When original HAS a blank line between HTML comment and text,
    // output should preserve it.
    let input = "<!-- comment -->\n\nText after comment.\n";
    let result = fmt(input);
    assert!(
        result.contains("<!-- comment -->\n\nText after comment."),
        "Blank line after HTML comment should be preserved when present: got {result:?}"
    );
}

#[test]
fn test_html_comment_pair_tight_with_text() {
    // HTML comment pair surrounding text with no blank lines
    let input = "<!-- start -->\nInner text here.\n<!-- end -->\n";
    let result = fmt(input);
    assert!(
        result.contains("<!-- start -->\nInner text here.\n<!-- end -->"),
        "HTML comment pair should stay tight with enclosed text: got {result:?}"
    );
}

// === GAP8: Sentence breaks after closing paren/quote ===

fn fmt_semantic(input: &str) -> String {
    fill_markdown(input, true, 88, true, false, false, false, None, ListSpacing::Preserve)
}

#[test]
fn test_sentence_break_after_period_paren() {
    // `word.)` should be recognized as end of sentence
    let input = "This is a long sentence that ends with a paren (like you.) Next sentence starts here and keeps going for a while.\n";
    let result = fmt_semantic(input);
    assert!(
        result.contains("you.)\n"),
        "Should break sentence after .): got {result:?}"
    );
}

#[test]
fn test_no_sentence_break_inside_link_paren() {
    // `[Google](url)."` should NOT be treated as sentence end (false positive from URL)
    let input = "He worked at [Google](https://en.wikipedia.org/wiki/Google).\" \"The next sentence starts here and keeps going.\n";
    let result = fmt_semantic(input);
    assert!(
        !result.contains("Google).\"\n\"The"),
        "Should NOT break sentence inside link construct: got {result:?}"
    );
}

// === GAP6: Escaped char backslash in code span counted in line width ===

fn fmt_plain(input: &str) -> String {
    fill_markdown(input, true, 88, false, false, false, false, None, ListSpacing::Preserve)
}

#[test]
fn test_escaped_char_in_code_span_width() {
    // The backslash in `\.` inside a code span should count correctly for width.
    // Escape placeholder substitution should NOT happen inside code spans.
    // Plain mode (non-semantic) wrapping to demonstrate the width issue.
    let input = "Backslashes that are NOT CommonMark escape sequences are preserved. Note: `\\.` is a valid CommonMark escape (escaped period).\n";
    let result = fmt_plain(input);
    // Python wraps before "valid" (83 chars on first line), Rust should too.
    let first_line = result.lines().next().unwrap();
    assert!(
        first_line.chars().count() <= 88,
        "First line should not exceed 88 chars (got {}): {first_line:?}",
        first_line.chars().count()
    );
}

// === GAP10: Typography transforms in footnote bodies ===

fn fmt_auto(input: &str) -> String {
    fill_markdown(input, true, 88, true, true, true, true, None, ListSpacing::Preserve)
}

#[test]
fn test_smart_quotes_in_footnote_body() {
    let input = "Text with footnote[^1].\n\n[^1]: He said \"hello\" and she said \"goodbye\".\n";
    let result = fmt_auto(input);
    // Smart quotes should be applied inside footnote body
    let fn_line = result.lines().find(|l| l.starts_with("[^1]:")).unwrap();
    assert!(
        fn_line.contains('\u{201c}') || fn_line.contains('\u{201d}'),
        "Smart quotes should be applied in footnote body: got {fn_line:?}"
    );
}

#[test]
fn test_ellipsis_in_footnote_body() {
    let input = "Text with footnote[^1].\n\n[^1]: This is a long footnote...\n";
    let result = fmt_auto(input);
    let fn_line = result.lines().find(|l| l.starts_with("[^1]:")).unwrap();
    assert!(
        fn_line.contains('\u{2026}'),
        "Ellipsis should be applied in footnote body: got {fn_line:?}"
    );
}
