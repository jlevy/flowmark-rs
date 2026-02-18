#![allow(clippy::unwrap_used)]

use flowmark::config::ListSpacing;
use flowmark::fill_markdown;

fn fmt(input: &str) -> String {
    fill_markdown(input, true, 88, true, false, false, false, None, ListSpacing::Preserve)
}

#[test]
fn test_simple_fenced_code_block() {
    let input = "```python\nprint('hello')\n```";
    let expected = "```python\nprint('hello')\n```\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn test_four_backtick_fence_preserved() {
    let input = "````value {% process=false %}\nUse {% callout %} for emphasis.\n````";
    let expected = "````value {% process=false %}\nUse {% callout %} for emphasis.\n````\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn test_nested_code_blocks() {
    let input = "````markdown\nThis is a code block with nested markdown:\n\n```python\nprint('hello')\n```\n````";
    let expected = "````markdown\nThis is a code block with nested markdown:\n\n```python\nprint('hello')\n```\n````\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn test_deeply_nested_code_blocks() {
    let input = "`````markdown\nHere's an example with 4-backtick code block:\n\n````python\nprint('hello')\n````\n`````";
    let expected = "`````markdown\nHere's an example with 4-backtick code block:\n\n````python\nprint('hello')\n````\n`````\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn test_code_block_with_inline_backticks() {
    let input = "```python\nx = \"`backtick`\"\ny = \"``double``\"\n```";
    let expected = "```python\nx = \"`backtick`\"\ny = \"``double``\"\n```\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn test_tilde_fence_stays_tilde() {
    let input = "~~~python\nprint('hello')\n~~~";
    let expected = "~~~python\nprint('hello')\n~~~\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn test_tilde_fence_with_backticks_in_content() {
    let input = "~~~markdown\nHere's some code:\n\n```python\nprint('hello')\n```\n~~~";
    let expected = "~~~markdown\nHere's some code:\n\n```python\nprint('hello')\n```\n~~~\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn test_empty_lines_in_code_block_no_trailing_whitespace() {
    let input = "```python\nline1\n\nline2\n```";
    let expected = "```python\nline1\n\nline2\n```\n";
    let result = fmt(input);
    assert_eq!(result, expected);
    // Verify the empty line has no trailing whitespace
    let lines: Vec<&str> = result.split('\n').collect();
    assert_eq!(lines[2], "");
}

#[test]
fn test_empty_lines_in_nested_code_block_no_trailing_whitespace() {
    let input = "\
- Example:

  ```python
  def foo():
      pass

  def bar():
      pass
  ```";

    let expected = "\
- Example:

  ```python
  def foo():
      pass

  def bar():
      pass
  ```
";

    let result = fmt(input);
    assert_eq!(result, expected);
    // Verify the empty line between functions has no trailing whitespace
    let lines: Vec<&str> = result.split('\n').collect();
    let empty_idx = lines
        .iter()
        .position(|&l| {
            l.is_empty()
                && lines
                    .get(lines.iter().position(|&x| x == l).unwrap().wrapping_sub(1))
                    .is_some_and(|x| x.ends_with("pass"))
        })
        .unwrap_or(0);
    if empty_idx > 0 {
        assert_eq!(lines[empty_idx], "");
    }
}

#[test]
fn test_empty_lines_in_quoted_code_block_no_trailing_whitespace() {
    let input = "> ```python\n> line1\n>\n> line2\n> ```";
    let expected = "> ```python\n> line1\n>\n> line2\n> ```\n";
    let result = fmt(input);
    assert_eq!(result, expected);
}

#[test]
fn test_multiple_empty_lines_in_code_block() {
    let input = "```python\nline1\n\n\nline2\n```";
    let expected = "```python\nline1\n\n\nline2\n```\n";
    assert_eq!(fmt(input), expected);
}

// ===== Tests ported from Python test_fenced_code_blocks.py =====

#[test]
fn test_minimum_backticks_computed_from_content() {
    // When content contains triple backticks, the fence must use 4+ backticks
    // This is tested indirectly via the nested code block tests above,
    // but we verify explicitly that nested backticks produce correct output
    let input = "````markdown\nShow this:\n\n```python\nprint('hello')\n```\n````";
    let result = fmt(input);
    // The outer fence should use 4 backticks to contain the inner triple backticks
    assert!(result.starts_with("````"), "Outer fence should use 4+ backticks");
    assert!(result.contains("```python"), "Inner fence should be preserved");
}
