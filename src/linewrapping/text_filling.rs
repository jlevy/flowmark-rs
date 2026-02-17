//! Text filling: wrap and fill paragraphs of plain text.
//!
//! Ported from Python: flowmark/linewrapping/text_filling.py

use once_cell::sync::Lazy;
use regex::Regex;

use crate::linewrapping::text_wrapping::{default_len, get_html_md_word_splitter, wrap_paragraph};

/// Default wrap width. Same as Black: a compromise between 80-char console width
/// and being too wide to read comfortably.
pub const DEFAULT_WRAP_WIDTH: usize = 88;

/// Default indent.
pub const DEFAULT_INDENT: &str = "    ";

/// Split text into paragraphs separated by two or more newlines.
pub fn split_paragraphs(text: &str) -> Vec<String> {
    static PARA_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n{2,}").unwrap());
    PARA_RE.split(text).map(|p| p.trim().to_string()).collect()
}

/// Text wrapping style options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wrap {
    /// No wrapping.
    None,
    /// Basic wrapping but preserves whitespace within paragraphs.
    Wrap,
    /// Wraps and also normalizes whitespace.
    WrapFull,
    /// Wrap and also indent.
    WrapIndent,
    /// Just indent.
    IndentOnly,
    /// Wrap with hanging indent (indented except for the first line).
    HangingIndent,
    /// 2-space hanging indent for markdown list items.
    MarkdownItem,
}

impl Wrap {
    pub fn initial_indent(self) -> &'static str {
        match self {
            Self::IndentOnly | Self::WrapIndent => DEFAULT_INDENT,
            _ => "",
        }
    }

    pub fn subsequent_indent(self) -> &'static str {
        match self {
            Self::MarkdownItem => "  ",
            Self::IndentOnly | Self::WrapIndent | Self::HangingIndent => DEFAULT_INDENT,
            _ => "",
        }
    }

    pub fn should_wrap(self) -> bool {
        matches!(
            self,
            Self::Wrap | Self::WrapFull | Self::WrapIndent | Self::HangingIndent | Self::MarkdownItem
        )
    }

    pub fn initial_indent_first_para_only(self) -> bool {
        matches!(self, Self::HangingIndent | Self::MarkdownItem)
    }

    pub fn replace_whitespace(self) -> bool {
        matches!(self, Self::WrapFull | Self::WrapIndent | Self::HangingIndent)
    }
}

/// Wrap and fill any number of paragraphs of plain text.
pub fn fill_text(
    text: &str,
    text_wrap: Wrap,
    width: usize,
    extra_indent: &str,
    empty_indent: &str,
    initial_column: usize,
    word_splitter: Option<fn(&str) -> Vec<String>>,
    len_fn: fn(&str) -> usize,
) -> String {
    let word_splitter = word_splitter.unwrap_or(get_html_md_word_splitter());

    if !text_wrap.should_wrap() {
        let indent = if text_wrap == Wrap::IndentOnly {
            format!("{extra_indent}{DEFAULT_INDENT}")
        } else {
            extra_indent.to_string()
        };
        let lines: Vec<&str> = text.lines().collect();
        if !lines.is_empty() {
            lines
                .iter()
                .map(|line| format!("{indent}{line}"))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            empty_indent.to_string()
        }
    } else {
        let empty_indent = empty_indent.trim();
        let initial_indent = format!("{extra_indent}{}", text_wrap.initial_indent());
        let subsequent_indent = format!("{extra_indent}{}", text_wrap.subsequent_indent());

        let width = width.saturating_sub(len_fn(&subsequent_indent));
        let replace_whitespace = text_wrap.replace_whitespace();

        let paragraphs = split_paragraphs(text);
        let mut wrapped_paragraphs: Vec<String> = Vec::new();

        for (i, paragraph) in paragraphs.iter().enumerate() {
            let cur_initial = if text_wrap.initial_indent_first_para_only() && i > 0 {
                &subsequent_indent
            } else {
                &initial_indent
            };

            wrapped_paragraphs.push(wrap_paragraph(
                paragraph,
                width,
                cur_initial,
                &subsequent_indent,
                initial_column,
                replace_whitespace,
                true, // drop_whitespace
                Some(word_splitter),
                len_fn,
                false, // is_markdown
            ));
        }

        let para_sep = format!("\n{empty_indent}\n");
        wrapped_paragraphs.join(&para_sep)
    }
}

/// Convenience wrapper with defaults.
pub fn fill_text_default(text: &str) -> String {
    fill_text(
        text,
        Wrap::Wrap,
        DEFAULT_WRAP_WIDTH,
        "",
        "",
        0,
        None,
        default_len,
    )
}
