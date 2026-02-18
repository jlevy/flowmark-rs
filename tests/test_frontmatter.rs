use flowmark::config::ListSpacing;
use flowmark::fill_markdown;

// Re-export frontmatter functions for testing
fn split_frontmatter(text: &str) -> (String, String) {
    flowmark::parser::frontmatter::split_frontmatter(text)
}

fn has_frontmatter(text: &str) -> bool {
    flowmark::parser::frontmatter::has_frontmatter(text)
}

#[test]
fn test_split_frontmatter_empty() {
    let (fm, content) = split_frontmatter("");
    assert_eq!(fm, "");
    assert_eq!(content, "");
}

#[test]
fn test_split_frontmatter_no_frontmatter() {
    let text = "# Heading\n\nThis is content.";
    let (fm, content) = split_frontmatter(text);
    assert_eq!(fm, "");
    assert_eq!(content, text);
}

#[test]
fn test_split_frontmatter_proper() {
    let text = "---\ntitle: Test\ndate: 2023-01-01\n---\n\n# Heading\n\nThis is content.";
    let (fm, content) = split_frontmatter(text);
    assert_eq!(fm, "---\ntitle: Test\ndate: 2023-01-01\n---\n");
    assert_eq!(content, "\n# Heading\n\nThis is content.");
}

#[test]
fn test_split_frontmatter_empty_lines_before() {
    let text = "\n\n---\ntitle: Test\n---\n\n# Content";
    let (fm, content) = split_frontmatter(text);
    assert_eq!(fm, "---\ntitle: Test\n---\n");
    assert_eq!(content, "\n# Content");
}

#[test]
fn test_split_frontmatter_unclosed() {
    let text = "---\ntitle: Test\ndate: 2023-01-01\n";
    let (fm, content) = split_frontmatter(text);
    assert_eq!(fm, text);
    assert_eq!(content, "");
}

#[test]
fn test_has_frontmatter() {
    assert!(!has_frontmatter(""));
    assert!(!has_frontmatter("# No frontmatter"));
    assert!(has_frontmatter("---\ntitle: Test\n---\n\nContent"));
    assert!(has_frontmatter("\n\n---\ntitle: Test\n---\n"));
}

#[test]
fn test_markdown_with_frontmatter() {
    let input = "---\ntitle: Test Document\ndate: 2023-01-01\nauthor: Test Author\n---\n\n# Heading\n\nThis is sentence one. This is sentence two.";

    let normalized =
        fill_markdown(input, true, 88, true, false, false, false, None, ListSpacing::Preserve);

    // Verify the frontmatter is preserved
    let (fm, _) = split_frontmatter(&normalized);
    assert_eq!(fm, "---\ntitle: Test Document\ndate: 2023-01-01\nauthor: Test Author\n---\n");

    // Verify content is formatted
    assert!(normalized.contains("# Heading"));
    assert!(normalized.contains("This is sentence one."));
    assert!(normalized.contains("This is sentence two."));
}

/// M4 regression test: CRLF line endings in frontmatter should not corrupt content.
/// `text.lines()` strips `\r\n` but rejoins with `\n`, silently converting CRLF files.
#[test]
fn test_split_frontmatter_crlf() {
    let text = "---\r\ntitle: Test\r\n---\r\n\r\n# Content\r\n";
    let (fm, content) = split_frontmatter(text);
    // Frontmatter should be extracted (line endings may be normalized)
    assert!(!fm.is_empty(), "Frontmatter should be detected with CRLF");
    assert!(fm.contains("title: Test"), "Title should be in frontmatter");
    // Content should not be corrupted
    assert!(content.contains("# Content"), "Content should be preserved");
}

/// M4 regression test: full pipeline with CRLF input.
#[test]
fn test_markdown_with_crlf_frontmatter() {
    let input = "---\r\ntitle: CRLF Test\r\n---\r\n\r\n# Heading\r\n\r\nParagraph text.\r\n";
    let result =
        fill_markdown(input, true, 88, true, false, false, false, None, ListSpacing::Preserve);
    // Content should not be corrupted
    assert!(result.contains("# Heading"), "Heading preserved with CRLF input");
    assert!(result.contains("Paragraph text."), "Paragraph preserved with CRLF input");
}
