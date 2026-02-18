use flowmark::config::ListSpacing;
use flowmark::fill_markdown;
use flowmark::typography::quotes::smart_quotes;
use flowmark::wrapping::tag_handling::{
    denormalize_adjacent_tags, normalize_adjacent_tags, preprocess_tag_block_spacing,
};

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
    assert!(
        result.contains(r#"kind="string""#),
        "HTML comment attribute quotes should be preserved"
    );
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
    let tags =
        [r#"{% field kind="string" id="name" %}"#, r#"{% callout type="warning" title="Note" %}"#];
    for tag in &tags {
        let result = fmt_smart(tag);
        let original_quote_count = tag.matches('"').count();
        let result_quote_count = result.matches('"').count();
        assert_eq!(
            original_quote_count, result_quote_count,
            "Pipeline should preserve straight quotes in tag: {tag}"
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

// ===== Tests ported from Python test_tag_formatting.py =====

#[test]
fn test_html_comment_multiline_closing() {
    use flowmark::wrapping::tag_handling::fix_multiline_opening_tag_with_closing;

    let text = "<!-- f:field kind='string'\nlabel='Name' --><!-- /f:field -->";
    let result = fix_multiline_opening_tag_with_closing(text);
    assert!(result.contains("-->\n<!-- /f:field -->"), "HTML closing tag not split: {result}");
}

#[test]
fn test_line_wrapper_preserves_multiline_tags() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "{% field kind=\"string\" %}\nContent here.\n{% /field %}";
    let result = wrapper(text, "", "");
    assert!(result.contains("{% field"));
    assert!(result.contains("{% /field %}"));
}

#[test]
fn test_list_item_with_tag_on_continuation_line() {
    let text = "**Phase 0 - Repository Setup:**\n\n- [ ] 0.1-0.2: Verify reference repos and initialize Bun monorepo with workspaces <!-- #kg-11h2 -->\n\n- [ ] 0.3: Configure TypeScript with strict settings (tsconfig.base.json)\n  <!-- #kg-32zz -->\n\n- [ ] 0.4: Configure Biome for formatting and linting <!-- #kg-ibof -->";
    let result = fmt(text);

    // The tag should NOT have an extra blank line before it
    assert!(
        !result.contains("\n\n  <!-- #kg-32zz -->"),
        "Extra blank line incorrectly added before tag on continuation line.\nResult:\n{result}"
    );
}

#[test]
fn test_multiline_opening_tag_closing_on_own_line() {
    use flowmark::wrapping::tag_handling::fix_multiline_opening_tag_with_closing;

    let problematic = "{% field kind='string'\nrequired=true %}{% /field %}";
    let result = fix_multiline_opening_tag_with_closing(problematic);
    assert!(result.contains("%}\n{% /field %}"), "Closing tag not on own line: {result}");
}

#[test]
fn test_multiline_tag_through_pipeline() {
    let long_tag = concat!(
        "{% field kind=\"string\" id=\"name\" label=\"Full Name\" role=\"user\" ",
        "required=true minLength=2 maxLength=100 placeholder=\"Enter your full name\" %}",
        "{% /field %}"
    );

    let result = fmt(long_tag);
    let lines: Vec<&str> = result.lines().collect();

    // Entire tag+closing is ONE token — stays on single line
    assert_eq!(lines.len(), 1, "Tag should be one line, got: {lines:?}");
    assert!(lines[0].contains("{% field") && lines[0].contains("{% /field %}"));
    assert!(result.contains("minLength=2"), "minLength=2 should stay together");
    assert!(result.contains("maxLength=100"), "maxLength=100 should stay together");
}

#[test]
fn test_multiline_tag_with_surrounding_text() {
    let text = "Here is some \"quoted text\" that should be converted.\n\n{% field kind=\"string\" examples=[\"one\",\n\"two\"] %}\n{% /field %}\n\nAnd more \"quoted text\" here.";

    let result = smart_quotes(text);

    assert!(result.contains("\u{201c}quoted text\u{201d}"), "Prose quotes should be converted");
    assert!(result.contains("kind=\"string\""), "Tag attribute quotes should be preserved");
    assert!(result.contains("examples=[\"one\","), "Array quotes should be preserved");
    assert!(result.contains("\"two\"]"), "Array quotes on continuation lines should be preserved");
}

#[test]
fn test_preprocess_tag_block_spacing_inline_tags() {
    // List items with inline tags — should NOT add blank lines between items
    let text = "{% field %}\n\n- Item 1 {% #item1 %}\n- Item 2 {% #item2 %}\n\n{% /field %}";
    let result = preprocess_tag_block_spacing(text);

    assert!(
        result.contains("{% #item1 %}\n- Item 2"),
        "Incorrectly added blank between items: {result}"
    );
}

#[test]
fn test_selection_field_with_task_list() {
    let input = "{% field kind=\"single_select\" id=\"priority\" label=\"Priority\" role=\"user\"\nrequired=true %}\n- [ ] Low {% #low %}\n- [ ] High {% #high %}\n{% /field %}";
    let result = fmt(input);

    assert!(result.contains("{% #low %}"));
    assert!(result.contains("{% #high %}"));
    assert!(result.contains("{% /field %}"));
}

#[test]
fn test_single_line_paired_tags_not_split() {
    use flowmark::wrapping::tag_handling::fix_multiline_opening_tag_with_closing;

    let single_line = "{% field kind='string' %}{% /field %}";
    let result = fix_multiline_opening_tag_with_closing(single_line);
    assert_eq!(result, single_line, "Single-line tag was incorrectly split: {result}");
}

#[test]
fn test_smart_quotes_multiline_tag_with_prose() {
    let text = "He said \"yes\" to the form.\n\n{% field kind=\"string\"\nlabel=\"Full Name\"\nrequired=true %}\n{% /field %}\n\nShe replied \"no\" later.";
    let result = smart_quotes(text);

    assert!(result.contains("\u{201c}yes\u{201d}"));
    assert!(result.contains("\u{201c}no\u{201d}"));
    assert!(result.contains("kind=\"string\""));
    assert!(result.contains("label=\"Full Name\""));
}

#[test]
fn test_tag_newlines_preserved_in_pipeline() {
    let tag6 = "{% field\n  kind=\"number\"\n  id=\"score\"\n  label=\"Score\"\n  role=\"user\"\n  min=0\n  max=100\n  required=true\n%}\n{% /field %}";
    let result = fmt(tag6);

    assert!(result.contains("{% field") || result.contains("{% field "));
    assert!(result.contains("%}\n{% /field %}") || result.contains("%}{% /field %}"));
}

#[test]
fn test_tag_with_array_spanning_lines() {
    let tag = "{% field examples=[\"one\",\n\"two\", \"three\"] %}";
    let result = smart_quotes(tag);
    assert_eq!(result, tag, "Array spanning lines was modified: {result}");
}

#[test]
fn test_tag_with_embedded_percent_brace() {
    let tag9 = "{% field kind=\"string\" id=\"pattern_test\" label=\"Pattern\"\npattern=\"test %} not end\" required=true %}\n{% /field %}";
    let result = fmt(tag9);

    assert!(result.contains("{% field"));
    assert!(result.contains("{% /field %}"));
    assert!(result.contains("pattern=") || result.contains("pattern_test"));
}

#[test]
fn test_tag_with_object_spanning_lines() {
    let tag = "{% field config={key: \"value\",\nother: \"data\"} %}";
    let result = smart_quotes(tag);
    assert_eq!(result, tag, "Object spanning lines was modified: {result}");
}

#[test]
fn test_word_splitter_handles_multiline_tags() {
    use flowmark::wrapping::text_wrapping::html_md_word_split;

    // Single line tag — should be kept together
    let single = "{% field kind=\"string\" id=\"name\" %}";
    let tokens = html_md_word_split(single);
    assert!(tokens.contains(&single.to_string()), "Single line tag should be atomic: {tokens:?}");

    // Multi-word tag — should be coalesced
    let multi = "{% callout type=\"warning\" title=\"Note\" %}";
    let tokens = html_md_word_split(multi);
    assert!(tokens.contains(&multi.to_string()), "Multi-word tag should be coalesced: {tokens:?}");
}
