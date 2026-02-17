use flowmark::fill_markdown;
use flowmark::config::ListSpacing;

fn fmt(input: &str) -> String {
    fill_markdown(input, true, 88, false, false, false, false, None, ListSpacing::Preserve)
}

#[test]
fn test_literal_tildes_before_numbers() {
    let result = fmt("Target: ~60 seconds, ~130 words total\n");
    assert_eq!(result, "Target: ~60 seconds, ~130 words total\n");
}

#[test]
fn test_literal_tildes_not_converted_to_double() {
    let result = fmt("Target: ~60 seconds, ~130 words total");
    assert!(!result.contains("~~"));
    assert_eq!(result.trim(), "Target: ~60 seconds, ~130 words total");
}

#[test]
fn test_double_tilde_strikethrough() {
    let result = fmt("This is ~~strikethrough~~ text\n");
    assert_eq!(result, "This is ~~strikethrough~~ text\n");
}

#[test]
fn test_single_tilde_strikethrough() {
    let result = fmt("This is ~strikethrough~ text\n");
    // GFM normalizes single tilde to double tilde
    assert_eq!(result, "This is ~~strikethrough~~ text\n");
}

#[test]
fn test_multiple_strikethroughs() {
    let result = fmt("~one~ and ~two~ items\n");
    assert_eq!(result, "~~one~~ and ~~two~~ items\n");
}

#[test]
fn test_single_tilde_no_closer() {
    let result = fmt("About ~50% of users\n");
    assert_eq!(result, "About ~50% of users\n");
}

#[test]
fn test_tildes_with_space_before_closer() {
    let result = fmt("costs ~100 to ~200\n");
    assert_eq!(result, "costs ~100 to ~200\n");
}

#[test]
fn test_escaped_tildes_preserved() {
    let result = fmt("Target: \\~60 seconds, \\~130 words total\n");
    assert_eq!(result, "Target: \\~60 seconds, \\~130 words total\n");
}

#[test]
fn test_strikethrough_in_paragraph() {
    let result = fmt("This paragraph has some ~~deleted text~~ in it and also mentions ~50 users.");
    assert!(result.contains("~~deleted text~~"));
    assert!(result.contains("~50 users"));
    assert!(!result.contains("~~50"));
}

// ===== Tests ported from Python test_strikethrough.py =====

#[test]
fn test_tilde_space_after_opener() {
    // A tilde followed by a space is not left-flanking, so no strikethrough
    let result = fmt("~ spaced ~\n");
    assert_eq!(result, "~ spaced ~\n");
}

#[test]
fn test_tilde_space_before_closer() {
    // A tilde preceded by a space is not right-flanking, so no strikethrough
    let result = fmt("~foo ~\n");
    assert_eq!(result, "~foo ~\n");
}
