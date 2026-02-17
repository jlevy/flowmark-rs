//! Tag handling for Jinja/Markdoc tags and HTML comments.
//!
//! Ported from Python: flowmark/linewrapping/tag_handling.py

use once_cell::sync::Lazy;
use regex::Regex;

use crate::linewrapping::atomic_patterns::{
    SINGLE_HTML_COMMENT, SINGLE_JINJA_COMMENT, SINGLE_JINJA_TAG, SINGLE_JINJA_VAR,
};
use crate::linewrapping::block_heuristics::line_is_block_content;
use crate::linewrapping::protocols::LineWrapper;

/// Pattern to detect adjacent tags (closing tag immediately followed by opening tag).
static ADJACENT_TAGS_RE: Lazy<Regex> = Lazy::new(|| {
    let pattern = format!(
        "({close1})({open1})|({close2})({open2})|({close3})({open3})|({close4})({open4})",
        close1 = SINGLE_JINJA_TAG.close_re,
        open1 = SINGLE_JINJA_TAG.open_re,
        close2 = SINGLE_JINJA_COMMENT.close_re,
        open2 = SINGLE_JINJA_COMMENT.open_re,
        close3 = SINGLE_JINJA_VAR.close_re,
        open3 = SINGLE_JINJA_VAR.open_re,
        close4 = SINGLE_HTML_COMMENT.close_re,
        open4 = SINGLE_HTML_COMMENT.open_re,
    );
    Regex::new(&pattern).unwrap()
});

/// Pattern to remove spaces between adjacent tags added during word splitting.
static DENORMALIZE_TAGS_RE: Lazy<Regex> = Lazy::new(|| {
    let pattern = format!(
        "({close1}) ({open1})|({close2}) ({open2})|({close3}) ({open3})|({close4}) ({open4})",
        close1 = SINGLE_JINJA_TAG.close_re,
        open1 = SINGLE_JINJA_TAG.open_re,
        close2 = SINGLE_JINJA_COMMENT.close_re,
        open2 = SINGLE_JINJA_COMMENT.open_re,
        close3 = SINGLE_JINJA_VAR.close_re,
        open3 = SINGLE_JINJA_VAR.open_re,
        close4 = SINGLE_HTML_COMMENT.close_re,
        open4 = SINGLE_HTML_COMMENT.open_re,
    );
    Regex::new(&pattern).unwrap()
});

/// Pattern for multiline closing tag detection.
static MULTILINE_CLOSING_PATTERN: Lazy<Regex> = Lazy::new(|| {
    let pattern = format!(
        r"{close1}\s*(?P<closing_tag>{open1}\s*/)|{close2}\s*(?P<closing_comment>{open2}\s*/)|{close3}\s*(?P<closing_var>{open3}\s*/)|{close4}\s*(?P<closing_html>{open4}\s*/)",
        close1 = SINGLE_JINJA_TAG.close_re,
        open1 = SINGLE_JINJA_TAG.open_re,
        close2 = SINGLE_JINJA_COMMENT.close_re,
        open2 = SINGLE_JINJA_COMMENT.open_re,
        close3 = SINGLE_JINJA_VAR.close_re,
        open3 = SINGLE_JINJA_VAR.open_re,
        close4 = SINGLE_HTML_COMMENT.close_re,
        open4 = SINGLE_HTML_COMMENT.open_re,
    );
    Regex::new(&pattern).unwrap()
});

/// Add a space between adjacent tags so they become separate tokens.
pub fn normalize_adjacent_tags(text: &str) -> String {
    ADJACENT_TAGS_RE
        .replace_all(text, |caps: &regex::Captures| {
            let groups: Vec<Option<regex::Match>> =
                (1..=caps.len()).map(|i| caps.get(i)).collect();
            for i in (0..groups.len()).step_by(2) {
                if let (Some(g1), Some(g2)) = (&groups.get(i).and_then(|g| *g), &groups.get(i + 1).and_then(|g| *g)) {
                    return format!("{} {}", g1.as_str(), g2.as_str());
                }
            }
            caps[0].to_string()
        })
        .into_owned()
}

/// Remove spaces between adjacent tags that were added during word splitting.
pub fn denormalize_adjacent_tags(text: &str) -> String {
    DENORMALIZE_TAGS_RE
        .replace_all(text, |caps: &regex::Captures| {
            let groups: Vec<Option<regex::Match>> =
                (1..=caps.len()).map(|i| caps.get(i)).collect();
            for i in (0..groups.len()).step_by(2) {
                if let (Some(g1), Some(g2)) = (&groups.get(i).and_then(|g| *g), &groups.get(i + 1).and_then(|g| *g)) {
                    return format!("{}{}", g1.as_str(), g2.as_str());
                }
            }
            caps[0].to_string()
        })
        .into_owned()
}

/// Check if a line contains only a tag (opening or closing), not inline tags in content.
fn is_tag_only_line(line: &str) -> bool {
    if line.starts_with(|c: char| c.is_whitespace()) {
        return false;
    }

    let stripped = line.trim();
    if stripped.is_empty() {
        return false;
    }

    let starts_tag = stripped.starts_with(SINGLE_JINJA_TAG.open_delim)
        || stripped.starts_with(SINGLE_JINJA_COMMENT.open_delim)
        || stripped.starts_with(SINGLE_JINJA_VAR.open_delim)
        || stripped.starts_with(SINGLE_HTML_COMMENT.open_delim);

    let ends_tag = stripped.ends_with(SINGLE_JINJA_TAG.close_delim)
        || stripped.ends_with(SINGLE_JINJA_COMMENT.close_delim)
        || stripped.ends_with(SINGLE_JINJA_VAR.close_delim)
        || stripped.ends_with(SINGLE_HTML_COMMENT.close_delim);

    starts_tag && ends_tag
}

/// Preprocess text to ensure proper blank lines around block content within tags.
pub fn preprocess_tag_block_spacing(text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();

    let has_tag_only_lines = lines.iter().any(|line| is_tag_only_line(line));
    if !has_tag_only_lines {
        return text.to_string();
    }

    let mut result_lines: Vec<String> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            let prev_line = lines[i - 1];
            let prev_is_empty = prev_line.trim().is_empty();

            // Case 1: Previous line is a tag-only line, current line is block content
            if !prev_is_empty && is_tag_only_line(prev_line) && line_is_block_content(line) {
                result_lines.push(String::new());
            }

            // Case 2: Previous line is block content, current line is a closing tag-only line
            if !prev_is_empty && line_is_block_content(prev_line) && is_tag_only_line(line) {
                result_lines.push(String::new());
            }
        }

        result_lines.push((*line).to_string());
    }

    result_lines.join("\n")
}

/// Check if a line ends with a Jinja/Markdoc tag or HTML comment.
pub fn line_ends_with_tag(line: &str) -> bool {
    let stripped = line.trim_end();
    if stripped.is_empty() {
        return false;
    }
    stripped.ends_with(SINGLE_JINJA_TAG.close_delim)
        || stripped.ends_with(SINGLE_JINJA_COMMENT.close_delim)
        || stripped.ends_with(SINGLE_JINJA_VAR.close_delim)
        || stripped.ends_with(SINGLE_HTML_COMMENT.close_delim)
}

/// Check if a line starts with a Jinja/Markdoc tag or HTML comment.
pub fn line_starts_with_tag(line: &str) -> bool {
    let stripped = line.trim_start();
    if stripped.is_empty() {
        return false;
    }
    stripped.starts_with(SINGLE_JINJA_TAG.open_delim)
        || stripped.starts_with(SINGLE_JINJA_COMMENT.open_delim)
        || stripped.starts_with(SINGLE_JINJA_VAR.open_delim)
        || stripped.starts_with(SINGLE_HTML_COMMENT.open_delim)
}

/// Check if a line is unindented and starts with a tag.
fn is_unindented_tag_line(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    if line.starts_with(|c: char| c.is_whitespace()) {
        return false;
    }
    line_starts_with_tag(line)
}

/// Check if a line is a closing tag.
fn is_closing_tag(line: &str) -> bool {
    let stripped = line.trim_start();
    stripped.starts_with("{% /")
        || stripped.starts_with("{# /")
        || stripped.starts_with("{{ /")
        || stripped.starts_with("<!-- /")
}

/// Fix closing tag spacing for block content only.
fn fix_closing_tag_spacing(text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut fixed_lines: Vec<String> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if is_closing_tag(line) {
            let stripped = line.trim_start().to_string();
            if i > 0 && !fixed_lines.is_empty() {
                let prev_line = &fixed_lines[fixed_lines.len() - 1];
                let prev_is_empty = prev_line.trim().is_empty();
                let prev_is_block = line_is_block_content(prev_line);
                if !prev_is_empty && prev_is_block {
                    fixed_lines.push(String::new());
                }
            }
            fixed_lines.push(stripped);
        } else {
            fixed_lines.push((*line).to_string());
        }
    }

    fixed_lines.join("\n")
}

/// Ensure closing tags are on their own line when the opening tag spans multiple lines.
fn fix_multiline_opening_tag_with_closing(text: &str) -> String {
    if !text.contains('\n') {
        return text.to_string();
    }

    let lines: Vec<&str> = text.split('\n').collect();
    let mut result_lines: Vec<String> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            result_lines.push((*line).to_string());
            continue;
        }

        let stripped = line.trim_start();
        let is_tag_start = stripped.starts_with(SINGLE_JINJA_TAG.open_delim)
            || stripped.starts_with(SINGLE_JINJA_COMMENT.open_delim)
            || stripped.starts_with(SINGLE_JINJA_VAR.open_delim)
            || stripped.starts_with(SINGLE_HTML_COMMENT.open_delim);

        if !is_tag_start {
            if let Some(m) = MULTILINE_CLOSING_PATTERN.find(line) {
                // Find the actual closing tag start position
                let closing_start = if let Some(caps) = MULTILINE_CLOSING_PATTERN.captures(line) {
                    caps.name("closing_tag")
                        .or_else(|| caps.name("closing_comment"))
                        .or_else(|| caps.name("closing_var"))
                        .or_else(|| caps.name("closing_html"))
                        .map(|m| m.start())
                } else {
                    Some(m.start())
                };

                if let Some(split_pos) = closing_start {
                    let before = line[..split_pos].trim_end();
                    let closing = line[split_pos..].trim_start();
                    result_lines.push(before.to_string());
                    result_lines.push(closing.to_string());
                    continue;
                }
            }
        }

        result_lines.push((*line).to_string());
    }

    result_lines.join("\n")
}

/// Augments a `LineWrapper` to preserve newlines around tags and HTML comments.
pub fn add_tag_newline_handling(base_wrapper: LineWrapper) -> LineWrapper {
    Box::new(move |text: &str, initial_indent: &str, subsequent_indent: &str| -> String {
        // If no newlines in input, just wrap and apply post-processing fixes
        if !text.contains('\n') {
            let result = base_wrapper(text, initial_indent, subsequent_indent);
            return fix_multiline_opening_tag_with_closing(&result);
        }

        let lines: Vec<&str> = text.split('\n').collect();

        if lines.len() <= 1 {
            let result = base_wrapper(text, initial_indent, subsequent_indent);
            return fix_multiline_opening_tag_with_closing(&result);
        }

        // Check if there are any tags in the text
        let has_tags = lines.iter().any(|line| line_ends_with_tag(line) || line_starts_with_tag(line));

        // Group lines into segments that should be wrapped together
        let mut segments: Vec<String> = Vec::new();
        let mut current_segment_lines: Vec<&str> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let is_first_line = i == 0;
            let prev_ends_with_tag = !is_first_line && line_ends_with_tag(lines[i - 1]);
            let curr_starts_with_tag = is_unindented_tag_line(line);
            let curr_is_block = has_tags && line_is_block_content(line);
            let prev_is_block = has_tags && !is_first_line && line_is_block_content(lines[i - 1]);

            if prev_ends_with_tag || curr_starts_with_tag || curr_is_block || prev_is_block {
                if !current_segment_lines.is_empty() {
                    segments.push(current_segment_lines.join("\n"));
                    current_segment_lines.clear();
                }
            }

            current_segment_lines.push(line);
        }
        if !current_segment_lines.is_empty() {
            segments.push(current_segment_lines.join("\n"));
        }

        if segments.len() == 1 {
            let result = base_wrapper(text, initial_indent, subsequent_indent);
            return fix_multiline_opening_tag_with_closing(&result);
        }

        // Wrap each segment separately
        let mut wrapped_segments: Vec<String> = Vec::new();
        for (i, segment) in segments.iter().enumerate() {
            let cur_indent = if i == 0 { initial_indent } else { subsequent_indent };
            let wrapped = base_wrapper(segment, cur_indent, subsequent_indent);
            wrapped_segments.push(wrapped);
        }

        // Rejoin segments, normalizing newlines around block content
        let mut result_parts: Vec<String> = Vec::new();
        for (i, wrapped) in wrapped_segments.iter().enumerate() {
            if i == 0 {
                result_parts.push(wrapped.clone());
                continue;
            }

            let prev_segment = &segments[i - 1];
            let curr_segment = &segments[i];

            let prev_is_block = prev_segment.split('\n').any(|l| line_is_block_content(l));
            let curr_is_block = curr_segment.split('\n').any(|l| line_is_block_content(l));
            let prev_is_tag = prev_segment.split('\n').last().map_or(false, line_ends_with_tag);
            let curr_is_tag = curr_segment.split('\n').next().map_or(false, |l| is_unindented_tag_line(l));

            if (prev_is_tag && curr_is_block) || (prev_is_block && curr_is_tag) {
                result_parts.push(String::new());
                result_parts.push(wrapped.clone());
            } else {
                result_parts.push(wrapped.clone());
            }
        }

        let result = result_parts.join("\n");
        let result = fix_closing_tag_spacing(&result);
        fix_multiline_opening_tag_with_closing(&result)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_denormalize_adjacent_tags() {
        let original = "{% field kind='string' %}{% /field %}";
        let normalized = normalize_adjacent_tags(original);
        assert_eq!(normalized, "{% field kind='string' %} {% /field %}");
        let denormalized = denormalize_adjacent_tags(&normalized);
        assert_eq!(denormalized, original);
    }

    #[test]
    fn test_normalize_html_comment_tags() {
        let original = r#"<!-- f:field kind="string" id="name" --><!-- /f:field -->"#;
        let normalized = normalize_adjacent_tags(original);
        assert!(normalized.contains(" <!-- /f:field -->"));
        let denormalized = denormalize_adjacent_tags(&normalized);
        assert_eq!(denormalized, original);
    }

    #[test]
    fn test_fix_closing_tag_spacing() {
        // Paragraph text - NO blank line added
        assert_eq!(
            fix_closing_tag_spacing("Regular text.\n{% /tag %}"),
            "Regular text.\n{% /tag %}"
        );

        // List item - blank line added
        assert_eq!(
            fix_closing_tag_spacing("- List item\n{% /tag %}"),
            "- List item\n\n{% /tag %}"
        );

        // Table row - blank line added
        assert_eq!(
            fix_closing_tag_spacing("| A | B |\n{% /tag %}"),
            "| A | B |\n\n{% /tag %}"
        );

        // Already has blank line - no change
        assert_eq!(
            fix_closing_tag_spacing("- Item\n\n{% /tag %}"),
            "- Item\n\n{% /tag %}"
        );
    }
}
