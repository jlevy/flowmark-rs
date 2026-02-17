//! Ellipsis replacement.
//!
//! Ported from Python: flowmark/typography/ellipses.py

use once_cell::sync::Lazy;
use regex::Regex;

static ELLIPSIS_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?m)(^|[\w"'\u{2018}\u{2019}\u{201c}\u{201d}])(\s*)(\.\.\.)([\.\,:;\?!\)\-\u{2014}"'\u{2018}\u{2019}\u{201c}\u{201d}]?)(\s*)"#).unwrap()
});

static WORD_CHAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\w").unwrap());

/// Replace three consecutive dots with a proper ellipsis character (\u{2026}).
///
/// Rules:
/// - `...` must be preceded by start of line OR a word character (with optional space)
/// - If immediately before is a word character, a space is inserted before
/// - If immediately after is a word character, a space is inserted after
/// - Punctuation adjacent to the ellipsis has no extra space
pub fn ellipses(text: &str) -> String {
    // We need access to the full text for boundary checking, so we use a manual approach
    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;

    for caps in ELLIPSIS_PATTERN.captures_iter(text) {
        let full_match = caps.get(0).unwrap();
        let start = full_match.start();
        let end = full_match.end();

        // Copy text before this match
        result.push_str(&text[last_end..start]);

        let prefix = &caps[1];
        let spaces_before = &caps[2];
        let punct = &caps[4];
        let spaces_after = &caps[5];

        // Get what follows the match
        let remaining = &text[end..];
        let next_char = remaining.chars().next().unwrap_or('\0');

        // Check boundary - must be followed by word or end
        if !remaining.is_empty() && !WORD_CHAR_RE.is_match(&next_char.to_string()) && next_char != '\0' {
            result.push_str(full_match.as_str());
            last_end = end;
            continue;
        }

        result.push_str(prefix);

        // Add space before ellipsis if word char with no existing space
        if !prefix.is_empty() && WORD_CHAR_RE.is_match(prefix) && spaces_before.is_empty() {
            result.push(' ');
        } else {
            result.push_str(spaces_before);
        }

        result.push('\u{2026}'); // ellipsis character
        result.push_str(punct);

        // Add space after if word char follows with no space and no punct
        if !remaining.is_empty()
            && WORD_CHAR_RE.is_match(&next_char.to_string())
            && spaces_after.is_empty()
            && punct.is_empty()
        {
            result.push(' ');
        } else {
            result.push_str(spaces_after);
        }

        last_end = end;
    }

    // Append remaining text
    result.push_str(&text[last_end..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipses_basic() {
        assert_eq!(ellipses("hello... world"), "hello \u{2026} world");
        assert_eq!(ellipses("hello ... world"), "hello \u{2026} world");
    }

    #[test]
    fn test_ellipses_with_punctuation() {
        assert_eq!(ellipses("hello...!"), "hello \u{2026}!");
    }
}
