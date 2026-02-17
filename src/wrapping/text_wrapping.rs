//! Word splitting and paragraph wrapping.
//!
//! Ported from Python: flowmark/linewrapping/text_wrapping.py

use regex::Regex;
use std::sync::LazyLock;

use crate::wrapping::atomic_patterns::ATOMIC_CONSTRUCT_PATTERN;
use crate::wrapping::tag_handling::{denormalize_adjacent_tags, normalize_adjacent_tags};

/// Placeholder format for atomic construct extraction.
const PLACEHOLDER_PREFIX: &str = "\x00AC";
const PLACEHOLDER_SUFFIX: &str = "\x00";

/// Pattern to identify words that need escaping if they start a wrapped markdown line.
static MD_SPECIALS_PAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([-*+>]|#+)$").unwrap());

/// Pattern for numbered list markers.
static MD_NUMERAL_PAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9]+[.)]$").unwrap());

/// Pattern for replacing whitespace.
static WHITESPACE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

/// Extract all atomic constructs from text, replacing them with placeholders.
fn extract_atomic_constructs(text: &str) -> (Vec<String>, String) {
    let mut construct_map: Vec<String> = Vec::new();
    let result = ATOMIC_CONSTRUCT_PATTERN.replace_all(text, |caps: &regex::Captures<'_>| {
        let construct = caps.get(0).unwrap().as_str().to_string();
        let idx = construct_map.len();
        construct_map.push(construct);
        format!("{PLACEHOLDER_PREFIX}{idx}{PLACEHOLDER_SUFFIX}")
    });
    (construct_map, result.into_owned())
}

/// Restore original constructs from placeholders in token list.
fn restore_atomic_constructs(tokens: &[String], construct_map: &[String]) -> Vec<String> {
    tokens
        .iter()
        .map(|token| {
            let mut result = token.clone();
            for (idx, construct) in construct_map.iter().enumerate() {
                let placeholder = format!("{PLACEHOLDER_PREFIX}{idx}{PLACEHOLDER_SUFFIX}");
                result = result.replace(&placeholder, construct);
            }
            result
        })
        .collect()
}

/// Word splitter for Markdown/HTML that keeps certain constructs together.
///
/// Uses a single-pass regex extraction approach:
/// 1. Extract all atomic constructs (tags, code spans, links) with placeholders
/// 2. Split on whitespace (placeholders become single "words")
/// 3. Restore original constructs
pub fn html_md_word_split(text: &str) -> Vec<String> {
    // Normalize adjacent tags to ensure proper tokenization
    let text = normalize_adjacent_tags(text);

    // Extract all atomic constructs and replace with placeholders
    let (construct_map, text_with_placeholders) = extract_atomic_constructs(&text);

    // Split on whitespace (placeholders are single tokens)
    let tokens: Vec<String> = text_with_placeholders
        .split_whitespace()
        .map(String::from)
        .collect();

    // Restore original constructs
    restore_atomic_constructs(&tokens, &construct_map)
}

/// Simple word splitter that splits on whitespace.
pub fn simple_word_split(text: &str) -> Vec<String> {
    text.split_whitespace().map(String::from).collect()
}

/// Prepends a backslash to a word if it matches markdown patterns
/// that need escaping at the start of a wrapped line.
pub fn markdown_escape_word(word: &str) -> String {
    if MD_NUMERAL_PAT.is_match(word) {
        // Insert backslash before the `.` or `)`
        let last = &word[word.len() - 1..];
        let prefix = &word[..word.len() - 1];
        format!("{prefix}\\{last}")
    } else if MD_SPECIALS_PAT.is_match(word) {
        format!("\\{word}")
    } else {
        word.to_string()
    }
}

/// Wrap a single paragraph of text, returning a list of wrapped lines.
///
/// Set `is_markdown` to `true` when wrapping markdown text to enable Markdown mode.
/// This automatically escapes special markdown characters at the start of wrapped lines.
#[allow(clippy::fn_params_excessive_bools)]
pub fn wrap_paragraph_lines(
    text: &str,
    width: usize,
    initial_column: usize,
    subsequent_offset: usize,
    replace_whitespace: bool,
    drop_whitespace: bool,
    splitter: Option<&dyn Fn(&str) -> Vec<String>>,
    is_markdown: bool,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    // Handle width == 0 as "no wrapping".
    if width == 0 {
        let mut text = text.to_string();
        if replace_whitespace {
            text = WHITESPACE_RE.replace_all(&text, " ").into_owned();
        }
        if drop_whitespace {
            text = text.trim().to_string();
        }
        if text.is_empty() {
            return vec![];
        }
        return vec![text];
    }

    let text = if replace_whitespace {
        WHITESPACE_RE.replace_all(text, " ").into_owned()
    } else {
        text.to_string()
    };

    // Use provided splitter or default
    let default_splitter = html_md_word_split;
    let splitter = splitter.unwrap_or(&default_splitter);
    let words = splitter(&text);

    let mut current_line: Vec<String> = Vec::new();
    let mut current_width = initial_column;
    let mut first_line = true;

    for word in &words {
        let word_width = word.chars().count();
        let space_width: usize = if current_line.is_empty() { 0 } else { 1 };

        if current_width + word_width + space_width <= width {
            current_line.push(word.clone());
            current_width += word_width + space_width;
        } else {
            // Start a new line
            if !current_line.is_empty() {
                let mut line = current_line.join(" ");
                if drop_whitespace {
                    line = line.trim().to_string();
                }
                lines.push(line);
                first_line = false;
            }

            // Check if word needs escaping at the start of this wrapped line
            let escaped_word = if is_markdown && !first_line {
                markdown_escape_word(word)
            } else {
                word.clone()
            };

            let escaped_word_width = escaped_word.chars().count();
            current_line = vec![escaped_word];
            current_width = subsequent_offset + escaped_word_width;
        }
    }

    // Add the last line if necessary
    if !current_line.is_empty() {
        let mut line = current_line.join(" ");
        if drop_whitespace {
            line = line.trim().to_string();
        }
        lines.push(line);
    }

    lines
}

/// Wrap lines of a single paragraph of plain text, returning a new string.
pub fn wrap_paragraph(
    text: &str,
    width: usize,
    initial_indent: &str,
    subsequent_indent: &str,
    initial_column: usize,
    replace_whitespace: bool,
    drop_whitespace: bool,
    splitter: Option<&dyn Fn(&str) -> Vec<String>>,
    is_markdown: bool,
) -> String {
    let mut lines = wrap_paragraph_lines(
        text,
        width,
        initial_column + initial_indent.chars().count(),
        subsequent_indent.chars().count(),
        replace_whitespace,
        drop_whitespace,
        splitter,
        is_markdown,
    );

    // Insert indents on first and subsequent lines
    if !initial_indent.is_empty() && initial_column == 0 && !lines.is_empty() {
        lines[0] = format!("{initial_indent}{}", lines[0]);
    }
    if !subsequent_indent.is_empty() && lines.len() > 1 {
        for line in lines.iter_mut().skip(1) {
            *line = format!("{subsequent_indent}{line}");
        }
    }

    let result = lines.join("\n");

    // Restore original adjacency for paired tags
    denormalize_adjacent_tags(&result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_escape_word() {
        assert_eq!(markdown_escape_word("1."), "1\\.");
        assert_eq!(markdown_escape_word("10."), "10\\.");
        assert_eq!(markdown_escape_word("1)"), "1\\)");
        assert_eq!(markdown_escape_word("-"), "\\-");
        assert_eq!(markdown_escape_word("*"), "\\*");
        assert_eq!(markdown_escape_word("+"), "\\+");
        assert_eq!(markdown_escape_word(">"), "\\>");
        assert_eq!(markdown_escape_word("#"), "\\#");
        assert_eq!(markdown_escape_word("##"), "\\##");
        assert_eq!(markdown_escape_word("hello"), "hello");
    }

    #[test]
    fn test_simple_wrapping() {
        let lines = wrap_paragraph_lines(
            "Hello world this is a test",
            10,
            0,
            0,
            true,
            true,
            Some(&simple_word_split),
            false,
        );
        assert!(!lines.is_empty());
        for line in &lines {
            assert!(line.chars().count() <= 10 || line.split_whitespace().count() == 1);
        }
    }

    #[test]
    fn test_no_wrap() {
        let lines = wrap_paragraph_lines(
            "Hello world this is a test",
            0,
            0,
            0,
            true,
            true,
            None,
            false,
        );
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Hello world this is a test");
    }

    #[test]
    fn test_html_md_word_split() {
        let words = html_md_word_split("Hello `code` world");
        assert_eq!(words, vec!["Hello", "`code`", "world"]);
    }

    #[test]
    fn test_html_md_word_split_links() {
        let words = html_md_word_split("See [link](url) here");
        assert_eq!(words, vec!["See", "[link](url)", "here"]);
    }
}
