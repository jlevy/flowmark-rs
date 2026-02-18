//! Smart quote conversion: straight quotes to typographic quotes.
//!
//! Ported from Python: flowmark/typography/smartquotes.py

use regex::Regex;
use std::sync::LazyLock;

use crate::wrapping::tag_handling::TEMPLATE_TAG_PATTERN;

/// Pattern to detect paragraph breaks.
static PARAGRAPH_BREAK_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n\s*\n").expect("valid PARAGRAPH_BREAK_PATTERN regex"));

/// Pattern for matching quoted text.
/// Handles double quotes and single quotes, excluding content with same-type quotes.
/// Also allows quotes to start after an em dash.
static QUOTE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?m)(^|\s|\u{2014})(?:"([^"\u{201c}\u{201d}]*)"|'([^'\u{2018}\u{2019}]*)')(\s|$|\.|,|;|:|\?|!|\u{2014}|\))"#,
    )
    .expect("valid QUOTE_PATTERN regex")
});

/// Pattern for apostrophes/contractions.
static APOSTROPHE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\w)'(\w)").expect("valid APOSTROPHE_PATTERN regex"));

/// Pattern for possessive after s/S.
static POSSESSIVE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\w*[sS]'$").expect("valid POSSESSIVE_PATTERN regex"));

/// Check if text contains paragraph breaks.
fn is_multi_paragraph(text: &str) -> bool {
    PARAGRAPH_BREAK_PATTERN.is_match(text)
}

/// Apply smart quote conversion to a text segment.
fn apply_smart_quotes_to_text(text: &str) -> String {
    // Handle quoted text - both single and double quotes
    let result = QUOTE_PATTERN
        .replace_all(text, |caps: &regex::Captures<'_>| {
            let prefix = caps.get(1).map_or("", |m| m.as_str());
            let double_content = caps.get(2);
            let single_content = caps.get(3);
            let suffix = caps.get(4).map_or("", |m| m.as_str());

            let content = double_content.or(single_content).map_or("", |m| m.as_str());

            if is_multi_paragraph(content) {
                return caps.get(0).expect("group 0 always exists").as_str().to_string();
            }

            if double_content.is_some() {
                format!("{prefix}\u{201c}{content}\u{201d}{suffix}")
            } else {
                format!("{prefix}\u{2018}{content}\u{2019}{suffix}")
            }
        })
        .into_owned();

    // Handle apostrophes/contractions - process word by word
    let mut output = String::new();

    // Split by whitespace while preserving the separators
    let mut remaining = result.as_str();
    loop {
        // Find the next whitespace boundary
        let ws_pos = remaining.find(char::is_whitespace);

        let (word, rest) = if let Some(pos) = ws_pos {
            // Find end of whitespace
            let after_word = &remaining[pos..];
            let ws_end = after_word.find(|c: char| !c.is_whitespace()).unwrap_or(after_word.len());
            let word_and_ws = &remaining[..pos + ws_end];
            let rest = &remaining[pos + ws_end..];
            (word_and_ws, rest)
        } else {
            (remaining, "")
        };

        if word.is_empty() && rest.is_empty() {
            break;
        }

        let processed_word = process_word_apostrophe(word);
        output.push_str(&processed_word);

        if rest.is_empty() {
            break;
        }
        remaining = rest;
    }

    output
}

/// Process a single word for apostrophe conversion.
fn process_word_apostrophe(word: &str) -> String {
    // Count straight quotes in the word (ignoring whitespace part)
    let trimmed = word.trim_end();
    let trailing_ws = &word[trimmed.len()..];
    let quote_count = trimmed.matches('\'').count();

    if quote_count != 1 {
        return word.to_string();
    }

    // Check if it's surrounded by word characters (contractions)
    if APOSTROPHE_PATTERN.is_match(trimmed) {
        let replaced = trimmed.replace('\'', "\u{2019}");
        return format!("{replaced}{trailing_ws}");
    }

    // Check if it's a possessive at the end of a word ending in s/S
    if POSSESSIVE_PATTERN.is_match(trimmed) {
        let replaced = trimmed.replace('\'', "\u{2019}");
        return format!("{replaced}{trailing_ws}");
    }

    word.to_string()
}

/// Replace straight ASCII quotes and apostrophes with typographic quotes.
///
/// Quotes inside template tags are NEVER converted.
pub fn smart_quotes(text: &str) -> String {
    let mut segments: Vec<String> = Vec::new();
    let mut last_end = 0;

    for m in TEMPLATE_TAG_PATTERN.find_iter(text) {
        let start = m.start();
        let end = m.end();

        // Add the text before this tag (apply smart quotes to it)
        if start > last_end {
            let before_text = &text[last_end..start];
            segments.push(apply_smart_quotes_to_text(before_text));
        }

        // Add the tag itself unchanged
        segments.push(m.as_str().to_string());
        last_end = end;
    }

    // Add any remaining text after the last tag
    if last_end < text.len() {
        let remaining = &text[last_end..];
        segments.push(apply_smart_quotes_to_text(remaining));
    }

    segments.join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_quotes() {
        assert_eq!(smart_quotes(r#"He said "hello" there"#), "He said \u{201c}hello\u{201d} there");
    }

    #[test]
    fn test_apostrophe() {
        assert_eq!(smart_quotes("I'm here"), "I\u{2019}m here");
    }

    #[test]
    fn test_possessive_s() {
        assert_eq!(smart_quotes("James' book"), "James\u{2019} book");
    }

    #[test]
    fn test_template_tag_preserved() {
        let input = r#"{% field kind="string" %}"#;
        assert_eq!(smart_quotes(input), input);
    }

    #[test]
    fn test_code_like_unchanged() {
        let input = r#"x="foo""#;
        assert_eq!(smart_quotes(input), input);
    }
}
