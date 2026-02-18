use flowmark::config::ListSpacing;
use flowmark::fill_markdown;

fn fmt(input: &str) -> String {
    fill_markdown(input, true, 88, true, true, false, false, None, ListSpacing::Loose)
}

#[test]
fn test_unbold_headings() {
    let input = "\
# **Bold Heading 1**

Some paragraph text.

## ***Bold Italic***

## **Simple Bold**

### Not Bold

#### **Partial** Bold

#### Other *partial* **bold** `code`

- **List Item Bold**

Another paragraph with **bold** text.

## **Nested `code`**

Final text.
";

    let result = fmt(input);

    // Bold removed from fully-bold heading
    assert!(result.contains("# Bold Heading 1"), "Bold should be removed from fully-bold H1");
    assert!(!result.contains("# **Bold Heading 1**"), "Should not have bold H1");

    // Bold removed from bold-italic, italic preserved
    eprintln!(
        "Result around bold-italic: {:?}",
        result.lines().find(|l| l.contains("Bold Italic"))
    );
    assert!(result.contains("## *Bold Italic*"), "Bold should be removed, italic preserved");
    assert!(!result.contains("## ***Bold Italic***"), "Should not have bold-italic");

    // Simple bold heading
    assert!(result.contains("## Simple Bold"), "Bold should be removed from simple bold heading");
    assert!(!result.contains("## **Simple Bold**"), "Should not have bold H2");

    // Not-bold heading unchanged
    assert!(result.contains("### Not Bold"), "Non-bold heading should be unchanged");

    // Partial bold stays unchanged (not entirely bold)
    assert!(result.contains("#### **Partial** Bold"), "Partially bold heading should be unchanged");

    // Mixed formatting stays unchanged
    assert!(
        result.contains("#### Other *partial* **bold** `code`"),
        "Mixed formatting heading should be unchanged"
    );

    // Bold in list items not affected
    assert!(result.contains("**List Item Bold**"), "Bold in list items should be unchanged");

    // Bold in paragraphs not affected
    assert!(result.contains("**bold** text"), "Bold in paragraphs should be unchanged");

    // Nested code in bold heading - bold removed, code preserved
    assert!(
        result.contains("## Nested `code`"),
        "Bold should be removed, code preserved in heading"
    );
}
