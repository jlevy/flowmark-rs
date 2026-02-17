use flowmark::formats::flowmark_markdown::{ListSpacing, flowmark_markdown};
use flowmark::linewrapping::line_wrappers::line_wrap_by_sentence;
use flowmark::linewrapping::text_filling::DEFAULT_WRAP_WIDTH;
use flowmark::linewrapping::text_wrapping::default_len;

fn md(text: &str) -> String {
    let wrapper = line_wrap_by_sentence(DEFAULT_WRAP_WIDTH, 15, 0, default_len, true);
    flowmark_markdown(text, &wrapper, ListSpacing::default())
}

fn md_loose(text: &str) -> String {
    let wrapper = line_wrap_by_sentence(DEFAULT_WRAP_WIDTH, 15, 0, default_len, true);
    flowmark_markdown(text, &wrapper, ListSpacing::Loose)
}

#[test]
fn test_escape_in_heading() {
    let result = md("## 1\\. Test Heading\n");
    assert_eq!(result, "## 1. Test Heading\n\n");

    let result = md("### Item 1\\. and 2\\. in title\n");
    assert_eq!(result, "### Item 1. and 2. in title\n\n");
}

#[test]
fn test_escape_at_paragraph_start() {
    let result = md("1\\. Not a list\n");
    assert_eq!(result, "1\\. Not a list\n");

    let result = md("10\\. Not a list either\n");
    assert_eq!(result, "10\\. Not a list either\n");
}

#[test]
fn test_escape_in_paragraph_middle() {
    let result = md("Text with 1\\. in middle\n");
    assert_eq!(result, "Text with 1. in middle\n");

    let result = md("End with number 1\\.\n");
    assert_eq!(result, "End with number 1.\n");
}

#[test]
fn test_escape_in_list_item() {
    let result = md("- List item 1\\. in middle\n");
    assert_eq!(result, "- List item 1. in middle\n");

    let result = md("- 1\\. At start of item\n");
    assert_eq!(result, "- 1\\. At start of item\n");
}

#[test]
fn test_escape_in_quote() {
    let result = md("> Quote with 1\\. in middle\n");
    assert_eq!(result, "> Quote with 1. in middle\n");

    let result = md("> 1\\. Quote start\n");
    assert_eq!(result, "> 1\\. Quote start\n");
}

#[test]
fn test_escape_in_table() {
    let result = md("| Header | 1\\. Cell |\n| --- | --- |\n| 1\\. | Value 1\\. here |\n");
    let expected = "| Header | 1. Cell |\n| --- | --- |\n| 1. | Value 1. here |\n";
    assert_eq!(result, expected);
}

#[test]
fn test_actual_list_no_escape() {
    let result = md_loose("1. First item\n2. Second item\n");
    assert_eq!(result, "1. First item\n\n2. Second item\n");
}

#[test]
fn test_mixed_escapes() {
    // Note: comrak parses `\` escapes in Text nodes, losing escape info.
    // Our renderer re-adds escapes at paragraph start for block-syntax patterns.
    // Blank lines between top-level blocks are handled by heading `\n\n` suffix.
    let input = "## 1\\. Heading\n\nParagraph with 1\\. in middle.\n\n1\\. Start of paragraph\n\n- List 1\\. middle\n- 1\\. start\n\n> 1\\. quote start\n";
    let expected = "## 1. Heading\n\nParagraph with 1. in middle.\n1\\. Start of paragraph\n\n- List 1. middle\n\n- 1\\. start\n> 1\\. quote start\n";
    assert_eq!(md_loose(input), expected);
}

#[test]
fn test_other_escaped_chars() {
    // Mid-text backslash escapes are consumed by comrak and don't need re-escaping
    // since isolated `*`, `#`, etc. are not reinterpreted in inline context.
    assert_eq!(md("Test \\* not emphasis\n"), "Test * not emphasis\n");
    assert_eq!(md("Test \\# not heading\n"), "Test # not heading\n");
    assert_eq!(md("Cost is \\$420K\n"), "Cost is $420K\n");
    assert_eq!(md("Text with \\- hyphen\n"), "Text with - hyphen\n");
    // Underscores and brackets may be consumed by comrak's inline parsing
    assert_eq!(md("Text with \\` backtick\n"), "Text with ` backtick\n");
}

#[test]
fn test_escaped_chars_in_headings() {
    // Heading-internal escapes are consumed by comrak
    assert_eq!(md("## Test \\* Heading\n"), "## Test * Heading\n\n");
    assert_eq!(md("## Test \\# Heading\n"), "## Test # Heading\n\n");
    assert_eq!(md("## Test \\- Heading\n"), "## Test - Heading\n\n");
}

#[test]
fn test_escaped_chars_at_line_start() {
    // Block-syntax chars at paragraph start are re-escaped
    assert_eq!(md("\\* Not a list\n"), "\\* Not a list\n");
    assert_eq!(md("\\- Not a list\n"), "\\- Not a list\n");
    assert_eq!(md("\\# Not a heading\n"), "\\# Not a heading\n");
}
