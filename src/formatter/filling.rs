//! Markdown filling and normalization pipeline.
//!
//! This is the core formatting pipeline that normalizes and wraps Markdown text.
//! It handles the complex interaction between comrak's AST rendering and the
//! text-level normalization needed to match Python/Marko behavior.
//!
//! Ported from Python: `flowmark/linewrapping/markdown_filling.py` and
//! parts of `flowmark/formats/flowmark_markdown.py`

use regex::Regex;
use std::fmt::Write as _;
use std::sync::LazyLock;

use comrak::nodes::{AstNode, ListType, NodeValue, TableAlignment};
use comrak::{Arena, Options};

use crate::config::{DEFAULT_MIN_LINE_LEN, ListSpacing};
use crate::formatter::markdown::flowmark_comrak_options;
use crate::parser::frontmatter::split_frontmatter;
use crate::transform::cleanups::doc_cleanups;
use crate::typography::ellipses::ellipses as apply_ellipses;
use crate::typography::quotes::smart_quotes;
use crate::wrapping::LineWrapper;
use crate::wrapping::line_wrappers::{line_wrap_by_sentence, line_wrap_to_width};
use crate::wrapping::tag_handling::preprocess_tag_block_spacing;

// ===== Regex patterns for normalization =====

/// Pattern for blank lines with trailing whitespace.
static BLANK_LINE_WS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[ \t]+$").expect("valid BLANK_LINE_WS regex"));

/// Pattern for code fence with space before language (horizontal whitespace only).
static CODE_FENCE_SPACE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^([ \t]*```)[^\S\n]+(\w)").expect("valid CODE_FENCE_SPACE regex")
});

/// Pattern for numbered list items with two spaces after period.
static NUMBERED_ITEM_TWO_SPACES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)\.  ").expect("valid NUMBERED_ITEM_TWO_SPACES regex"));

/// Normalize blank lines by removing trailing whitespace.
fn normalize_blank_lines(text: &str) -> String {
    BLANK_LINE_WS.replace_all(text, "").into_owned()
}

/// Remove space between code fence and language identifier.
fn normalize_code_fences(text: &str) -> String {
    CODE_FENCE_SPACE.replace_all(text, "$1$2").into_owned()
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

/// Apply all text-level normalizations to comrak output.
fn normalize_comrak_output(text: &str) -> String {
    let text = normalize_blank_lines(text);
    let text = normalize_code_fences(&text);
    let text = normalize_numbered_lists(&text);
    collapse_blank_lines_outside_code(&text)
}

/// Check if a trimmed line is a closing fence matching the given fence string.
fn is_closing_fence(trimmed: &str, fence_str: &str) -> bool {
    if fence_str.is_empty() || !trimmed.starts_with(fence_str) {
        return false;
    }
    let fence_char = fence_str.chars().next().unwrap_or('`');
    trimmed[fence_str.len()..]
        .chars()
        .all(|c| c == fence_char || c.is_whitespace())
}

/// Detect an opening code fence and return the fence string if found.
fn detect_opening_fence(trimmed: &str) -> Option<String> {
    let is_backtick_fence = trimmed.starts_with("```");
    let is_tilde_fence = trimmed.starts_with("~~~");
    if is_backtick_fence || is_tilde_fence {
        let fence_char = if is_backtick_fence { '`' } else { '~' };
        let fence_len = trimmed.chars().take_while(|&c| c == fence_char).count();
        Some(std::iter::repeat_n(fence_char, fence_len).collect())
    } else {
        None
    }
}

/// Process text line-by-line, applying a transformation only outside fenced code blocks.
///
/// `process_outside` receives each non-code, non-fence line and returns zero or more
/// output lines. Code block lines and fence lines are included in the output unchanged.
fn transform_outside_code_fences<F>(text: &str, mut process_outside: F) -> String
where
    F: FnMut(&str) -> Vec<String>,
{
    let lines: Vec<&str> = text.lines().collect();
    let had_trailing_newline = text.ends_with('\n');
    let mut result: Vec<String> = Vec::new();
    let mut in_code = false;
    let mut fence_str = String::new();

    for line in &lines {
        if in_code {
            result.push((*line).to_string());
            if is_closing_fence(line.trim(), &fence_str) {
                in_code = false;
            }
        } else if let Some(fs) = detect_opening_fence(line.trim()) {
            fence_str = fs;
            in_code = true;
            result.push((*line).to_string());
        } else {
            result.extend(process_outside(line));
        }
    }

    let mut output = result.join("\n");
    if had_trailing_newline {
        output.push('\n');
    }
    output
}

/// Collapse multiple blank lines to single blank lines, but preserve
/// content inside code blocks (fenced with backticks or tildes).
///
/// Uses the fence helpers directly (rather than `transform_outside_code_fences`)
/// because it needs to reset the blank-line counter at fence boundaries.
fn collapse_blank_lines_outside_code(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let had_trailing_newline = text.ends_with('\n');
    let mut result: Vec<&str> = Vec::new();
    let mut in_code = false;
    let mut fence_str = String::new();
    let mut consecutive_empty: usize = 0;

    for line in &lines {
        if in_code {
            result.push(line);
            if is_closing_fence(line.trim(), &fence_str) {
                in_code = false;
                consecutive_empty = 0;
            }
        } else if let Some(fs) = detect_opening_fence(line.trim()) {
            fence_str = fs;
            in_code = true;
            consecutive_empty = 0;
            result.push(line);
        } else if line.trim().is_empty() {
            consecutive_empty += 1;
            if consecutive_empty <= 1 {
                result.push(line);
            }
        } else {
            consecutive_empty = 0;
            result.push(line);
        }
    }

    let mut output = result.join("\n");
    if had_trailing_newline {
        output.push('\n');
    }
    output
}

/// Replace escaped characters with placeholders, but only outside fenced code blocks.
/// This prevents comrak from stripping backslash escapes during parsing.
fn protect_escapes_outside_code(text: &str, placeholders: &[(String, String)]) -> String {
    transform_outside_code_fences(text, |line| {
        let mut processed = line.to_string();
        for (escaped, placeholder) in placeholders {
            processed = processed.replace(escaped.as_str(), placeholder.as_str());
        }
        vec![processed]
    })
}

/// Remove unnecessary period escapes from the formatted output.
/// Period escapes (\.) are only needed at the start of a line where
/// `DIGITS\.` would be interpreted as an ordered list marker.
/// In headings and mid-paragraph, period escapes are unnecessary.
/// Preserves content inside code spans (backtick-delimited) and fenced code blocks.
fn postprocess_period_escapes(text: &str) -> String {
    transform_outside_code_fences(text, |line| {
        let trimmed_start = line.trim_start();

        if trimmed_start.starts_with('#') {
            // Heading line: remove period escapes but preserve code spans
            return vec![remove_period_escapes_preserving_code(line)];
        }

        // Strip blockquote markers for content analysis
        let after_quotes =
            trimmed_start.trim_start_matches(|c: char| c == '>' || c.is_whitespace());

        // Strip unordered list markers (- , * , + ) and optional task list markers
        let after_list_marker = after_quotes
            .strip_prefix("- ")
            .or_else(|| after_quotes.strip_prefix("* "))
            .or_else(|| after_quotes.strip_prefix("+ "))
            .map_or(after_quotes, |rest| {
                // Also strip task list markers: [ ] , [x] , [X]
                rest.strip_prefix("[ ] ")
                    .or_else(|| rest.strip_prefix("[x] "))
                    .or_else(|| rest.strip_prefix("[X] "))
                    .unwrap_or(rest)
            });

        // Check if content starts with DIGITS\.
        let digit_end = after_list_marker
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(after_list_marker.len());

        if digit_end > 0 && after_list_marker[digit_end..].starts_with("\\.") {
            // DIGITS\. at effective line start: keep the escape to prevent list interpretation
            vec![line.to_string()]
        } else {
            // No list-like pattern at start: remove period escapes, preserving code spans
            vec![remove_period_escapes_preserving_code(line)]
        }
    })
}

/// Remove `\.` → `.` on a single line, but preserve content inside backtick code spans.
fn remove_period_escapes_preserving_code(line: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '`' {
            // Found backtick(s) - measure opening sequence length
            let bt_count = chars[i..].iter().take_while(|&&c| c == '`').count();
            // Copy opening backticks
            for _ in 0..bt_count {
                result.push('`');
            }
            i += bt_count;

            // Find matching closing backtick sequence (same length)
            let mut found_close = false;
            while i < len {
                if chars[i] == '`' {
                    let close_count = chars[i..].iter().take_while(|&&c| c == '`').count();
                    if close_count == bt_count {
                        // Matching close: copy and exit code span
                        for _ in 0..close_count {
                            result.push('`');
                        }
                        i += close_count;
                        found_close = true;
                        break;
                    }
                    // Non-matching backticks: copy as code content
                    for _ in 0..close_count {
                        result.push('`');
                    }
                    i += close_count;
                } else {
                    // Inside code span: copy literally (no escape processing)
                    result.push(chars[i]);
                    i += 1;
                }
            }
            if !found_close {
                // Unmatched backticks: already copied, continue
            }
        } else if chars[i] == '\\' && i + 1 < len && chars[i + 1] == '.' {
            // \. outside code span → just .
            result.push('.');
            i += 2;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Check if a node is a block-level element that needs blank line separation.
fn is_block_element(node: &AstNode) -> bool {
    matches!(
        node.data.borrow().value,
        NodeValue::Paragraph
            | NodeValue::Heading(_)
            | NodeValue::List(_)
            | NodeValue::BlockQuote
            | NodeValue::CodeBlock(_)
            | NodeValue::ThematicBreak
            | NodeValue::HtmlBlock(_)
            | NodeValue::Table(_)
            | NodeValue::FootnoteDefinition(_)
            | NodeValue::Alert(_)
    )
}

/// Check if inline content ends with a hard break (backslash before newline).
fn inline_ends_with_hard_break<'a>(node: &'a AstNode<'a>) -> bool {
    let children: Vec<_> = node.children().collect();
    if let Some(last_child) = children.last() {
        // Check for explicit LineBreak node
        if matches!(last_child.data.borrow().value, NodeValue::LineBreak) {
            return true;
        }
        // Check if last text node ends with backslash (hard break in headings)
        if let NodeValue::Text(ref text) = last_child.data.borrow().value {
            if text.ends_with('\\') {
                return true;
            }
        }
    }
    false
}

/// Render block children with proper blank line separation between them.
fn render_block_children<'a>(
    node: &'a AstNode<'a>,
    line_wrapper: &LineWrapper,
    list_spacing: ListSpacing,
    prefix: &str,
    subsequent_prefix: &str,
    in_heading: &mut bool,
    options: &Options,
) -> String {
    let mut output = String::new();
    let mut prev_was_block = false;
    let mut prev_ended_with_double_newline = false;
    let mut prev_was_hard_break_heading = false;

    for child in node.children() {
        let child_is_block = is_block_element(child);

        // Check if current child is a hard-break heading
        let child_is_hard_break_heading =
            matches!(child.data.borrow().value, NodeValue::Heading(_))
                && inline_ends_with_hard_break(child);

        // Add blank line between consecutive block elements,
        // unless adjacent to a heading ending with a hard break
        if child_is_block
            && prev_was_block
            && !prev_ended_with_double_newline
            && !prev_was_hard_break_heading
            && !child_is_hard_break_heading
        {
            output.push('\n');
        }

        let block_output = render_block(
            child,
            line_wrapper,
            list_spacing,
            prefix,
            subsequent_prefix,
            in_heading,
            options,
        );
        prev_ended_with_double_newline = block_output.ends_with("\n\n");
        prev_was_hard_break_heading = matches!(child.data.borrow().value, NodeValue::Heading(_))
            && inline_ends_with_hard_break(child);
        output.push_str(&block_output);
        prev_was_block = child_is_block;
    }

    output
}

/// Render block children within a quoted context (blockquote or alert).
/// Uses `blank_prefix` (e.g., ">") for blank separator lines between blocks.
#[allow(clippy::too_many_arguments)]
fn render_block_children_quoted<'a>(
    node: &'a AstNode<'a>,
    line_wrapper: &LineWrapper,
    list_spacing: ListSpacing,
    prefix: &str,
    subsequent_prefix: &str,
    blank_prefix: &str,
    in_heading: &mut bool,
    options: &Options,
) -> String {
    let mut output = String::new();
    let mut prev_was_block = false;
    let mut prev_ended_with_double_newline = false;

    for child in node.children() {
        let child_is_block = is_block_element(child);

        // Add blank line between consecutive block elements
        // Use the blank_prefix (e.g., "> ") to maintain the quote context
        if child_is_block && prev_was_block && !prev_ended_with_double_newline {
            output.push_str(blank_prefix);
            output.push_str(" \n");
        }

        let block_output = render_block(
            child,
            line_wrapper,
            list_spacing,
            prefix,
            subsequent_prefix,
            in_heading,
            options,
        );
        prev_ended_with_double_newline = block_output.ends_with("\n\n");
        output.push_str(&block_output);
        prev_was_block = child_is_block;
    }

    output
}

/// Render a single block-level node.
fn render_block<'a>(
    node: &'a AstNode<'a>,
    line_wrapper: &LineWrapper,
    list_spacing: ListSpacing,
    prefix: &str,
    subsequent_prefix: &str,
    in_heading: &mut bool,
    options: &Options,
) -> String {
    let mut output = String::new();

    match &node.data.borrow().value {
        NodeValue::Document => {
            output = render_block_children(
                node,
                line_wrapper,
                list_spacing,
                prefix,
                subsequent_prefix,
                in_heading,
                options,
            );
        }

        NodeValue::Paragraph => {
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

            // Check if heading ends with a hard break (either LineBreak node or trailing backslash)
            let ends_with_hard_break =
                inline_ends_with_hard_break(node) || inline_text.ends_with('\\');

            let _ = writeln!(output, "{prefix}{hashes} {inline_text}");
            if !ends_with_hard_break {
                output.push('\n');
            }
        }

        NodeValue::List(list) => {
            // Determine effective tightness
            let is_tight = match list_spacing {
                ListSpacing::Preserve => list.tight,
                ListSpacing::Tight => can_be_tight(node),
                ListSpacing::Loose => false,
            };

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
                    (format!("{prefix}{marker} "), format!("{subsequent_prefix}  "))
                };

                // For loose lists, add blank line between items (except before first)
                // Use the outer prefix (without list indentation) for the blank line
                // to maintain blockquote context
                if !is_tight && i > 0 {
                    let blank_prefix = subsequent_prefix.trim_end();
                    if blank_prefix.is_empty() {
                        output.push('\n');
                    } else {
                        output.push_str(blank_prefix);
                        output.push('\n');
                    }
                }

                render_list_item(
                    child,
                    &mut output,
                    line_wrapper,
                    list_spacing,
                    &item_prefix,
                    &item_subsequent,
                    in_heading,
                    options,
                );
            }
        }

        NodeValue::BlockQuote => {
            let q_prefix = format!("{prefix}> ");
            let q_subsequent = format!("{subsequent_prefix}> ");

            let inner = render_block_children_quoted(
                node,
                line_wrapper,
                list_spacing,
                &q_prefix,
                &q_subsequent,
                &format!("{subsequent_prefix}>"),
                in_heading,
                options,
            );

            // Trim trailing newlines and re-add single newline
            output.push_str(inner.trim_end_matches('\n'));
            output.push('\n');
        }

        NodeValue::CodeBlock(code_block) => {
            let info = &code_block.info;
            let literal = &code_block.literal;
            let code_content = literal.trim_end_matches('\n');

            let fence_char = if code_block.fenced {
                if code_block.fence_char == b'~' { '~' } else { '`' }
            } else {
                '`'
            };

            // Calculate minimum fence length needed
            let fence_len = min_fence_length(code_content, fence_char).max(if code_block.fenced {
                code_block.fence_length
            } else {
                3
            });
            let fence: String = std::iter::repeat_n(fence_char, fence_len).collect();

            let lang_text = if info.is_empty() { String::new() } else { info.clone() };

            let _ = writeln!(output, "{prefix}{fence}{lang_text}");
            let empty_prefix = subsequent_prefix.trim_end();
            for line in code_content.split('\n') {
                if line.is_empty() {
                    output.push_str(empty_prefix);
                    output.push('\n');
                } else {
                    let _ = writeln!(output, "{subsequent_prefix}{line}");
                }
            }
            let _ = writeln!(output, "{subsequent_prefix}{fence}");
        }

        NodeValue::ThematicBreak => {
            let _ = writeln!(output, "{prefix}* * *");
        }

        NodeValue::HtmlBlock(html) => {
            let literal = &html.literal;
            // Check if this HTML block has wrappable text content
            // (e.g., HTML comments/tags mixed with regular text)
            let trimmed = literal.trim();
            let has_text_content = !trimmed.is_empty()
                && trimmed.contains(|c: char| c.is_alphabetic())
                && trimmed.chars().filter(|&c| c == '<').count() > 0;

            if has_text_content && trimmed.len() > 40 {
                // Collapse internal whitespace and wrap as text
                // Join all lines into a single line first
                let single_line: String =
                    literal.lines().map(str::trim).collect::<Vec<_>>().join(" ").trim().to_string();
                let wrapped = line_wrapper(&single_line, prefix, subsequent_prefix);
                output.push_str(&wrapped);
                output.push('\n');
            } else {
                // Short or non-wrappable HTML: pass through as-is
                output.push_str(prefix);
                output.push_str(literal);
                if !literal.ends_with('\n') {
                    output.push('\n');
                }
            }
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
            let _ = writeln!(output, "| {} |", delimiters.join(" | "));

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

            let mut first_child = true;
            for child in node.children() {
                let (p, sp) = if first_child {
                    (fn_prefix.clone(), fn_subsequent.clone())
                } else {
                    (fn_subsequent.clone(), fn_subsequent.clone())
                };
                let child_output =
                    render_block(child, line_wrapper, list_spacing, &p, &sp, in_heading, options);
                output.push_str(&child_output);
                first_child = false;
            }

            // Ensure proper ending
            if !output.ends_with("\n\n") {
                if output.ends_with('\n') {
                    output.push('\n');
                } else {
                    output.push_str("\n\n");
                }
            }
        }

        NodeValue::Alert(alert) => {
            let alert_type = format!("{:?}", alert.alert_type).to_uppercase();
            let _ = writeln!(output, "> [!{alert_type}]");

            let q_prefix = format!("{prefix}> ");
            let q_subsequent = format!("{subsequent_prefix}> ");

            let inner = render_block_children_quoted(
                node,
                line_wrapper,
                list_spacing,
                &q_prefix,
                &q_subsequent,
                &format!("{subsequent_prefix}>"),
                in_heading,
                options,
            );

            output.push_str(inner.trim_end_matches('\n'));
            output.push('\n');
        }

        // Inline elements and other node types
        _ => {
            for child in node.children() {
                output.push_str(&render_block(
                    child,
                    line_wrapper,
                    list_spacing,
                    prefix,
                    subsequent_prefix,
                    in_heading,
                    options,
                ));
            }
        }
    }

    output
}

/// Check if a list item needs blank lines between its children.
/// This is true when:
/// - The item's parent list is loose (not tight)
/// - OR the item has multiple block children that aren't just para+nested-list
fn item_needs_child_spacing<'a>(node: &'a AstNode<'a>, parent_is_tight: bool) -> bool {
    if !parent_is_tight {
        // For loose lists, always add blank lines between children
        return true;
    }
    let children: Vec<_> = node.children().collect();
    if children.len() <= 1 {
        return false;
    }
    // For tight lists, only add spacing if there are multiple paragraphs
    // (not counting para + nested list as needing spacing)
    let para_count =
        children.iter().filter(|c| matches!(c.data.borrow().value, NodeValue::Paragraph)).count();
    para_count > 1
}

/// Render a list item's children.
#[allow(clippy::too_many_arguments)]
fn render_list_item<'a>(
    node: &'a AstNode<'a>,
    output: &mut String,
    line_wrapper: &LineWrapper,
    list_spacing: ListSpacing,
    item_prefix: &str,
    item_subsequent: &str,
    in_heading: &mut bool,
    options: &Options,
) {
    let mut first_child = true;
    let children: Vec<_> = node.children().collect();

    // Check if parent list is tight by looking at the list spacing context
    // We determine this by checking if the parent list node is tight
    let parent_is_tight = node.parent().is_some_and(|parent| {
        if let NodeValue::List(list) = &parent.data.borrow().value {
            match list_spacing {
                ListSpacing::Preserve => list.tight,
                ListSpacing::Tight => can_be_tight(parent),
                ListSpacing::Loose => false,
            }
        } else {
            false
        }
    });

    let needs_spacing = item_needs_child_spacing(node, parent_is_tight);

    for (i, child) in children.iter().enumerate() {
        let (p, sp) = if first_child {
            (item_prefix.to_string(), item_subsequent.to_string())
        } else {
            (item_subsequent.to_string(), item_subsequent.to_string())
        };

        // Add blank line between children in a list item
        if !first_child && needs_spacing {
            // Check if previous child was heading that ends with double newline
            let prev_ended_double = if i > 0 {
                matches!(children[i - 1].data.borrow().value, NodeValue::Heading(_))
            } else {
                false
            };

            // Don't add blank line before a heading that ends with hard break
            // (it connects tightly to the following content)
            let current_is_hard_break_heading =
                matches!(&child.data.borrow().value, NodeValue::Heading(_))
                    && inline_ends_with_hard_break(child);

            // Don't add blank line before a short tag-only HTML block
            // (e.g., <!-- comment --> on a continuation line in a list item)
            let current_is_tag_block =
                if let NodeValue::HtmlBlock(html) = &child.data.borrow().value {
                    let trimmed = html.literal.trim();
                    !trimmed.contains('\n')
                        && ((trimmed.starts_with("<!--") && trimmed.ends_with("-->"))
                            || (trimmed.starts_with("{%") && trimmed.ends_with("%}"))
                            || (trimmed.starts_with("{#") && trimmed.ends_with("#}"))
                            || (trimmed.starts_with("{{") && trimmed.ends_with("}}")))
                } else {
                    false
                };

            if !prev_ended_double && !current_is_hard_break_heading && !current_is_tag_block {
                output.push('\n');
            }
        }

        let child_output =
            render_block(child, line_wrapper, list_spacing, &p, &sp, in_heading, options);
        output.push_str(&child_output);
        first_child = false;
    }
}

/// Render inline children of a node to a flat string.
fn render_inline_children<'a>(
    node: &'a AstNode<'a>,
    options: &Options,
    in_heading: bool,
) -> String {
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
            // Escaped character - the children will contain the character.
            // Most escapes are handled via placeholders (pre-processing), but
            // comrak may still create Escaped nodes for some characters.
            let inner = render_inline_children(node, options, in_heading);
            format!("\\{inner}")
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
            if parent.children().next().is_some_and(|c| std::ptr::eq(c, para_node)) {
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
    let re = Regex::new(&pattern).expect("valid fence length regex");
    let max_len = re
        .captures_iter(code_content)
        .map(|caps| caps.get(1).expect("capture group 1 always exists").as_str().len())
        .max()
        .unwrap_or(0);
    std::cmp::max(3, max_len + 1)
}

/// Normalize and wrap Markdown text filling paragraphs to the full width.
///
/// This is the main entry point for Markdown formatting.
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
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
    // Escaped characters to protect from comrak stripping.
    // comrak strips backslash escapes (e.g., \~ → ~, \* → *) in the AST for most chars.
    // Period (\.) is handled by comrak's Escaped node, so we exclude it here.
    // We use Unicode Private Use Area placeholders to preserve escapes through the pipeline.
    // IMPORTANT: \\ must be first so \\X doesn't get partially matched as \X.
    // Period (.) IS included: comrak converts `1\.` to list items, losing the escape.
    const ESCAPE_CHARS: &[char] = &[
        '\\', '~', '*', '#', '-', '+', '>', '.', '!', '[', ']', '(', ')', '{', '}', '$', '_', '|',
        '`',
    ];

    let line_wrapper = line_wrapper.unwrap_or_else(|| {
        if semantic {
            line_wrap_by_sentence(width, DEFAULT_MIN_LINE_LEN, true)
        } else {
            line_wrap_to_width(width, true)
        }
    });

    // Extract frontmatter before any processing
    let (frontmatter, content) = split_frontmatter(markdown_text);

    let mut text = if frontmatter.is_empty() { markdown_text.to_string() } else { content };

    if dedent_input {
        text = dedent(&text);
    }

    text = text.trim().to_string();
    text.push('\n');

    // Preprocess: ensure proper blank lines around block content within tags
    text = preprocess_tag_block_spacing(&text);
    let mut escape_placeholders: Vec<(String, String)> = Vec::new();
    for &ch in ESCAPE_CHARS {
        let escaped = format!("\\{ch}");
        // Use a single PUA character per escape char for consistent width measurement.
        // Map to U+E000 + ASCII code point of the escaped character.
        let placeholder =
            char::from_u32(0xE000 + ch as u32).expect("valid PUA code point").to_string();
        escape_placeholders.push((escaped, placeholder));
    }
    // Apply replacements, but skip inside fenced code blocks
    text = protect_escapes_outside_code(&text, &escape_placeholders);

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
    let mut in_heading = false;

    let result = render_block(root, &line_wrapper, list_spacing, "", "", &mut in_heading, &options);

    // Restore all escaped characters from placeholders
    let mut result = result;
    for (escaped, placeholder) in &escape_placeholders {
        result = result.replace(placeholder.as_str(), escaped.as_str());
    }

    // Remove unnecessary period escapes (keep only at line starts where they prevent list interpretation)
    let result = postprocess_period_escapes(&result);

    // Apply text-level normalizations
    let result = normalize_comrak_output(&result);

    // Reattach frontmatter if present
    if frontmatter.is_empty() { result } else { format!("{frontmatter}{result}") }
}

/// Apply smart quotes to all text nodes in the AST.
/// Works at the paragraph level so quotes spanning inline elements are handled.
fn apply_smart_quotes_to_ast<'a>(root: &'a AstNode<'a>) {
    for node in root.descendants() {
        let is_para = matches!(
            node.data.borrow().value,
            NodeValue::Paragraph | NodeValue::Heading(_) | NodeValue::TableCell
        );
        if is_para {
            apply_smart_quotes_to_inline_tree(node);
        }
    }
}

/// Collect text nodes from inline tree, apply smart quotes to concatenated text,
/// then redistribute back.
#[allow(clippy::items_after_statements)]
fn apply_smart_quotes_to_inline_tree<'a>(node: &'a AstNode<'a>) {
    // Collect all text nodes with their content
    let mut text_nodes: Vec<&'a AstNode<'a>> = Vec::new();
    let mut concatenated = String::new();
    let mut char_boundaries: Vec<(usize, usize)> = Vec::new(); // (start, len) in chars

    fn collect_text_nodes<'a>(
        node: &'a AstNode<'a>,
        text_nodes: &mut Vec<&'a AstNode<'a>>,
        concatenated: &mut String,
        char_boundaries: &mut Vec<(usize, usize)>,
    ) {
        for child in node.children() {
            let data = child.data.borrow();
            match &data.value {
                NodeValue::Text(text) => {
                    let start = concatenated.chars().count();
                    let len = text.chars().count();
                    concatenated.push_str(text);
                    char_boundaries.push((start, len));
                    text_nodes.push(child);
                }
                NodeValue::Code(_) | NodeValue::HtmlInline(_) => {
                    // Skip code spans and raw HTML - don't apply smart quotes
                    // But add placeholder chars to maintain context
                    concatenated.push('X'); // placeholder for quote context
                }
                NodeValue::SoftBreak => {
                    concatenated.push(' ');
                }
                _ => {
                    // Recurse into emphasis, strong, link, etc.
                    drop(data);
                    collect_text_nodes(child, text_nodes, concatenated, char_boundaries);
                }
            }
        }
    }

    collect_text_nodes(node, &mut text_nodes, &mut concatenated, &mut char_boundaries);

    if text_nodes.is_empty() {
        return;
    }

    // Apply smart quotes to the full concatenated text
    let converted = smart_quotes(&concatenated);

    // Redistribute characters back to text nodes
    let converted_chars: Vec<char> = converted.chars().collect();
    for (i, text_node) in text_nodes.iter().enumerate() {
        let (start, len) = char_boundaries[i];
        if start + len <= converted_chars.len() {
            let new_text: String = converted_chars[start..start + len].iter().collect();
            let mut data = text_node.data.borrow_mut();
            if let NodeValue::Text(ref mut text) = data.value {
                *text = new_text;
            }
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
        .map(|l| if l.len() >= min_indent { &l[min_indent..] } else { l })
        .collect::<Vec<_>>()
        .join("\n")
}
