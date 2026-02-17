//! Comrak-based Markdown parsing and rendering.
//!
//! This module handles the core Markdown normalization by parsing with comrak
//! and rendering back to normalized `CommonMark` format.

use comrak::nodes::AstNode;
use comrak::{Arena, Options};

/// Create comrak options configured for GFM parsing with Flowmark conventions.
pub fn flowmark_comrak_options<'c>() -> Options<'c> {
    let mut options = Options::default();

    // Extension options - enable GFM features
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.front_matter_delimiter = None;
    options.extension.header_ids = None;
    options.extension.description_lists = false;
    options.extension.multiline_block_quotes = false;
    options.extension.math_dollars = true;
    options.extension.math_code = false;
    options.extension.wikilinks_title_after_pipe = false;
    options.extension.wikilinks_title_before_pipe = false;
    options.extension.underline = false;
    options.extension.subscript = false;
    options.extension.superscript = false;
    options.extension.spoiler = false;
    options.extension.greentext = false;
    options.extension.image_url_rewriter = None;
    options.extension.link_url_rewriter = None;
    options.extension.alerts = true;

    // Parse options
    options.parse.smart = false;

    // Render options
    options.render.hardbreaks = true;
    options.render.width = 0;
    options.render.unsafe_ = true;
    options.render.escape = false;
    options.render.list_style = comrak::ListStyleType::Dash;
    options.render.sourcepos = false;
    options.render.escaped_char_spans = false;
    options.render.ignore_setext = false;
    options.render.ignore_empty_links = false;
    options.render.gfm_quirks = false;
    options.render.prefer_fenced = true;
    options.render.figure_with_caption = false;
    options.render.tasklist_classes = false;
    options.render.ol_width = 0;
    options.render.experimental_inline_sourcepos = false;

    options
}

/// Parse markdown text and render it with comrak, returning the normalized output.
pub fn parse_and_render(text: &str) -> String {
    let arena = Arena::new();
    let options = flowmark_comrak_options();
    let root = comrak::parse_document(&arena, text, &options);
    let mut output = vec![];
    comrak::format_commonmark(root, &options, &mut output).unwrap();
    String::from_utf8(output).unwrap()
}

/// Parse markdown text and provide access to the AST via a closure.
pub fn with_markdown_ast<F, R>(text: &str, f: F) -> R
where
    F: for<'a, 'c> FnOnce(&'a AstNode<'a>, &Options<'c>) -> R,
{
    let arena = Arena::new();
    let options = flowmark_comrak_options();
    let root = comrak::parse_document(&arena, text, &options);
    f(root, &options)
}
