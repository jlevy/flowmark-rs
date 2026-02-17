//! Document tree transforms using comrak AST.
//!
//! Ported from Python: flowmark/transforms/doc_transforms.py

use comrak::nodes::{AstNode, NodeValue};

/// A segment of inline content with optional reference to its source Text node.
struct InlineSegment<'a> {
    text: String,
    /// Some(node) for mutable Text nodes, None for immutable segments (code, HTML, etc.)
    node: Option<&'a AstNode<'a>>,
}

/// Collect inline content segments from a node and its descendants.
fn collect_inline_segments<'a>(node: &'a AstNode<'a>) -> Vec<InlineSegment<'a>> {
    let ast = node.data.borrow();
    match &ast.value {
        NodeValue::Text(text) => {
            vec![InlineSegment {
                text: text.to_string(),
                node: Some(node),
            }]
        }
        NodeValue::SoftBreak => {
            vec![InlineSegment {
                text: "\n".to_string(),
                node: None,
            }]
        }
        NodeValue::LineBreak => {
            vec![InlineSegment {
                text: "\n".to_string(),
                node: None,
            }]
        }
        NodeValue::Code(code) => {
            vec![InlineSegment {
                text: code.literal.clone(),
                node: None,
            }]
        }
        NodeValue::HtmlInline(html) => {
            vec![InlineSegment {
                text: html.clone(),
                node: None,
            }]
        }
        NodeValue::FootnoteReference(fref) => {
            vec![InlineSegment {
                text: fref.name.clone(),
                node: None,
            }]
        }
        // Container nodes: recurse into children
        NodeValue::Emph
        | NodeValue::Strong
        | NodeValue::Strikethrough
        | NodeValue::Link(_)
        | NodeValue::Image(_)
        | NodeValue::Escaped => {
            drop(ast);
            let mut segments = Vec::new();
            for child in node.children() {
                segments.extend(collect_inline_segments(child));
            }
            segments
        }
        _ => vec![],
    }
}

/// Apply a rewrite function to all text nodes in the document that are not in code.
pub fn rewrite_text_content<'a>(
    root: &'a AstNode<'a>,
    rewrite_func: &dyn Fn(&str) -> String,
) {
    for node in root.descendants() {
        let mut ast = node.data.borrow_mut();
        if matches!(
            &ast.value,
            NodeValue::CodeBlock(_) | NodeValue::Code(_)
        ) {
            continue;
        }

        let text_str = if let NodeValue::Text(text) = &ast.value {
            Some(text.to_string())
        } else {
            None
        };
        if let Some(text_str) = text_str {
            let rewritten = rewrite_func(&text_str);
            if rewritten != text_str {
                ast.value = NodeValue::Text(rewritten.into());
            }
        }
    }
}

/// Apply a length-preserving rewrite function across all inline elements within
/// each inline scope (Paragraph, Heading, TableCell).
///
/// Unlike `rewrite_text_content` which processes each text node independently,
/// this function concatenates all text within an inline scope and applies the
/// rewrite function to the composite text. This allows the rewrite function to
/// handle patterns (like quote pairs) that span across inline elements such as
/// code spans, emphasis, or links.
///
/// The rewrite function must be character-length-preserving (each input character
/// maps to exactly one output character).
pub fn rewrite_text_across_inlines<'a>(
    root: &'a AstNode<'a>,
    rewrite_func: &dyn Fn(&str) -> String,
) {
    for node in root.descendants() {
        let is_inline_scope = {
            let ast = node.data.borrow();
            matches!(
                &ast.value,
                NodeValue::Paragraph | NodeValue::Heading(_) | NodeValue::TableCell
            )
        };

        if !is_inline_scope {
            continue;
        }

        // Collect all inline segments from this scope
        let mut segments: Vec<InlineSegment<'a>> = Vec::new();
        for child in node.children() {
            segments.extend(collect_inline_segments(child));
        }

        if segments.is_empty() {
            continue;
        }

        // Build composite text from all segments
        let composite: String = segments.iter().map(|s| s.text.as_str()).collect();
        if composite.is_empty() {
            continue;
        }

        // Apply rewrite to the full composite text
        let converted = rewrite_func(&composite);

        // Character-level length must be preserved for correct mapping
        let composite_chars: Vec<char> = composite.chars().collect();
        let converted_chars: Vec<char> = converted.chars().collect();
        debug_assert_eq!(
            composite_chars.len(),
            converted_chars.len(),
            "rewrite function must be character-length-preserving"
        );

        if converted == composite {
            continue;
        }

        // Map changes back only to mutable (Text) segments using character positions
        let mut char_pos = 0;
        for segment in &segments {
            let segment_char_count = segment.text.chars().count();
            if let Some(text_node) = segment.node {
                let new_text: String = converted_chars[char_pos..char_pos + segment_char_count]
                    .iter()
                    .collect();
                if new_text != segment.text {
                    let mut ast = text_node.data.borrow_mut();
                    ast.value = NodeValue::Text(new_text.into());
                }
            }
            char_pos += segment_char_count;
        }
    }
}
