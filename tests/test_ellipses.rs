use flowmark::typography::ellipses::ellipses;

#[test]
fn test_ellipses_basic_conversions() {
    // Basic conversions, adding space only if needed next to a word character
    assert_eq!(ellipses("word..."), "word \u{2026}");
    assert_eq!(ellipses("word ..."), "word \u{2026}");
    assert_eq!(ellipses("word  ..."), "word  \u{2026}");
    assert_eq!(ellipses("word ... "), "word \u{2026} ");
    assert_eq!(ellipses("word  ...  "), "word  \u{2026}  ");

    assert_eq!(ellipses("word...word"), "word \u{2026} word");
    assert_eq!(ellipses("word ... word"), "word \u{2026} word");
    assert_eq!(ellipses("word  ...  word"), "word  \u{2026}  word");
    assert_eq!(ellipses("Hello...World"), "Hello \u{2026} World");
}

#[test]
fn test_ellipses_at_boundaries() {
    assert_eq!(ellipses("...word"), "\u{2026} word");
    assert_eq!(ellipses("... word"), "\u{2026} word");
    assert_eq!(ellipses(" ... word"), " \u{2026} word");
    assert_eq!(ellipses("word..."), "word \u{2026}");
    assert_eq!(ellipses("..."), "\u{2026}");
}

#[test]
fn test_ellipses_multiple() {
    assert_eq!(
        ellipses("I think... well... maybe..."),
        "I think \u{2026} well \u{2026} maybe \u{2026}"
    );
    assert_eq!(ellipses("First...second...third"), "First \u{2026} second \u{2026} third");
    assert_eq!(ellipses("Wait... what... really?"), "Wait \u{2026} what \u{2026} really?");
    assert_eq!(
        ellipses("I was thinking... maybe we should go."),
        "I was thinking \u{2026} maybe we should go."
    );
    assert_eq!(
        ellipses("The options are... well... complicated."),
        "The options are \u{2026} well \u{2026} complicated."
    );
}

#[test]
fn test_ellipses_punctuation() {
    assert_eq!(ellipses("word...."), "word \u{2026}.");
    assert_eq!(ellipses("word.... "), "word \u{2026}. ");
    assert_eq!(ellipses("word....  text"), "word \u{2026}.  text");
    assert_eq!(ellipses("word....word"), "word \u{2026}.word");
    assert_eq!(ellipses("He said..."), "He said \u{2026}");
    assert_eq!(ellipses("Really...?"), "Really \u{2026}?");
    assert_eq!(ellipses("Wait...!"), "Wait \u{2026}!");
    assert_eq!(ellipses("Well...,"), "Well \u{2026},");
    assert_eq!(ellipses("word .... Another"), "word \u{2026}. Another");

    assert_eq!(ellipses("word....."), "word.....");
    assert_eq!(ellipses("word......"), "word......");
}

#[test]
fn test_ellipses_does_not_apply() {
    assert_eq!(ellipses(".."), "..");
    assert_eq!(ellipses("."), ".");
    assert_eq!(ellipses(". . ."), ". . .");
    assert_eq!(ellipses("$..."), "$...");
    assert_eq!(ellipses("@..."), "@...");
    assert_eq!(ellipses("#..."), "#...");
    assert_eq!(ellipses("...@"), "...@");
    assert_eq!(ellipses("...$"), "...$");
    assert_eq!(ellipses("...#"), "...#");
}

#[test]
fn test_ellipses_multiline() {
    assert_eq!(
        ellipses("First line...\nSecond line... continues\n...starts here"),
        "First line \u{2026}\nSecond line \u{2026} continues\n\u{2026} starts here"
    );
    assert_eq!(ellipses("Hello....\n"), "Hello \u{2026}.\n");
}

#[test]
fn test_ellipses_edge_cases() {
    assert_eq!(ellipses(""), "");
    assert_eq!(ellipses("No ellipses here"), "No ellipses here");
    assert_eq!(ellipses("   "), "   ");
    assert_eq!(ellipses("...."), "\u{2026}.");
    assert_eq!(ellipses(" ...."), " \u{2026}.");
}

#[test]
fn test_ellipses_code_like() {
    assert_eq!(ellipses("if (x...) {"), "if (x...) {");
    assert_eq!(ellipses("[...]"), "[...]");
    assert_eq!(ellipses("{...}"), "{...}");
    assert_eq!(ellipses("path/to/..."), "path/to/...");
}

#[test]
fn test_ellipses_four_dots() {
    assert_eq!(ellipses("word....word"), "word \u{2026}.word");
    assert_eq!(ellipses("word....123"), "word \u{2026}.123");
}

#[test]
fn test_ellipses_quotes() {
    assert_eq!(ellipses("'...word"), "'\u{2026} word");
    assert_eq!(ellipses("'... word"), "'\u{2026} word");
    assert_eq!(ellipses("\"...word"), "\"\u{2026} word");
    assert_eq!(ellipses("\"... word"), "\"\u{2026} word");
    assert_eq!(ellipses("'...word'"), "'\u{2026} word'");
    assert_eq!(ellipses("\"...word\""), "\"\u{2026} word\"");
    assert_eq!(ellipses("'...'"), "'\u{2026}'");
    assert_eq!(ellipses("\"...\""), "\"\u{2026}\"");
    assert_eq!(ellipses("word...'"), "word \u{2026}'");
    assert_eq!(ellipses("word...\""), "word \u{2026}\"");
    assert_eq!(ellipses("word...'next"), "word \u{2026}'next");
    assert_eq!(ellipses("word...\"next"), "word \u{2026}\"next");
    assert_eq!(ellipses("He said '...'"), "He said '\u{2026}'");
    assert_eq!(ellipses("She said \"...\""), "She said \"\u{2026}\"");
}

/// C2 regression test: ellipsis adjacent to curly quotes must be converted.
/// Previously, `ELLIPSIS_PATTERN` group 4 didn't include \u{201c}/\u{201d},
/// so `...` next to curly double quotes was skipped.
#[test]
fn test_ellipses_curly_quotes() {
    // Curly double quotes
    assert_eq!(ellipses("word...\u{201d}"), "word \u{2026}\u{201d}",);
    assert_eq!(ellipses("\u{201c}...word"), "\u{201c}\u{2026} word",);
    assert_eq!(ellipses("word...\u{201c}"), "word \u{2026}\u{201c}",);
    assert_eq!(ellipses("\u{201d}...word"), "\u{201d}\u{2026} word",);
    // Curly single quotes
    assert_eq!(ellipses("word...\u{2019}"), "word \u{2026}\u{2019}",);
    assert_eq!(ellipses("\u{2018}...word"), "\u{2018}\u{2026} word",);
}

/// C2 integration test: ellipsis + smartquotes together via `fill_markdown`.
/// In --auto mode both are enabled. The `...` adjacent to a quote that
/// smartquotes has converted to curly must still be replaced with \u{2026}.
#[test]
fn test_ellipses_smartquotes_integration() {
    use flowmark::config::ListSpacing;
    use flowmark::fill_markdown;

    // Both smartquotes and ellipses enabled (like --auto mode)
    let result = fill_markdown(
        "He said \"well...\"\n",
        true,
        88,
        false,
        false,
        true,
        true,
        None,
        ListSpacing::Preserve,
    );
    assert!(
        result.contains('\u{2026}'),
        "Ellipsis should be converted even adjacent to curly quote: {result:?}"
    );
    assert!(result.contains("\u{201c}"), "Opening quote should be converted: {result:?}");

    let result = fill_markdown(
        "\"Hello...\" she said\n",
        true,
        88,
        false,
        false,
        true,
        true,
        None,
        ListSpacing::Preserve,
    );
    assert!(result.contains('\u{2026}'), "Ellipsis should be converted: {result:?}");
}
