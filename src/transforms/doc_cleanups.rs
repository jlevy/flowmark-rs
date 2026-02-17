//! Document cleanup transforms.
//!
//! Ported from Python: flowmark/transforms/doc_cleanups.py

use comrak::nodes::{AstNode, NodeValue};

/// Remove bold from headings where the entire text is bold.
pub fn unbold_headings<'a>(root: &'a AstNode<'a>) {
    for node in root.descendants() {
        let ast = node.data.borrow();
        if !matches!(&ast.value, NodeValue::Heading(_)) {
            continue;
        }
        drop(ast);

        let children: Vec<&AstNode<'a>> = node.children().collect();
        if children.len() == 1 {
            let child_ast = children[0].data.borrow();
            if matches!(&child_ast.value, NodeValue::Strong) {
                drop(child_ast);
                let grandchildren: Vec<&AstNode<'a>> = children[0].children().collect();
                for gc in &grandchildren {
                    gc.detach();
                }
                children[0].detach();
                for gc in grandchildren {
                    node.append(gc);
                }
            }
        }
    }
}

/// Apply safe cleanups to the document.
pub fn doc_cleanups<'a>(root: &'a AstNode<'a>) {
    unbold_headings(root);
}
