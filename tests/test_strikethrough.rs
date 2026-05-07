use flowmark::config::ListSpacing;
use flowmark::fill_markdown;

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

// ===== Tilde-in-parentheses bug (GFM punctuation flanking rules) =====
// Ported from Python test_strikethrough.py (v0.6.5, fix `443861f`).
// comrak already implements full GFM flanking rules including punctuation,
// so these tests verify continued parity rather than driving a Rust-side fix.

#[test]
fn test_tilde_before_and_inside_parens() {
    // `~100 (~200)` must NOT be parsed as strikethrough.
    // Closing `~` in `(~200)` is preceded by `(` (punctuation) and followed by
    // `2` (word char), so it is NOT right-flanking per GFM spec.
    let result = fmt("~100 (~200)\n");
    assert_eq!(result, "~100 (~200)\n");
}

#[test]
fn test_tilde_before_and_inside_parens_fill() {
    let result = fmt("~100 (~200)");
    assert!(!result.contains("~~"));
    assert_eq!(result.trim(), "~100 (~200)");
}

#[test]
fn test_tilde_only_inside_parens() {
    let result = fmt("100 (~200)\n");
    assert_eq!(result, "100 (~200)\n");
}

#[test]
fn test_tilde_inside_parens_with_text() {
    let result = fmt("~100 (x ~200)\n");
    assert_eq!(result, "~100 (x ~200)\n");
}

#[test]
fn test_tilde_in_parens_then_outside() {
    let result = fmt("(~200) ~100\n");
    assert_eq!(result, "(~200) ~100\n");
}

#[test]
fn test_tilde_before_parens_no_tilde_inside() {
    let result = fmt("~100 (200)\n");
    assert_eq!(result, "~100 (200)\n");
}

#[test]
fn test_strikethrough_inside_parens() {
    // Valid double-tilde strikethrough inside parens is preserved.
    let result = fmt("(~~text~~) end\n");
    assert_eq!(result, "(~~text~~) end\n");
}

#[test]
fn test_strikethrough_after_punctuation() {
    // Opening `~` after `"` (punctuation) is left-flanking because preceded by
    // punctuation. Closing `~` before `"` is right-flanking because followed
    // by punctuation.
    let result = fmt("\"~~text~~\" end\n");
    assert_eq!(result, "\"~~text~~\" end\n");
}

#[test]
fn test_strikethrough_with_punctuation_content() {
    // Closing `~~` after `!` (punctuation) is right-flanking because the next
    // char is whitespace.
    let result = fmt("~~hello!~~ end\n");
    assert_eq!(result, "~~hello!~~ end\n");
}

#[test]
fn test_tilde_in_brackets() {
    // `[` before closing `~` is punctuation, followed by word char → not
    // right-flanking. Also `[~200]` has no matching closer for the outer `~`.
    let result = fmt("~100 [~200]\n");
    assert!(!result.contains("~~"));
}
