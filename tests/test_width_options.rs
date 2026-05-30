use flowmark::config::ListSpacing;
use flowmark::reformat_text;

#[test]
fn test_normal_width_wrapping() {
    let text = "This is a long line that should definitely be wrapped at narrow widths but fits on one line at wide widths.";

    // Test narrow width
    let result_40 =
        reformat_text(text, 40, true, false, false, false, false, ListSpacing::Preserve);
    let lines_40: Vec<&str> = result_40.lines().collect();
    assert!(lines_40.len() > 1, "Text should be wrapped at width 40");

    // Test wide width
    let result_200 =
        reformat_text(text, 200, true, false, false, false, false, ListSpacing::Preserve);
    let lines_200: Vec<&str> = result_200.lines().collect();
    assert_eq!(lines_200.len(), 1, "Text should fit on one line at width 200");
}

#[test]
fn test_zero_width_disables_wrapping() {
    let text = "This is a very long line that would normally be wrapped at any reasonable width setting but should remain as a single line when wrapping is disabled.";

    let result = reformat_text(text, 0, true, false, false, false, false, ListSpacing::Preserve);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 1, "Width 0 should disable wrapping and keep text on one line");
    assert_eq!(result.trim(), text, "Text should be unchanged except for whitespace normalization");
}

#[test]
fn test_width_zero_with_semantic() {
    let text = "This is sentence one. This is sentence two. This is sentence three.";

    let result = reformat_text(text, 0, true, true, false, false, false, ListSpacing::Preserve);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 1, "Width 0 with semantic should still keep text on one line");
}

#[test]
fn test_width_zero_with_markdown() {
    let text = "This is a long paragraph that would normally be wrapped but should remain on one line when width is 0.";

    let result = reformat_text(text, 0, false, false, false, false, false, ListSpacing::Preserve);
    let content = result.trim_end_matches('\n');
    let lines: Vec<&str> = content.split('\n').collect();
    assert_eq!(lines.len(), 1, "Width 0 with markdown should keep paragraph on one line");
}

#[test]
fn test_width_zero_with_markdown_semantic() {
    let text = "This is sentence one. This is sentence two. This is sentence three.";

    let result = reformat_text(text, 0, false, true, false, false, false, ListSpacing::Preserve);
    let content = result.trim_end_matches('\n');
    let lines: Vec<&str> = content.split('\n').collect();
    assert_eq!(lines.len(), 1, "Width 0 with markdown semantic should keep text on one line");
}

#[test]
fn test_existing_behavior_unchanged() {
    let text = "This is a test line that should be wrapped at the default width of 88 characters.";

    let result_default =
        reformat_text(text, 88, true, false, false, false, false, ListSpacing::Preserve);
    let result_88 =
        reformat_text(text, 88, true, false, false, false, false, ListSpacing::Preserve);
    assert_eq!(result_default, result_88, "Default behavior should match explicit width=88");

    let long_text = "This is a very long line that definitely exceeds 88 characters and should be wrapped when using the default width setting.";
    let result =
        reformat_text(long_text, 88, true, false, false, false, false, ListSpacing::Preserve);
    let lines: Vec<&str> = result.lines().collect();
    assert!(lines.len() > 1, "Long text should be wrapped at default width");
}

// ===== Tests ported from Python test_width_options.py =====

#[test]
fn test_negative_width_disables_wrapping() {
    // Python uses negative width (-1) to disable wrapping.
    // In Rust, width is usize (unsigned), so width=0 serves the same purpose.
    // This test verifies the Rust equivalent behavior.
    let text = "This is a very long line that would normally be wrapped at any reasonable width setting but should remain as a single line when wrapping is disabled.";

    let result = reformat_text(text, 0, true, false, false, false, false, ListSpacing::Preserve);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 1, "Width 0 should disable wrapping and keep text on one line");
    assert_eq!(result.trim(), text, "Text should be unchanged except for whitespace normalization");
}

/// H1 regression test: `fill_text` with width < indent length must not panic.
#[test]
fn test_very_small_width_does_not_panic() {
    use flowmark::fill_text;
    use flowmark::wrapping::text_filling::Wrap;

    // WrapIndent uses 4-char indent, width=2 would underflow without saturating_sub
    let result = fill_text("Hello world test", Wrap::WrapIndent, 2, "", "", 0, None);
    assert!(!result.is_empty(), "should produce output, not panic");

    // Also test fill_markdown with very small width on a nested list
    let result = reformat_text(
        "- Item one with some text\n  - Nested item\n",
        3,
        false,
        false,
        false,
        false,
        false,
        ListSpacing::Preserve,
    );
    assert!(!result.is_empty(), "fill_markdown with width=3 should not panic");
}

// ===== Issue #42: task-list items must not accumulate spaces after the checkbox =====
// Ported from test_width_options.py. Comrak already renders the checkbox correctly; these
// pin the no-double-space + idempotency contract.

#[test]
fn test_task_list_idempotent_semantic_width0() {
    let text = "- [ ] Unchecked item one\n- [x] Checked item one\n- Normal list item\n";
    let once = reformat_text(text, 0, false, true, false, false, false, ListSpacing::Preserve);
    let twice = reformat_text(&once, 0, false, true, false, false, false, ListSpacing::Preserve);
    assert_eq!(once, twice);
    assert!(once.contains("- [ ] Unchecked item one"));
    assert!(once.contains("- [x] Checked item one"));
    assert!(!once.contains("]  "), "no double space after the checkbox");
}

#[test]
fn test_task_list_idempotent_semantic_default_width() {
    let text = "- [ ] First task\n- [x] Second task\n";
    let once = reformat_text(text, 88, false, true, false, false, false, ListSpacing::Preserve);
    let twice = reformat_text(&once, 88, false, true, false, false, false, ListSpacing::Preserve);
    assert_eq!(once, twice);
    assert!(!once.contains("]  "));
}
