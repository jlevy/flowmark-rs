//! Tag handling for Jinja/Markdoc tags and HTML comments.
//!
//! Ported from Python: `flowmark/linewrapping/tag_handling.py`

use regex::Regex;
use std::sync::LazyLock;

use crate::wrapping::atomic_patterns::{
    SINGLE_HTML_COMMENT, SINGLE_JINJA_COMMENT, SINGLE_JINJA_TAG, SINGLE_JINJA_VAR,
};
use crate::wrapping::block_heuristics::line_is_block_content;
use crate::wrapping::LineWrapper;

/// Pattern to match complete template tags (for protecting content inside tags).
pub(crate) static TEMPLATE_TAG_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    let patterns = [
        SINGLE_JINJA_TAG.pattern,
        SINGLE_JINJA_COMMENT.pattern,
        SINGLE_JINJA_VAR.pattern,
        SINGLE_HTML_COMMENT.pattern,
    ];
    Regex::new(&format!("(?s){}", patterns.join("|"))).unwrap()
});

/// Pattern to detect adjacent tags (closing tag immediately followed by opening tag).
static ADJACENT_TAGS_RE: LazyLock<Regex> = LazyLock::new(|| {
    let pattern = format!(
        "({close_jt})({open_jt})|({close_jc})({open_jc})|({close_jv})({open_jv})|({close_hc})({open_hc})",
        close_jt = SINGLE_JINJA_TAG.close_re,
        open_jt = SINGLE_JINJA_TAG.open_re,
        close_jc = SINGLE_JINJA_COMMENT.close_re,
        open_jc = SINGLE_JINJA_COMMENT.open_re,
        close_jv = SINGLE_JINJA_VAR.close_re,
        open_jv = SINGLE_JINJA_VAR.open_re,
        close_hc = SINGLE_HTML_COMMENT.close_re,
        open_hc = SINGLE_HTML_COMMENT.open_re,
    );
    Regex::new(&pattern).unwrap()
});

/// Pattern to remove spaces between adjacent tags.
static DENORMALIZE_TAGS_RE: LazyLock<Regex> = LazyLock::new(|| {
    let pattern = format!(
        "({close_jt}) ({open_jt})|({close_jc}) ({open_jc})|({close_jv}) ({open_jv})|({close_hc}) ({open_hc})",
        close_jt = SINGLE_JINJA_TAG.close_re,
        open_jt = SINGLE_JINJA_TAG.open_re,
        close_jc = SINGLE_JINJA_COMMENT.close_re,
        open_jc = SINGLE_JINJA_COMMENT.open_re,
        close_jv = SINGLE_JINJA_VAR.close_re,
        open_jv = SINGLE_JINJA_VAR.open_re,
        close_hc = SINGLE_HTML_COMMENT.close_re,
        open_hc = SINGLE_HTML_COMMENT.open_re,
    );
    Regex::new(&pattern).unwrap()
});

/// Pattern for detecting multiline closing tags.
static MULTILINE_CLOSING_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    let pattern = format!(
        r"{close_jt}\s*(?P<closing_tag>{open_jt}\s*/)|{close_jc}\s*(?P<closing_comment>{open_jc}\s*/)|{close_jv}\s*(?P<closing_var>{open_jv}\s*/)|{close_hc}\s*(?P<closing_html>{open_hc}\s*/)",
        close_jt = SINGLE_JINJA_TAG.close_re,
        open_jt = SINGLE_JINJA_TAG.open_re,
        close_jc = SINGLE_JINJA_COMMENT.close_re,
        open_jc = SINGLE_JINJA_COMMENT.open_re,
        close_jv = SINGLE_JINJA_VAR.close_re,
        open_jv = SINGLE_JINJA_VAR.open_re,
        close_hc = SINGLE_HTML_COMMENT.close_re,
        open_hc = SINGLE_HTML_COMMENT.open_re,
    );
    Regex::new(&pattern).unwrap()
});

/// Add a space between adjacent tags so they become separate tokens.
pub fn normalize_adjacent_tags(text: &str) -> String {
    ADJACENT_TAGS_RE
        .replace_all(text, |caps: &regex::Captures<'_>| {
            let groups: Vec<Option<regex::Match<'_>>> =
                (1..=caps.len()).map(|i| caps.get(i)).collect();
            for i in (0..groups.len()).step_by(2) {
                if let (Some(a), Some(b)) = (&groups.get(i).copied().flatten(), &groups.get(i + 1).copied().flatten()) {
                    return format!("{} {}", a.as_str(), b.as_str());
                }
            }
            caps.get(0).unwrap().as_str().to_string()
        })
        .into_owned()
}

/// Remove spaces between adjacent tags that were added during word splitting.
pub fn denormalize_adjacent_tags(text: &str) -> String {
    DENORMALIZE_TAGS_RE
        .replace_all(text, |caps: &regex::Captures<'_>| {
            let groups: Vec<Option<regex::Match<'_>>> =
                (1..=caps.len()).map(|i| caps.get(i)).collect();
            for i in (0..groups.len()).step_by(2) {
                if let (Some(a), Some(b)) = (&groups.get(i).copied().flatten(), &groups.get(i + 1).copied().flatten()) {
                    return format!("{}{}", a.as_str(), b.as_str());
                }
            }
            caps.get(0).unwrap().as_str().to_string()
        })
        .into_owned()
}

/// Check if a line is a tag-only line (starts and ends with tag delimiters).
fn is_tag_only_line(line: &str) -> bool {
    // Indented lines are continuations, not standalone tag blocks
    if !line.is_empty() && line.starts_with(char::is_whitespace) {
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

    // Check if there are any tag-only lines
    let has_tag_only_lines = lines.iter().any(|line| is_tag_only_line(line));
    if !has_tag_only_lines {
        return text.to_string();
    }

    let mut result_lines: Vec<&str> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            let prev_line = lines[i - 1];
            let prev_is_empty = prev_line.trim().is_empty();

            // Case 1: Previous line is a tag-only line, current line is block content
            if !prev_is_empty && is_tag_only_line(prev_line) && line_is_block_content(line) {
                result_lines.push("");
            }

            // Case 2: Previous line is block content, current line is a closing tag-only line
            if !prev_is_empty && line_is_block_content(prev_line) && is_tag_only_line(line) {
                result_lines.push("");
            }
        }

        result_lines.push(line);
    }

    result_lines.join("\n")
}

/// Check if a line ends with a Jinja/Markdoc tag or HTML comment.
pub(crate) fn line_ends_with_tag(line: &str) -> bool {
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
pub(crate) fn line_starts_with_tag(line: &str) -> bool {
    let stripped = line.trim_start();
    if stripped.is_empty() {
        return false;
    }
    stripped.starts_with(SINGLE_JINJA_TAG.open_delim)
        || stripped.starts_with(SINGLE_JINJA_COMMENT.open_delim)
        || stripped.starts_with(SINGLE_JINJA_VAR.open_delim)
        || stripped.starts_with(SINGLE_HTML_COMMENT.open_delim)
}

/// Check if a line is an unindented line that starts with a tag.
fn is_unindented_tag_line(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    if line.starts_with(char::is_whitespace) {
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

/// Fix closing tag spacing for block content.
pub fn fix_closing_tag_spacing(text: &str) -> String {
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
            fixed_lines.push(line.to_string());
        }
    }

    fixed_lines.join("\n")
}

/// Ensure closing tags are on their own line when the opening tag spans multiple lines.
pub fn fix_multiline_opening_tag_with_closing(text: &str) -> String {
    if !text.contains('\n') {
        return text.to_string();
    }

    let lines: Vec<&str> = text.split('\n').collect();
    let mut result_lines: Vec<String> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            result_lines.push(line.to_string());
            continue;
        }

        let stripped = line.trim_start();
        let is_tag_start = stripped.starts_with(SINGLE_JINJA_TAG.open_delim)
            || stripped.starts_with(SINGLE_JINJA_COMMENT.open_delim)
            || stripped.starts_with(SINGLE_JINJA_VAR.open_delim)
            || stripped.starts_with(SINGLE_HTML_COMMENT.open_delim);

        if !is_tag_start {
            if let Some(_m) = MULTILINE_CLOSING_PATTERN.find(line) {
                // Find which named group matched
                let caps = MULTILINE_CLOSING_PATTERN.captures(line).unwrap();
                let mut found = false;
                for group_name in &["closing_tag", "closing_comment", "closing_var", "closing_html"]
                {
                    if caps.name(group_name).is_some() {
                        let split_pos = caps.name(group_name).unwrap().start();
                        let before = line[..split_pos].trim_end();
                        let closing = line[split_pos..].trim_start();
                        result_lines.push(before.to_string());
                        result_lines.push(closing.to_string());
                        found = true;
                        break;
                    }
                }
                if found {
                    continue;
                }
            }
        }

        result_lines.push(line.to_string());
    }

    result_lines.join("\n")
}

/// Augments a `LineWrapper` to preserve newlines around Jinja/Markdoc tags
/// and HTML comments.
#[allow(clippy::type_complexity)]
pub(crate) fn add_tag_newline_handling(base_wrapper: Box<dyn Fn(&str, &str, &str) -> String + Send + Sync>) -> LineWrapper {
    Box::new(move |text: &str, initial_indent: &str, subsequent_indent: &str| -> String {
        // If no newlines in input, just wrap and apply post-processing fixes.
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
        let has_tags = lines
            .iter()
            .any(|line| line_ends_with_tag(line) || line_starts_with_tag(line));

        // Group lines into segments
        let mut segments: Vec<String> = Vec::new();
        let mut current_segment_lines: Vec<&str> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let is_first_line = i == 0;
            let prev_ends_with_tag =
                !is_first_line && line_ends_with_tag(lines[i - 1]);
            let curr_starts_with_tag = is_unindented_tag_line(line);
            let curr_is_block = has_tags && line_is_block_content(line);
            let prev_is_block =
                has_tags && !is_first_line && line_is_block_content(lines[i - 1]);

            if (prev_ends_with_tag || curr_starts_with_tag || curr_is_block || prev_is_block) && !current_segment_lines.is_empty() {
                segments.push(current_segment_lines.join("\n"));
                current_segment_lines.clear();
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
            let is_first = i == 0;
            let cur_initial_indent = if is_first {
                initial_indent
            } else {
                subsequent_indent
            };
            let wrapped = base_wrapper(segment, cur_initial_indent, subsequent_indent);
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

            let prev_is_block = prev_segment
                .split('\n')
                .any(line_is_block_content);
            let curr_is_block = curr_segment
                .split('\n')
                .any(line_is_block_content);

            let prev_last_line = prev_segment.split('\n').next_back().unwrap_or("");
            let curr_first_line = curr_segment.split('\n').next().unwrap_or("");

            let prev_is_tag = line_ends_with_tag(prev_last_line);
            let curr_is_tag = is_unindented_tag_line(curr_first_line);

            if (prev_is_tag && curr_is_block) || (prev_is_block && curr_is_tag) {
                result_parts.push(String::new());
                result_parts.push(wrapped.clone());
            } else {
                result_parts.push(wrapped.clone());
            }
        }

        let result = result_parts.join("\n");

        // Post-process
        let result = fix_closing_tag_spacing(&result);
        fix_multiline_opening_tag_with_closing(&result)
    })
}
