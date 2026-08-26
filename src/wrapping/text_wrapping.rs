//! Word splitting and paragraph wrapping.
//!
//! Ported from Python: `flowmark/linewrapping/text_wrapping.py`

use regex::Regex;
use std::sync::LazyLock;

use crate::preservation::ProtectedSource;
use crate::wrapping::atomic_patterns::ATOMIC_CONSTRUCT_PATTERN;
use crate::wrapping::tag_handling::{denormalize_adjacent_tags, normalize_adjacent_tags};

/// Placeholder format for atomic construct extraction.
const PLACEHOLDER_PREFIX: &str = "\x00AC";
const PLACEHOLDER_SUFFIX: &str = "\x00";
/// Filler character used to pad placeholders to match original construct width.
const PLACEHOLDER_FILLER: char = '\x01';

/// Pattern to identify words that need escaping if they start a wrapped markdown line.
static MD_SPECIALS_PAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([-*+>]|#+)$").expect("valid MD_SPECIALS_PAT regex"));

/// Pattern for numbered list markers.
static MD_NUMERAL_PAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9]+[.)]$").expect("valid MD_NUMERAL_PAT regex"));

/// Pattern for replacing whitespace.
static WHITESPACE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+").expect("valid WHITESPACE_RE regex"));

/// Extract all atomic constructs from text, replacing them with width-preserving
/// placeholders. The placeholder is padded with filler chars so the wrapping
/// algorithm counts the same character width as the original construct.
fn extract_atomic_constructs(text: &str) -> (Vec<String>, Vec<String>, String) {
    let mut constructs: Vec<String> = Vec::new();
    let mut placeholders: Vec<String> = Vec::new();
    let result = ATOMIC_CONSTRUCT_PATTERN.replace_all(text, |caps: &regex::Captures<'_>| {
        let construct = caps.get(0).expect("group 0 always exists").as_str().to_string();
        let idx = constructs.len();
        let construct_len = construct.chars().count();
        let core = format!("{PLACEHOLDER_PREFIX}{idx}{PLACEHOLDER_SUFFIX}");
        let core_len = core.chars().count();
        let placeholder = if construct_len > core_len {
            let padding: String =
                std::iter::repeat_n(PLACEHOLDER_FILLER, construct_len - core_len).collect();
            format!("{PLACEHOLDER_PREFIX}{idx}{padding}{PLACEHOLDER_SUFFIX}")
        } else {
            core
        };
        constructs.push(construct);
        placeholders.push(placeholder.clone());
        placeholder
    });
    (constructs, placeholders, result.into_owned())
}

/// Restore original constructs from placeholders in token list.
///
/// Uses a single-pass scan per token: looks for the placeholder prefix byte (`\x00`)
/// and checks for the full `\x00AC<idx><filler>\x00` pattern, avoiding O(N×M) string
/// searches.
fn restore_atomic_constructs(
    tokens: &[String],
    constructs: &[String],
    placeholders: &[String],
) -> Vec<String> {
    if constructs.is_empty() {
        return tokens.to_vec();
    }

    // Build a lookup from placeholder string → construct index for O(1) matching.
    let placeholder_map: std::collections::HashMap<&str, &str> =
        placeholders.iter().zip(constructs.iter()).map(|(p, c)| (p.as_str(), c.as_str())).collect();

    tokens
        .iter()
        .map(|token| {
            // Fast path: if the token doesn't contain the prefix byte, no placeholders.
            if !token.contains('\x00') {
                return token.clone();
            }

            // Check if the entire token is a placeholder (common case: atomic constructs
            // become single whitespace-delimited tokens).
            if let Some(construct) = placeholder_map.get(token.as_str()) {
                return (*construct).to_string();
            }

            // Slow path: token contains embedded placeholders mixed with other text.
            // Fall back to sequential replacement (rare in practice).
            let mut result = token.clone();
            for (placeholder, construct) in placeholders.iter().zip(constructs.iter()) {
                if result.contains(placeholder.as_str()) {
                    result = result.replace(placeholder.as_str(), construct);
                }
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

    // Extract all atomic constructs and replace with width-preserving placeholders
    let (constructs, placeholders, text_with_placeholders) = extract_atomic_constructs(&text);

    // Split on whitespace (placeholders are single tokens)
    let tokens: Vec<String> = text_with_placeholders.split_whitespace().map(String::from).collect();

    // Restore original constructs
    restore_atomic_constructs(&tokens, &constructs, &placeholders)
}

/// Simple word splitter that splits on whitespace.
/// Not used in the production pipeline — available for tests and external consumers.
pub fn simple_word_split(text: &str) -> Vec<String> {
    text.split_whitespace().map(String::from).collect()
}

/// Prepends a backslash to a word if it matches markdown patterns
/// that need escaping at the start of a wrapped line.
pub fn markdown_escape_word(word: &str) -> String {
    if MD_NUMERAL_PAT.is_match(word) {
        // Insert backslash before the last character (`.` or `)`)
        let last_char_len = word.chars().next_back().map_or(0, char::len_utf8);
        let prefix = &word[..word.len() - last_char_len];
        let last = &word[word.len() - last_char_len..];
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
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools, clippy::type_complexity)]
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
        let space_width: usize = usize::from(!current_line.is_empty());

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
            let escaped_word =
                if is_markdown && !first_line { markdown_escape_word(word) } else { word.clone() };

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

/// Wrap parser-facing words while measuring protected tokens by their authored source.
#[allow(clippy::too_many_arguments)]
pub(crate) fn wrap_paragraph_lines_protected(
    text: &str,
    width: usize,
    initial_column: usize,
    subsequent_offset: usize,
    drop_whitespace: bool,
    is_markdown: bool,
    protected: &ProtectedSource,
) -> Vec<String> {
    if width == 0 {
        let collapsed = WHITESPACE_RE.replace_all(text, " ");
        let collapsed = if drop_whitespace { collapsed.trim() } else { &collapsed };
        return (!collapsed.is_empty()).then(|| collapsed.to_string()).into_iter().collect();
    }

    let normalized = WHITESPACE_RE.replace_all(text, " ");
    let words = html_md_word_split(&normalized);
    let mut lines = Vec::new();
    let mut current_line = Vec::<String>::new();
    let mut current_width = initial_column;
    let mut first_line = true;

    for word in words {
        let metrics = protected
            .measure_inline_text(&word)
            .expect("scanner-selected preservation tokens must remain canonical during wrapping");
        let space_width = usize::from(!current_line.is_empty());
        if current_width + space_width + metrics.first_width <= width {
            current_line.push(word);
            if metrics.has_authored_break {
                current_width = metrics.final_width;
                first_line = false;
            } else {
                current_width += space_width + metrics.final_width;
            }
            continue;
        }

        if !current_line.is_empty() {
            let line = current_line.join(" ");
            let line = if drop_whitespace { line.trim().to_owned() } else { line };
            lines.push(line);
            first_line = false;
        }
        let escaped = if is_markdown && !first_line { markdown_escape_word(&word) } else { word };
        let escaped_metrics = protected
            .measure_inline_text(&escaped)
            .expect("escaped preservation word must remain canonical");
        current_line = vec![escaped];
        current_width = if escaped_metrics.has_authored_break {
            first_line = false;
            escaped_metrics.final_width
        } else {
            subsequent_offset + escaped_metrics.final_width
        };
    }
    if !current_line.is_empty() {
        let line = current_line.join(" ");
        let line = if drop_whitespace { line.trim().to_owned() } else { line };
        lines.push(line);
    }
    lines
}

/// Wrap lines of a single paragraph of plain text, returning a new string.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn wrap_paragraph_protected(
    text: &str,
    width: usize,
    initial_indent: &str,
    subsequent_indent: &str,
    protected: &ProtectedSource,
    is_markdown: bool,
) -> String {
    let mut lines = wrap_paragraph_lines_protected(
        text,
        width,
        initial_indent.chars().count(),
        subsequent_indent.chars().count(),
        true,
        is_markdown,
        protected,
    );
    if !initial_indent.is_empty() && !lines.is_empty() {
        lines[0] = format!("{initial_indent}{}", lines[0]);
    }
    if !subsequent_indent.is_empty() && lines.len() > 1 {
        for line in lines.iter_mut().skip(1) {
            *line = format!("{subsequent_indent}{line}");
        }
    }
    denormalize_adjacent_tags(&lines.join("\n"))
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
        let lines =
            wrap_paragraph_lines("Hello world this is a test", 0, 0, 0, true, true, None, false);
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
