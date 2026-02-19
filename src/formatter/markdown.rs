//! Comrak configuration for Markdown parsing.
//!
//! Configures comrak options for GFM parsing with Flowmark conventions.
//! See `filling.rs` module docs for the full list of comrak workarounds
//! (COMRAK-WORKAROUND1–12) needed to achieve Python/marko parity.

use comrak::Options;

/// Create comrak options configured for GFM parsing with Flowmark conventions.
pub(crate) fn flowmark_comrak_options<'c>() -> Options<'c> {
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
    options.parse.ignore_setext = false;
    options.parse.escaped_char_spans = false;

    // Render options
    options.render.hardbreaks = true;
    options.render.width = 0;
    options.render.r#unsafe = true;
    options.render.escape = false;
    options.render.list_style = comrak::options::ListStyleType::Dash;
    options.render.sourcepos = false;
    options.render.escaped_char_spans = false;
    options.render.ignore_empty_links = false;
    options.render.gfm_quirks = false;
    options.render.prefer_fenced = true;
    options.render.figure_with_caption = false;
    options.render.tasklist_classes = false;
    options.render.ol_width = 0;

    options
}
