use flowmark::config::ListSpacing;
use flowmark::fill_markdown;
use flowmark::formatter::filling::{
    get_fill_perf_stats, reset_fill_perf_stats, set_fill_perf_stats_enabled,
};

static ORIGINAL_DOC: &str = "\
# This is a header

This is sentence one. This is sentence two.
This is sentence three.
This is sentence four. This is sentence 5. This is sentence six.
Seven. Eight. Nine. Ten.
A [link](https://example.com). Some *emphasis* and **strong emphasis** and `code`.
And a     super-super-super-super-super-super-super-hyphenated veeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeery long word.
This is a sentence with many words and words and words and words and words and words and words and words.
And another with words and
words and words split across a line.

A second paragraph.


- This is a list item
- This is another list item
    - A sub item
        - A sub sub item
- This is a third list item with many words and words and words and words and words and words and words and words

    - A sub item
    - Another sub item


    - Another sub item (after a line break)

- This is a nice [Markdown auto-formatter](https://github.com/jlevy/kmd/blob/main/kmd/text_formatting/markdown_normalization.py),
  so text documents are saved in a normalized form that can be diffed consistently.

A third paragraph.

## Sub-heading

1. This is a numbered list item
2. This is another numbered list item

<!--window-br-->

<!--window-br--> Words and words and words and words and words and <span data-foo=\"bar\">some HTML</span> and words and words and words and words and words and words.

<span data-foo=\"bar\">Inline HTML.</span> And some following words and words and words and words and words and words.

<h1 data-foo=\"bar\">Block HTML.</h1> And some following words.

<div class=\"foo\">
Some more HTML. Words and words and words and words and    words and <span data-foo=\"bar\">more HTML</span> and words and words and words and words and words and words.</div>

> This is a quote block. With a couple sentences. Note we have a `>` on this line.
>
> - Quotes can also contain lists.
> - With items. Like this. And these items may have long sentences in them.

```python
def hello_world():
    print(\"Hello, World!\")

# End of code
```


```
more code
```


Indented code:

    more code here

    and more

- **Intelligent:** Kmd understands itself. It reads its own code and docs and gives you assistance!


<p style=\"max-width: 450px;\">
\"*Simple should be simple.
Complex should be possible.*\" —Alan Kay
</p>

### Building

1. Lorem ipsum dolor sit amet, consectetur adipiscing elit. [Fork](https://github.com/jlevy/kmd/fork) this repo
   (having your own fork
   will make it
   easier to contribute actions, add models, etc.).

2. [Check out](https://docs.github.com/en/repositories/creating-and-managing-repositories/cloning-a-repository)
   the code. Lorem [another link](https://docs.github.com/en/repositories/creating-and-managing-repositories/cloning-a-repository).

3. Install the package dependencies:

   ```shell
   poetry install
   ```
";

static EXPECTED_DOC: &str = "\
# This is a header

This is sentence one.
This is sentence two.
This is sentence three.
This is sentence four.
This is sentence 5. This is sentence six.
Seven. Eight. Nine. Ten.
A [link](https://example.com).
Some *emphasis* and **strong emphasis** and `code`. And a
super-super-super-super-super-super-super-hyphenated
veeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeery
long word.
This is a sentence with many words and words and words and words and words and
words and words and words.
And another with words and words and words split across a line.

A second paragraph.

- This is a list item

- This is another list item

  - A sub item

    - A sub sub item

- This is a third list item with many words and words and words and words and words and
  words and words and words

  - A sub item

  - Another sub item

  - Another sub item (after a line break)

- This is a nice
  [Markdown auto-formatter](https://github.com/jlevy/kmd/blob/main/kmd/text_formatting/markdown_normalization.py),
  so text documents are saved in a normalized form that can be diffed consistently.

A third paragraph.

## Sub-heading

1. This is a numbered list item

2. This is another numbered list item

<!--window-br-->

<!--window-br--> Words and words and words and words and words and
<span data-foo=\"bar\">some HTML</span> and words and words and words and words and words
and words.

<span data-foo=\"bar\">Inline HTML.</span> And some following words and words and words
and words and words and words.

<h1 data-foo=\"bar\">Block HTML.</h1> And some following words.

<div class=\"foo\"> Some more HTML. Words and words and words and words and words and
<span data-foo=\"bar\">more HTML</span> and words and words and words and words and words
and words.</div>

> This is a quote block.
> With a couple sentences.
> Note we have a `>` on this line.
> \n\
> - Quotes can also contain lists.
>
> - With items. Like this.
>   And these items may have long sentences in them.

```python
def hello_world():
    print(\"Hello, World!\")

# End of code
```

```
more code
```

Indented code:

```
more code here

and more
```

- **Intelligent:** Kmd understands itself.
  It reads its own code and docs and gives you assistance!

<p style=\"max-width: 450px;\"> \"*Simple should be simple.
Complex should be possible.*\" —Alan Kay </p>

### Building

1. Lorem ipsum dolor sit amet, consectetur adipiscing elit.
   [Fork](https://github.com/jlevy/kmd/fork) this repo (having your own fork will make
   it easier to contribute actions, add models, etc.).

2. [Check out](https://docs.github.com/en/repositories/creating-and-managing-repositories/cloning-a-repository)
   the code. Lorem
   [another link](https://docs.github.com/en/repositories/creating-and-managing-repositories/cloning-a-repository).

3. Install the package dependencies:

   ```shell
   poetry install
   ```
";

#[test]
fn test_normalize_markdown() {
    let normalized_doc =
        fill_markdown(ORIGINAL_DOC, true, 88, true, false, false, false, None, ListSpacing::Loose);

    if normalized_doc != EXPECTED_DOC {
        // Print a diff to help debug
        let expected_lines: Vec<&str> = EXPECTED_DOC.lines().collect();
        let actual_lines: Vec<&str> = normalized_doc.lines().collect();
        for (i, (exp, act)) in expected_lines.iter().zip(actual_lines.iter()).enumerate() {
            if exp != act {
                eprintln!("First difference at line {}:", i + 1);
                eprintln!("  Expected: {exp:?}");
                eprintln!("  Actual:   {act:?}");
                break;
            }
        }
        if expected_lines.len() != actual_lines.len() {
            eprintln!(
                "Line count differs: expected {}, got {}",
                expected_lines.len(),
                actual_lines.len()
            );
        }
    }

    assert_eq!(normalized_doc, EXPECTED_DOC);
}

#[test]
fn test_multi_paragraph_list_items() {
    let input_doc = "\
- **`make_parent_dirs(path: str | Path, mode: int = 0o777) -> Path`**

  Ensures that the parent directories for a file exist, creating them if necessary.
- **`rmtree_or_file(path: str | Path, ignore_errors: bool = False)`**

  Removes the target even if it's a file, directory, or symlink.";

    let expected_doc = "\
- **`make_parent_dirs(path: str | Path, mode: int = 0o777) -> Path`**

  Ensures that the parent directories for a file exist, creating them if necessary.

- **`rmtree_or_file(path: str | Path, ignore_errors: bool = False)`**

  Removes the target even if it's a file, directory, or symlink.
";

    let normalized_doc =
        fill_markdown(input_doc, true, 88, true, false, false, false, None, ListSpacing::Preserve);

    assert_eq!(normalized_doc, expected_doc);
}

#[test]
fn test_fill_perf_stats_records_stage_times_when_enabled() {
    reset_fill_perf_stats();
    set_fill_perf_stats_enabled(true);

    let _ = fill_markdown(
        "# Perf\n\nSome content.\n",
        true,
        88,
        true,
        true,
        true,
        true,
        None,
        ListSpacing::Preserve,
    );

    set_fill_perf_stats_enabled(false);
    let stats = get_fill_perf_stats();
    assert!(stats.files >= 1, "expected at least one recorded file");
    assert!(stats.total_ns() > 0, "expected non-zero total measured time");
    assert!(stats.parse_ns > 0, "expected non-zero parse stage");
    assert!(stats.render_ns > 0, "expected non-zero render stage");
    reset_fill_perf_stats();
}

#[test]
fn test_reference_image_roundtrips() {
    // COMRAK-WORKAROUND1 covers reference links via PUA-encoded URLs that
    // are decoded back to `[text][label]` during rendering. Reference
    // images need the same decoding so the encoded URL does not survive
    // into the output.
    let input = "\
![alt][img]

[img]: https://example.com/img.png
";
    let output =
        fill_markdown(input, true, 88, false, false, false, false, None, ListSpacing::Preserve);
    assert!(
        !output.contains('\u{F000}') && !output.contains('\u{F001}'),
        "output should not contain PUA sentinels: {output:?}"
    );
    assert!(output.contains("![alt][img]"), "output should preserve reference image: {output:?}");
}

#[test]
fn test_reference_image_inside_reference_link_roundtrips() {
    // Badge pattern: a reference image nested inside a reference link.
    // Both the inner image and the outer link should preserve their
    // reference form, and the encoded URL should not survive into the
    // output.
    let input = "\
[![alt][img]][url]

[img]: https://example.com/img.png
[url]: https://example.com/page
";
    let output =
        fill_markdown(input, true, 88, false, false, false, false, None, ListSpacing::Preserve);
    assert!(
        !output.contains('\u{F000}') && !output.contains('\u{F001}'),
        "output should not contain PUA sentinels: {output:?}"
    );
    assert!(output.contains("![alt][img]"), "output should preserve reference image: {output:?}");
}
