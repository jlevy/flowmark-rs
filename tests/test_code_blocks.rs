use flowmark::linewrapping::markdown_filling::fill_markdown;
use flowmark::formats::flowmark_markdown::ListSpacing;
use flowmark::linewrapping::text_filling::DEFAULT_WRAP_WIDTH;

fn fm(text: &str) -> String {
    fill_markdown(text, true, DEFAULT_WRAP_WIDTH, false, false, false, None, ListSpacing::default())
}

#[test]
fn test_simple_fenced_code_block() {
    let input = "```python\nprint('hello')\n```";
    let expected = "```python\nprint('hello')\n```\n";
    assert_eq!(fm(input), expected);
}

#[test]
fn test_four_backtick_fence_preserved() {
    let input = "````value {% process=false %}\nUse {% callout %} for emphasis.\n````";
    let expected = "````value {% process=false %}\nUse {% callout %} for emphasis.\n````\n";
    assert_eq!(fm(input), expected);
}

#[test]
fn test_nested_code_blocks() {
    let input = "\
````markdown
This is a code block with nested markdown:

```python
print('hello')
```
````";
    let expected = "\
````markdown
This is a code block with nested markdown:

```python
print('hello')
```
````
";
    assert_eq!(fm(input), expected);
}

#[test]
fn test_deeply_nested_code_blocks() {
    let input = "\
`````markdown
Here's an example with 4-backtick code block:

````python
print('hello')
````
`````";
    let expected = "\
`````markdown
Here's an example with 4-backtick code block:

````python
print('hello')
````
`````
";
    assert_eq!(fm(input), expected);
}

#[test]
fn test_code_block_with_inline_backticks() {
    let input = "```python\nx = \"`backtick`\"\ny = \"``double``\"\n```";
    let expected = "```python\nx = \"`backtick`\"\ny = \"``double``\"\n```\n";
    assert_eq!(fm(input), expected);
}

#[test]
fn test_tilde_fence_stays_tilde() {
    let input = "~~~python\nprint('hello')\n~~~";
    let expected = "~~~python\nprint('hello')\n~~~\n";
    assert_eq!(fm(input), expected);
}

#[test]
fn test_tilde_fence_with_backticks_in_content() {
    let input = "\
~~~markdown
Here's some code:

```python
print('hello')
```
~~~";
    let expected = "\
~~~markdown
Here's some code:

```python
print('hello')
```
~~~
";
    assert_eq!(fm(input), expected);
}

#[test]
fn test_empty_lines_in_code_block_no_trailing_whitespace() {
    let input = "```python\nline1\n\nline2\n```";
    let expected = "```python\nline1\n\nline2\n```\n";
    let result = fm(input);
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
    // Code blocks within list items don't get blank line before them
    let expected = "\
- Example:
  ```python
  def foo():
      pass

  def bar():
      pass
  ```
";
    let result = fm(input);
    assert_eq!(result, expected);
    // Verify the empty line between functions has no trailing whitespace
    let lines: Vec<&str> = result.split('\n').collect();
    let empty_line_idx = lines.iter().position(|&l| l.is_empty() && lines.get(lines.iter().position(|&x| x == l).unwrap().wrapping_sub(1)).map_or(false, |prev| prev.ends_with("pass"))).unwrap_or(0);
    if empty_line_idx > 0 {
        assert_eq!(lines[empty_line_idx], "");
    }
}

#[test]
fn test_empty_lines_in_quoted_code_block_no_trailing_whitespace() {
    let input = "> ```python\n> line1\n>\n> line2\n> ```";
    let expected = "> ```python\n> line1\n>\n> line2\n> ```\n";
    assert_eq!(fm(input), expected);
}

#[test]
fn test_multiple_empty_lines_in_code_block() {
    let input = "```python\nline1\n\n\nline2\n```";
    let expected = "```python\nline1\n\n\nline2\n```\n";
    assert_eq!(fm(input), expected);
}
