//! Document cleanup transforms.
//!
//! Ported from Python: flowmark/transforms/doc_cleanups.py

use comrak::nodes::{AstNode, NodeValue};

/// Unwrap a node: move its children to its parent, then remove it.
fn unwrap_node<'a>(parent: &'a AstNode<'a>, to_remove: &'a AstNode<'a>) {
    let children: Vec<&AstNode<'a>> = to_remove.children().collect();
    for child in &children {
        child.detach();
    }
    to_remove.detach();
    for child in children {
        parent.append(child);
    }
}

/// Remove bold from headings where the entire text is bold.
pub fn unbold_headings<'a>(root: &'a AstNode<'a>) {
    // Collect heading nodes first to avoid modifying tree during iteration.
    let headings: Vec<&'a AstNode<'a>> = root
        .descendants()
        .filter(|node| {
            let ast = node.data.borrow();
            matches!(&ast.value, NodeValue::Heading(_))
        })
        .collect();

    for heading in headings {
        let children: Vec<&AstNode<'a>> = heading.children().collect();
        if children.len() != 1 {
            continue;
        }

        let child = children[0];
        let child_ast = child.data.borrow();

        if matches!(&child_ast.value, NodeValue::Strong) {
            // Case: Heading > Strong > ... → Heading > ...
            drop(child_ast);
            unwrap_node(heading, child);
        } else if matches!(&child_ast.value, NodeValue::Emph) {
            // Case: Heading > Emph > Strong > ... → Heading > Emph > ...
            drop(child_ast);
            let emph_children: Vec<&AstNode<'a>> = child.children().collect();
            if emph_children.len() == 1 {
                let gc = emph_children[0];
                let gc_ast = gc.data.borrow();
                if matches!(&gc_ast.value, NodeValue::Strong) {
                    drop(gc_ast);
                    unwrap_node(child, gc);
                }
            }
        }
    }
}

/// Apply safe cleanups to the document.
pub fn doc_cleanups<'a>(root: &'a AstNode<'a>) {
    unbold_headings(root);
}
