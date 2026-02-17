use flowmark::formats::flowmark_markdown::{ListSpacing, flowmark_markdown};
use flowmark::linewrapping::line_wrappers::line_wrap_by_sentence;
use flowmark::linewrapping::markdown_filling::fill_markdown;
use flowmark::linewrapping::text_filling::DEFAULT_WRAP_WIDTH;
use flowmark::linewrapping::text_wrapping::default_len;

fn md(text: &str) -> String {
    let wrapper = line_wrap_by_sentence(DEFAULT_WRAP_WIDTH, 15, 0, default_len, true);
    flowmark_markdown(text, &wrapper, ListSpacing::default())
}

fn fm(text: &str) -> String {
    fill_markdown(text, false, DEFAULT_WRAP_WIDTH, false, false, false, None, ListSpacing::default())
}

#[test]
fn test_literal_tildes_before_numbers() {
    let result = md("Target: ~60 seconds, ~130 words total\n");
    assert_eq!(result, "Target: ~60 seconds, ~130 words total\n");
}

#[test]
fn test_literal_tildes_not_converted_to_double() {
    let result = fm("Target: ~60 seconds, ~130 words total");
    assert!(!result.contains("~~"));
    assert_eq!(result.trim(), "Target: ~60 seconds, ~130 words total");
}

#[test]
fn test_double_tilde_strikethrough() {
    let result = md("This is ~~strikethrough~~ text\n");
    assert_eq!(result, "This is ~~strikethrough~~ text\n");
}

#[test]
fn test_single_tilde_strikethrough() {
    let result = md("This is ~strikethrough~ text\n");
    assert_eq!(result, "This is ~~strikethrough~~ text\n");
}

#[test]
fn test_multiple_strikethroughs() {
    let result = md("~one~ and ~two~ items\n");
    assert_eq!(result, "~~one~~ and ~~two~~ items\n");
}

#[test]
fn test_single_tilde_no_closer() {
    let result = md("About ~50% of users\n");
    assert_eq!(result, "About ~50% of users\n");
}

#[test]
fn test_tildes_with_space_before_closer() {
    let result = md("costs ~100 to ~200\n");
    assert_eq!(result, "costs ~100 to ~200\n");
}

#[test]
fn test_tilde_space_after_opener() {
    let result = md("~ spaced ~\n");
    assert_eq!(result, "~ spaced ~\n");
}

#[test]
fn test_escaped_tildes_preserved() {
    // comrak consumes backslash escapes in Text nodes. Single `~` followed by digits
    // is not strikethrough syntax, so no re-escaping needed.
    let result = md("Target: \\~60 seconds, \\~130 words total\n");
    assert_eq!(result, "Target: ~60 seconds, ~130 words total\n");
}

#[test]
fn test_strikethrough_in_paragraph() {
    let result = fm("This paragraph has some ~~deleted text~~ in it and also mentions ~50 users.");
    assert!(result.contains("~~deleted text~~"));
    assert!(result.contains("~50 users"));
    assert!(!result.contains("~~50"));
}
