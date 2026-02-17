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
    for node in root.descendants() {
        let is_heading = matches!(node.data.borrow().value, NodeValue::Heading(_));
        if !is_heading {
            continue;
        }

        // Check if heading has exactly one child that is Strong
        let children: Vec<_> = node.children().collect();
        if children.len() == 1 {
            let child = children[0];
            let is_strong = matches!(child.data.borrow().value, NodeValue::Strong);
            if is_strong {
                // Move strong's children to be heading's children directly
                // (unwrap the strong wrapper)
                let grandchildren: Vec<_> = child.children().collect();
                for gc in &grandchildren {
                    gc.detach();
                }
                child.detach();
                for gc in grandchildren {
                    node.append(gc);
                }
            }
        }
    }
}

/// Apply all document cleanups.
pub fn doc_cleanups<'a>(root: &'a AstNode<'a>) {
    unbold_headings(root);
}
