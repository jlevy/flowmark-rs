use flowmark::fill_markdown;
use flowmark::config::ListSpacing;

// Helper to call fill_markdown with default options
fn fmt(input: &str, list_spacing: ListSpacing) -> String {
    fill_markdown(input, true, 88, false, false, false, false, None, list_spacing)
}

fn fmt_semantic(input: &str, list_spacing: ListSpacing) -> String {
    fill_markdown(input, true, 88, true, false, false, false, None, list_spacing)
}

// --- Tests for preserve mode (default) ---

#[test]
fn test_tight_list_preserved() {
    let input = "- one\n- two\n- three\n";
    assert_eq!(fmt(input, ListSpacing::Preserve), "- one\n- two\n- three\n");
}

#[test]
fn test_loose_list_preserved() {
    let input = "- one\n\n- two\n\n- three\n";
    assert_eq!(fmt(input, ListSpacing::Preserve), "- one\n\n- two\n\n- three\n");
}

#[test]
fn test_preserve_is_default() {
    let input_tight = "- one\n- two\n- three\n";
    let input_loose = "- one\n\n- two\n\n- three\n";

    assert_eq!(fmt(input_tight, ListSpacing::Preserve), "- one\n- two\n- three\n");
    assert_eq!(fmt(input_loose, ListSpacing::Preserve), "- one\n\n- two\n\n- three\n");
}

#[test]
fn test_numbered_list_preserve() {
    let input_tight = "1. one\n2. two\n3. three\n";
    let input_loose = "1. one\n\n2. two\n\n3. three\n";

    assert_eq!(fmt(input_tight, ListSpacing::Preserve), "1. one\n2. two\n3. three\n");
    assert_eq!(fmt(input_loose, ListSpacing::Preserve), "1. one\n\n2. two\n\n3. three\n");
}

// --- Tests for loose mode ---

#[test]
fn test_tight_list_to_loose() {
    let input = "- one\n- two\n- three\n";
    assert_eq!(fmt(input, ListSpacing::Loose), "- one\n\n- two\n\n- three\n");
}

#[test]
fn test_loose_list_stays_loose() {
    let input = "- one\n\n- two\n\n- three\n";
    assert_eq!(fmt(input, ListSpacing::Loose), "- one\n\n- two\n\n- three\n");
}

#[test]
fn test_numbered_list_to_loose() {
    let input = "1. one\n2. two\n3. three\n";
    assert_eq!(fmt(input, ListSpacing::Loose), "1. one\n\n2. two\n\n3. three\n");
}

// --- Tests for tight mode ---

#[test]
fn test_loose_list_to_tight() {
    let input = "- one\n\n- two\n\n- three\n";
    assert_eq!(fmt(input, ListSpacing::Tight), "- one\n- two\n- three\n");
}

#[test]
fn test_tight_list_stays_tight() {
    let input = "- one\n- two\n- three\n";
    assert_eq!(fmt(input, ListSpacing::Tight), "- one\n- two\n- three\n");
}

#[test]
fn test_multi_para_stays_loose_in_tight_mode() {
    let input = "- para1\n\n  para2\n- item2\n";
    let output = fmt(input, ListSpacing::Tight);
    assert!(output.contains("\n\n"));
}

// --- Tests for nested lists ---

#[test]
fn test_nested_lists_independent_preserve() {
    let input = "- outer tight\n  - inner tight\n  - inner tight\n- outer tight\n";
    let expected = "- outer tight\n  - inner tight\n  - inner tight\n- outer tight\n";
    assert_eq!(fmt(input, ListSpacing::Preserve), expected);
}

#[test]
fn test_nested_lists_loose_outer_tight_inner() {
    let input = "- outer loose\n\n  - inner tight\n  - inner tight\n\n- outer loose\n";
    let expected = "- outer loose\n\n  - inner tight\n  - inner tight\n\n- outer loose\n";
    assert_eq!(fmt(input, ListSpacing::Preserve), expected);
}

// --- Tests for complex content ---

#[test]
fn test_list_items_with_code_blocks_preserve() {
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

    let normalized = fmt_semantic(input, ListSpacing::Preserve);
    assert_eq!(normalized, expected);
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

    let normalized = fmt_semantic(input, ListSpacing::Loose);
    assert_eq!(normalized, expected);
}

#[test]
fn test_list_items_with_quote_blocks() {
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

    let normalized = fmt_semantic(input, ListSpacing::Preserve);
    assert_eq!(normalized, expected);
}

// --- Tests for spacing normalization ---

#[test]
fn test_input_spacing_normalization_loose() {
    let input_tight = "- First item\n- Second item\n- Third item\n";
    let input_loose = "- First item\n\n- Second item\n\n- Third item\n";
    let input_extra = "- First item\n\n\n- Second item\n\n\n- Third item\n";

    let expected = "- First item\n\n- Second item\n\n- Third item\n";

    assert_eq!(fmt(input_tight, ListSpacing::Loose), expected);
    assert_eq!(fmt(input_loose, ListSpacing::Loose), expected);
    assert_eq!(fmt(input_extra, ListSpacing::Loose), expected);
}

#[test]
fn test_input_spacing_normalization_tight() {
    let input_tight = "- First item\n- Second item\n- Third item\n";
    let input_loose = "- First item\n\n- Second item\n\n- Third item\n";
    let input_extra = "- First item\n\n\n- Second item\n\n\n- Third item\n";

    let expected = "- First item\n- Second item\n- Third item\n";

    assert_eq!(fmt(input_tight, ListSpacing::Tight), expected);
    assert_eq!(fmt(input_loose, ListSpacing::Tight), expected);
    assert_eq!(fmt(input_extra, ListSpacing::Tight), expected);
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

    assert_eq!(fmt_semantic(input, ListSpacing::Loose), expected);
}

#[test]
fn test_multi_paragraph_spacing_loose_mode() {
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

    assert_eq!(fmt_semantic(input, ListSpacing::Loose), expected);
}
