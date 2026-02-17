//! Multi-paragraph text filling.
//!
//! Ported from Python: flowmark/linewrapping/text_filling.py

use regex::Regex;
use std::sync::LazyLock;

use crate::config::DEFAULT_WRAP_WIDTH;
use crate::wrapping::text_wrapping::{html_md_word_split, wrap_paragraph};

/// Default indent string.
const DEFAULT_INDENT: &str = "    ";

static PARA_SPLIT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n{2,}").unwrap());

/// Split text into paragraphs separated by blank lines.
pub fn split_paragraphs(text: &str) -> Vec<String> {
    PARA_SPLIT_RE
        .split(text)
        .map(|p| p.trim().to_string())
        .collect()
}

/// Text wrapping styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wrap {
    None,
    Wrap,
    WrapFull,
    WrapIndent,
    IndentOnly,
    HangingIndent,
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

/// Fill any number of paragraphs of plain text.
pub fn fill_text(
    text: &str,
    text_wrap: Wrap,
    width: usize,
    extra_indent: &str,
    empty_indent: &str,
    initial_column: usize,
    word_splitter: Option<&dyn Fn(&str) -> Vec<String>>,
) -> String {
    let splitter: &dyn Fn(&str) -> Vec<String> = word_splitter.unwrap_or(&html_md_word_split);

    if !text_wrap.should_wrap() {
        let indent = if text_wrap == Wrap::IndentOnly {
            format!("{extra_indent}{DEFAULT_INDENT}")
        } else {
            extra_indent.to_string()
        };
        let lines: Vec<&str> = text.lines().collect();
        if !lines.is_empty() {
            return lines
                .iter()
                .map(|line| format!("{indent}{line}"))
                .collect::<Vec<_>>()
                .join("\n");
        }
        return empty_indent.trim().to_string();
    }

    let empty_indent_trimmed = empty_indent.trim();
    let mut initial_indent = format!("{extra_indent}{}", text_wrap.initial_indent());
    let subsequent_indent = format!("{extra_indent}{}", text_wrap.subsequent_indent());

    let width = width - subsequent_indent.chars().count();
    let replace_whitespace = text_wrap.replace_whitespace();

    let paragraphs = split_paragraphs(text);
    let mut wrapped_paragraphs: Vec<String> = Vec::new();

    for (i, paragraph) in paragraphs.iter().enumerate() {
        if text_wrap.initial_indent_first_para_only() && i > 0 {
            initial_indent = subsequent_indent.clone();
        }

        wrapped_paragraphs.push(wrap_paragraph(
            paragraph,
            width,
            &initial_indent,
            &subsequent_indent,
            initial_column,
            replace_whitespace,
            true,
            Some(splitter),
            false,
        ));
    }

    let para_sep = format!("\n{empty_indent_trimmed}\n");
    wrapped_paragraphs.join(&para_sep)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_paragraphs() {
        let text = "Para one.\n\nPara two.\n\nPara three.";
        let paras = split_paragraphs(text);
        assert_eq!(paras.len(), 3);
        assert_eq!(paras[0], "Para one.");
        assert_eq!(paras[1], "Para two.");
        assert_eq!(paras[2], "Para three.");
    }
}
