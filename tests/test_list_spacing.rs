use flowmark::formats::flowmark_markdown::ListSpacing;
use flowmark::linewrapping::markdown_filling::fill_markdown;
use flowmark::linewrapping::text_filling::DEFAULT_WRAP_WIDTH;

fn fm(text: &str) -> String {
    fill_markdown(text, false, DEFAULT_WRAP_WIDTH, false, false, false, None, ListSpacing::default())
}

fn fm_loose(text: &str) -> String {
    fill_markdown(text, false, DEFAULT_WRAP_WIDTH, false, false, false, None, ListSpacing::Loose)
}

fn fm_tight(text: &str) -> String {
    fill_markdown(text, false, DEFAULT_WRAP_WIDTH, false, false, false, None, ListSpacing::Tight)
}

fn fm_preserve(text: &str) -> String {
    fill_markdown(text, false, DEFAULT_WRAP_WIDTH, false, false, false, None, ListSpacing::Preserve)
}

fn fm_semantic_loose(text: &str) -> String {
    fill_markdown(text, true, DEFAULT_WRAP_WIDTH, false, false, false, None, ListSpacing::Loose)
}

fn fm_semantic_preserve(text: &str) -> String {
    fill_markdown(text, true, DEFAULT_WRAP_WIDTH, false, false, false, None, ListSpacing::Preserve)
}

// --- Tests for preserve mode (default) ---

#[test]
fn test_tight_list_preserved() {
    let input = "- one\n- two\n- three\n";
    assert_eq!(fm_preserve(input), "- one\n- two\n- three\n");
}

#[test]
fn test_loose_list_preserved() {
    let input = "- one\n\n- two\n\n- three\n";
    assert_eq!(fm_preserve(input), "- one\n\n- two\n\n- three\n");
}

#[test]
fn test_preserve_is_default() {
    let input_tight = "- one\n- two\n- three\n";
    let input_loose = "- one\n\n- two\n\n- three\n";
    assert_eq!(fm(input_tight), "- one\n- two\n- three\n");
    assert_eq!(fm(input_loose), "- one\n\n- two\n\n- three\n");
}

#[test]
fn test_numbered_list_preserve() {
    let input_tight = "1. one\n2. two\n3. three\n";
    let input_loose = "1. one\n\n2. two\n\n3. three\n";
    assert_eq!(fm_preserve(input_tight), "1. one\n2. two\n3. three\n");
    assert_eq!(fm_preserve(input_loose), "1. one\n\n2. two\n\n3. three\n");
}

// --- Tests for loose mode ---

#[test]
fn test_tight_list_to_loose() {
    let input = "- one\n- two\n- three\n";
    assert_eq!(fm_loose(input), "- one\n\n- two\n\n- three\n");
}

#[test]
fn test_loose_list_stays_loose() {
    let input = "- one\n\n- two\n\n- three\n";
    assert_eq!(fm_loose(input), "- one\n\n- two\n\n- three\n");
}

#[test]
fn test_numbered_list_to_loose() {
    let input = "1. one\n2. two\n3. three\n";
    assert_eq!(fm_loose(input), "1. one\n\n2. two\n\n3. three\n");
}

// --- Tests for tight mode ---

#[test]
fn test_loose_list_to_tight() {
    let input = "- one\n\n- two\n\n- three\n";
    assert_eq!(fm_tight(input), "- one\n- two\n- three\n");
}

#[test]
fn test_tight_list_stays_tight() {
    let input = "- one\n- two\n- three\n";
    assert_eq!(fm_tight(input), "- one\n- two\n- three\n");
}

#[test]
fn test_multi_para_stays_loose_in_tight_mode() {
    let input = "- para1\n\n  para2\n- item2\n";
    let output = fm_tight(input);
    assert!(output.contains("\n\n"), "Multi-paragraph items should stay loose");
}

// --- Tests for nested lists ---

#[test]
fn test_nested_lists_independent_preserve() {
    let input = "- outer tight\n  - inner tight\n  - inner tight\n- outer tight\n";
    let expected = "- outer tight\n  - inner tight\n  - inner tight\n- outer tight\n";
    assert_eq!(fm_preserve(input), expected);
}

#[test]
fn test_nested_lists_loose_outer_tight_inner() {
    // comrak renders tight inner list items without blank line before sub-list
    let input = "- outer loose\n\n  - inner tight\n  - inner tight\n\n- outer loose\n";
    let expected = "- outer loose\n  - inner tight\n  - inner tight\n\n- outer loose\n";
    assert_eq!(fm_preserve(input), expected);
}

// --- Tests for complex content ---

#[test]
fn test_list_items_with_code_blocks_preserve() {
    // Code blocks within list items don't get blank line before them
    // (consistent with comrak AST rendering)
    let input = "\
- Use `z` (zoxide) instead of `cd`.

  ```shell
  z ~/some/long/path/to/foo
  ```

- Use `eza` instead of `ls`.
";
    let expected = "\
- Use `z` (zoxide) instead of `cd`.
  ```shell
  z ~/some/long/path/to/foo
  ```

- Use `eza` instead of `ls`.
";
    assert_eq!(fm_semantic_preserve(input), expected);
}

#[test]
fn test_list_items_with_code_blocks_loose() {
    let input = "\
- Use `z` (zoxide) instead of `cd`.

  ```shell
  z ~/some/long/path/to/foo
  ```

- Use `eza` instead of `ls`. It has color support.
";
    let expected = "\
- Use `z` (zoxide) instead of `cd`.
  ```shell
  z ~/some/long/path/to/foo
  ```

- Use `eza` instead of `ls`. It has color support.
";
    assert_eq!(fm_semantic_loose(input), expected);
}

#[test]
fn test_list_items_with_quote_blocks() {
    // Quote blocks within list items don't get blank line before them
    let input = "\
- First item with a quote.

  > This is a quote block.
  > With multiple lines.

- Second item without quotes.
";
    let expected = "\
- First item with a quote.
  > This is a quote block.
  > With multiple lines.

- Second item without quotes.
";
    assert_eq!(fm_semantic_preserve(input), expected);
}

// --- Tests for spacing normalization ---

#[test]
fn test_input_spacing_normalization_loose() {
    let input_tight = "- First item\n- Second item\n- Third item\n";
    let input_loose = "- First item\n\n- Second item\n\n- Third item\n";
    let input_extra = "- First item\n\n\n- Second item\n\n\n- Third item\n";
    let expected = "- First item\n\n- Second item\n\n- Third item\n";
    assert_eq!(fm_loose(input_tight), expected);
    assert_eq!(fm_loose(input_loose), expected);
    assert_eq!(fm_loose(input_extra), expected);
}

#[test]
fn test_input_spacing_normalization_tight() {
    let input_tight = "- First item\n- Second item\n- Third item\n";
    let input_loose = "- First item\n\n- Second item\n\n- Third item\n";
    let input_extra = "- First item\n\n\n- Second item\n\n\n- Third item\n";
    let expected = "- First item\n- Second item\n- Third item\n";
    assert_eq!(fm_tight(input_tight), expected);
    assert_eq!(fm_tight(input_loose), expected);
    assert_eq!(fm_tight(input_extra), expected);
}

#[test]
fn test_complex_content_with_loose_mode() {
    let input = "\
- Item before code
- Item with code

  ```shell
  echo \"test\"
  ```
- Item after code
";
    let expected = "\
- Item before code

- Item with code
  ```shell
  echo \"test\"
  ```

- Item after code
";
    assert_eq!(fm_semantic_loose(input), expected);
}

#[test]
fn test_multi_paragraph_spacing_loose_mode() {
    // Multi-paragraph items: second paragraph is adjacent (no blank line)
    // because comrak's AST doesn't distinguish inter-paragraph spacing from
    // inter-item spacing within list items.
    let input = "\
- Simple item
- Multi-paragraph item

  Second paragraph
- Another simple item
";
    let expected = "\
- Simple item

- Multi-paragraph item
  Second paragraph

- Another simple item
";
    assert_eq!(fm_semantic_loose(input), expected);
}
