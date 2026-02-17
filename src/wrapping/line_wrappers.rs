//! `LineWrapper` factory functions.
//!
//! Ported from Python: `flowmark/linewrapping/line_wrappers.py`

use regex::Regex;
use std::sync::{Arc, LazyLock};

use crate::wrapping::sentence::split_sentences_regex;
use crate::wrapping::tag_handling::{add_tag_newline_handling, denormalize_adjacent_tags};
use crate::wrapping::text_wrapping::{wrap_paragraph, wrap_paragraph_lines};
use crate::wrapping::LineWrapper;

/// Pattern to match Markdown hard line breaks.
static LINE_BREAK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\\\n|  \n").unwrap());

/// Split text by explicit Markdown line breaks.
fn split_markdown_hard_breaks(text: &str) -> Vec<String> {
    LINE_BREAK_RE.split(text).map(String::from).collect()
}

/// Augments a `LineWrapper` to first split the text by Markdown hard breaks.
#[allow(clippy::type_complexity)]
fn add_markdown_hard_break_handling(base_wrapper: LineWrapper) -> LineWrapper {
    let base: Arc<dyn Fn(&str, &str, &str) -> String + Send + Sync> = Arc::from(base_wrapper);

    Box::new(move |text: &str, initial_indent: &str, subsequent_indent: &str| -> String {
        let segments = split_markdown_hard_breaks(text);

        if segments.is_empty() {
            return String::new();
        }
        if segments.len() == 1 {
            return base(text, initial_indent, subsequent_indent);
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
            let wrapped = base(segment, cur_initial_indent, subsequent_indent);
            if is_last {
                wrapped_segments.push(wrapped);
            } else {
                wrapped_segments.push(format!("{wrapped}\\"));
            }
        }

        wrapped_segments.join("\n")
    })
}

/// Wrap lines of text to a given width.
pub fn line_wrap_to_width(width: usize, is_markdown: bool) -> LineWrapper {
    let line_wrapper: LineWrapper =
        Box::new(move |text: &str, initial_indent: &str, subsequent_indent: &str| -> String {
            wrap_paragraph(
                text,
                width,
                initial_indent,
                subsequent_indent,
                0,
                true,
                true,
                None,
                is_markdown,
            )
        });

    if is_markdown {
        let enhanced = add_tag_newline_handling(line_wrapper);
        add_markdown_hard_break_handling(enhanced)
    } else {
        line_wrapper
    }
}

/// Wrap lines of text to a given width but also keep sentences on their own lines.
pub fn line_wrap_by_sentence(
    width: usize,
    min_line_len: usize,
    is_markdown: bool,
) -> LineWrapper {
    let line_wrapper: LineWrapper =
        Box::new(move |text: &str, initial_indent: &str, subsequent_indent: &str| -> String {
            let text = text.replace('\n', " ");

            // Handle width == 0 as "no wrapping"
            if width == 0 {
                return format!("{initial_indent}{}", text.trim());
            }

            let mut lines: Vec<String> = Vec::new();
            let mut first_line = true;
            let initial_indent_len = initial_indent.chars().count();
            let subsequent_indent_len = subsequent_indent.chars().count();

            let sentences = split_sentences_regex(&text, 0);

            for sentence in &sentences {
                let current_column = if first_line {
                    initial_indent_len
                } else {
                    subsequent_indent_len
                };

                let current_column = if !lines.is_empty()
                    && lines.last().unwrap().chars().count() < min_line_len
                {
                    current_column + lines.last().unwrap().chars().count()
                } else {
                    current_column
                };

                let mut wrapped = wrap_paragraph_lines(
                    sentence,
                    width,
                    current_column,
                    subsequent_indent_len,
                    true,
                    true,
                    None,
                    is_markdown,
                );

                // If last line is shorter than min_line_len, combine with next line.
                if !lines.is_empty()
                    && !wrapped.is_empty()
                    && lines.last().unwrap().chars().count() < min_line_len
                    && lines.last().unwrap().chars().count() + 1 + wrapped[0].chars().count()
                        <= width
                {
                    let last = lines.last_mut().unwrap();
                    *last = format!("{last} {}", wrapped[0]);
                    wrapped.remove(0);
                }

                lines.extend(wrapped);
                first_line = false;
            }

            // Insert indents
            if !initial_indent.is_empty() && !lines.is_empty() {
                lines[0] = format!("{initial_indent}{}", lines[0]);
            }
            if !subsequent_indent.is_empty() && lines.len() > 1 {
                for line in lines.iter_mut().skip(1) {
                    *line = format!("{subsequent_indent}{line}");
                }
            }

            let result = lines.join("\n");
            denormalize_adjacent_tags(&result)
        });

    if is_markdown {
        let enhanced = add_tag_newline_handling(line_wrapper);
        add_markdown_hard_break_handling(enhanced)
    } else {
        line_wrapper
    }
}

