//! High-level line wrapper constructors.
//!
//! Ported from Python: flowmark/linewrapping/line_wrappers.py

use once_cell::sync::Lazy;
use regex::Regex;

use crate::linewrapping::protocols::LineWrapper;
use crate::linewrapping::sentence_split_regex::split_sentences_regex;
use crate::linewrapping::tag_handling::{add_tag_newline_handling, denormalize_adjacent_tags};
use crate::linewrapping::text_filling::DEFAULT_WRAP_WIDTH;
use crate::linewrapping::text_wrapping::{default_len, wrap_paragraph, wrap_paragraph_lines};

/// Default minimum line length for sentence breaking.
pub const DEFAULT_MIN_LINE_LEN: usize = 20;

/// Split text by explicit Markdown line breaks (backslash-newline or two-space-newline).
fn split_markdown_hard_breaks(text: &str) -> Vec<String> {
    static LINE_BREAK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\\n|  \n").unwrap());
    LINE_BREAK_RE
        .split(text)
        .map(String::from)
        .collect()
}

/// Augments a `LineWrapper` to handle Markdown hard breaks.
fn add_markdown_hard_break_handling(base_wrapper: LineWrapper) -> LineWrapper {
    Box::new(move |text: &str, initial_indent: &str, subsequent_indent: &str| -> String {
        let segments = split_markdown_hard_breaks(text);

        if segments.is_empty() {
            return String::new();
        }
        if segments.len() == 1 {
            return base_wrapper(text, initial_indent, subsequent_indent);
        }

        let mut wrapped_segments: Vec<String> = Vec::new();

        for (i, segment) in segments.iter().enumerate() {
            let is_first = i == 0;
            let is_last = i == segments.len() - 1;

            let cur_initial_indent = if is_first {
                initial_indent
            } else {
                subsequent_indent
            };
            let wrapped_segment = base_wrapper(segment, cur_initial_indent, subsequent_indent);
            if is_last {
                wrapped_segments.push(wrapped_segment);
            } else {
                wrapped_segments.push(format!("{wrapped_segment}\\"));
            }
        }

        wrapped_segments.join("\n")
    })
}

/// Create a line wrapper that wraps lines to a given width.
pub fn line_wrap_to_width(
    width: usize,
    len_fn: fn(&str) -> usize,
    is_markdown: bool,
) -> LineWrapper {
    let base: LineWrapper = Box::new(move |text: &str, initial_indent: &str, subsequent_indent: &str| -> String {
        wrap_paragraph(
            text,
            width,
            initial_indent,
            subsequent_indent,
            0,
            true,
            true,
            None,
            len_fn,
            is_markdown,
        )
    });

    if is_markdown {
        let enhanced = add_tag_newline_handling(base);
        add_markdown_hard_break_handling(enhanced)
    } else {
        base
    }
}

/// Create a line wrapper that wraps by sentence boundaries.
pub fn line_wrap_by_sentence(
    width: usize,
    min_line_len: usize,
    min_sentence_len: usize,
    len_fn: fn(&str) -> usize,
    is_markdown: bool,
) -> LineWrapper {
    let base: LineWrapper = Box::new(move |text: &str, initial_indent: &str, subsequent_indent: &str| -> String {
        let text = text.replace('\n', " ");

        // Handle width == 0 as "no wrapping"
        if width == 0 {
            return format!("{initial_indent}{}", text.trim());
        }

        let mut lines: Vec<String> = Vec::new();
        let mut first_line = true;
        let initial_indent_len = len_fn(initial_indent);
        let subsequent_indent_len = len_fn(subsequent_indent);

        let sentences = split_sentences_regex(&text, min_sentence_len);

        for sentence in &sentences {
            let current_column = if first_line {
                initial_indent_len
            } else {
                subsequent_indent_len
            };
            let extra = if !lines.is_empty() && len_fn(&lines[lines.len() - 1]) < min_line_len {
                len_fn(&lines[lines.len() - 1])
            } else {
                0
            };
            let effective_column = current_column + extra;

            let wrapped = wrap_paragraph_lines(
                sentence,
                width,
                effective_column,
                subsequent_indent_len,
                true,
                true,
                None,
                len_fn,
                is_markdown,
            );

            // If last line is shorter than min_line_len, combine with next line
            if !lines.is_empty()
                && !wrapped.is_empty()
                && len_fn(&lines[lines.len() - 1]) < min_line_len
                && len_fn(&lines[lines.len() - 1]) + 1 + len_fn(&wrapped[0]) <= width
            {
                let last_idx = lines.len() - 1;
                lines[last_idx] = format!("{} {}", lines[last_idx], wrapped[0]);
                lines.extend(wrapped.into_iter().skip(1));
            } else {
                lines.extend(wrapped);
            }

            first_line = false;
        }

        // Insert indents and assemble
        if !initial_indent.is_empty() && !lines.is_empty() {
            lines[0] = format!("{initial_indent}{}", lines[0]);
        }
        if !subsequent_indent.is_empty() && lines.len() > 1 {
            for line in &mut lines[1..] {
                *line = format!("{subsequent_indent}{line}");
            }
        }

        let result = lines.join("\n");
        denormalize_adjacent_tags(&result)
    });

    if is_markdown {
        let enhanced = add_tag_newline_handling(base);
        add_markdown_hard_break_handling(enhanced)
    } else {
        base
    }
}

/// Convenience constructors with common defaults.
pub fn line_wrap_to_width_default(is_markdown: bool) -> LineWrapper {
    line_wrap_to_width(DEFAULT_WRAP_WIDTH, default_len, is_markdown)
}

pub fn line_wrap_by_sentence_default(is_markdown: bool) -> LineWrapper {
    line_wrap_by_sentence(DEFAULT_WRAP_WIDTH, DEFAULT_MIN_LINE_LEN, 0, default_len, is_markdown)
}
