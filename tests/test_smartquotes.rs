use flowmark::config::ListSpacing;
use flowmark::fill_markdown;
use flowmark::typography::quotes::smart_quotes;

#[test]
fn test_basic_double_quotes() {
    assert_eq!(
        smart_quotes("I'm there with \"George\""),
        "I\u{2019}m there with \u{201c}George\u{201d}"
    );
    assert_eq!(smart_quotes("\"Hello,\" he said."), "\u{201c}Hello,\u{201d} he said.");
    assert_eq!(smart_quotes("\"I know!\""), "\u{201c}I know!\u{201d}");
}

#[test]
fn test_basic_single_quotes() {
    assert_eq!(
        smart_quotes("Words in 'single quotes' work too"),
        "Words in \u{2018}single quotes\u{2019} work too"
    );
    assert_eq!(smart_quotes("X is 'foo'"), "X is \u{2018}foo\u{2019}");
}

#[test]
fn test_apostrophes_and_contractions() {
    assert_eq!(smart_quotes("I'm there"), "I\u{2019}m there");
    assert_eq!(
        smart_quotes("I'll be there, don't worry"),
        "I\u{2019}ll be there, don\u{2019}t worry"
    );
    assert_eq!(smart_quotes("Jill's"), "Jill\u{2019}s");
    assert_eq!(smart_quotes("James'"), "James\u{2019}");
}

#[test]
fn test_possessives_at_end_of_words() {
    assert_eq!(smart_quotes("James'"), "James\u{2019}");
    assert_eq!(smart_quotes("The students' books"), "The students\u{2019} books");
    assert_eq!(smart_quotes("Mr. Jones' house"), "Mr. Jones\u{2019} house");
    assert_eq!(smart_quotes("The cats' toys"), "The cats\u{2019} toys");
    assert_eq!(smart_quotes("Jesus' disciples"), "Jesus\u{2019} disciples");
    assert_eq!(smart_quotes("The class' performance"), "The class\u{2019} performance");
}

#[test]
fn test_patterns_left_unchanged() {
    assert_eq!(smart_quotes("In the '60s"), "In the '60s");
    assert_eq!(smart_quotes("x=\"foo\""), "x=\"foo\"");
    assert_eq!(smart_quotes("x='foo'"), "x='foo'");
    assert_eq!(smart_quotes("Blah'blah'blah"), "Blah'blah'blah");
    assert_eq!(smart_quotes("\"\"quotes\"s"), "\"\"quotes\"s");
    assert_eq!(smart_quotes("\\\"escaped\\\""), "\\\"escaped\\\"");
    assert_eq!(smart_quotes("'apos'trophes"), "'apos'trophes");
}

#[test]
fn test_quotes_with_punctuation() {
    assert_eq!(smart_quotes("\"Hello,\""), "\u{201c}Hello,\u{201d}");
    assert_eq!(smart_quotes("\"Wait;\""), "\u{201c}Wait;\u{201d}");
    assert_eq!(smart_quotes("\"Stop:\""), "\u{201c}Stop:\u{201d}");
    assert_eq!(smart_quotes("\"Really?\""), "\u{201c}Really?\u{201d}");
    assert_eq!(smart_quotes("\"Yes!\""), "\u{201c}Yes!\u{201d}");
    assert_eq!(smart_quotes("\"End.\""), "\u{201c}End.\u{201d}");
    assert_eq!(smart_quotes("\"Em dash\"\u{2014}"), "\u{201c}Em dash\u{201d}\u{2014}");
    assert_eq!(smart_quotes("\"Parenthesis\")"), "\u{201c}Parenthesis\u{201d})");
    assert_eq!(smart_quotes("'Single em dash'\u{2014}"), "\u{2018}Single em dash\u{2019}\u{2014}");
    assert_eq!(smart_quotes("'Single parenthesis')"), "\u{2018}Single parenthesis\u{2019})");
}

#[test]
fn test_quotes_at_boundaries() {
    assert_eq!(smart_quotes("\"Start of sentence\""), "\u{201c}Start of sentence\u{201d}");
    assert_eq!(
        smart_quotes("He said \"middle of sentence\" and continued"),
        "He said \u{201c}middle of sentence\u{201d} and continued"
    );
}

#[test]
fn test_mixed_quotes_and_apostrophes() {
    assert_eq!(
        smart_quotes("I'm reading \"The Great Gatsby\" today"),
        "I\u{2019}m reading \u{201c}The Great Gatsby\u{201d} today"
    );
    assert_eq!(
        smart_quotes("She said \"I can't believe it!\""),
        "She said \u{201c}I can\u{2019}t believe it!\u{201d}"
    );
}

#[test]
fn test_edge_cases() {
    assert_eq!(smart_quotes(""), "");
    assert_eq!(smart_quotes("No quotes here"), "No quotes here");
    assert_eq!(smart_quotes("Just \"quotes\""), "Just \u{201c}quotes\u{201d}");
    assert_eq!(smart_quotes("'Single'"), "\u{2018}Single\u{2019}");
}

#[test]
fn test_multiple_quotes_in_text() {
    assert_eq!(
        smart_quotes("He said \"hello\" and she said \"goodbye\""),
        "He said \u{201c}hello\u{201d} and she said \u{201c}goodbye\u{201d}"
    );
    assert_eq!(
        smart_quotes("The words 'yes' and 'no' are opposites"),
        "The words \u{2018}yes\u{2019} and \u{2018}no\u{2019} are opposites"
    );
}

#[test]
fn test_complex_sentences() {
    let text = "John said \"I can't believe it's not butter!\" at the store.";
    let expected =
        "John said \u{201c}I can\u{2019}t believe it\u{2019}s not butter!\u{201d} at the store.";
    assert_eq!(smart_quotes(text), expected);
}

#[test]
fn test_technical_content_unchanged() {
    assert_eq!(smart_quotes("function(\"param\")"), "function(\"param\")");
    assert_eq!(smart_quotes("array['key']"), "array['key']");
    assert_eq!(smart_quotes("height=\"100px\""), "height=\"100px\"");
    assert_eq!(smart_quotes("class='my-class'"), "class='my-class'");
}

#[test]
fn test_complex_cases_unchanged() {
    assert_eq!(smart_quotes("quote\"in\"quote"), "quote\"in\"quote");
    assert_eq!(smart_quotes("\"\"nested\"\""), "\"\"nested\"\"");
    assert_eq!(smart_quotes("''nested''"), "''nested''");
    assert_eq!(smart_quotes("\"\"nested\""), "\"\"nested\"");
    assert_eq!(smart_quotes("'nested''"), "'nested''");
    assert_eq!(smart_quotes("x=\"foo\""), "x=\"foo\"");
    assert_eq!(smart_quotes("x='foo'"), "x='foo'");
    assert_eq!(smart_quotes("Blah'blah'blah"), "Blah'blah'blah");
    assert_eq!(smart_quotes("\"\"quotes\"s"), "\"\"quotes\"s");
    assert_eq!(smart_quotes("\\\"escaped\\\""), "\\\"escaped\\\"");
    assert_eq!(smart_quotes("'apos"), "'apos");
    assert_eq!(smart_quotes("'apos'trophes"), "'apos'trophes");
    assert_eq!(smart_quotes("$James'"), "$James'");
}

#[test]
fn test_quotes_with_newlines() {
    // Double quotes with newlines
    assert_eq!(smart_quotes("\"Hello\nWorld\""), "\u{201c}Hello\nWorld\u{201d}");
    assert_eq!(
        smart_quotes("He said \"Hello\nWorld\" today"),
        "He said \u{201c}Hello\nWorld\u{201d} today"
    );
    assert_eq!(
        smart_quotes("\"First line\nSecond line\nThird line\""),
        "\u{201c}First line\nSecond line\nThird line\u{201d}"
    );

    // Single quotes with newlines
    assert_eq!(smart_quotes("'Hello\nWorld'"), "\u{2018}Hello\nWorld\u{2019}");
    assert_eq!(
        smart_quotes("She said 'Hello\nWorld' today"),
        "She said \u{2018}Hello\nWorld\u{2019} today"
    );
    assert_eq!(
        smart_quotes("'First line\nSecond line\nThird line'"),
        "\u{2018}First line\nSecond line\nThird line\u{2019}"
    );

    // With punctuation after newline quotes
    assert_eq!(smart_quotes("\"Hello\nWorld\"."), "\u{201c}Hello\nWorld\u{201d}.");
    assert_eq!(smart_quotes("\"Hello\nWorld\"!"), "\u{201c}Hello\nWorld\u{201d}!");
    assert_eq!(smart_quotes("'Hello\nWorld'?"), "\u{2018}Hello\nWorld\u{2019}?");

    // Mixed with contractions
    assert_eq!(
        smart_quotes("I'm reading \"Hello\nWorld\" today"),
        "I\u{2019}m reading \u{201c}Hello\nWorld\u{201d} today"
    );

    // Multiple paragraphs in quotes should NOT be converted
    let text = "\"This is paragraph one.\n\nThis is paragraph two.\"";
    let expected = "\"This is paragraph one.\n\nThis is paragraph two.\"";
    assert_eq!(smart_quotes(text), expected);

    // Quotes at start and end of lines
    let text = "\"Start of text\nMiddle line\nEnd of text\"";
    let expected = "\u{201c}Start of text\nMiddle line\nEnd of text\u{201d}";
    assert_eq!(smart_quotes(text), expected);

    // Basic paragraph break
    assert_eq!(smart_quotes("\"Para 1.\n\nPara 2.\""), "\"Para 1.\n\nPara 2.\"");
    assert_eq!(smart_quotes("'Para 1.\n\nPara 2.'"), "'Para 1.\n\nPara 2.'");

    // Paragraph break with spaces
    assert_eq!(smart_quotes("\"Para 1.\n \nPara 2.\""), "\"Para 1.\n \nPara 2.\"");
    assert_eq!(smart_quotes("\"Para 1.\n  \nPara 2.\""), "\"Para 1.\n  \nPara 2.\"");
    assert_eq!(smart_quotes("\"Para 1.\n\t\nPara 2.\""), "\"Para 1.\n\t\nPara 2.\"");

    // Multiple paragraph breaks
    assert_eq!(
        smart_quotes("\"Para 1.\n\nPara 2.\n\nPara 3.\""),
        "\"Para 1.\n\nPara 2.\n\nPara 3.\""
    );

    // Paragraph break in context
    let text = "He said \"Para 1.\n\nPara 2.\" yesterday.";
    let expected = "He said \"Para 1.\n\nPara 2.\" yesterday.";
    assert_eq!(smart_quotes(text), expected);

    // Mixed: some with paragraph breaks, some without
    let text = "She said \"Hello world\" and he said \"Para 1.\n\nPara 2.\" today.";
    let expected = "She said \u{201c}Hello world\u{201d} and he said \"Para 1.\n\nPara 2.\" today.";
    assert_eq!(smart_quotes(text), expected);
}

// Integration tests: smart quoting in container types

#[test]
fn test_smart_quotes_in_table_cells() {
    let text = "| User Says | Response |\n| --- | --- |\n| \"Hello there\" | \"Goodbye\" |\n";
    let result =
        fill_markdown(text, true, 88, false, false, true, false, None, ListSpacing::Preserve);
    assert!(result.contains("\u{201c}Hello there\u{201d}"));
    assert!(result.contains("\u{201c}Goodbye\u{201d}"));
}

#[test]
fn test_smart_quotes_apostrophes_in_table_cells() {
    let text = "| User Says |\n| --- |\n| There's a bug |\n";
    let result =
        fill_markdown(text, true, 88, false, false, true, false, None, ListSpacing::Preserve);
    assert!(result.contains("There\u{2019}s"));
}

#[test]
fn test_smart_quotes_in_strikethrough() {
    let text = "~~\"Hello\" and don't~~ rest of text\n";
    let result =
        fill_markdown(text, true, 88, false, false, true, false, None, ListSpacing::Preserve);
    assert!(result.contains("\u{201c}Hello\u{201d}"));
    assert!(result.contains("don\u{2019}t"));
}

#[test]
fn test_smart_quotes_spanning_code_span() {
    let text = "**Tell the user:** \"First, install the `markform` command.\"\n";
    let result =
        fill_markdown(text, true, 88, false, false, true, false, None, ListSpacing::Preserve);
    assert!(result.contains("\u{201c}First,"));
    assert!(result.contains("command.\u{201d}"));
}

#[test]
fn test_smart_quotes_spanning_emphasis() {
    let text = "He said \"this is *really* important.\"\n";
    let result =
        fill_markdown(text, true, 88, false, false, true, false, None, ListSpacing::Preserve);
    assert!(result.contains("\u{201c}this"));
    assert!(result.contains("important.\u{201d}"));
}

#[test]
fn test_smart_quotes_spanning_strong_emphasis() {
    let text = "She said \"this is **very** important.\"\n";
    let result =
        fill_markdown(text, true, 88, false, false, true, false, None, ListSpacing::Preserve);
    assert!(result.contains("\u{201c}this"));
    assert!(result.contains("important.\u{201d}"));
}

#[test]
fn test_smart_quotes_spanning_link() {
    let text = "Read \"the [documentation](https://example.com) first.\"\n";
    let result =
        fill_markdown(text, true, 88, false, false, true, false, None, ListSpacing::Preserve);
    assert!(result.contains("\u{201c}the"));
    assert!(result.contains("first.\u{201d}"));
}

#[test]
fn test_smart_quotes_not_modifying_code_content() {
    let text = "Use \"the `x=\"value\"` syntax\" for this.\n";
    let result =
        fill_markdown(text, true, 88, false, false, true, false, None, ListSpacing::Preserve);
    assert!(result.contains("`x=\"value\"`"));
}

#[test]
fn test_smart_quotes_apostrophe_spanning_code_span() {
    let text = "I'll use the `markform` tool and it'll work.\n";
    let result =
        fill_markdown(text, true, 88, false, false, true, false, None, ListSpacing::Preserve);
    assert!(result.contains("I\u{2019}ll"));
    assert!(result.contains("it\u{2019}ll"));
}

// ===== Tests ported from Python test_smartquotes.py =====

fn fmt_sq(input: &str) -> String {
    fill_markdown(input, true, 88, false, false, true, false, None, ListSpacing::Preserve)
}

fn fmt_sq_semantic(input: &str) -> String {
    fill_markdown(input, true, 88, true, false, true, false, None, ListSpacing::Preserve)
}

#[test]
fn test_smart_quotes_blockquote_multiline_with_code_span() {
    let text = "> **Tell the user:** \"First, I'll make sure Markform is installed.\n> Markform is a CLI tool for creating structured forms that agents can fill via tool\n> calls. I'll install it globally so we can use the `markform` command.\"\n";
    let result = fmt_sq_semantic(text);
    assert!(result.contains("\u{201c}First,"), "Outer quotes should be converted");
    assert!(result.contains("command.\u{201d}"), "Closing quote should be converted");
    assert!(result.contains("I\u{2019}ll"), "Apostrophes should be converted");
    assert!(result.contains("`markform`"), "Code span must be preserved");
}

#[test]
fn test_smart_quotes_complex_table() {
    let text = "| User Says | You (the Agent) Run |\n| --- | --- |\n| **Issues/Beads** |  |\n| \"There's a bug where ...\" | `tbd create \"...\" --type=bug` |\n| \"Create a task/feature for ...\" | `tbd create \"...\" --type=task` or `--type=feature` |\n";
    let result = fmt_sq(text);
    // Prose quotes should be converted
    assert!(
        result.contains("\u{201c}There\u{2019}s a bug where \u{2026}\u{201d}")
            || result.contains("\u{201c}There\u{2019}s a bug where ...\u{201d}"),
        "Prose quotes should be converted in table: {result}"
    );
    // Code spans should be unchanged
    assert!(result.contains("`tbd create \"...\" --type=bug`"), "Code spans should be unchanged");
    assert!(result.contains("`tbd create \"...\" --type=task`"), "Code spans should be unchanged");
}

#[test]
fn test_smart_quotes_in_table_preserve_code_spans() {
    let text = "| Description | Command |\n| --- | --- |\n| \"Fix a bug\" | `tbd create \"...\" --type=bug` |\n";
    let result = fmt_sq(text);
    assert!(result.contains("\u{201c}Fix a bug\u{201d}"), "Prose quotes should be converted");
    assert!(result.contains("`tbd create \"...\" --type=bug`"), "Code span should be unchanged");
}

#[test]
fn test_smart_quotes_in_table_with_bold() {
    let text = "| Column |\n| --- |\n| **Issues/Beads** |\n| \"There's a bug\" |\n";
    let result = fmt_sq(text);
    assert!(
        result.contains("\u{201c}There\u{2019}s a bug\u{201d}"),
        "Smart quotes should be applied in table with bold"
    );
}

#[test]
fn test_smart_quotes_spanning_code_span_in_blockquote() {
    let text = "> **Tell the user:** \"First, install the `markform` command.\"\n";
    let result = fmt_sq(text);
    assert!(result.contains("\u{201c}First,"), "Opening quote should be converted");
    assert!(result.contains("command.\u{201d}"), "Closing quote should be converted");
}
