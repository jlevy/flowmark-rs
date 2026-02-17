use flowmark::fill_markdown;
use flowmark::config::ListSpacing;

fn fmt(input: &str) -> String {
    fill_markdown(input, true, 88, true, false, false, false, None, ListSpacing::Preserve)
}

fn fmt_loose(input: &str) -> String {
    fill_markdown(input, true, 88, true, false, false, false, None, ListSpacing::Loose)
}

#[test]
fn test_escape_in_heading() {
    // Escapes in headings are removed (period after number not ambiguous)
    let result = fmt("## 1\\. Test Heading\n");
    assert!(result.contains("## 1. Test Heading"), "Escape should be removed in heading: got {:?}", result);

    let result = fmt("### Item 1\\. and 2\\. in title\n");
    assert!(result.contains("### Item 1. and 2. in title"), "Escapes should be removed in heading: got {:?}", result);
}

#[test]
fn test_escape_at_paragraph_start() {
    // Escape at paragraph start preserved to prevent list interpretation
    let result = fmt("1\\. Not a list\n");
    assert!(result.contains("1\\."), "Escape should be preserved at paragraph start: got {:?}", result);

    let result = fmt("10\\. Not a list either\n");
    assert!(result.contains("10\\."), "Escape should be preserved at paragraph start: got {:?}", result);
}

#[test]
fn test_escape_in_paragraph_middle() {
    // Escapes in middle of paragraph removed (not ambiguous)
    let result = fmt("Text with 1\\. in middle\n");
    assert!(result.contains("Text with 1. in middle"), "Escape should be removed mid-paragraph: got {:?}", result);

    let result = fmt("End with number 1\\.\n");
    assert!(result.contains("End with number 1."), "Escape should be removed at end: got {:?}", result);
}

#[test]
fn test_actual_list_no_escape() {
    // Real ordered lists without escapes stay as lists
    let result = fmt_loose("1. First item\n2. Second item\n");
    assert!(result.contains("1. First item"), "List items should be preserved");
    assert!(result.contains("2. Second item"), "List items should be preserved");
}

#[test]
fn test_other_escaped_chars() {
    // Non-period escaped characters are preserved
    let result = fmt("Test \\* not emphasis\n");
    assert!(result.contains("\\*"), "Asterisk escape should be preserved: got {:?}", result);

    let result = fmt("Test \\# not heading\n");
    assert!(result.contains("\\#"), "Hash escape should be preserved: got {:?}", result);

    let result = fmt("Text with \\- hyphen\n");
    assert!(result.contains("\\-"), "Hyphen escape should be preserved: got {:?}", result);
}

#[test]
fn test_escaped_chars_in_headings() {
    // Non-period escapes in headings ARE preserved
    let result = fmt("## Test \\* Heading\n");
    assert!(result.contains("\\*"), "Asterisk escape should be preserved in heading: got {:?}", result);

    let result = fmt("## Test \\# Heading\n");
    assert!(result.contains("\\#"), "Hash escape should be preserved in heading: got {:?}", result);
}

#[test]
fn test_escaped_chars_at_line_start() {
    // Escaped chars at line start preserved
    let result = fmt("\\* Not a list\n");
    assert!(result.contains("\\*"), "Asterisk escape at start should be preserved: got {:?}", result);

    let result = fmt("\\- Not a list\n");
    assert!(result.contains("\\-"), "Hyphen escape at start should be preserved: got {:?}", result);
}
