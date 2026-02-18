#![allow(clippy::unwrap_used)]

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
    assert_eq!(wrap_paragraph_lines("- word", 10, 0, 0, true, true, None, true), vec!["- word"]);

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
        vec!["word - word *", "word + word >", "word # word ##", "word 1. word 2)", "word",]
    );

    assert_eq!(
        wrap_paragraph_lines(text, 20, 0, 0, true, true, None, true),
        vec!["word - word * word +", "word > word # word", "\\## word 1. word 2)", "word",]
    );

    assert_eq!(
        wrap_paragraph_lines(text, 20, 0, 0, true, true, None, false),
        vec!["word - word * word +", "word > word # word", "## word 1. word 2)", "word",]
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
        vec!["Text", "with", "<b>bold</b>", "and", "[a link](https://example.com).",]
    );
}

#[test]
fn test_wrap_text() {
    let sample_text = "This is a sample text with a [Markdown link](https://example.com) and an <a href='#'>tag</a>. It should demonstrate the functionality of our enhanced text wrapping implementation.";

    let filled =
        wrap_paragraph(sample_text, 40, ">", ">>", 0, true, true, Some(&simple_word_split), false);
    let filled_expected = "\
>This is a sample text with a [Markdown
>>link](https://example.com) and an <a
>>href='#'>tag</a>. It should
>>demonstrate the functionality of our
>>enhanced text wrapping implementation.";

    let filled_smart =
        wrap_paragraph(sample_text, 40, ">", ">>", 0, true, true, Some(&html_md_word_split), false);
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
        assert!(line.chars().count() <= width, "Line exceeds width: {line:?}");
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
    let code_token = result.iter().find(|r| r.contains("`getRequiredEnv()`")).unwrap();
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
    assert_eq!(wrapped_spaces, "This line ends with spaces\\\nThis is a new line");

    // Test backslash line breaks
    let text_with_backslash = "This line ends with backslash\\\nThis is a new line";
    let wrapped_backslash = wrapper(text_with_backslash, "", "");
    assert_eq!(wrapped_backslash, "This line ends with backslash\\\nThis is a new line");

    // Test single segment (no line breaks)
    let single_segment = "Text with no breaks";
    let wrapped_single = wrapper(single_segment, "> ", "  ");
    assert_eq!(wrapped_single, "> Text with no breaks");
}

#[test]
fn test_adjacent_jinja_tags_no_space() {
    use flowmark::config::DEFAULT_MIN_LINE_LEN;
    use flowmark::wrapping::line_wrappers::{line_wrap_by_sentence, line_wrap_to_width};
    use flowmark::wrapping::tag_handling::{denormalize_adjacent_tags, normalize_adjacent_tags};

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
    use flowmark::config::DEFAULT_MIN_LINE_LEN;
    use flowmark::wrapping::line_wrappers::{line_wrap_by_sentence, line_wrap_to_width};
    use flowmark::wrapping::tag_handling::{denormalize_adjacent_tags, normalize_adjacent_tags};

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
    use flowmark::config::ListSpacing;
    use flowmark::fill_markdown;

    // Jinja tags
    let jinja_input = "{% field kind='string' %}{% /field %}";
    let jinja_result = fill_markdown(
        jinja_input,
        true,
        88,
        true,
        false,
        false,
        false,
        None,
        ListSpacing::Preserve,
    );
    assert_eq!(jinja_result.trim(), jinja_input);

    // HTML comment tags
    let html_input = "<!-- f:field kind=\"string\" id=\"name\" --><!-- /f:field -->";
    let html_result =
        fill_markdown(html_input, true, 88, true, false, false, false, None, ListSpacing::Preserve);
    assert_eq!(html_result.trim(), html_input);

    // With surrounding text
    let mixed_input = "Before {% field %}{% /field %} after.";
    let mixed_result = fill_markdown(
        mixed_input,
        true,
        88,
        true,
        false,
        false,
        false,
        None,
        ListSpacing::Preserve,
    );
    assert!(mixed_result.contains("{% field %}{% /field %}"));
}

// ===== Tests ported from Python test_wrapping.py =====

#[test]
fn test_adjacent_jinja_comment_tags_no_space() {
    use flowmark::config::DEFAULT_MIN_LINE_LEN;
    use flowmark::wrapping::line_wrappers::line_wrap_by_sentence;
    use flowmark::wrapping::tag_handling::{denormalize_adjacent_tags, normalize_adjacent_tags};

    let original = "{# first #}{# second #}";
    let normalized = normalize_adjacent_tags(original);
    assert_eq!(normalized, "{# first #} {# second #}");
    let denormalized = denormalize_adjacent_tags(&normalized);
    assert_eq!(denormalized, original);

    let wrapper = line_wrap_by_sentence(80, DEFAULT_MIN_LINE_LEN, true);
    let result = wrapper(original, "", "");
    assert_eq!(result, original);
}

#[test]
fn test_adjacent_jinja_variable_tags_no_space() {
    use flowmark::config::DEFAULT_MIN_LINE_LEN;
    use flowmark::wrapping::line_wrappers::line_wrap_by_sentence;
    use flowmark::wrapping::tag_handling::{denormalize_adjacent_tags, normalize_adjacent_tags};

    let original = "{{ a }}{{ b }}";
    let normalized = normalize_adjacent_tags(original);
    assert_eq!(normalized, "{{ a }} {{ b }}");
    let denormalized = denormalize_adjacent_tags(&normalized);
    assert_eq!(denormalized, original);

    let wrapper = line_wrap_by_sentence(80, DEFAULT_MIN_LINE_LEN, true);
    let result = wrapper(original, "", "");
    assert_eq!(result, original);
}

#[test]
fn test_backslash_in_tag_attributes() {
    let tag_with_backslash = r"{% field pattern='^[^@]+\.[^@]+$' %}";
    let text = format!("Use {tag_with_backslash} for email.");
    let result = html_md_word_split(&text);
    assert!(result.contains(&tag_with_backslash.to_string()));

    let wrapped = wrap_paragraph_lines(&text, 80, 0, 0, true, true, None, true);
    let full_result = wrapped.join(" ");
    assert!(full_result.contains(r"\."));
}

#[test]
fn test_block_heuristics_blank_line_normalization() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "{% field %}\n- Item 1\n- Item 2\n{% /field %}";
    let result = wrapper(text, "", "");

    assert!(
        result.contains("{% field %}\n\n"),
        "Expected blank line after opening tag, got: {result}"
    );
    assert!(
        result.contains("\n\n{% /field %}"),
        "Expected blank line before closing tag, got: {result}"
    );
}

#[test]
fn test_block_heuristics_list_items() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "{% field %}\n- Item 1\n- Item 2\n- Item 3\n{% /field %}";
    let result = wrapper(text, "", "");

    assert!(result.contains("{% field %}\n"));
    assert!(result.contains("\n- Item 1\n"));
    assert!(result.contains("\n- Item 2\n"));
    assert!(result.contains("\n- Item 3\n"));
    assert!(result.contains("\n{% /field %}"));
}

#[test]
fn test_block_heuristics_mixed_content() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "{% field %}\nIntro text here.\n| Col1 | Col2 |\n|------|------|\nOutro text.\n{% /field %}";
    let result = wrapper(text, "", "");

    assert!(result.contains("{% field %}\n"));
    assert!(result.contains("\n| Col1 | Col2 |\n"));
    assert!(result.contains("\n|------|------|\n"));
    assert!(result.contains("\n{% /field %}"));
}

#[test]
fn test_block_heuristics_only_with_tags() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "Some text\n| A | B |\nMore text";
    let result = wrapper(text, "", "");
    assert!(result.contains("| A | B |"));
}

#[test]
fn test_block_heuristics_preserves_existing_blank_lines() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "{% field %}\n\n- Item 1\n\n{% /field %}";
    let result = wrapper(text, "", "");

    assert!(result.contains("{% field %}\n\n"));
    // Should not have more than one consecutive blank line
    let mut max_consecutive_empty = 0;
    let mut current = 0;
    for line in result.split('\n') {
        if line.trim().is_empty() {
            current += 1;
            max_consecutive_empty = max_consecutive_empty.max(current);
        } else {
            current = 0;
        }
    }
    assert!(max_consecutive_empty <= 1, "Too many consecutive blank lines: {result}");
}

#[test]
fn test_block_heuristics_table_blank_lines() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "{% field %}\n| A | B |\n|---|---|\n{% /field %}";
    let result = wrapper(text, "", "");

    assert!(result.contains("{% field %}\n\n"));
    assert!(result.contains("\n\n{% /field %}"));
}

#[test]
fn test_block_heuristics_table_rows() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "{% field %}\n| A | B |\n|---|---|\n| 1 | 2 |\n{% /field %}";
    let result = wrapper(text, "", "");

    assert!(result.contains("{% field %}\n"));
    assert!(result.contains("\n| A | B |\n"));
    assert!(result.contains("\n|---|---|\n"));
    assert!(result.contains("\n| 1 | 2 |\n"));
    assert!(result.contains("\n{% /field %}"));
}

#[test]
fn test_closing_tag_spacing_function() {
    use flowmark::wrapping::tag_handling::fix_closing_tag_spacing;

    // Paragraph text - NO blank line added
    let text1 = "Regular text.\n{% /tag %}";
    let result1 = fix_closing_tag_spacing(text1);
    assert_eq!(result1, "Regular text.\n{% /tag %}");

    // List item - blank line added
    let text2 = "- List item\n{% /tag %}";
    let result2 = fix_closing_tag_spacing(text2);
    assert_eq!(result2, "- List item\n\n{% /tag %}");

    // Table row - blank line added
    let text3 = "| A | B |\n{% /tag %}";
    let result3 = fix_closing_tag_spacing(text3);
    assert_eq!(result3, "| A | B |\n\n{% /tag %}");

    // Already has blank line - no change
    let text4 = "- Item\n\n{% /tag %}";
    let result4 = fix_closing_tag_spacing(text4);
    assert_eq!(result4, "- Item\n\n{% /tag %}");

    // HTML comment closing tags
    let text6 = "Regular text.\n<!-- /tag -->";
    let result6 = fix_closing_tag_spacing(text6);
    assert_eq!(result6, "Regular text.\n<!-- /tag -->");

    let text7 = "- Item\n<!-- /tag -->";
    let result7 = fix_closing_tag_spacing(text7);
    assert_eq!(result7, "- Item\n\n<!-- /tag -->");
}

#[test]
fn test_inline_code_in_table_cells() {
    // Typical table cell with inline code
    let cell1 = "the field is `runId: v.id('experimentRuns')` or maybe `foo`?";
    let result1 = html_md_word_split(cell1);
    assert!(result1.contains(&"`runId: v.id('experimentRuns')`".to_string()));

    // Simple code-only cell
    let cell2 = "`detailedLogs`";
    let result2 = html_md_word_split(cell2);
    assert_eq!(result2, vec!["`detailedLogs`"]);

    // Cell with multiple code spans
    let cell3 = "`a` and `b` and `c`";
    let result3 = html_md_word_split(cell3);
    assert!(result3.contains(&"`a`".to_string()));
    assert!(result3.contains(&"`b`".to_string()));
    assert!(result3.contains(&"`c`".to_string()));
    assert!(result3.contains(&"and".to_string()));
}

#[test]
fn test_inline_code_with_surrounding_punctuation() {
    // Inline code with parentheses
    let text = "syntax (`<!--% ... -->`)**";
    let result = html_md_word_split(text);
    assert!(result.contains(&"(`<!--% ... -->`)**".to_string()));

    // Inline code followed by punctuation
    let text2 = "Use `foo bar`.";
    let result2 = html_md_word_split(text2);
    assert!(result2.contains(&"`foo bar`.".to_string()));
}

#[test]
fn test_list_content_gets_blank_lines() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "{% field %}\n- Item 1\n- Item 2\n{% /field %}";
    let result = wrapper(text, "", "");

    assert!(result.contains("{% field %}\n\n"), "Expected blank line after opening tag: {result}");
    assert!(
        result.contains("\n\n{% /field %}"),
        "Expected blank line before closing tag: {result}"
    );
}

#[test]
fn test_long_html_tags() {
    let long_html =
        "<div class='container' id='main' data-value='test' style='color: red'>content</div>";
    let text = format!("Before {long_html} after.");
    let result = html_md_word_split(&text);
    assert!(result.contains(&long_html.to_string()));
}

#[test]
fn test_long_jinja_comments() {
    let long_comment = "{# This is a long comment that spans many words here #}";
    let text = format!("Before {long_comment} after.");
    let result = html_md_word_split(&text);
    assert!(result.contains(&long_comment.to_string()));
}

#[test]
fn test_long_template_tags() {
    // 10-word template tag
    let long_tag =
        "{% component name='widget' type='button' size='large' color='blue' disabled=true %}";
    let text = format!("Before {long_tag} after.");
    let result = html_md_word_split(&text);
    assert!(result.contains(&long_tag.to_string()));

    // 12-word template tag (at the limit)
    let very_long_tag =
        "{% table columns=[a, b, c] rows=[1, 2, 3] border=true striped=true hover=true %}";
    let text2 = format!("Before {very_long_tag} after.");
    let result2 = html_md_word_split(&text2);
    assert!(result2.contains(&very_long_tag.to_string()));
}

#[test]
fn test_mixed_content_blank_lines_correct() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "{% field %}\nSome intro text.\n- Item 1\n- Item 2\n{% /field %}";
    let result = wrapper(text, "", "");

    assert!(
        result.contains("\n\n{% /field %}"),
        "Expected blank line before closing tag: {result}"
    );
}

#[test]
fn test_multiple_single_word_inline_codes() {
    let text = r#"via `getRequiredEnv()` and must live in files with `"use node"`."#;
    let result = html_md_word_split(text);

    assert!(result.contains(&"`getRequiredEnv()`".to_string()));
    assert!(result.contains(&"via".to_string()));
    assert!(result.contains(&"and".to_string()));
}

#[test]
fn test_nested_tags_newlines_preserved() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "{% form id='test' %}\n{% group id='section' %}\n{% field id='name' %}{% /field %}\n{% /group %}\n{% /form %}";
    let result = wrapper(text, "", "");

    assert!(result.contains("{% form id='test' %}\n"));
    assert!(result.contains("\n{% group id='section' %}\n"));
    assert!(result.contains("\n{% /group %}\n"));
    assert!(result.contains("\n{% /form %}"));
}

#[test]
fn test_newline_after_opening_tag() {
    use flowmark::config::DEFAULT_MIN_LINE_LEN;
    use flowmark::wrapping::line_wrappers::{line_wrap_by_sentence, line_wrap_to_width};

    let wrapper = line_wrap_to_width(80, true);

    let text = "{% description ref='example' %}\nThis is content after the tag.";
    let result = wrapper(text, "", "");
    assert!(result.contains("{% description ref='example' %}\n"));

    let text2 = "<!-- f:description ref='example' -->\nContent after HTML comment tag.";
    let result2 = wrapper(text2, "", "");
    assert!(result2.contains("<!-- f:description ref='example' -->\n"));

    let wrapper2 = line_wrap_by_sentence(80, DEFAULT_MIN_LINE_LEN, true);
    let result3 = wrapper2(text, "", "");
    assert!(result3.contains("{% description ref='example' %}\n"));
}

#[test]
fn test_newline_before_closing_tag() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "Some content here.\n{% /description %}";
    let result = wrapper(text, "", "");
    assert!(result.contains("\n{% /description %}"));

    let text2 = "Some content.\n<!-- /f:description -->";
    let result2 = wrapper(text2, "", "");
    assert!(result2.contains("\n<!-- /f:description -->"));
}

#[test]
fn test_paired_tags_not_broken() {
    let paired = "{% field kind='string' id='email' %}{% /field %}";
    let text = format!("Some text before {paired} and after.");
    let result = html_md_word_split(&text);
    let full_result = result.join(" ");
    assert!(full_result.contains("{% field kind='string' id='email' %}"));
    assert!(full_result.contains("{% /field %}"));

    // HTML comment paired tags
    let paired_html = "<!-- f:field kind='string' --><!-- /f:field -->";
    let text2 = format!("Before {paired_html} after.");
    let result2 = html_md_word_split(&text2);
    let full_result2 = result2.join(" ");
    assert!(full_result2.contains("<!-- f:field kind='string' -->"));
    assert!(full_result2.contains("<!-- /f:field -->"));

    // Wrapping should not break tags
    let long_text = format!("This is a longer piece of text with {paired} embedded in the middle.");
    let wrapped = wrap_paragraph_lines(&long_text, 40, 0, 0, true, true, None, true);
    let full_result3 = wrapped.join(" ");
    assert!(full_result3.contains("{% field kind='string' id='email' %}"));
    assert!(full_result3.contains("{% /field %}"));
}

#[test]
fn test_paragraph_only_content_various_tags() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    // Jinja tags
    let jinja = "{% note %}\nSimple paragraph.\n{% /note %}";
    let jinja_result = wrapper(jinja, "", "");
    assert!(!jinja_result.contains("\n\n{% /note %}"));

    // HTML comment tags
    let html = "<!-- f:warning -->\nWarning text here.\n<!-- /f:warning -->";
    let html_result = wrapper(html, "", "");
    assert!(!html_result.contains("\n\n<!-- /f:warning -->"));

    // Longer paragraph
    let long = "{% tip %}\nThis is a longer paragraph with more text that spans across multiple sentences. It should all be wrapped normally.\n{% /tip %}";
    let long_result = wrapper(long, "", "");
    assert!(!long_result.contains("\n\n{% /tip %}"));
}

#[test]
fn test_paragraph_text_no_extra_blank_lines() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text =
        "{% description %}\nThis is a simple note.\nJust paragraph text.\n{% /description %}";
    let result = wrapper(text, "", "");

    assert!(
        !result.contains("\n\n{% /description %}"),
        "Unexpected blank line before closing tag: {result}"
    );
    assert!(result.contains("\n{% /description %}"));

    let text2 = "<!-- f:note -->\nThis is text content.\n<!-- /f:note -->";
    let result2 = wrapper(text2, "", "");
    assert!(
        !result2.contains("\n\n<!-- /f:note -->"),
        "Unexpected blank line before closing tag: {result2}"
    );
    assert!(result2.contains("\n<!-- /f:note -->"));
}

#[test]
fn test_self_closing_html_comment_tags() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "Some content.\n<!-- note: important -->\nMore content.";
    let result = wrapper(text, "", "");
    assert!(result.contains("\n<!-- note: important -->\n"));

    let text2 = "Before.\n<!-- TODO: refactor this section later -->\nAfter.";
    let result2 = wrapper(text2, "", "");
    assert!(result2.contains("\n<!-- TODO: refactor this section later -->\n"));

    let text3 = "<!-- start -->\nContent here.\n<!-- end -->";
    let result3 = wrapper(text3, "", "");
    assert!(result3.contains("<!-- start -->\n"));
    assert!(result3.contains("\n<!-- end -->"));

    // Self-closing comment inline
    let inline = "See <!-- ref: section 3 --> for details.";
    let tokens = html_md_word_split(inline);
    assert!(tokens.contains(&"<!-- ref: section 3 -->".to_string()));
}

#[test]
fn test_self_closing_jinja_comment_tags() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "Code here.\n{# TODO: optimize this #}\nMore code.";
    let result = wrapper(text, "", "");
    assert!(result.contains("\n{# TODO: optimize this #}\n"));

    // Comment inline
    let inline = "Value {# in bytes #} is 1024.";
    let tokens = html_md_word_split(inline);
    assert!(tokens.contains(&"{# in bytes #}".to_string()));
}

#[test]
fn test_self_closing_jinja_tags() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    // Self-closing tag on its own line
    let text = "Some content.\n{% break %}\nMore content.";
    let result = wrapper(text, "", "");
    assert!(result.contains("\n{% break %}\n"));

    // Self-closing tag with attributes
    let text2 = "Before.\n{% include 'header.html' %}\nAfter.";
    let result2 = wrapper(text2, "", "");
    assert!(result2.contains("\n{% include 'header.html' %}\n"));

    // Multiple self-closing tags
    let text3 = "{% set x = 1 %}\n{% set y = 2 %}\n{% set z = 3 %}";
    let result3 = wrapper(text3, "", "");
    assert!(result3.contains("{% set x = 1 %}\n"));
    assert!(result3.contains("\n{% set y = 2 %}\n"));
    assert!(result3.contains("\n{% set z = 3 %}"));

    // Self-closing tag inline
    let inline = "Use {% include 'partial.html' %} to include.";
    let tokens = html_md_word_split(inline);
    assert!(tokens.contains(&"{% include 'partial.html' %}".to_string()));
}

#[test]
fn test_self_closing_jinja_variable_tags() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "Name:\n{{ user.name }}\nEmail:";
    let result = wrapper(text, "", "");
    assert!(result.contains("\n{{ user.name }}\n"));

    let text2 = "Count:\n{{ items | length }}\nDone.";
    let result2 = wrapper(text2, "", "");
    assert!(result2.contains("\n{{ items | length }}\n"));

    // Variable inline
    let inline = "Hello {{ name }}, welcome!";
    let tokens = html_md_word_split(inline);
    assert!(tokens.iter().any(|t| t.contains("{{ name }}")));
}

#[test]
fn test_table_content_gets_blank_lines() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "{% field %}\n| A | B |\n|---|---|\n| 1 | 2 |\n{% /field %}";
    let result = wrapper(text, "", "");

    assert!(result.contains("{% field %}\n\n"), "Expected blank line after opening tag: {result}");
    assert!(
        result.contains("\n\n{% /field %}"),
        "Expected blank line before closing tag: {result}"
    );
}

#[test]
fn test_tag_with_list_items() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    let text = "- Option A {% #option_a %}\n{% /field %}";
    let result = wrapper(text, "", "");
    assert!(result.contains("\n{% /field %}"));
}

#[test]
fn test_various_tag_types_with_tables() {
    use flowmark::wrapping::line_wrappers::line_wrap_to_width;

    let wrapper = line_wrap_to_width(80, true);

    // Jinja tags with table
    let jinja = "{% table %}\n| A | B |\n{% /table %}";
    let jinja_result = wrapper(jinja, "", "");
    assert!(jinja_result.contains("{% table %}\n\n"));
    assert!(jinja_result.contains("\n\n{% /table %}"));

    // HTML comment tags with table
    let html = "<!-- f:table -->\n| A | B |\n<!-- /f:table -->";
    let html_result = wrapper(html, "", "");
    assert!(html_result.contains("<!-- f:table -->\n\n"));
    assert!(html_result.contains("\n\n<!-- /f:table -->"));

    // Jinja variable tags with table
    let var = "{{ header }}\n| A | B |\n{{ footer }}";
    let var_result = wrapper(var, "", "");
    assert!(var_result.contains("{{ header }}\n\n"));
}
