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

// ===== Tests ported from Python test_escape_handling.py =====

#[test]
fn test_escape_in_list_item() {
    // In middle of list item — remove escape
    let result = fmt("- List item 1\\. in middle\n");
    assert!(result.contains("1. in middle"), "Escape should be removed in list item middle: got {:?}", result);
}

#[test]
#[ignore] // Known bug: fmr-2tll — escape at start of list item content not preserved
fn test_escape_in_list_item_start_preserved() {
    // At start of list item — preserve escape
    let result = fmt("- 1\\. At start of item\n");
    assert!(result.contains("1\\."), "Escape should be preserved at list item start: got {:?}", result);
}

#[test]
fn test_escape_in_quote() {
    // In middle of quote line — remove escape
    let result = fmt("> Quote with 1\\. in middle\n");
    assert!(result.contains("1. in middle"), "Escape should be removed in quote middle: got {:?}", result);

    // At start of quote content — preserve escape
    let result = fmt("> 1\\. Quote start\n");
    assert!(result.contains("1\\."), "Escape should be preserved at quote start: got {:?}", result);
}

#[test]
fn test_escape_in_table() {
    let input = "| Header | 1\\. Cell |\n| --- | --- |\n| 1\\. | Value 1\\. here |\n";
    let result = fmt(input);

    assert!(result.contains("1. Cell"), "Escape should be removed in table header: got {:?}", result);
    assert!(result.contains("Value 1. here"), "Escape should be removed in table cell: got {:?}", result);
}

#[test]
#[ignore] // Known bug: fmr-2tll — escape at start of list item content not preserved
fn test_mixed_escapes() {
    let input = "## 1\\. Heading\n\nParagraph with 1\\. in middle.\n\n1\\. Start of paragraph\n\n- List 1\\. middle\n- 1\\. start\n\n> 1\\. quote start\n";
    let expected = "## 1. Heading\n\nParagraph with 1. in middle.\n\n1\\. Start of paragraph\n\n- List 1. middle\n\n- 1\\. start\n\n> 1\\. quote start\n";
    assert_eq!(fmt_loose(input), expected);
}

#[test]
fn test_mixed_escapes_comprehensive() {
    let input = "## 1\\. Heading with \\* asterisk\n\nText with 1\\. period and \\* asterisk and \\# hash.\n\n1\\. Not a list (period escape kept)\n\\* Not a list (asterisk escape kept)\n\nCost: \\$100 (dollar escape kept)\n";
    let expected = "## 1. Heading with \\* asterisk\n\nText with 1. period and \\* asterisk and \\# hash.\n\n1\\. Not a list (period escape kept) \\* Not a list (asterisk escape kept)\n\nCost: \\$100 (dollar escape kept)\n";
    assert_eq!(fmt(input), expected);
}
