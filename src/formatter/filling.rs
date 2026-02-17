//! Markdown filling and normalization pipeline.
//!
//! This is the core formatting pipeline that normalizes and wraps Markdown text.
//! It handles the complex interaction between comrak's AST rendering and the
//! text-level normalization needed to match Python/Marko behavior.
//!
//! Ported from Python: flowmark/linewrapping/markdown_filling.py and
//! parts of flowmark/formats/flowmark_markdown.py

use regex::Regex;
use std::sync::LazyLock;

use comrak::nodes::{AstNode, ListType, NodeValue, TableAlignment};
use comrak::{Arena, Options};

use crate::config::{ListSpacing, DEFAULT_WRAP_WIDTH};
use crate::formatter::markdown::flowmark_comrak_options;
use crate::parser::frontmatter::split_frontmatter;
use crate::transform::cleanups::doc_cleanups;
use crate::typography::ellipses::ellipses as apply_ellipses;
use crate::typography::quotes::smart_quotes;
use crate::wrapping::line_wrappers::{line_wrap_by_sentence, line_wrap_to_width};
use crate::wrapping::sentence::split_sentences_regex;
use crate::wrapping::tag_handling::preprocess_tag_block_spacing;
use crate::wrapping::text_wrapping::wrap_paragraph_lines;
use crate::wrapping::LineWrapper;
use crate::config::DEFAULT_MIN_LINE_LEN;

// ===== Regex patterns for normalization =====

/// Pattern for blank lines with trailing whitespace.
static BLANK_LINE_WS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[ \t]+$").unwrap());

/// Pattern for multiple consecutive blank lines (3+ newlines).
static MULTI_BLANK_LINES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n{3,}").unwrap());

/// Pattern for code fence with space before language.
static CODE_FENCE_SPACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^([ \t]*```)\s+(\w)").unwrap());

/// Pattern for numbered list items with two spaces after period.
static NUMBERED_ITEM_TWO_SPACES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)\.  ").unwrap());

/// Pattern for blockquote markers with trailing space only.
static QUOTE_TRAILING_SPACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^> $").unwrap());

/// Pattern for HTML comments that should be on their own line.
static HTML_COMMENT_INLINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[ \t]*(<!--.*?-->)[ \t]*").unwrap());

/// Normalize blank lines by removing trailing whitespace.
fn normalize_blank_lines(text: &str) -> String {
    BLANK_LINE_WS.replace_all(text, "").into_owned()
}

/// Collapse multiple blank lines to single blank lines.
fn collapse_blank_lines(text: &str) -> String {
    MULTI_BLANK_LINES.replace_all(text, "\n\n").into_owned()
}

/// Remove space between code fence and language identifier.
fn normalize_code_fences(text: &str) -> String {
    CODE_FENCE_SPACE
        .replace_all(text, "$1$2")
        .into_owned()
}

/// Fix numbered list items: convert two spaces to one space after period.
fn normalize_numbered_lists(text: &str) -> String {
    let mut result = String::new();
    for line in text.lines() {
        if let Some(caps) = NUMBERED_ITEM_TWO_SPACES.captures(line) {
            let num = &caps[1];
            let fixed = line.replacen(&format!("{num}.  "), &format!("{num}. "), 1);
            result.push_str(&fixed);
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }
    // Remove trailing newline if original didn't have one
    if !text.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }
    result
}

/// Fix quote markers with trailing space only.
fn normalize_quote_markers(text: &str) -> String {
    QUOTE_TRAILING_SPACE.replace_all(text, ">").into_owned()
}

/// Normalize HTML comments by ensuring they are on their own lines.
fn normalize_html_comments(text: &str) -> String {
    let result = HTML_COMMENT_INLINE
        .replace_all(text, "\n$1\n\n")
        .into_owned();
    let result = result.trim_start().to_string();
    collapse_blank_lines(&result)
}

/// Apply all text-level normalizations to comrak output.
fn normalize_comrak_output(text: &str) -> String {
    let text = normalize_blank_lines(text);
    let text = normalize_code_fences(&text);
    let text = normalize_numbered_lists(&text);
    let text = normalize_quote_markers(&text);
    let text = collapse_blank_lines(&text);
    text
}

/// Format a single paragraph by wrapping it with the line wrapper.
fn format_paragraph(
    text: &str,
    line_wrapper: &LineWrapper,
    prefix: &str,
    subsequent_prefix: &str,
) -> String {
    let wrapped = line_wrapper(text, prefix, subsequent_prefix);
    format!("{wrapped}\n")
}

/// Recursively render a comrak AST node to normalized Markdown.
///
/// This is the core rendering function that walks the AST and produces
/// normalized Markdown output.
fn render_node<'a>(
    node: &'a AstNode<'a>,
    line_wrapper: &LineWrapper,
    list_spacing: ListSpacing,
    prefix: &str,
    subsequent_prefix: &str,
    suppress_item_break: &mut bool,
    skip_next_blank: &mut bool,
    in_heading: &mut bool,
    current_list_tight: &mut bool,
    options: &Options,
) -> String {
    let mut output = String::new();

    match &node.data.borrow().value {
        NodeValue::Document => {
            for child in node.children() {
                output.push_str(&render_node(
                    child,
                    line_wrapper,
                    list_spacing,
                    prefix,
                    subsequent_prefix,
                    suppress_item_break,
                    skip_next_blank,
                    in_heading,
                    current_list_tight,
                    options,
                ));
            }
        }

        NodeValue::Paragraph => {
            *skip_next_blank = false;
            *suppress_item_break = false;

            // Collect all inline content
            let inline_text = render_inline_children(node, options, *in_heading);

            // Handle GFM tasklist checkbox
            let inline_text = if let Some(tasklist) = get_tasklist_marker(node) {
                format!("{tasklist}{inline_text}")
            } else {
                inline_text
            };

            // Wrap the text
            let wrapped = line_wrapper(&inline_text, prefix, subsequent_prefix);
            output.push_str(&wrapped);
            output.push('\n');
        }

        NodeValue::Heading(heading) => {
            *in_heading = true;
            let level = heading.level;
            let hashes = "#".repeat(level as usize);

            let inline_text = render_inline_children(node, options, true);
            *in_heading = false;

            output.push_str(&format!("{prefix}{hashes} {inline_text}\n\n"));
            *skip_next_blank = true;
            *suppress_item_break = true;
        }

        NodeValue::List(list) => {
            *skip_next_blank = false;

            // Determine effective tightness
            let is_tight = match list_spacing {
                ListSpacing::Preserve => list.tight,
                ListSpacing::Tight => can_be_tight(node),
                ListSpacing::Loose => false,
            };

            let old_tight = *current_list_tight;
            *current_list_tight = is_tight;

            let is_ordered = matches!(list.list_type, ListType::Ordered);
            let start = list.start;
            let bullet = list.bullet_char;

            for (i, child) in node.children().enumerate() {
                let (item_prefix, item_subsequent) = if is_ordered {
                    let num = start + i;
                    let p = format!("{num}. ");
                    let s = " ".repeat(num.to_string().len() + 2);
                    (format!("{prefix}{p}"), format!("{subsequent_prefix}{s}"))
                } else {
                    let marker = bullet as char;
                    (
                        format!("{prefix}{marker} "),
                        format!("{subsequent_prefix}  "),
                    )
                };

                let item_output = render_node(
                    child,
                    line_wrapper,
                    list_spacing,
                    &item_prefix,
                    &item_subsequent,
                    suppress_item_break,
                    skip_next_blank,
                    in_heading,
                    current_list_tight,
                    options,
                );
                output.push_str(&item_output);
            }

            *current_list_tight = old_tight;
        }

        NodeValue::Item(_) => {
            let mut item_output = String::new();

            // For loose lists, add blank line between items
            if !*current_list_tight {
                if *suppress_item_break {
                    *suppress_item_break = false;
                } else {
                    let sep = subsequent_prefix.trim_end();
                    item_output.push_str(sep);
                    item_output.push('\n');
                }
            }

            // Render the item's children using the prefix passed to us
            let mut first_child = true;
            for child in node.children() {
                let (p, sp) = if first_child {
                    (prefix.to_string(), subsequent_prefix.to_string())
                } else {
                    (subsequent_prefix.to_string(), subsequent_prefix.to_string())
                };
                item_output.push_str(&render_node(
                    child,
                    line_wrapper,
                    list_spacing,
                    &p,
                    &sp,
                    suppress_item_break,
                    skip_next_blank,
                    in_heading,
                    current_list_tight,
                    options,
                ));
                first_child = false;
            }

            output.push_str(&item_output);
        }

        NodeValue::BlockQuote => {
            *skip_next_blank = false;
            let q_prefix = format!("{prefix}> ");
            let q_subsequent = format!("{subsequent_prefix}> ");

            let mut inner = String::new();
            for child in node.children() {
                inner.push_str(&render_node(
                    child,
                    line_wrapper,
                    list_spacing,
                    &q_prefix,
                    &q_subsequent,
                    suppress_item_break,
                    skip_next_blank,
                    in_heading,
                    current_list_tight,
                    options,
                ));
            }

            output.push_str(&inner.trim_end_matches('\n'));
            output.push('\n');
            *suppress_item_break = false;
        }

        NodeValue::CodeBlock(code_block) => {
            *skip_next_blank = false;
            let info = &code_block.info;
            let literal = &code_block.literal;
            let code_content = literal.trim_end_matches('\n');

            let fence_char = if code_block.fenced {
                if code_block.fence_char == b'~' { '~' } else { '`' }
            } else {
                '`'
            };

            // Calculate minimum fence length needed
            let fence_len = min_fence_length(code_content, fence_char)
                .max(if code_block.fenced { code_block.fence_length as usize } else { 3 });
            let fence: String = std::iter::repeat(fence_char).take(fence_len).collect();

            let lang_text = if info.is_empty() {
                String::new()
            } else {
                info.to_string()
            };

            output.push_str(&format!("{prefix}{fence}{lang_text}\n"));
            let empty_prefix = subsequent_prefix.trim_end();
            for line in code_content.lines() {
                if line.is_empty() {
                    output.push_str(empty_prefix);
                    output.push('\n');
                } else {
                    output.push_str(&format!("{subsequent_prefix}{line}\n"));
                }
            }
            // Handle case where code content is empty
            if code_content.is_empty() {
                // No lines to output
            }
            output.push_str(&format!("{subsequent_prefix}{fence}\n"));
            *suppress_item_break = false;
        }

        NodeValue::ThematicBreak => {
            output.push_str(&format!("{prefix}* * *\n"));
        }

        NodeValue::HtmlBlock(html) => {
            output.push_str(&format!("{prefix}{}", html.literal));
        }

        NodeValue::Table(_) => {
            let children: Vec<_> = node.children().collect();
            if children.is_empty() {
                return output;
            }

            // Render header row
            let header = &children[0];
            output.push_str(&render_table_row(header, options));

            // Render delimiter row
            let alignments = get_table_alignments(node);
            let delimiters: Vec<String> = alignments
                .iter()
                .map(|a| match a {
                    TableAlignment::None => "---".to_string(),
                    TableAlignment::Left => ":---".to_string(),
                    TableAlignment::Center => ":---:".to_string(),
                    TableAlignment::Right => "---:".to_string(),
                })
                .collect();
            output.push_str(&format!("| {} |\n", delimiters.join(" | ")));

            // Render body rows
            for child in children.iter().skip(1) {
                output.push_str(&render_table_row(child, options));
            }
        }

        NodeValue::TableRow(_) | NodeValue::TableCell => {
            // Handled by render_table_row
        }

        NodeValue::FootnoteDefinition(footnote) => {
            let label = &footnote.name;
            let label_prefix = format!("[^{label}]: ");
            let fn_prefix = format!("{prefix}{label_prefix}");
            let fn_subsequent = format!("{subsequent_prefix}    ");

            let mut inner = String::new();
            let mut first_child = true;
            for child in node.children() {
                let (p, sp) = if first_child {
                    (fn_prefix.clone(), fn_subsequent.clone())
                } else {
                    (fn_subsequent.clone(), fn_subsequent.clone())
                };
                inner.push_str(&render_node(
                    child,
                    line_wrapper,
                    list_spacing,
                    &p,
                    &sp,
                    suppress_item_break,
                    skip_next_blank,
                    in_heading,
                    current_list_tight,
                    options,
                ));
                first_child = false;
            }

            output.push_str(&inner.trim_end_matches('\n'));
            output.push_str("\n\n");
            *suppress_item_break = true;
        }

        NodeValue::SoftBreak => {
            output.push('\n');
        }

        NodeValue::LineBreak => {
            output.push_str("\\\n");
        }

        // Note: comrak v0.36 has no BlankLine variant; blank lines are handled
        // during post-processing normalization.

        NodeValue::Alert(alert) => {
            *skip_next_blank = false;
            let alert_type = format!("{:?}", alert.alert_type).to_uppercase();
            output.push_str(&format!("> [!{alert_type}]\n"));

            let q_prefix = format!("{prefix}> ");
            let q_subsequent = format!("{subsequent_prefix}> ");

            let mut inner = String::new();
            for child in node.children() {
                inner.push_str(&render_node(
                    child,
                    line_wrapper,
                    list_spacing,
                    &q_prefix,
                    &q_subsequent,
                    suppress_item_break,
                    skip_next_blank,
                    in_heading,
                    current_list_tight,
                    options,
                ));
            }

            output.push_str(&inner.trim_end_matches('\n'));
            output.push('\n');
            *suppress_item_break = false;
        }

        // Note: comrak v0.36 has no LinkReference variant; link references
        // are resolved during parsing.

        // Inline elements - handled by render_inline_children
        _ => {
            // For any other node types, render children
            for child in node.children() {
                output.push_str(&render_node(
                    child,
                    line_wrapper,
                    list_spacing,
                    prefix,
                    subsequent_prefix,
                    suppress_item_break,
                    skip_next_blank,
                    in_heading,
                    current_list_tight,
                    options,
                ));
            }
        }
    }

    output
}

/// Render inline children of a node to a flat string.
fn render_inline_children<'a>(node: &'a AstNode<'a>, options: &Options, in_heading: bool) -> String {
    let mut output = String::new();
    for child in node.children() {
        output.push_str(&render_inline(child, options, in_heading));
    }
    output
}

/// Render a single inline node to string.
fn render_inline<'a>(node: &'a AstNode<'a>, options: &Options, in_heading: bool) -> String {
    match &node.data.borrow().value {
        NodeValue::Text(text) => text.clone(),

        NodeValue::Code(code) => {
            let text = &code.literal;
            if text.starts_with('`') || text.ends_with('`') {
                format!("`` {text} ``")
            } else {
                format!("`{text}`")
            }
        }

        NodeValue::Emph => {
            let inner = render_inline_children(node, options, in_heading);
            format!("*{inner}*")
        }

        NodeValue::Strong => {
            let inner = render_inline_children(node, options, in_heading);
            format!("**{inner}**")
        }

        NodeValue::Strikethrough => {
            let inner = render_inline_children(node, options, in_heading);
            format!("~~{inner}~~")
        }

        NodeValue::Link(link) => {
            let inner = render_inline_children(node, options, in_heading);
            let title = if link.title.is_empty() {
                String::new()
            } else {
                format!(" \"{}\"", link.title.replace('"', "\\\""))
            };
            format!("[{inner}]({}{})", link.url, title)
        }

        NodeValue::Image(image) => {
            let inner = render_inline_children(node, options, in_heading);
            let title = if image.title.is_empty() {
                String::new()
            } else {
                format!(" \"{}\"", image.title.replace('"', "\\\""))
            };
            format!("![{inner}]({}{})", image.url, title)
        }

        NodeValue::HtmlInline(html) => html.clone(),

        NodeValue::SoftBreak => "\n".to_string(),

        NodeValue::LineBreak => "\\\n".to_string(),

        NodeValue::Escaped => {
            // Escaped character - the children will contain the character
            let inner = render_inline_children(node, options, in_heading);
            if inner == "." && in_heading {
                // In headings, periods don't need escaping
                ".".to_string()
            } else if inner == "." {
                // Only escape if it would form a list marker
                // For now, preserve the escape
                format!("\\{inner}")
            } else {
                format!("\\{inner}")
            }
        }

        NodeValue::FootnoteReference(fr) => {
            format!("[^{}]", fr.name)
        }

        NodeValue::Math(math) => {
            if math.display_math {
                format!("$${}$$", math.literal)
            } else {
                format!("${}$", math.literal)
            }
        }

        NodeValue::WikiLink(wl) => {
            format!("[[{}]]", wl.url)
        }

        _ => {
            // Fallback: render children
            render_inline_children(node, options, in_heading)
        }
    }
}

/// Check if a list can be rendered tight.
fn can_be_tight<'a>(list_node: &'a AstNode<'a>) -> bool {
    for item in list_node.children() {
        if !matches!(item.data.borrow().value, NodeValue::Item(_)) {
            continue;
        }
        // If the item has more than one child, it must be loose
        if item.children().count() > 1 {
            return false;
        }
    }
    true
}

/// Get tasklist marker for a paragraph if its parent is a tasklist item.
fn get_tasklist_marker<'a>(para_node: &'a AstNode<'a>) -> Option<String> {
    if let Some(parent) = para_node.parent() {
        if let NodeValue::TaskItem(checked) = &parent.data.borrow().value {
            let marker = if checked.is_some() { "[x] " } else { "[ ] " };
            // Only add marker to first paragraph in the item
            if parent.children().next().map(|c| std::ptr::eq(c, para_node)).unwrap_or(false) {
                return Some(marker.to_string());
            }
        }
    }
    None
}

/// Get table column alignments.
fn get_table_alignments<'a>(table_node: &'a AstNode<'a>) -> Vec<TableAlignment> {
    if let NodeValue::Table(table) = &table_node.data.borrow().value {
        table.alignments.clone()
    } else {
        vec![]
    }
}

/// Render a table row.
fn render_table_row<'a>(row_node: &'a AstNode<'a>, options: &Options) -> String {
    let cells: Vec<String> = row_node
        .children()
        .map(|cell| {
            let content = render_inline_children(cell, options, false);
            content.replace('|', "\\|")
        })
        .collect();
    format!("| {} |\n", cells.join(" | "))
}

/// Calculate minimum fence length needed for code content.
fn min_fence_length(code_content: &str, fence_char: char) -> usize {
    let pattern = format!(
        r"(?m)^[ ]{{0,3}}({escaped}{{3,}})",
        escaped = regex::escape(&fence_char.to_string())
    );
    let re = Regex::new(&pattern).unwrap();
    let max_len = re
        .captures_iter(code_content)
        .map(|caps| caps.get(1).unwrap().as_str().len())
        .max()
        .unwrap_or(0);
    std::cmp::max(3, max_len + 1)
}

/// Normalize and wrap Markdown text filling paragraphs to the full width.
///
/// This is the main entry point for Markdown formatting.
pub fn fill_markdown(
    markdown_text: &str,
    dedent_input: bool,
    width: usize,
    semantic: bool,
    cleanups: bool,
    smartquotes: bool,
    ellipses: bool,
    line_wrapper: Option<LineWrapper>,
    list_spacing: ListSpacing,
) -> String {
    let line_wrapper = line_wrapper.unwrap_or_else(|| {
        if semantic {
            line_wrap_by_sentence(width, DEFAULT_MIN_LINE_LEN, true)
        } else {
            line_wrap_to_width(width, true)
        }
    });

    // Extract frontmatter before any processing
    let (frontmatter, content) = split_frontmatter(markdown_text);

    let mut text = if !frontmatter.is_empty() {
        content
    } else {
        markdown_text.to_string()
    };

    if dedent_input {
        text = dedent(&text);
    }

    text = text.trim().to_string();
    text.push('\n');

    // Preprocess: ensure proper blank lines around block content within tags
    text = preprocess_tag_block_spacing(&text);

    // Parse with comrak
    let arena = Arena::new();
    let options = flowmark_comrak_options();
    let root = comrak::parse_document(&arena, &text, &options);

    // Apply cleanups if enabled
    if cleanups {
        doc_cleanups(root);
    }

    // Apply typography transforms
    if smartquotes {
        apply_smart_quotes_to_ast(root);
    }
    if ellipses {
        apply_ellipses_to_ast(root);
    }

    // Render the AST to normalized markdown
    let mut suppress_item_break = true;
    let mut skip_next_blank = false;
    let mut in_heading = false;
    let mut current_list_tight = false;

    let result = render_node(
        root,
        &line_wrapper,
        list_spacing,
        "",
        "",
        &mut suppress_item_break,
        &mut skip_next_blank,
        &mut in_heading,
        &mut current_list_tight,
        &options,
    );

    // Apply text-level normalizations
    let result = normalize_comrak_output(&result);

    // Reattach frontmatter if present
    if !frontmatter.is_empty() {
        format!("{frontmatter}{result}")
    } else {
        result
    }
}

/// Apply smart quotes to all text nodes in the AST.
fn apply_smart_quotes_to_ast<'a>(root: &'a AstNode<'a>) {
    for node in root.descendants() {
        let mut data = node.data.borrow_mut();
        if let NodeValue::Text(ref mut text) = data.value {
            *text = smart_quotes(text);
        }
    }
}

/// Apply ellipsis conversion to all text nodes in the AST.
fn apply_ellipses_to_ast<'a>(root: &'a AstNode<'a>) {
    for node in root.descendants() {
        let mut data = node.data.borrow_mut();
        if let NodeValue::Text(ref mut text) = data.value {
            *text = apply_ellipses(text);
        }
    }
}

/// Simple dedent: remove common leading whitespace from all lines.
fn dedent(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return text.to_string();
    }

    // Find minimum indentation (ignoring empty lines)
    let min_indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    if min_indent == 0 {
        return text.to_string();
    }

    lines
        .iter()
        .map(|l| {
            if l.len() >= min_indent {
                &l[min_indent..]
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
