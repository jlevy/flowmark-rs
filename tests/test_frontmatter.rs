use flowmark::formats::flowmark_markdown::ListSpacing;
use flowmark::formats::frontmatter::{has_frontmatter, split_frontmatter};
use flowmark::linewrapping::markdown_filling::fill_markdown;
use flowmark::linewrapping::text_filling::DEFAULT_WRAP_WIDTH;

fn fm_semantic(text: &str) -> String {
    fill_markdown(text, true, DEFAULT_WRAP_WIDTH, false, false, false, None, ListSpacing::default())
}

#[test]
fn test_split_frontmatter() {
    let (fm, content) = split_frontmatter("");
    assert_eq!(fm, "");
    assert_eq!(content, "");

    let text = "# Heading\n\nThis is content.";
    let (fm, content) = split_frontmatter(text);
    assert_eq!(fm, "");
    assert_eq!(content, text);

    let text = "---\ntitle: Test\ndate: 2023-01-01\n---\n\n# Heading\n\nThis is content.";
    let (fm, content) = split_frontmatter(text);
    assert_eq!(fm, "---\ntitle: Test\ndate: 2023-01-01\n---\n");
    assert_eq!(content, "\n# Heading\n\nThis is content.");
}

#[test]
fn test_has_frontmatter() {
    assert!(!has_frontmatter(""));
    assert!(!has_frontmatter("# No frontmatter"));
    assert!(has_frontmatter("---\ntitle: Test\n---\n\nContent"));
}

#[test]
fn test_markdown_with_frontmatter() {
    let input = "---\ntitle: Test Document\ndate: 2023-01-01\nauthor: Test Author\n---\n\n# Heading\n\nThis is sentence one. This is sentence two.";
    let result = fm_semantic(input);

    let (frontmatter, _) = split_frontmatter(&result);
    let expected_fm = "---\ntitle: Test Document\ndate: 2023-01-01\nauthor: Test Author\n---\n";
    assert_eq!(frontmatter, expected_fm);

    assert!(result.contains("# Heading"));
    assert!(result.contains("This is sentence one."));
    assert!(result.contains("This is sentence two."));
}
