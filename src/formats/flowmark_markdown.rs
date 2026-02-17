//! Markdown normalizer using comrak.
//!
//! Ported from Python: flowmark/formats/flowmark_markdown.py
//!
//! This module renders a Markdown document AST back to normalized Markdown text,
//! using comrak for parsing and a custom renderer for output.

use comrak::nodes::{AstNode, ListDelimType, ListType, NodeList, NodeValue, TableAlignment};
use comrak::{Arena, Options, parse_document};
use regex::Regex;

use crate::linewrapping::protocols::LineWrapper;

/// Controls how list item spacing is handled during Markdown normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListSpacing {
    /// Keep lists tight or loose as authored (default).
    Preserve,
    /// Convert all lists to loose format (blank lines between items).
    Loose,
    /// Convert all lists to tight format where possible.
    Tight,
}

impl Default for ListSpacing {
    fn default() -> Self {
        Self::Preserve
    }
}

fn normalize_title_quotes(title: &str) -> String {
    let escaped = title.trim_matches('"').replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn min_fence_length(code_content: &str, fence_char: &str) -> usize {
    let pattern_str = format!(r"(?m)^[ ]{{0,3}}({}{{3,}})", regex::escape(fence_char));
    let re = Regex::new(&pattern_str).unwrap();
    let mut max_len = 0;
    for caps in re.captures_iter(code_content) {
        let fence = &caps[1];
        max_len = max_len.max(fence.len());
    }
    3.max(max_len + 1)
}

/// Render state for tracking prefix context.
struct RenderState<'w> {
    prefix: String,
    second_prefix: String,
    suppress_item_break: bool,
    skip_next_blank_line: bool,
    current_inline_text: String,
    in_heading: bool,
    list_spacing: ListSpacing,
    current_list_tight: bool,
    line_wrapper: &'w LineWrapper,
}

impl<'w> RenderState<'w> {
    fn new(line_wrapper: &'w LineWrapper, list_spacing: ListSpacing) -> Self {
        Self {
            prefix: String::new(),
            second_prefix: String::new(),
            suppress_item_break: true,
            skip_next_blank_line: false,
            current_inline_text: String::new(),
            in_heading: false,
            list_spacing,
            current_list_tight: false,
            line_wrapper,
        }
    }

    fn wrap(&self, text: &str, initial: &str, subsequent: &str) -> String {
        (self.line_wrapper)(text, initial, subsequent)
    }
}

/// Render a comrak AST node tree to normalized Markdown.
fn render_node<'a>(node: &'a AstNode<'a>, state: &mut RenderState<'_>) -> String {
    let ast = node.data.borrow();

    match &ast.value {
        NodeValue::Document => {
            drop(ast);
            render_children(node, state)
        }

        NodeValue::Paragraph => {
            drop(ast);
            state.skip_next_blank_line = false;
            state.suppress_item_break = false;
            state.current_inline_text.clear();

            let mut children_text = render_children(node, state);

            // GFM task list checkbox support
            if let Some(parent) = node.parent() {
                let parent_ast = parent.data.borrow();
                if let NodeValue::TaskItem(Some(checked)) = &parent_ast.value {
                    let marker = if *checked != '\0' { "x" } else { " " };
                    children_text = format!("[{marker}] {children_text}");
                }
            }

            let wrapped =
                state.wrap(&children_text, &state.prefix, &state.second_prefix);
            state.prefix = state.second_prefix.clone();
            state.current_inline_text.clear();
            format!("{wrapped}\n")
        }

        NodeValue::List(list) => {
            let list = list.clone();
            drop(ast);
            state.skip_next_blank_line = false;

            let is_tight = match state.list_spacing {
                ListSpacing::Preserve => list.tight,
                ListSpacing::Tight => can_be_tight(node),
                ListSpacing::Loose => false,
            };

            let old_tight = state.current_list_tight;
            state.current_list_tight = is_tight;

            let mut result = String::new();
            let mut item_index: usize = 0;

            for child in node.children() {
                let child_ast = child.data.borrow();
                if matches!(&child_ast.value, NodeValue::Item(_) | NodeValue::TaskItem(_)) {
                    let (prefix, subsequent) = get_list_item_prefix(&list, item_index);
                    drop(child_ast);

                    let old_prefix = state.prefix.clone();
                    let old_second = state.second_prefix.clone();
                    state.prefix = format!("{}{prefix}", state.prefix);
                    state.second_prefix = format!("{}{subsequent}", state.second_prefix);

                    let rendered = render_list_item(child, state);
                    result.push_str(&rendered);

                    state.prefix = old_prefix;
                    state.second_prefix = old_second;
                    item_index += 1;
                }
            }

            state.current_list_tight = old_tight;
            state.prefix = state.second_prefix.clone();
            result
        }

        NodeValue::Heading(heading) => {
            let level = heading.level;
            drop(ast);
            state.in_heading = true;
            state.current_inline_text.clear();
            let children_content = render_children(node, state);
            state.in_heading = false;
            state.current_inline_text.clear();

            let hashes = "#".repeat(level as usize);
            if children_content.ends_with('\\') {
                let result = format!("{}{hashes} {children_content}\n", state.prefix);
                state.prefix = state.second_prefix.clone();
                result
            } else {
                let result = format!("{}{hashes} {children_content}\n\n", state.prefix);
                state.prefix = state.second_prefix.clone();
                state.skip_next_blank_line = true;
                state.suppress_item_break = true;
                result
            }
        }

        NodeValue::BlockQuote => {
            drop(ast);
            state.skip_next_blank_line = false;

            let old_prefix = state.prefix.clone();
            let old_second = state.second_prefix.clone();
            state.prefix = format!("{}> ", state.prefix);
            state.second_prefix = format!("{}> ", state.second_prefix);

            let result = render_children(node, state);
            let result = result.trim_end_matches('\n');

            state.prefix = old_prefix;
            state.second_prefix = old_second;
            state.prefix = state.second_prefix.clone();
            state.suppress_item_break = false;
            format!("{result}\n")
        }

        NodeValue::CodeBlock(code_block) => {
            let code_block = code_block.clone();
            drop(ast);
            state.skip_next_blank_line = false;

            let code_content = code_block.literal.trim_end_matches('\n');
            let lang = code_block.info.trim();

            let (fence_char, original_fence_len) = if code_block.fenced {
                (
                    if code_block.fence_char == b'~' {
                        "~"
                    } else {
                        "`"
                    },
                    code_block.fence_length as usize,
                )
            } else {
                ("`", 3)
            };

            let min_len = min_fence_length(code_content, fence_char);
            let fence_len = original_fence_len.max(min_len);
            let fence = fence_char.repeat(fence_len);

            let mut lines = vec![format!("{}{fence}{lang}", state.prefix)];
            let empty_line_prefix = state.second_prefix.trim_end().to_string();
            for line in code_content.split('\n') {
                if line.is_empty() {
                    lines.push(empty_line_prefix.clone());
                } else {
                    lines.push(format!("{}{line}", state.second_prefix));
                }
            }
            lines.push(format!("{}{fence}", state.second_prefix));
            state.prefix = state.second_prefix.clone();
            state.suppress_item_break = false;
            lines.join("\n") + "\n"
        }

        NodeValue::HtmlBlock(html) => {
            let literal = html.literal.clone();
            drop(ast);
            let result = format!("{}{literal}", state.prefix);
            state.prefix = state.second_prefix.clone();
            result
        }

        NodeValue::ThematicBreak => {
            drop(ast);
            let result = format!("{}* * *\n", state.prefix);
            state.prefix = state.second_prefix.clone();
            result
        }

        NodeValue::SoftBreak => {
            drop(ast);
            "\n".to_string()
        }

        NodeValue::LineBreak => {
            drop(ast);
            "\\\n".to_string()
        }

        NodeValue::Text(text) => {
            let text_str = text.to_string();
            state.current_inline_text.push_str(&text_str);
            drop(ast);
            text_str
        }

        NodeValue::Code(code) => {
            let text = &code.literal;
            let result = if text.starts_with('`') || text.ends_with('`') {
                format!("`` {text} ``")
            } else {
                format!("`{text}`")
            };
            drop(ast);
            result
        }

        NodeValue::Emph => {
            drop(ast);
            let children = render_children(node, state);
            format!("*{children}*")
        }

        NodeValue::Strong => {
            drop(ast);
            let children = render_children(node, state);
            format!("**{children}**")
        }

        NodeValue::Strikethrough => {
            drop(ast);
            let children = render_children(node, state);
            format!("~~{children}~~")
        }

        NodeValue::Link(link) => {
            let url = link.url.clone();
            let title = link.title.clone();
            drop(ast);
            let text = render_children(node, state);
            if title.is_empty() {
                format!("[{text}]({url})")
            } else {
                let normalized_title = normalize_title_quotes(&title);
                format!("[{text}]({url} {normalized_title})")
            }
        }

        NodeValue::Image(image) => {
            let url = image.url.clone();
            let title = image.title.clone();
            drop(ast);
            let alt = render_children(node, state);
            if title.is_empty() {
                format!("![{alt}]({url})")
            } else {
                let normalized_title = normalize_title_quotes(&title);
                format!("![{alt}]({url} {normalized_title})")
            }
        }

        NodeValue::HtmlInline(html) => {
            let result = html.clone();
            drop(ast);
            result
        }

        NodeValue::Table(_) => {
            drop(ast);
            render_table(node, state)
        }

        NodeValue::TableRow(_) => {
            drop(ast);
            render_table_row(node, state)
        }

        NodeValue::TableCell => {
            drop(ast);
            let rendered = render_children(node, state);
            rendered.replace('|', "\\|")
        }

        NodeValue::FootnoteDefinition(def) => {
            let label = def.name.clone();
            drop(ast);
            let label_part = format!("[^{label}]: ");
            let indent = "    ";

            let old_prefix = state.prefix.clone();
            let old_second = state.second_prefix.clone();
            state.prefix = format!("{}{label_part}", state.prefix);
            state.second_prefix = format!("{}{indent}", state.second_prefix);

            let content = render_children(node, state);

            state.prefix = old_prefix;
            state.second_prefix = old_second;
            state.prefix = state.second_prefix.clone();
            state.suppress_item_break = true;

            let content = content.trim_end_matches('\n');
            format!("{content}\n\n")
        }

        NodeValue::FootnoteReference(fref) => {
            let label = fref.name.clone();
            drop(ast);
            format!("[^{label}]")
        }

        NodeValue::Escaped => {
            drop(ast);
            let children = render_children(node, state);
            children
        }

        NodeValue::Item(_) | NodeValue::TaskItem(_) => {
            drop(ast);
            render_children(node, state)
        }

        NodeValue::Alert(alert) => {
            let alert_type = format!("{:?}", alert.alert_type).to_uppercase();
            drop(ast);
            state.skip_next_blank_line = false;

            let alert_header = format!("> [!{alert_type}]\n");

            let old_prefix = state.prefix.clone();
            let old_second = state.second_prefix.clone();
            state.prefix = format!("{}> ", state.prefix);
            state.second_prefix = format!("{}> ", state.second_prefix);

            let result = render_children(node, state);
            let result = result.trim_end_matches('\n');

            state.prefix = old_prefix;
            state.second_prefix = old_second;
            state.prefix = state.second_prefix.clone();
            state.suppress_item_break = false;
            format!("{alert_header}{result}\n")
        }

        // Catch-all for anything else
        _ => {
            drop(ast);
            render_children(node, state)
        }
    }
}

fn render_children<'a>(node: &'a AstNode<'a>, state: &mut RenderState<'_>) -> String {
    let mut result = String::new();
    for child in node.children() {
        result.push_str(&render_node(child, state));
    }
    result
}

fn render_list_item<'a>(node: &'a AstNode<'a>, state: &mut RenderState<'_>) -> String {
    let mut result = String::new();

    if !state.current_list_tight {
        if state.suppress_item_break {
            state.suppress_item_break = false;
        } else {
            result.push_str(&format!("{}\n", state.second_prefix.trim_end()));
        }
    }

    result.push_str(&render_children(node, state));
    result
}

fn get_list_item_prefix(list: &NodeList, index: usize) -> (String, String) {
    match list.list_type {
        ListType::Ordered => {
            let num = list.start + index;
            let delim = match list.delimiter {
                ListDelimType::Paren => ")",
                _ => ".",
            };
            let prefix = format!("{num}{delim} ");
            let indent = " ".repeat(prefix.len());
            (prefix, indent)
        }
        ListType::Bullet => {
            let bullet = char::from(list.bullet_char);
            let prefix = format!("{bullet} ");
            (prefix, "  ".to_string())
        }
    }
}

fn can_be_tight<'a>(list_node: &'a AstNode<'a>) -> bool {
    for item in list_node.children() {
        let item_ast = item.data.borrow();
        if matches!(&item_ast.value, NodeValue::Item(_) | NodeValue::TaskItem(_)) {
            drop(item_ast);
            let child_count = item.children().count();
            if child_count > 1 {
                return false;
            }
        }
    }
    true
}

fn render_table<'a>(node: &'a AstNode<'a>, state: &mut RenderState<'_>) -> String {
    let ast = node.data.borrow();
    let alignments = match &ast.value {
        NodeValue::Table(table) => table.alignments.clone(),
        _ => vec![],
    };
    drop(ast);

    let mut lines: Vec<String> = Vec::new();
    let mut is_first = true;

    for child in node.children() {
        let child_ast = child.data.borrow();
        if matches!(&child_ast.value, NodeValue::TableRow(_)) {
            drop(child_ast);
            lines.push(render_table_row(child, state));

            if is_first {
                let mut delimiters: Vec<&str> = Vec::new();
                for align in &alignments {
                    let delim = match align {
                        TableAlignment::Center => ":---:",
                        TableAlignment::Left => ":---",
                        TableAlignment::Right => "---:",
                        _ => "---",
                    };
                    delimiters.push(delim);
                }
                lines.push(format!("| {} |\n", delimiters.join(" | ")));
                is_first = false;
            }
        }
    }

    lines.join("")
}

fn render_table_row<'a>(node: &'a AstNode<'a>, state: &mut RenderState<'_>) -> String {
    let mut cells: Vec<String> = Vec::new();
    for child in node.children() {
        let child_ast = child.data.borrow();
        if matches!(&child_ast.value, NodeValue::TableCell) {
            drop(child_ast);
            let content = render_children(child, state).replace('|', "\\|");
            cells.push(content);
        }
    }
    format!("| {} |\n", cells.join(" | "))
}

/// Get standard comrak parse options for Flowmark.
pub fn flowmark_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.alerts = true;
    options.parse.smart = false;
    options
}

/// Render a comrak AST to normalized Markdown.
pub fn render_markdown<'a>(root: &'a AstNode<'a>, line_wrapper: &LineWrapper, list_spacing: ListSpacing) -> String {
    let mut state = RenderState::new(line_wrapper, list_spacing);
    render_node(root, &mut state)
}

/// Parse and render Markdown using comrak with Flowmark conventions.
pub fn flowmark_markdown(text: &str, line_wrapper: &LineWrapper, list_spacing: ListSpacing) -> String {
    let arena = Arena::new();
    let options = flowmark_options();
    let root = parse_document(&arena, text, &options);

    render_markdown(root, line_wrapper, list_spacing)
}
