//! Document cleanup transforms.
//!
//! Ported from Python: flowmark/transforms/doc_cleanups.py and doc_transforms.py
//!
//! In Python, these operate on the Marko AST. In Rust, we operate on comrak's AST.
//! The approach is different but the effect is the same.

use comrak::nodes::{AstNode, NodeValue};

/// Remove bold formatting from headings that contain only bold text.
///
/// Many documents have headings like `# **Title**` where the bold is redundant
/// since headings are already visually prominent.
pub fn unbold_headings<'a>(root: &'a AstNode<'a>) {
    // Collect heading nodes first to avoid modifying tree during iteration
    let headings: Vec<_> = root
        .descendants()
        .filter(|node| matches!(node.data.borrow().value, NodeValue::Heading(_)))
        .collect();

    for node in headings {
        let children: Vec<_> = node.children().collect();
        if children.len() != 1 {
            continue;
        }

        let child = children[0];
        let child_value = child.data.borrow().value.clone();

        match child_value {
            // # **Bold Heading** → # Bold Heading
            NodeValue::Strong => {
                let grandchildren: Vec<_> = child.children().collect();
                for gc in &grandchildren {
                    gc.detach();
                }
                child.detach();
                for gc in grandchildren {
                    node.append(gc);
                }
            }
            // # ***Bold Italic*** → # *Italic*
            // comrak parses as Emph(Strong(text))
            NodeValue::Emph => {
                let emph_children: Vec<_> = child.children().collect();
                if emph_children.len() == 1
                    && matches!(emph_children[0].data.borrow().value, NodeValue::Strong)
                {
                    let strong = emph_children[0];
                    // Move strong's children to the emph, removing the strong wrapper
                    let strong_children: Vec<_> = strong.children().collect();
                    for gc in &strong_children {
                        gc.detach();
                    }
                    strong.detach();
                    for gc in strong_children {
                        child.append(gc);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Apply all document cleanups.
pub fn doc_cleanups<'a>(root: &'a AstNode<'a>) {
    unbold_headings(root);
}
