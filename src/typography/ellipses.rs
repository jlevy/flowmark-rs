//! Ellipsis conversion: `...` to `\u{2026}`.
//!
//! Ported from Python: flowmark/typography/ellipses.py

use regex::Regex;
use std::sync::LazyLock;

/// Pattern to match three consecutive dots with context.
static ELLIPSIS_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)(^|[\w\x22\x27\u{2018}\u{2019}])(\s*)(\.\.\.)([\.,;:\?!\)\-\u{2014}\x22\x27\u{2018}\u{2019}]?)(\s*)",
    )
    .expect("valid ELLIPSIS_PATTERN regex")
});

static WORD_CHAR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\w").expect("valid WORD_CHAR_RE regex"));

/// Replace three consecutive dots with a proper ellipsis character (\u{2026}).
pub fn ellipses(text: &str) -> String {
    // We need to process matches with access to the rest of the text,
    // so we do manual iteration instead of replace_all.
    let mut result = String::new();
    let mut last_end = 0;

    for m in ELLIPSIS_PATTERN.find_iter(text) {
        let caps = ELLIPSIS_PATTERN
            .captures(&text[m.start()..])
            .expect("captures must succeed after find");
        let full_match_start = m.start();
        let full_match_end = m.end();

        let prefix = caps.get(1).map_or("", |m| m.as_str());
        let spaces_before = caps.get(2).map_or("", |m| m.as_str());
        let punct = caps.get(4).map_or("", |m| m.as_str());
        let spaces_after = caps.get(5).map_or("", |m| m.as_str());

        // Get what follows the match
        let remaining = &text[full_match_end..];
        let next_char = remaining.chars().next();

        // Check boundary - must be followed by word or end of line
        if let Some(nc) = next_char {
            if !WORD_CHAR_RE.is_match(&nc.to_string()) {
                // Not a valid boundary, keep original
                result.push_str(&text[last_end..full_match_end]);
                last_end = full_match_end;
                continue;
            }
        }

        result.push_str(&text[last_end..full_match_start]);

        // Build replacement
        result.push_str(prefix);

        // Add space before ellipsis if word char with no existing space
        if !prefix.is_empty() && WORD_CHAR_RE.is_match(prefix) && spaces_before.is_empty() {
            result.push(' ');
        } else {
            result.push_str(spaces_before);
        }

        result.push('\u{2026}');
        result.push_str(punct);

        // Add space after ellipsis if word char follows with no space and no punct
        if let Some(nc) = next_char {
            if WORD_CHAR_RE.is_match(&nc.to_string()) && spaces_after.is_empty() && punct.is_empty()
            {
                result.push(' ');
            } else {
                result.push_str(spaces_after);
            }
        } else {
            result.push_str(spaces_after);
        }

        last_end = full_match_end;
    }

    result.push_str(&text[last_end..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ellipsis() {
        assert_eq!(ellipses("Hello... world"), "Hello \u{2026} world");
    }

    #[test]
    fn test_ellipsis_with_space() {
        assert_eq!(ellipses("Hello ... world"), "Hello \u{2026} world");
    }

    #[test]
    fn test_ellipsis_at_end() {
        assert_eq!(ellipses("Hello..."), "Hello \u{2026}");
    }

    #[test]
    fn test_ellipsis_with_punct() {
        assert_eq!(ellipses("Hello...!"), "Hello \u{2026}!");
    }
}
