//! Auto-formatting of Markdown text.
//!
//! Ported from Python: flowmark/linewrapping/markdown_filling.py
//!
//! This is the main entry point for Markdown normalization.

use comrak::{Arena, parse_document};

use crate::formats::flowmark_markdown::{ListSpacing, flowmark_options, render_markdown};
use crate::formats::frontmatter::split_frontmatter;
use crate::linewrapping::line_wrappers::{
    line_wrap_by_sentence, line_wrap_to_width, DEFAULT_MIN_LINE_LEN,
};
use crate::linewrapping::protocols::LineWrapper;
use crate::linewrapping::tag_handling::preprocess_tag_block_spacing;
use crate::linewrapping::text_filling::DEFAULT_WRAP_WIDTH;
use crate::linewrapping::text_wrapping::default_len;
use crate::transforms::doc_cleanups::doc_cleanups;
use crate::transforms::doc_transforms::{rewrite_text_across_inlines, rewrite_text_content};
use crate::typography::ellipses::ellipses;
use crate::typography::smartquotes::smart_quotes;

/// Normalize and wrap Markdown text.
///
/// This is the main public API for formatting Markdown documents.
///
/// - `semantic`: Break lines at sentence boundaries for better diffs
/// - `cleanups`: Apply safe document cleanups (e.g., unbold headings)
/// - `smartquotes`: Convert straight quotes to typographic quotes
/// - `ellipses`: Convert `...` to proper ellipsis character
/// - `list_spacing`: Control list item spacing
pub fn fill_markdown(
    markdown_text: &str,
    semantic: bool,
    width: usize,
    cleanups: bool,
    apply_smartquotes: bool,
    apply_ellipses: bool,
    line_wrapper: Option<&LineWrapper>,
    list_spacing: ListSpacing,
) -> String {
    let default_wrapper: LineWrapper;
    let wrapper = if let Some(lw) = line_wrapper {
        lw
    } else {
        default_wrapper = if semantic {
            line_wrap_by_sentence(width, DEFAULT_MIN_LINE_LEN, 0, default_len, true)
        } else {
            line_wrap_to_width(width, default_len, true)
        };
        &default_wrapper
    };

    // Extract frontmatter before any processing
    let (frontmatter, content) = split_frontmatter(markdown_text);

    let mut text = if !frontmatter.is_empty() {
        content
    } else {
        markdown_text.to_string()
    };

    // Dedent and strip
    text = dedent_and_strip(&text);
    text = text.trim().to_string() + "\n";

    // Preprocess tag block spacing
    text = preprocess_tag_block_spacing(&text);

    // Parse with comrak
    let arena = Arena::new();
    let options = flowmark_options();
    let root = parse_document(&arena, &text, &options);

    // Apply AST transforms
    if cleanups {
        doc_cleanups(root);
    }
    if apply_smartquotes {
        rewrite_text_across_inlines(root, &smart_quotes);
    }
    if apply_ellipses {
        rewrite_text_content(root, &ellipses);
    }

    // Render back to Markdown
    let result = render_markdown(root, wrapper, list_spacing);

    // Reattach frontmatter if present
    if !frontmatter.is_empty() {
        format!("{frontmatter}{result}")
    } else {
        result
    }
}

/// Simple dedent: remove common leading whitespace from all lines.
fn dedent_and_strip(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    // Find minimum indentation (ignoring empty lines)
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    if min_indent == 0 {
        return text.to_string();
    }

    lines
        .iter()
        .map(|line| {
            if line.len() >= min_indent {
                &line[min_indent..]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Convenience function with common defaults.
pub fn fill_markdown_default(markdown_text: &str, semantic: bool) -> String {
    fill_markdown(
        markdown_text,
        semantic,
        DEFAULT_WRAP_WIDTH,
        false,
        false,
        false,
        None,
        ListSpacing::default(),
    )
}
