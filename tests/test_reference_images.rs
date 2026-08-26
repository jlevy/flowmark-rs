//! Reference-style image rendering tests.
//!
//! Ported from Python: `test_reference_images.py`. Reference images
//! (`![alt][label]`, `![alt][]`, `![alt]`) always render as the inline form
//! `![alt](url{title})` regardless of source syntax. `flowmark_markdown()` uses the
//! default semantic wrapper, so the Rust analog formats with `semantic = true`.

use flowmark::config::ListSpacing;
use flowmark::fill_markdown;

/// Analog of Python's `md = flowmark_markdown(); md(src)`.
fn md(src: &str) -> String {
    fill_markdown(src, true, 88, true, false, false, false, None, ListSpacing::Preserve)
}

// --- Every syntactic form inlines to ![alt](url). ---

#[test]
fn test_full_ref_image_inlined() {
    let src = "![alt][img]\n\n[img]: https://example.com/img.png\n";
    let expected = "![alt](https://example.com/img.png)\n\n[img]: https://example.com/img.png\n";
    assert_eq!(md(src), expected);
}

#[test]
fn test_collapsed_ref_image_inlined() {
    let src = "![alt][]\n\n[alt]: https://example.com/img.png\n";
    let expected = "![alt](https://example.com/img.png)\n\n[alt]: https://example.com/img.png\n";
    assert_eq!(md(src), expected);
}

#[test]
fn test_shortcut_ref_image_inlined() {
    let src = "![alt]\n\n[alt]: https://example.com/img.png\n";
    let expected = "![alt](https://example.com/img.png)\n\n[alt]: https://example.com/img.png\n";
    assert_eq!(md(src), expected);
}

#[test]
fn test_full_ref_image_with_title() {
    let src = "![alt][img]\n\n[img]: https://example.com/img.png \"My title\"\n";
    let expected = "![alt](https://example.com/img.png \"My title\")\n\n[img]: https://example.com/img.png \"My title\"\n";
    assert_eq!(md(src), expected);
}

#[test]
fn test_full_ref_image_label_with_spaces() {
    let src = "![Logo][company logo]\n\n[company logo]: https://example.com/logo.png\n";
    let expected =
        "![Logo](https://example.com/logo.png)\n\n[company logo]: https://example.com/logo.png\n";
    assert_eq!(md(src), expected);
}

#[test]
fn test_full_ref_image_label_case_insensitive() {
    let src = "![alt][IMG]\n\n[Img]: https://example.com/img.png\n";
    let expected = "![alt](https://example.com/img.png)\n\n[img]: https://example.com/img.png\n";
    assert_eq!(md(src), expected);
}

#[test]
fn test_full_ref_image_empty_alt() {
    let src = "![][img]\n\n[img]: https://example.com/img.png\n";
    let expected = "![](https://example.com/img.png)\n\n[img]: https://example.com/img.png\n";
    assert_eq!(md(src), expected);
}

// --- Badge pattern: image nested inside a reference link. ---

#[test]
fn test_badge_full_ref_image_in_full_ref_link() {
    let src = "[![alt][img]][url]\n\n[img]: https://example.com/img.png\n[url]: https://example.com/page\n";
    let expected = "[![alt](https://example.com/img.png)][url]\n\n[img]: https://example.com/img.png\n[url]: https://example.com/page\n";
    assert_eq!(md(src), expected);
}

#[test]
fn test_badge_collapsed_ref_image_in_full_ref_link() {
    let src =
        "[![alt][]][url]\n\n[alt]: https://example.com/img.png\n[url]: https://example.com/page\n";
    let expected = "[![alt](https://example.com/img.png)][url]\n\n[alt]: https://example.com/img.png\n[url]: https://example.com/page\n";
    assert_eq!(md(src), expected);
}

#[test]
fn test_badge_shortcut_ref_image_in_full_ref_link() {
    let src =
        "[![alt]][url]\n\n[alt]: https://example.com/img.png\n[url]: https://example.com/page\n";
    let expected = "[![alt](https://example.com/img.png)][url]\n\n[alt]: https://example.com/img.png\n[url]: https://example.com/page\n";
    assert_eq!(md(src), expected);
}

#[test]
fn test_badge_inline_image_in_full_ref_link() {
    let src = "[![alt](https://example.com/img.png)][url]\n\n[url]: https://example.com/page\n";
    let expected =
        "[![alt](https://example.com/img.png)][url]\n\n[url]: https://example.com/page\n";
    assert_eq!(md(src), expected);
}

// --- Idempotence. ---

#[test]
fn test_full_ref_image_idempotent() {
    let once = md("![alt][img]\n\n[img]: https://example.com/img.png\n");
    assert_eq!(md(&once), once);
}

#[test]
fn test_badge_full_ref_image_in_full_ref_link_idempotent() {
    let once = md(
        "[![alt][img]][url]\n\n[img]: https://example.com/img.png\n[url]: https://example.com/page\n",
    );
    assert_eq!(md(&once), once);
}
