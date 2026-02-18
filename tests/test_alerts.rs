use flowmark::config::ListSpacing;
use flowmark::fill_markdown;

fn fmt(input: &str) -> String {
    fill_markdown(input, true, 88, true, false, false, false, None, ListSpacing::Preserve)
}

#[test]
fn test_basic_note_alert() {
    let input = "> [!NOTE]\n> This is a note alert.";
    let expected = "> [!NOTE]\n> This is a note alert.\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn test_all_valid_alert_types() {
    let alert_types = ["NOTE", "TIP", "IMPORTANT", "WARNING", "CAUTION"];
    for alert_type in &alert_types {
        let input =
            format!("> [!{alert_type}]\n> Content for {} alert.", alert_type.to_lowercase());
        let result = fmt(&input);
        assert!(
            result.contains(&format!("> [!{alert_type}]")),
            "Alert type {alert_type} was not preserved"
        );
        assert!(
            result.contains(&format!("{} alert", alert_type.to_lowercase())),
            "Content for {alert_type} was lost"
        );
        assert!(result.starts_with('>'), "Quote formatting lost for {alert_type}");
    }
}

#[test]
fn test_lowercase_alert_normalized_to_uppercase() {
    let input = "> [!note]\n> This lowercase alert should be normalized.";
    let result = fmt(input);
    assert!(result.contains("> [!NOTE]"));
    assert!(!result.contains("> [!note]"));
    assert!(result.contains("normalized"));
}

#[test]
fn test_misspelled_alert_preserves_quote() {
    let test_cases = [
        ("> [!NOOT]\n> Content here", "[!NOOT]"),
        ("> [!WARNNG]\n> Content here", "[!WARNNG]"),
        ("> [!WARNUNG]\n> Content here", "[!WARNUNG]"),
        ("> [!NOTEE]\n> Content here", "[!NOTEE]"),
        ("> [!HINT]\n> Content here", "[!HINT]"),
    ];
    for (input, misspelled) in &test_cases {
        let result = fmt(input);
        assert!(result.starts_with('>'), "Quote formatting lost for {misspelled}");
        assert!(result.contains(misspelled), "Content lost for {misspelled}");
        assert!(result.contains("Content here"), "Body content lost for {misspelled}");
    }
}

#[test]
fn test_unknown_alert_types_preserve_quote() {
    let test_cases = [
        "> [!FOO]\n> Foo type",
        "> [!CUSTOM]\n> Custom type",
        "> [!INFO]\n> Info type",
        "> [!DANGER]\n> Danger type",
        "> [!SUCCESS]\n> Success type",
    ];
    for input in &test_cases {
        let result = fmt(input);
        assert!(result.starts_with('>'), "Quote formatting lost for: {}", &input[..20]);
    }
}

#[test]
fn test_malformed_alert_preserves_quote() {
    let test_cases = [
        "> [NOTE]\n> Missing exclamation mark",
        "> [!NOTE\n> Missing closing bracket",
        "> ![NOTE]\n> Wrong order of symbols",
        "> [! NOTE]\n> Space after exclamation",
    ];
    for input in &test_cases {
        let result = fmt(input);
        assert!(
            result.starts_with('>'),
            "Quote formatting lost for: {}",
            &input[..30.min(input.len())]
        );
    }
}

#[test]
fn test_alert_with_multiline_content() {
    let input =
        "> [!NOTE]\n> First line of content.\n> Second line of content.\n> Third line of content.";
    let result = fmt(input);
    assert!(result.contains("> [!NOTE]"));
    assert!(result.contains("First line"));
    assert!(result.contains("content"));
}

#[test]
fn test_alert_with_multiple_paragraphs() {
    let input = "> [!TIP]\n> First paragraph.\n>\n> Second paragraph.";
    let expected = "> [!TIP]\n> First paragraph.\n> \n> Second paragraph.\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn test_alert_with_code_block() {
    let input = "> [!WARNING]\n> Be careful with this code:\n>\n> ```python\n> dangerous_operation()\n> ```";
    let result = fmt(input);
    assert!(result.contains("> [!WARNING]"));
    assert!(result.contains("```python"));
    assert!(result.contains("dangerous_operation()"));
}

#[test]
fn test_alert_with_list() {
    let input = "> [!IMPORTANT]\n> Remember:\n>\n> - First item\n> - Second item";
    let result = fmt(input);
    assert!(result.contains("> [!IMPORTANT]"));
    assert!(result.contains("First item"));
    assert!(result.contains("Second item"));
}

#[test]
fn test_multiple_alerts_in_document() {
    let input = "> [!NOTE]\n> First note.\n\nSome text between.\n\n> [!WARNING]\n> A warning.";
    let result = fmt(input);
    assert!(result.contains("> [!NOTE]"));
    assert!(result.contains("> [!WARNING]"));
    assert!(result.contains("First note"));
    assert!(result.contains("A warning"));
    assert!(result.contains("Some text between"));
}

#[test]
fn test_alert_after_heading() {
    let input = "## Section Title\n\n> [!NOTE]\n> Important note for this section.";
    let result = fmt(input);
    assert!(result.contains("## Section Title"));
    assert!(result.contains("> [!NOTE]"));
    assert!(result.contains("Important note"));
}

#[test]
fn test_regular_quote_still_works() {
    let input = "> This is a regular quote.\n> It has multiple lines.";
    let result = fmt(input);
    assert!(result.starts_with('>'));
    assert!(result.contains("regular quote"));
}

// ===== Tests ported from Python test_alerts.py =====

#[test]
fn test_empty_alert_type_preserves_quote() {
    let input = "> [!]\n> Some content";
    let result = fmt(input);
    assert!(result.starts_with('>'), "Quote formatting lost for empty alert type");
    assert!(result.contains("Some content"));
}

#[test]
fn test_quote_with_link_like_content() {
    let input = "> Check out [!this link](https://example.com) for more info.";
    let result = fmt(input);
    assert!(result.starts_with('>'));
    assert!(result.contains("this link") || result.contains("[!this link]"));
}
