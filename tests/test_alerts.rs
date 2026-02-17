use flowmark::formats::flowmark_markdown::ListSpacing;
use flowmark::linewrapping::markdown_filling::fill_markdown;
use flowmark::linewrapping::text_filling::DEFAULT_WRAP_WIDTH;

fn fm(text: &str) -> String {
    fill_markdown(text, true, DEFAULT_WRAP_WIDTH, false, false, false, None, ListSpacing::default())
}

#[test]
fn test_basic_note_alert() {
    let input = "> [!NOTE]\n> This is a note alert.";
    let expected = "> [!NOTE]\n> This is a note alert.\n";
    assert_eq!(fm(input), expected);
}

#[test]
fn test_all_valid_alert_types() {
    for alert_type in &["NOTE", "TIP", "IMPORTANT", "WARNING", "CAUTION"] {
        let input = format!("> [!{alert_type}]\n> Content for {} alert.", alert_type.to_lowercase());
        let result = fm(&input);
        assert!(result.contains(&format!("> [!{alert_type}]")), "Alert type {alert_type} was not preserved");
        assert!(result.contains(&format!("{} alert", alert_type.to_lowercase())), "Content for {alert_type} was lost");
        assert!(result.starts_with('>'), "Quote formatting lost for {alert_type}");
    }
}

#[test]
fn test_lowercase_alert_normalized_to_uppercase() {
    let input = "> [!note]\n> This lowercase alert should be normalized.";
    let result = fm(input);
    assert!(result.contains("> [!NOTE]"));
    assert!(!result.contains("> [!note]"));
    assert!(result.contains("normalized"));
}

#[test]
fn test_misspelled_alert_preserves_quote() {
    let test_cases = vec![
        ("> [!NOOT]\n> Content here", "[!NOOT]"),
        ("> [!WARNNG]\n> Content here", "[!WARNNG]"),
        ("> [!NOTEE]\n> Content here", "[!NOTEE]"),
        ("> [!HINT]\n> Content here", "[!HINT]"),
    ];

    for (input, misspelled_type) in test_cases {
        let result = fm(input);
        assert!(result.starts_with('>'), "Quote formatting lost for {misspelled_type}");
        assert!(result.contains(misspelled_type), "Content lost for {misspelled_type}");
        assert!(result.contains("Content here"), "Body content lost for {misspelled_type}");
    }
}

#[test]
fn test_unknown_alert_types_preserve_quote() {
    let test_cases = vec![
        "> [!FOO]\n> Foo type",
        "> [!CUSTOM]\n> Custom type",
        "> [!INFO]\n> Info type",
        "> [!DANGER]\n> Danger type",
    ];

    for input in test_cases {
        let result = fm(input);
        assert!(result.starts_with('>'), "Quote formatting lost for: {}", &input[..20]);
    }
}

#[test]
fn test_alert_with_multiline_content() {
    let input = "> [!NOTE]\n> First line of content.\n> Second line of content.\n> Third line of content.";
    let result = fm(input);
    assert!(result.contains("> [!NOTE]"));
    assert!(result.contains("First line"));
    assert!(result.contains("content"));
}

#[test]
fn test_alert_with_multiple_paragraphs() {
    let input = "> [!TIP]\n> First paragraph.\n> \n> Second paragraph.";
    // Note: blank lines in quotes are rendered as ">" or "> " depending on implementation
    let result = fm(input);
    assert!(result.contains("> [!TIP]"));
    assert!(result.contains("First paragraph"));
    assert!(result.contains("Second paragraph"));
}

#[test]
fn test_alert_with_code_block() {
    let input = "> [!WARNING]\n> Be careful with this code:\n> \n> ```python\n> dangerous_operation()\n> ```";
    let result = fm(input);
    assert!(result.contains("> [!WARNING]"));
    assert!(result.contains("```python"));
    assert!(result.contains("dangerous_operation()"));
}

#[test]
fn test_alert_with_list() {
    let input = "> [!IMPORTANT]\n> Remember:\n> \n> - First item\n> - Second item";
    let result = fm(input);
    assert!(result.contains("> [!IMPORTANT]"));
    assert!(result.contains("First item"));
    assert!(result.contains("Second item"));
}

#[test]
fn test_multiple_alerts_in_document() {
    let input = "> [!NOTE]\n> First note.\n\nSome text between.\n\n> [!WARNING]\n> A warning.";
    let result = fm(input);
    assert!(result.contains("> [!NOTE]"));
    assert!(result.contains("> [!WARNING]"));
    assert!(result.contains("First note"));
    assert!(result.contains("A warning"));
    assert!(result.contains("Some text between"));
}

#[test]
fn test_alert_after_heading() {
    let input = "## Section Title\n\n> [!NOTE]\n> Important note for this section.";
    let result = fm(input);
    assert!(result.contains("## Section Title"));
    assert!(result.contains("> [!NOTE]"));
    assert!(result.contains("Important note"));
}

#[test]
fn test_regular_quote_still_works() {
    let input = "> This is a regular quote.\n> It has multiple lines.";
    let result = fm(input);
    assert!(result.starts_with('>'));
    assert!(result.contains("regular quote"));
}
