use flowmark::wrapping::text_wrapping::{
    html_md_word_split, markdown_escape_word, simple_word_split, wrap_paragraph,
    wrap_paragraph_lines,
};

#[test]
fn test_markdown_escape_word_function() {
    // Cases that should be escaped
    assert_eq!(markdown_escape_word("-"), "\\-");
    assert_eq!(markdown_escape_word("+"), "\\+");
    assert_eq!(markdown_escape_word("*"), "\\*");
    assert_eq!(markdown_escape_word(">"), "\\>");
    assert_eq!(markdown_escape_word("#"), "\\#");
    assert_eq!(markdown_escape_word("##"), "\\##");
    assert_eq!(markdown_escape_word("1."), "1\\.");
    assert_eq!(markdown_escape_word("10."), "10\\.");
    assert_eq!(markdown_escape_word("1)"), "1\\)");
    assert_eq!(markdown_escape_word("99)"), "99\\)");

    // Cases that should NOT be escaped
    assert_eq!(markdown_escape_word("word"), "word");
    assert_eq!(markdown_escape_word("-word"), "-word");
    assert_eq!(markdown_escape_word("word-"), "word-");
    assert_eq!(markdown_escape_word("#word"), "#word");
    assert_eq!(markdown_escape_word("word#"), "word#");
    assert_eq!(markdown_escape_word("1.word"), "1.word");
    assert_eq!(markdown_escape_word("word1."), "word1.");
    assert_eq!(markdown_escape_word("1)word"), "1)word");
    assert_eq!(markdown_escape_word("word1)"), "word1)");
    assert_eq!(markdown_escape_word("<tag>"), "<tag>");
    assert_eq!(markdown_escape_word("[link]"), "[link]");
    assert_eq!(markdown_escape_word("1"), "1");
    assert_eq!(markdown_escape_word("."), ".");
}

#[test]
fn test_wrap_paragraph_lines_markdown_escaping() {
    assert_eq!(
        wrap_paragraph_lines("- word", 10, 0, 0, true, true, None, true),
        vec!["- word"]
    );

    let text = "word - word * word + word > word # word ## word 1. word 2) word";

    assert_eq!(
        wrap_paragraph_lines(text, 5, 0, 0, true, true, None, true),
        vec![
            "word", "\\-", "word", "\\*", "word", "\\+", "word", "\\>", "word", "\\#", "word",
            "\\##", "word", "1\\.", "word", "2\\)", "word",
        ]
    );

    assert_eq!(
        wrap_paragraph_lines(text, 10, 0, 0, true, true, None, true),
        vec![
            "word -", "word *", "word +", "word >", "word #", "word ##", "word 1.", "word 2)",
            "word",
        ]
    );

    assert_eq!(
        wrap_paragraph_lines(text, 15, 0, 0, true, true, None, true),
        vec![
            "word - word *",
            "word + word >",
            "word # word ##",
            "word 1. word 2)",
            "word",
        ]
    );

    assert_eq!(
        wrap_paragraph_lines(text, 20, 0, 0, true, true, None, true),
        vec![
            "word - word * word +",
            "word > word # word",
            "\\## word 1. word 2)",
            "word",
        ]
    );

    assert_eq!(
        wrap_paragraph_lines(text, 20, 0, 0, true, true, None, false),
        vec![
            "word - word * word +",
            "word > word # word",
            "## word 1. word 2)",
            "word",
        ]
    );
}

#[test]
fn test_smart_splitter() {
    let html_text =
        "This is <span class='test'>some text</span> and <a href='#'>this is a link</a>.";
    assert_eq!(
        html_md_word_split(html_text),
        vec![
            "This",
            "is",
            "<span class='test'>some",
            "text</span>",
            "and",
            "<a href='#'>this",
            "is",
            "a",
            "link</a>.",
        ]
    );

    let md_text =
        "Here's a [Markdown link](https://example.com) and [another one](https://test.com).";
    assert_eq!(
        html_md_word_split(md_text),
        vec![
            "Here's",
            "a",
            "[Markdown link](https://example.com)",
            "and",
            "[another one](https://test.com).",
        ]
    );

    let mixed_text = "Text with <b>bold</b> and [a link](https://example.com).";
    assert_eq!(
        html_md_word_split(mixed_text),
        vec![
            "Text",
            "with",
            "<b>bold</b>",
            "and",
            "[a link](https://example.com).",
        ]
    );
}

#[test]
fn test_wrap_text() {
    let sample_text =
        "This is a sample text with a [Markdown link](https://example.com) and an <a href='#'>tag</a>. It should demonstrate the functionality of our enhanced text wrapping implementation.";

    let filled = wrap_paragraph(
        sample_text,
        40,
        ">",
        ">>",
        0,
        true,
        true,
        Some(&simple_word_split),
        false,
    );
    let filled_expected = "\
>This is a sample text with a [Markdown
>>link](https://example.com) and an <a
>>href='#'>tag</a>. It should
>>demonstrate the functionality of our
>>enhanced text wrapping implementation.";

    let filled_smart = wrap_paragraph(
        sample_text,
        40,
        ">",
        ">>",
        0,
        true,
        true,
        Some(&html_md_word_split),
        false,
    );
    let filled_smart_expected = "\
>This is a sample text with a
>>[Markdown link](https://example.com)
>>and an <a href='#'>tag</a>. It should
>>demonstrate the functionality of our
>>enhanced text wrapping implementation.";

    let filled_smart_offset = wrap_paragraph(
        sample_text,
        40,
        ">",
        ">>",
        35,
        true,
        true,
        Some(&html_md_word_split),
        false,
    );
    let filled_smart_offset_expected = "This
>>is a sample text with a
>>[Markdown link](https://example.com)
>>and an <a href='#'>tag</a>. It should
>>demonstrate the functionality of our
>>enhanced text wrapping implementation.";

    assert_eq!(filled, filled_expected);
    assert_eq!(filled_smart, filled_smart_expected);
    assert_eq!(filled_smart_offset, filled_smart_offset_expected);
}

#[test]
fn test_wrap_width() {
    let text = "You may also simply ask a question and the kmd assistant will help you. Press `?` or just press space twice, then write your question or request. Press `?` and tab to get suggested questions.";
    let width = 80;
    let wrapped = wrap_paragraph_lines(text, width, 0, 0, true, true, None, false);
    for line in &wrapped {
        assert!(
            line.chars().count() <= width,
            "Line exceeds width: {:?}",
            line
        );
    }
}

#[test]
fn test_template_tag_splitter() {
    // Markdoc-style tags
    let markdoc_text = "Text with {% if $condition %} template tags {% endif %} here.";
    let result = html_md_word_split(markdoc_text);
    assert!(result.contains(&"{% if $condition %}".to_string()));
    assert!(result.contains(&"{% endif %}".to_string()));

    // Self-closing Markdoc tags
    let self_closing = "Include {% partial file='header.md' /%} here.";
    let result = html_md_word_split(self_closing);
    assert!(result.contains(&"{% partial file='header.md' /%}".to_string()));

    // Jinja/Nunjucks comments
    let comment_text = "Text with {# this is a comment #} inline.";
    let result = html_md_word_split(comment_text);
    assert!(result.contains(&"{# this is a comment #}".to_string()));

    // Jinja/Nunjucks variables
    let variable_text = "Hello {{ user.name }} welcome.";
    let result = html_md_word_split(variable_text);
    assert!(result.contains(&"{{ user.name }}".to_string()));

    // Complex Markdoc tag with attributes
    let complex_tag = "Use {% callout type='warning' title='Note' %} for emphasis.";
    let result = html_md_word_split(complex_tag);
    assert!(result.contains(&"{% callout type='warning' title='Note' %}".to_string()));
}

#[test]
fn test_template_tag_wrapping() {
    // Template tag should stay together even if it's long
    let text_with_tag = "Some text {% callout type='warning' %} more text after the tag.";
    let result = wrap_paragraph_lines(text_with_tag, 30, 0, 0, true, true, None, true);
    let full_result = result.join(" ");
    assert!(full_result.contains("{% callout type='warning' %}"));

    // Jinja variable should stay together
    let text_with_var = "Hello {{ user.first_name }} and welcome to the site.";
    let result = wrap_paragraph_lines(text_with_var, 25, 0, 0, true, true, None, true);
    let full_result = result.join(" ");
    assert!(full_result.contains("{{ user.first_name }}"));

    // Comment should stay together
    let text_with_comment = "Text {# TODO: fix this later #} and more text here.";
    let result = wrap_paragraph_lines(text_with_comment, 20, 0, 0, true, true, None, true);
    let full_result = result.join(" ");
    assert!(full_result.contains("{# TODO: fix this later #}"));
}

#[test]
fn test_mixed_html_and_template_tags() {
    let mixed = "Text <span class='x'>html</span> and {% if $y %} template {% endif %} here.";
    let result = html_md_word_split(mixed);

    assert!(result.contains(&"<span class='x'>html</span>".to_string()));
    assert!(result.contains(&"{% if $y %}".to_string()));
    assert!(result.contains(&"{% endif %}".to_string()));
}

#[test]
fn test_inline_code_with_spaces() {
    let code = "`code with spaces`";
    let text = format!("Some {code} here.");
    let result = html_md_word_split(&text);
    assert!(result.contains(&code.to_string()));
}

#[test]
fn test_html_comments_kept_together() {
    let comment = "<!-- a comment -->";
    let text = format!("Text with {comment} inline.");
    let result = html_md_word_split(&text);
    assert!(result.contains(&comment.to_string()));

    let long_comment = "<!-- this is a longer comment with more words -->";
    let text2 = format!("Before {long_comment} after.");
    let result2 = html_md_word_split(&text2);
    assert!(result2.contains(&long_comment.to_string()));
}

#[test]
fn test_single_word_inline_code_not_coalesced() {
    let text = "access env vars via `getRequiredEnv()` and must live in files";
    let result = html_md_word_split(text);
    assert!(result.contains(&"`getRequiredEnv()`".to_string()));
    let code_token = result
        .iter()
        .find(|r| r.contains("`getRequiredEnv()`"))
        .unwrap();
    assert_eq!(code_token, "`getRequiredEnv()`");
    assert!(result.contains(&"and".to_string()));
}

#[test]
fn test_line_wrap_to_width_with_markdown_breaks() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    // Test trailing space line breaks
    let text_with_spaces = "This line ends with spaces  \nThis is a new line";
    let wrapped_spaces = wrapper(text_with_spaces, "", "");
    assert_eq!(
        wrapped_spaces,
        "This line ends with spaces\\\nThis is a new line"
    );

    // Test backslash line breaks
    let text_with_backslash = "This line ends with backslash\\\nThis is a new line";
    let wrapped_backslash = wrapper(text_with_backslash, "", "");
    assert_eq!(
        wrapped_backslash,
        "This line ends with backslash\\\nThis is a new line"
    );

    // Test single segment (no line breaks)
    let single_segment = "Text with no breaks";
    let wrapped_single = wrapper(single_segment, "> ", "  ");
    assert_eq!(wrapped_single, "> Text with no breaks");
}

#[test]
fn test_adjacent_jinja_tags_no_space() {
    use flowmark::wrapping::line_wrappers::{line_wrap_by_sentence, line_wrap_to_width};
    use flowmark::wrapping::tag_handling::{denormalize_adjacent_tags, normalize_adjacent_tags};
    use flowmark::config::DEFAULT_MIN_LINE_LEN;

    let original = "{% field kind='string' %}{% /field %}";
    let normalized = normalize_adjacent_tags(original);
    assert_eq!(normalized, "{% field kind='string' %} {% /field %}");
    let denormalized = denormalize_adjacent_tags(&normalized);
    assert_eq!(denormalized, original);

    let wrapper1 = line_wrap_to_width(80, true);
    let result1 = wrapper1(original, "", "");
    assert_eq!(result1, original);

    let wrapper2 = line_wrap_by_sentence(80, DEFAULT_MIN_LINE_LEN, true);
    let result2 = wrapper2(original, "", "");
    assert_eq!(result2, original);
}

#[test]
fn test_adjacent_html_comment_tags_no_space() {
    use flowmark::wrapping::line_wrappers::{line_wrap_by_sentence, line_wrap_to_width};
    use flowmark::wrapping::tag_handling::{denormalize_adjacent_tags, normalize_adjacent_tags};
    use flowmark::config::DEFAULT_MIN_LINE_LEN;

    let original = "<!-- f:field kind=\"string\" id=\"name\" --><!-- /f:field -->";
    let normalized = normalize_adjacent_tags(original);
    assert!(normalized.contains(" <!-- /f:field -->"));
    let denormalized = denormalize_adjacent_tags(&normalized);
    assert_eq!(denormalized, original);

    let wrapper1 = line_wrap_to_width(80, true);
    let result1 = wrapper1(original, "", "");
    assert_eq!(result1, original);

    let wrapper2 = line_wrap_by_sentence(80, DEFAULT_MIN_LINE_LEN, true);
    let result2 = wrapper2(original, "", "");
    assert_eq!(result2, original);
}

#[test]
fn test_adjacent_tags_full_pipeline() {
    use flowmark::fill_markdown;
    use flowmark::config::ListSpacing;

    // Jinja tags
    let jinja_input = "{% field kind='string' %}{% /field %}";
    let jinja_result = fill_markdown(jinja_input, true, 88, true, false, false, false, None, ListSpacing::Preserve);
    assert_eq!(jinja_result.trim(), jinja_input);

    // HTML comment tags
    let html_input = "<!-- f:field kind=\"string\" id=\"name\" --><!-- /f:field -->";
    let html_result = fill_markdown(html_input, true, 88, true, false, false, false, None, ListSpacing::Preserve);
    assert_eq!(html_result.trim(), html_input);

    // With surrounding text
    let mixed_input = "Before {% field %}{% /field %} after.";
    let mixed_result = fill_markdown(mixed_input, true, 88, true, false, false, false, None, ListSpacing::Preserve);
    assert!(mixed_result.contains("{% field %}{% /field %}"));
}
