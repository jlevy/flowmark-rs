use flowmark::config::ListSpacing;
use flowmark::fill_markdown;

fn fmt(input: &str) -> String {
    fill_markdown(input, true, 88, true, false, false, false, None, ListSpacing::Preserve)
}

fn fmt_nodedent(input: &str) -> String {
    fill_markdown(input, false, 88, true, false, false, false, None, ListSpacing::Preserve)
}

fn fmt_loose(input: &str) -> String {
    fill_markdown(input, true, 88, true, false, false, false, None, ListSpacing::Loose)
}

#[test]
fn test_heading_spacing_basic() {
    let input = "## Heading One\nFirst paragraph.\n\n### Heading Two\nSecond paragraph.";
    let expected = "## Heading One\n\nFirst paragraph.\n\n### Heading Two\n\nSecond paragraph.\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn test_heading_spacing_before_list() {
    let input = "## Section Title\n- First item\n- Second item";
    let expected = "## Section Title\n\n- First item\n\n- Second item\n";
    assert_eq!(fmt_loose(input), expected);
}

#[test]
fn test_heading_spacing_before_quote() {
    let input = "## Section Title\n> This is a quote.";
    let expected = "## Section Title\n\n> This is a quote.\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn test_heading_spacing_before_code() {
    let input = "## Section Title\n```python\nprint(\"hello\")\n```";
    let expected = "## Section Title\n\n```python\nprint(\"hello\")\n```\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn test_heading_with_hard_break() {
    let input = "# Comment before\\\ncode()\n# Another comment\\\nmore_code()";
    let expected = "# Comment before\\\ncode()\n# Another comment\\\nmore_code()\n";
    assert_eq!(fmt_nodedent(input), expected);
}

#[test]
fn test_hard_breaks_in_paragraphs() {
    let input = "First line\\\nsecond line\\\nthird line";
    let expected = "First line\\\nsecond line\\\nthird line\n";
    assert_eq!(fmt_nodedent(input), expected);
}

#[test]
fn test_hard_breaks_after_comments() {
    let input = "# Write data\\\ntemp_file.write(...)\n# Ensure data is on disk\\\ntemp_file.flush()\n# Close the file\\\ntemp_file.close()";
    let expected = "# Write data\\\ntemp_file.write(...)\n# Ensure data is on disk\\\ntemp_file.flush()\n# Close the file\\\ntemp_file.close()\n";
    assert_eq!(fmt_nodedent(input), expected);
}

#[test]
fn test_hard_breaks_in_list_items() {
    let input = "- First line of item\\\n  second line of item\\\n  third line of item";
    let expected = "- First line of item\\\n  second line of item\\\n  third line of item\n";
    assert_eq!(fmt(input), expected);
}

// ===== Tests ported from Python test_heading_spacing.py =====

#[test]
fn test_heading_with_hard_break_in_list() {
    let input = "- Item with heading and hard break:\n  ## Heading\\\n  continuation text";
    let expected = "- Item with heading and hard break:\n  ## Heading\\\n  continuation text\n";
    assert_eq!(fmt(input), expected);
}
