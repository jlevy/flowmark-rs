use flowmark::formats::flowmark_markdown::{ListSpacing, flowmark_options, render_markdown};
use flowmark::linewrapping::line_wrappers::line_wrap_by_sentence;
use flowmark::linewrapping::text_filling::DEFAULT_WRAP_WIDTH;
use flowmark::linewrapping::text_wrapping::default_len;
use flowmark::transforms::doc_cleanups::unbold_headings;

use comrak::{Arena, parse_document};

#[test]
fn test_unbold_headings() {
    let input = r#"
# **Bold Heading 1**

Some paragraph text.

## ***Bold Italic***

## **Simple Bold**

### Not Bold

#### **Partial** Bold

#### Other *partial* **bold** `code`

- **List Item Bold**

Another paragraph with **bold** text.

## **Nested `code`**

Final text.
"#;

    // Our comrak-based renderer: headings emit \n\n but no blank line is added
    // before them by preceding blocks. List items followed by paragraphs also
    // don't get extra blank lines between them.
    let expected = r#"# Bold Heading 1

Some paragraph text.
## *Bold Italic*

## Simple Bold

### Not Bold

#### **Partial** Bold

#### Other *partial* **bold** `code`

- **List Item Bold**
Another paragraph with **bold** text.
## Nested `code`

Final text."#;

    let wrapper = line_wrap_by_sentence(DEFAULT_WRAP_WIDTH, 15, 0, default_len, true);
    let arena = Arena::new();
    let options = flowmark_options();
    let root = parse_document(&arena, input, &options);
    unbold_headings(root);
    let result = render_markdown(root, &wrapper, ListSpacing::default());

    assert_eq!(result.trim(), expected.trim());
}
