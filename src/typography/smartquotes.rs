//! Smart quote conversion.
//!
//! Ported from Python: flowmark/typography/smartquotes.py

use once_cell::sync::Lazy;
use regex::Regex;

use crate::linewrapping::atomic_patterns::TEMPLATE_TAG_PATTERN;

static PARAGRAPH_BREAK_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n\s*\n").unwrap());

/// Pattern for quoted text: matches "..." and '...' with proper boundary checks.
static QUOTE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?m)(^|\s|\u{2014})(?:"([^"\u{201c}\u{201d}]*)"|'([^'\u{2018}\u{2019}]*)')(\s|$|\.|,|;|:|\?|!|\u{2014}|\))"#,
    )
    .unwrap()
});

static APOSTROPHE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\w)'(\w)").unwrap());
static POSSESSIVE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\w*[sS]'$").unwrap());

/// Check if text contains paragraph breaks.
fn is_multi_paragraph(text: &str) -> bool {
    PARAGRAPH_BREAK_PATTERN.is_match(text)
}

/// Apply smart quote conversion to a text segment.
fn apply_smart_quotes_to_text(text: &str) -> String {
    let result = QUOTE_PATTERN
        .replace_all(text, |caps: &regex::Captures| {
            let prefix = &caps[1];
            let double_content = caps.get(2);
            let single_content = caps.get(3);
            let suffix = &caps[4];

            let content = double_content
                .or(single_content)
                .map(|m| m.as_str())
                .unwrap_or("");

            if is_multi_paragraph(content) {
                return caps[0].to_string();
            }

            if double_content.is_some() {
                format!("{prefix}\u{201c}{content}\u{201d}{suffix}")
            } else {
                format!("{prefix}\u{2018}{content}\u{2019}{suffix}")
            }
        })
        .into_owned();

    // Handle apostrophes/contractions
    // Split by whitespace preserving the whitespace
    let parts: Vec<String> = {
        static SPLIT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\s+)").unwrap());
        let mut result_parts = Vec::new();
        let mut last = 0;
        for m in SPLIT_RE.find_iter(&result) {
            if m.start() > last {
                result_parts.push(result[last..m.start()].to_string());
            }
            result_parts.push(m.as_str().to_string());
            last = m.end();
        }
        if last < result.len() {
            result_parts.push(result[last..].to_string());
        }
        result_parts
    };

    let mut output_parts: Vec<String> = Vec::new();
    for part in parts {
        if part.chars().all(char::is_whitespace) {
            output_parts.push(part);
            continue;
        }

        let quote_count = part.chars().filter(|&c| c == '\'').count();
        if quote_count == 1 {
            if APOSTROPHE_PATTERN.is_match(&part) {
                output_parts.push(part.replace('\'', "\u{2019}"));
            } else if POSSESSIVE_PATTERN.is_match(&part) {
                output_parts.push(part.replace('\'', "\u{2019}"));
            } else {
                output_parts.push(part);
            }
        } else {
            output_parts.push(part);
        }
    }

    output_parts.join("")
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

        if start > last_end {
            let before_text = &text[last_end..start];
            segments.push(apply_smart_quotes_to_text(before_text));
        }
        segments.push(m.as_str().to_string());
        last_end = end;
    }

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
    fn test_smart_quotes_basic() {
        assert_eq!(
            smart_quotes(r#"I'm there with "George""#),
            "I\u{2019}m there with \u{201c}George\u{201d}"
        );
    }

    #[test]
    fn test_smart_quotes_single() {
        assert_eq!(
            smart_quotes("Words in 'single quotes' work too"),
            "Words in \u{2018}single quotes\u{2019} work too"
        );
    }

    #[test]
    fn test_smart_quotes_apostrophes() {
        assert_eq!(
            smart_quotes("I'm there"),
            "I\u{2019}m there"
        );
        assert_eq!(
            smart_quotes("don't worry"),
            "don\u{2019}t worry"
        );
    }

    #[test]
    fn test_smart_quotes_possessive() {
        assert_eq!(smart_quotes("Jill's"), "Jill\u{2019}s");
        assert_eq!(smart_quotes("James'"), "James\u{2019}");
    }

    #[test]
    fn test_smart_quotes_unchanged() {
        // Code-like patterns should not be converted
        assert_eq!(smart_quotes(r#"x="foo""#), r#"x="foo""#);
        assert_eq!(smart_quotes("x='foo'"), "x='foo'");
    }

    #[test]
    fn test_smart_quotes_template_tags() {
        // Template tags should not be modified
        assert_eq!(
            smart_quotes(r#"{% field kind="string" %}"#),
            r#"{% field kind="string" %}"#
        );
        assert_eq!(
            smart_quotes("{{ variable }}"),
            "{{ variable }}"
        );
    }
}
