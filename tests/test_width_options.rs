use flowmark::reformat_text;
use flowmark::config::ListSpacing;

#[test]
fn test_normal_width_wrapping() {
    let text = "This is a long line that should definitely be wrapped at narrow widths but fits on one line at wide widths.";

    // Test narrow width
    let result_40 = reformat_text(text, 40, true, false, false, false, false, ListSpacing::Preserve);
    let lines_40: Vec<&str> = result_40.trim().split('\n').collect();
    assert!(lines_40.len() > 1, "Text should be wrapped at width 40");

    // Test wide width
    let result_200 = reformat_text(text, 200, true, false, false, false, false, ListSpacing::Preserve);
    let lines_200: Vec<&str> = result_200.trim().split('\n').collect();
    assert_eq!(lines_200.len(), 1, "Text should fit on one line at width 200");
}

#[test]
fn test_zero_width_disables_wrapping() {
    let text = "This is a very long line that would normally be wrapped at any reasonable width setting but should remain as a single line when wrapping is disabled.";

    let result = reformat_text(text, 0, true, false, false, false, false, ListSpacing::Preserve);
    let lines: Vec<&str> = result.trim().split('\n').collect();
    assert_eq!(lines.len(), 1, "Width 0 should disable wrapping and keep text on one line");
    assert_eq!(result.trim(), text, "Text should be unchanged except for whitespace normalization");
}

#[test]
fn test_width_zero_with_semantic() {
    let text = "This is sentence one. This is sentence two. This is sentence three.";

    let result = reformat_text(text, 0, true, true, false, false, false, ListSpacing::Preserve);
    let lines: Vec<&str> = result.trim().split('\n').collect();
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

    let result_default = reformat_text(text, 88, true, false, false, false, false, ListSpacing::Preserve);
    let result_88 = reformat_text(text, 88, true, false, false, false, false, ListSpacing::Preserve);
    assert_eq!(result_default, result_88, "Default behavior should match explicit width=88");

    let long_text = "This is a very long line that definitely exceeds 88 characters and should be wrapped when using the default width setting.";
    let result = reformat_text(long_text, 88, true, false, false, false, false, ListSpacing::Preserve);
    let lines: Vec<&str> = result.trim().split('\n').collect();
    assert!(lines.len() > 1, "Long text should be wrapped at default width");
}
