//! Text wrapping with HTML/Markdown-aware word splitting.
//!
//! Ported from Python: flowmark/linewrapping/text_wrapping.py

use once_cell::sync::Lazy;
use regex::Regex;

use crate::linewrapping::atomic_patterns::ATOMIC_CONSTRUCT_PATTERN;
use crate::linewrapping::tag_handling::{denormalize_adjacent_tags, normalize_adjacent_tags};

/// Default length function: just character count.
pub fn default_len(s: &str) -> usize {
    s.len()
}

/// Simple word splitter: split on whitespace.
pub fn simple_word_splitter(text: &str) -> Vec<String> {
    text.split_whitespace().map(String::from).collect()
}

// Placeholder format for atomic construct extraction.
const PLACEHOLDER_PREFIX: &str = "\x00AC";
const PLACEHOLDER_SUFFIX: &str = "\x00";

/// Extract all atomic constructs from text, replacing them with placeholders.
fn extract_atomic_constructs(text: &str) -> (Vec<(usize, String)>, String) {
    let mut construct_map: Vec<(usize, String)> = Vec::new();
    let mut result = String::new();
    let mut last_end = 0;
    let mut placeholder_idx = 0;

    for mat_result in ATOMIC_CONSTRUCT_PATTERN.find_iter(text) {
        let mat = mat_result.expect("atomic construct regex match failed");
        // Add text before match
        result.push_str(&text[last_end..mat.start()]);
        // Add placeholder for the matched construct
        let construct = mat.as_str().to_string();
        construct_map.push((placeholder_idx, construct));
        result.push_str(&format!("{PLACEHOLDER_PREFIX}{placeholder_idx}{PLACEHOLDER_SUFFIX}"));
        placeholder_idx += 1;
        last_end = mat.end();
    }
    result.push_str(&text[last_end..]);

    (construct_map, result)
}

/// Restore original constructs from placeholders in token list.
fn restore_atomic_constructs(tokens: &[String], construct_map: &[(usize, String)]) -> Vec<String> {
    tokens
        .iter()
        .map(|token| {
            let mut result = token.clone();
            for (idx, construct) in construct_map {
                let placeholder = format!("{PLACEHOLDER_PREFIX}{idx}{PLACEHOLDER_SUFFIX}");
                result = result.replace(&placeholder, construct);
            }
            result
        })
        .collect()
}

/// HTML/Markdown-aware word splitter that keeps certain constructs together.
///
/// Uses a single-pass regex extraction approach:
/// 1. Extract all atomic constructs (tags, code spans, links) with placeholders
/// 2. Split on whitespace (placeholders become single "words")
/// 3. Restore original constructs
pub fn html_md_word_splitter(text: &str) -> Vec<String> {
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

/// Get the cached word splitter function.
/// In Rust, we just return the function directly since there's no instance to cache.
pub fn get_html_md_word_splitter() -> fn(&str) -> Vec<String> {
    html_md_word_splitter
}

/// Pattern to identify words that need escaping if they start a wrapped markdown line.
static MD_SPECIALS_PAT: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([-*+>]|#+)$").unwrap());

/// Pattern for numbered list markers.
static MD_NUMERAL_PAT: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[0-9]+[.)]$").unwrap());

/// Prepends a backslash to a word if it matches markdown patterns
/// that need escaping at the start of a wrapped line.
pub fn markdown_escape_word(word: &str) -> String {
    if MD_NUMERAL_PAT.is_match(word) {
        // Insert backslash before the `.` or `)`
        let (prefix, suffix) = word.split_at(word.len() - 1);
        format!("{prefix}\\{suffix}")
    } else if MD_SPECIALS_PAT.is_match(word) {
        format!("\\{word}")
    } else {
        word.to_string()
    }
}

/// Wrap a single paragraph of text, returning a list of wrapped lines.
///
/// Set `is_markdown` to True when wrapping markdown text to enable Markdown mode.
/// This automatically escapes special markdown characters at the start of wrapped lines.
pub fn wrap_paragraph_lines(
    text: &str,
    width: usize,
    initial_column: usize,
    subsequent_offset: usize,
    replace_whitespace: bool,
    drop_whitespace: bool,
    splitter: Option<fn(&str) -> Vec<String>>,
    len_fn: fn(&str) -> usize,
    is_markdown: bool,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    // Handle width == 0 as "no wrapping"
    if width == 0 {
        let text = if replace_whitespace {
            static WS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
            WS_RE.replace_all(text, " ").into_owned()
        } else {
            text.to_string()
        };
        let text = if drop_whitespace {
            text.trim().to_string()
        } else {
            text
        };
        if !text.is_empty() {
            return vec![text];
        }
        return vec![];
    }

    let text = if replace_whitespace {
        static WS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
        WS_RE.replace_all(text, " ").into_owned()
    } else {
        text.to_string()
    };

    let splitter = splitter.unwrap_or(html_md_word_splitter);
    let words = splitter(&text);

    let mut current_line: Vec<String> = Vec::new();
    let mut current_width = initial_column;
    let mut first_line = true;

    for word in words {
        let word_width = len_fn(&word);
        let space_width: usize = if current_line.is_empty() { 0 } else { 1 };

        if current_width + word_width + space_width <= width {
            current_line.push(word);
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
                markdown_escape_word(&word)
            } else {
                word
            };

            let escaped_word_width = len_fn(&escaped_word);
            current_line = vec![escaped_word];
            current_width = subsequent_offset + escaped_word_width;
        }
    }

    // Add the last line
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
    word_splitter: Option<fn(&str) -> Vec<String>>,
    len_fn: fn(&str) -> usize,
    is_markdown: bool,
) -> String {
    let mut lines = wrap_paragraph_lines(
        text,
        width,
        initial_column + len_fn(initial_indent),
        len_fn(subsequent_indent),
        replace_whitespace,
        drop_whitespace,
        word_splitter,
        len_fn,
        is_markdown,
    );

    // Insert indents on first and subsequent lines
    if !initial_indent.is_empty() && initial_column == 0 && !lines.is_empty() {
        lines[0] = format!("{initial_indent}{}", lines[0]);
    }
    if !subsequent_indent.is_empty() && lines.len() > 1 {
        for line in &mut lines[1..] {
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
    fn test_markdown_escape_word_function() {
        assert_eq!(markdown_escape_word("-"), "\\-");
        assert_eq!(markdown_escape_word("+"), "\\+");
        assert_eq!(markdown_escape_word("*"), "\\*");
        assert_eq!(markdown_escape_word(">"), "\\>");
        assert_eq!(markdown_escape_word("#"), "\\#");
        assert_eq!(markdown_escape_word("##"), "\\##");
        assert_eq!(markdown_escape_word("1."), "1\\.");
        assert_eq!(markdown_escape_word("10."), "10\\.");
        assert_eq!(markdown_escape_word("1)"), "1\\)");
        assert_eq!(markdown_escape_word("99)"), "99\\)");

        // Cases that should NOT be escaped
        assert_eq!(markdown_escape_word("word"), "word");
        assert_eq!(markdown_escape_word("-word"), "-word");
        assert_eq!(markdown_escape_word("word-"), "word-");
        assert_eq!(markdown_escape_word("#word"), "#word");
        assert_eq!(markdown_escape_word("word#"), "word#");
        assert_eq!(markdown_escape_word("1.word"), "1.word");
        assert_eq!(markdown_escape_word("word1."), "word1.");
        assert_eq!(markdown_escape_word("1)word"), "1)word");
        assert_eq!(markdown_escape_word("word1)"), "word1)");
        assert_eq!(markdown_escape_word("<tag>"), "<tag>");
        assert_eq!(markdown_escape_word("[link]"), "[link]");
        assert_eq!(markdown_escape_word("1"), "1");
        assert_eq!(markdown_escape_word("."), ".");
    }

    #[test]
    fn test_wrap_paragraph_lines_markdown_escaping() {
        let result = wrap_paragraph_lines("- word", 10, 0, 0, true, true, None, default_len, true);
        assert_eq!(result, vec!["- word"]);

        let text = "word - word * word + word > word # word ## word 1. word 2) word";
        let result = wrap_paragraph_lines(text, 5, 0, 0, true, true, None, default_len, true);
        assert_eq!(
            result,
            vec![
                "word", "\\-", "word", "\\*", "word", "\\+", "word", "\\>", "word", "\\#",
                "word", "\\##", "word", "1\\.", "word", "2\\)", "word",
            ]
        );

        let result = wrap_paragraph_lines(text, 10, 0, 0, true, true, None, default_len, true);
        assert_eq!(
            result,
            vec![
                "word -", "word *", "word +", "word >", "word #", "word ##", "word 1.",
                "word 2)", "word",
            ]
        );

        let result = wrap_paragraph_lines(text, 15, 0, 0, true, true, None, default_len, true);
        assert_eq!(
            result,
            vec![
                "word - word *",
                "word + word >",
                "word # word ##",
                "word 1. word 2)",
                "word",
            ]
        );

        let result = wrap_paragraph_lines(text, 20, 0, 0, true, true, None, default_len, true);
        assert_eq!(
            result,
            vec![
                "word - word * word +",
                "word > word # word",
                "\\## word 1. word 2)",
                "word",
            ]
        );

        let result = wrap_paragraph_lines(text, 20, 0, 0, true, true, None, default_len, false);
        assert_eq!(
            result,
            vec![
                "word - word * word +",
                "word > word # word",
                "## word 1. word 2)",
                "word",
            ]
        );
    }

    #[test]
    fn test_smart_splitter() {
        let html_text =
            "This is <span class='test'>some text</span> and <a href='#'>this is a link</a>.";
        let result = html_md_word_splitter(html_text);
        assert_eq!(
            result,
            vec![
                "This",
                "is",
                "<span class='test'>some",
                "text</span>",
                "and",
                "<a href='#'>this",
                "is",
                "a",
                "link</a>.",
            ]
        );

        let md_text =
            "Here's a [Markdown link](https://example.com) and [another one](https://test.com).";
        let result = html_md_word_splitter(md_text);
        assert_eq!(
            result,
            vec![
                "Here's",
                "a",
                "[Markdown link](https://example.com)",
                "and",
                "[another one](https://test.com).",
            ]
        );
    }

    #[test]
    fn test_wrap_text() {
        let sample_text = "This is a sample text with a [Markdown link](https://example.com) \
            and an <a href='#'>tag</a>. It should demonstrate the functionality of \
            our enhanced text wrapping implementation.";

        let filled = wrap_paragraph(
            sample_text,
            40,
            ">",
            ">>",
            0,
            true,
            true,
            Some(simple_word_splitter),
            default_len,
            false,
        );
        let expected = ">This is a sample text with a [Markdown\n\
            >>link](https://example.com) and an <a\n\
            >>href='#'>tag</a>. It should\n\
            >>demonstrate the functionality of our\n\
            >>enhanced text wrapping implementation.";
        assert_eq!(filled, expected);

        let filled_smart = wrap_paragraph(
            sample_text,
            40,
            ">",
            ">>",
            0,
            true,
            true,
            Some(html_md_word_splitter),
            default_len,
            false,
        );
        let expected_smart = ">This is a sample text with a\n\
            >>[Markdown link](https://example.com)\n\
            >>and an <a href='#'>tag</a>. It should\n\
            >>demonstrate the functionality of our\n\
            >>enhanced text wrapping implementation.";
        assert_eq!(filled_smart, expected_smart);

        let filled_offset = wrap_paragraph(
            sample_text,
            40,
            ">",
            ">>",
            35,
            true,
            true,
            Some(html_md_word_splitter),
            default_len,
            false,
        );
        let expected_offset = "This\n\
            >>is a sample text with a\n\
            >>[Markdown link](https://example.com)\n\
            >>and an <a href='#'>tag</a>. It should\n\
            >>demonstrate the functionality of our\n\
            >>enhanced text wrapping implementation.";
        assert_eq!(filled_offset, expected_offset);
    }

    #[test]
    fn test_inline_code_with_spaces() {
        let code = "`code with spaces`";
        let text = format!("Some {code} here.");
        let result = html_md_word_splitter(&text);
        assert!(result.contains(&code.to_string()));
    }

    #[test]
    fn test_html_comments_kept_together() {
        let comment = "<!-- a comment -->";
        let text = format!("Text with {comment} inline.");
        let result = html_md_word_splitter(&text);
        assert!(result.contains(&comment.to_string()));
    }

    #[test]
    fn test_template_tag_splitter() {
        let text = "Text with {% if $condition %} template tags {% endif %} here.";
        let result = html_md_word_splitter(text);
        assert!(result.contains(&"{% if $condition %}".to_string()));
        assert!(result.contains(&"{% endif %}".to_string()));
    }
}
