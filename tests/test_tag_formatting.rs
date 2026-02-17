use flowmark::fill_markdown;
use flowmark::typography::quotes::smart_quotes;
use flowmark::config::ListSpacing;
use flowmark::wrapping::tag_handling::{normalize_adjacent_tags, denormalize_adjacent_tags, preprocess_tag_block_spacing};

fn fmt_smart(input: &str) -> String {
    fill_markdown(input, true, 88, true, false, true, false, None, ListSpacing::Preserve)
}

fn fmt(input: &str) -> String {
    fill_markdown(input, true, 88, true, false, false, false, None, ListSpacing::Preserve)
}

// ===== Smart quotes and tag interaction tests =====

#[test]
fn test_smart_quotes_preserves_quotes_in_tags() {
    // Quotes inside template tags should NOT be converted
    let input = r#"{% field kind="string" %}"#;
    let result = smart_quotes(input);
    assert_eq!(result, input, "Template tag quotes should not be converted");
}

#[test]
fn test_smart_quotes_preserves_quotes_in_include() {
    let input = r#"{% include "header.html" %}"#;
    let result = smart_quotes(input);
    assert_eq!(result, input);
}

#[test]
fn test_smart_quotes_preserves_single_quotes_in_attributes() {
    let input = "{% field kind='string' label='Name' %}";
    let result = smart_quotes(input);
    assert_eq!(result, input);
}

#[test]
fn test_smart_quotes_preserves_quotes_in_jinja_comments() {
    let input = r#"{# "quoted text" in comment #}"#;
    let result = smart_quotes(input);
    assert_eq!(result, input);
}

#[test]
fn test_smart_quotes_preserves_quotes_in_html_comments() {
    let input = r#"<!-- f:field kind="string" -->"#;
    let result = smart_quotes(input);
    assert_eq!(result, input);
}

#[test]
fn test_smart_quotes_preserves_apostrophe_in_jinja_variable() {
    let input = "{{ won't }}";
    let result = smart_quotes(input);
    assert_eq!(result, input);
}

#[test]
fn test_smart_quotes_converts_prose_but_not_tags() {
    let input = r#"She said "hello" and {% field label="Name" %} was set."#;
    let result = smart_quotes(input);
    assert!(result.contains("\u{201c}hello\u{201d}"), "Prose quotes should be converted");
    assert!(result.contains(r#"label="Name""#), "Tag quotes should be preserved");
}

#[test]
fn test_jinja_variable_tags_in_prose() {
    let input = r#"Hello {{ user.name }}, welcome to "our site"."#;
    let result = smart_quotes(input);
    assert!(result.contains("{{ user.name }}"), "Jinja variable tag should be preserved");
    assert!(result.contains("\u{201c}our site\u{201d}"), "Prose quotes should be converted");
}

#[test]
fn test_jinja_comment_tags() {
    let input = r#"{# TODO: fix "this" later #} Some "quoted" text."#;
    let result = smart_quotes(input);
    assert!(result.contains("\u{201c}quoted\u{201d}"), "Prose quotes should be converted");
}

#[test]
fn test_html_comment_tags_with_quotes() {
    let input = r#"<!-- f:field kind="string" --> Some "quoted" text <!-- /f:field -->"#;
    let result = smart_quotes(input);
    assert!(result.contains("\u{201c}quoted\u{201d}"), "Prose quotes should be converted");
    assert!(result.contains(r#"kind="string""#), "HTML comment attribute quotes should be preserved");
}

// ===== Adjacent tag tests =====

#[test]
fn test_adjacent_closing_tags_roundtrip() {
    let input = "{% field %}{% /field %}";
    let normalized = normalize_adjacent_tags(input);
    let denormalized = denormalize_adjacent_tags(&normalized);
    assert_eq!(denormalized, input, "Adjacent tags should roundtrip correctly");
}

// ===== Pipeline tests with tags =====

#[test]
fn test_pipeline_preserves_tag_quotes() {
    let tags = [
        r#"{% field kind="string" id="name" %}"#,
        r#"{% callout type="warning" title="Note" %}"#,
    ];
    for tag in &tags {
        let result = fmt_smart(tag);
        let original_quote_count = tag.matches('"').count();
        let result_quote_count = result.matches('"').count();
        assert_eq!(
            original_quote_count, result_quote_count,
            "Pipeline should preserve straight quotes in tag: {}",
            tag
        );
    }
}

#[test]
fn test_smart_quotes_with_nunjucks_raw_block() {
    let input = "{% raw %}This {{ won't }} be {% processed %}{% endraw %}";
    let result = smart_quotes(input);
    assert_eq!(result, input, "Raw block content should be preserved");
}

// ===== Preprocess tag block spacing tests =====

#[test]
fn test_preprocess_tag_block_spacing_lists() {
    let input = "{% field %}\n- Item 1\n- Item 2\n{% /field %}";
    let result = preprocess_tag_block_spacing(input);
    // Should add blank lines around block content within tags
    assert!(result.contains("%}\n\n-"), "Should have blank line after opening tag before list");
    assert!(result.contains("\n\n{% /field"), "Should have blank line before closing tag");
}

#[test]
fn test_preprocess_tag_block_spacing_tables() {
    let input = "{% table %}\n| H1 | H2 |\n| --- | --- |\n| A | B |\n{% /table %}";
    let result = preprocess_tag_block_spacing(input);
    assert!(result.contains("%}\n\n|"), "Should have blank line after opening tag before table");
}

#[test]
fn test_preprocess_tag_block_spacing_already_spaced() {
    let input = "{% field %}\n\n- Item 1\n- Item 2\n\n{% /field %}";
    let result = preprocess_tag_block_spacing(input);
    assert!(!result.contains("\n\n\n"), "Should not introduce triple newlines");
}

#[test]
fn test_fill_markdown_with_list_in_tags() {
    let input = "{% field kind='selection' %}\n- [ ] Low {% #low %}\n- [ ] Medium {% #medium %}\n- [ ] High {% #high %}\n{% /field %}";
    let result = fmt(input);
    // Opening tag should have blank line before list
    assert!(result.contains("{% field"), "Should preserve opening tag");
    assert!(result.contains("{% /field %}"), "Should preserve closing tag");
}
