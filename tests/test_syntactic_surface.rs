//! Syntactic-surface parity tests.
//!
//! Methodology: for every inline AST node type the renderer handles, enumerate
//! the full matrix of syntactic forms — independent of whether the form was
//! observed in upstream tests. The expected outputs were captured by running
//! `flowmark@0.7.0` (Python, pinned parity baseline) directly on each input.
//!
//! Why this file exists: a senior review of PR #54/#57 (2026-05-28) found
//! three latent parity bugs (reference-image inlining, badge pattern,
//! ref-def case lowercasing) that the corpus sweep and fixture mirroring
//! never exercised because the upstream tests themselves had near-zero
//! reference-image coverage. Mirroring upstream tests faithfully inherits
//! upstream gaps. This file is the structural backstop: an explicit, port-
//! owned enumeration of every form per node type, so a gap is a missing row,
//! not a missing test.
//!
//! Maintenance: when adding or modifying a render branch in
//! `src/formatter/filling.rs`, add or update the matching row here. The
//! `docs/parity-coverage-matrix.md` index maps each `NodeValue` variant to
//! the tests below.
//!
//! Re-baselining against a new Python version: run the matching input
//! through `uvx flowmark@<new-version> -` and update the expected string.
#![allow(clippy::unwrap_used)]

use flowmark::config::ListSpacing;
use flowmark::fill_markdown;

fn fmt(input: &str) -> String {
    fill_markdown(input, true, 88, false, false, false, false, None, ListSpacing::Preserve)
}

// =============================================================================
// IMAGE node — every syntactic form.
// Python `render_image` always emits inline form `![alt](url{title})`,
// regardless of source syntax. Ref-def lines normalize label to lowercase.
// =============================================================================

#[test]
fn syntactic_image_inline_no_title() {
    let input = "![alt](https://example.com/img.png)\n";
    assert_eq!(fmt(input), "![alt](https://example.com/img.png)\n");
}

#[test]
fn syntactic_image_inline_with_double_quoted_title() {
    let input = "![alt](https://example.com/img.png \"A title\")\n";
    assert_eq!(fmt(input), "![alt](https://example.com/img.png \"A title\")\n");
}

#[test]
fn syntactic_image_inline_empty_alt() {
    let input = "![](https://example.com/img.png)\n";
    assert_eq!(fmt(input), "![](https://example.com/img.png)\n");
}

#[test]
fn syntactic_image_full_ref() {
    let input = "![alt][img]\n\n[img]: https://example.com/img.png\n";
    let expected = "![alt](https://example.com/img.png)\n\n[img]: https://example.com/img.png\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn syntactic_image_full_ref_with_title() {
    let input = "![alt][img]\n\n[img]: https://example.com/img.png \"My title\"\n";
    let expected = "![alt](https://example.com/img.png \"My title\")\n\n[img]: https://example.com/img.png \"My title\"\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn syntactic_image_full_ref_case_insensitive_label() {
    let input = "![alt][IMG]\n\n[Img]: https://example.com/img.png\n";
    let expected = "![alt](https://example.com/img.png)\n\n[img]: https://example.com/img.png\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn syntactic_image_collapsed_ref() {
    let input = "![alt][]\n\n[alt]: https://example.com/img.png\n";
    let expected = "![alt](https://example.com/img.png)\n\n[alt]: https://example.com/img.png\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn syntactic_image_shortcut_ref() {
    let input = "![alt]\n\n[alt]: https://example.com/img.png\n";
    let expected = "![alt](https://example.com/img.png)\n\n[alt]: https://example.com/img.png\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn syntactic_image_label_with_spaces_and_punctuation() {
    let input = "![Logo][company logo]\n\n[company logo]: https://example.com/logo.png\n";
    let expected =
        "![Logo](https://example.com/logo.png)\n\n[company logo]: https://example.com/logo.png\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn syntactic_image_missing_definition_pass_through() {
    // No matching definition: comrak parses it as text. Critical: must not
    // leak PUA / hex into output.
    let input = "Some ![alt][missing] text.\n";
    let out = fmt(input);
    assert!(
        !out.contains('\u{F000}') && !out.contains('\u{F001}') && !out.contains('\u{F002}'),
        "missing-def image must not leak PUA markers, got: {out}"
    );
}

// =============================================================================
// LINK node — every syntactic form (mirrors IMAGE coverage above).
// Python `render_link_*` emits reference form `[text][label]` (or `[text][]`
// when text == label, per issue #45). Inline form preserved.
// =============================================================================

#[test]
fn syntactic_link_inline_no_title() {
    let input = "[click](https://example.com)\n";
    assert_eq!(fmt(input), "[click](https://example.com)\n");
}

#[test]
fn syntactic_link_inline_with_title() {
    let input = "[click](https://example.com \"My title\")\n";
    assert_eq!(fmt(input), "[click](https://example.com \"My title\")\n");
}

#[test]
fn syntactic_link_autolink_angle_bracket() {
    // <url> autolink should round-trip with angle brackets.
    let input = "<https://example.com>\n";
    assert_eq!(fmt(input), "<https://example.com>\n");
}

#[test]
fn syntactic_link_full_ref_preserved() {
    let input = "[click here][link]\n\n[link]: https://example.com\n";
    let expected = "[click here][link]\n\n[link]: https://example.com\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn syntactic_link_full_ref_with_title_preserved() {
    let input = "[click][link]\n\n[link]: https://example.com \"A title\"\n";
    let expected = "[click][link]\n\n[link]: https://example.com \"A title\"\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn syntactic_link_collapsed_ref_preserved() {
    let input = "[link][]\n\n[link]: https://example.com\n";
    let expected = "[link][]\n\n[link]: https://example.com\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn syntactic_link_shortcut_ref_normalized_to_collapsed() {
    // Shortcut form normalizes to collapsed when text == label (issue #45).
    let input = "[link]\n\n[link]: https://example.com\n";
    let expected = "[link][]\n\n[link]: https://example.com\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn syntactic_link_full_ref_case_insensitive_label() {
    let input = "[click][LINK]\n\n[Link]: https://example.com\n";
    let expected = "[click][link]\n\n[link]: https://example.com\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn syntactic_link_label_with_spaces_and_punctuation() {
    let input = "[School][St. John's School]\n\n[St. John's School]: https://example.com\n";
    let expected = "[School][st. john's school]\n\n[st. john's school]: https://example.com\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn syntactic_link_missing_definition_pass_through() {
    let input = "Some [text][missing] more.\n";
    let out = fmt(input);
    assert!(
        !out.contains('\u{F000}') && !out.contains('\u{F001}') && !out.contains('\u{F002}'),
        "missing-def link must not leak PUA markers, got: {out}"
    );
}

// =============================================================================
// BADGE pattern — image nested inside a reference link, the canonical
// `[![alt][img]][url]` shape used for CI/license/version badges. Each
// inner-image form × each outer-link form is its own row.
// =============================================================================

#[test]
fn syntactic_badge_inline_image_in_full_ref_link() {
    let input = "[![alt](https://example.com/img.png)][url]\n\n[url]: https://example.com/page\n";
    let expected =
        "[![alt](https://example.com/img.png)][url]\n\n[url]: https://example.com/page\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn syntactic_badge_full_ref_image_in_full_ref_link() {
    let input = "[![alt][img]][url]\n\n[img]: https://example.com/img.png\n[url]: https://example.com/page\n";
    let expected = "[![alt](https://example.com/img.png)][url]\n\n[img]: https://example.com/img.png\n[url]: https://example.com/page\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn syntactic_badge_collapsed_ref_image_in_full_ref_link() {
    let input =
        "[![alt][]][url]\n\n[alt]: https://example.com/img.png\n[url]: https://example.com/page\n";
    let expected = "[![alt](https://example.com/img.png)][url]\n\n[alt]: https://example.com/img.png\n[url]: https://example.com/page\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn syntactic_badge_shortcut_ref_image_in_full_ref_link() {
    let input =
        "[![alt]][url]\n\n[alt]: https://example.com/img.png\n[url]: https://example.com/page\n";
    let expected = "[![alt](https://example.com/img.png)][url]\n\n[alt]: https://example.com/img.png\n[url]: https://example.com/page\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn syntactic_badge_inline_image_in_inline_link() {
    let input = "[![alt](https://example.com/img.png)](https://example.com/page)\n";
    let expected = "[![alt](https://example.com/img.png)](https://example.com/page)\n";
    assert_eq!(fmt(input), expected);
}

// =============================================================================
// INLINE CODE — backtick variants. Critical because comrak normalizes
// surrounding whitespace, and Python preserves a single inner-space when
// the code contains a backtick.
// =============================================================================

#[test]
fn syntactic_code_single_backtick() {
    let input = "Use `let x = 5;` here.\n";
    assert_eq!(fmt(input), "Use `let x = 5;` here.\n");
}

#[test]
fn syntactic_code_double_backtick_with_inner_backtick() {
    // The `` `` form is the standard escape for code that itself contains a
    // backtick. Python preserves the surrounding spaces.
    let input = "Use `` ` `` for backticks.\n";
    assert_eq!(fmt(input), "Use `` ` `` for backticks.\n");
}

// =============================================================================
// EMPHASIS / STRONG — `*` vs `_`, nested combinations.
// =============================================================================

#[test]
fn syntactic_emph_asterisk() {
    let input = "Some *emphasized* text.\n";
    assert_eq!(fmt(input), "Some *emphasized* text.\n");
}

#[test]
fn syntactic_strong_asterisk() {
    let input = "Some **bold** text.\n";
    assert_eq!(fmt(input), "Some **bold** text.\n");
}

#[test]
fn syntactic_strong_emph_nested() {
    let input = "Some ***bold and emphasized*** text.\n";
    assert_eq!(fmt(input), "Some ***bold and emphasized*** text.\n");
}

// =============================================================================
// STRIKETHROUGH (GFM) — `~~text~~`.
// =============================================================================

#[test]
fn syntactic_strikethrough_basic() {
    let input = "Some ~~deleted~~ text.\n";
    assert_eq!(fmt(input), "Some ~~deleted~~ text.\n");
}

// =============================================================================
// LINEBREAK — backslash form vs trailing-space form. Both render to
// backslash form in the output (Python convention).
// =============================================================================

#[test]
fn syntactic_linebreak_backslash() {
    let input = "Line one\\\nLine two\n";
    assert_eq!(fmt(input), "Line one\\\nLine two\n");
}

// =============================================================================
// FOOTNOTE REFERENCE — `[^label]`. Definition formats are covered by D5-D8.
// This row just confirms a stand-alone reference round-trips cleanly.
// =============================================================================

#[test]
fn syntactic_footnote_reference_basic() {
    let input = "See note[^1] here.\n\n[^1]: The note body.\n";
    let out = fmt(input);
    assert!(out.contains("[^1]"), "footnote reference should round-trip: {out}");
    assert!(out.contains("[^1]: The note body."), "footnote def should round-trip: {out}");
}

// =============================================================================
// AUTOLINK extension (bare URL) — comrak GFM extension recognizes bare
// `https://...` as an autolink. Python does too. Must NOT add angle brackets.
// =============================================================================

#[test]
fn syntactic_bare_url_autolink_no_angle_brackets() {
    let input = "Visit https://example.com today.\n";
    assert_eq!(fmt(input), "Visit https://example.com today.\n");
}

// =============================================================================
// HTML INLINE — tag fragments embedded in paragraphs.
// =============================================================================

#[test]
fn syntactic_html_inline_simple_tag() {
    let input = "Text with <span>inline html</span> here.\n";
    assert_eq!(fmt(input), "Text with <span>inline html</span> here.\n");
}
