# Parity Coverage Matrix

> **Doc status:** Rust port-specific (no upstream equivalent).
> Maps every AST node type the renderer handles to the test(s) that prove its per-form
> parity with Python flowmark.
> Built to make coverage gaps visible at a glance rather than discoverable only through
> bug reports.

## Why this exists

The senior review of PRs #54 and #57 (2026-05-28) surfaced three latent parity bugs —
reference-image inlining, the badge pattern (`[![alt][img]][url]`), and ref-def label
lowercasing — that were never exercised by the existing fixtures, the differential
corpus sweep, or the mirrored upstream tests.
The root cause is structural, not local: upstream Python flowmark itself has near-zero
test coverage for reference-image syntactic forms (`grep "!\\[" tests/*.py` returns only
an alert-syntax false positive and one inline-image example), and mirroring upstream
tests faithfully inherits upstream gaps.

This document is the structural backstop.
For every `NodeValue::*` variant the renderer handles (see
`render_block`/`render_inline` in
[`src/formatter/filling.rs`](../src/formatter/filling.rs)), this index lists:

- the **forms** the variant can take in source markdown
- the **tests** that exercise each form
- any **known gaps** (rows where coverage is partial)

When you add, modify, or remove a render branch, update this index and
[`tests/test_syntactic_surface.rs`](../tests/test_syntactic_surface.rs) in the same
commit.

## How to read this

Each row is one `(NodeValue variant, syntactic form)` pair.
The right-most column links to a test that asserts byte-identical Python parity for that
specific row, OR explains why the row has no direct coverage.

## Inline nodes

| Node | Form | Source example | Covered by |
| --- | --- | --- | --- |
| `Text` | n/a | `hello world` | implicit in every test |
| `SoftBreak` | source line break inside paragraph | `line one\nline two` | `tests/wrapping/`, `test_ref_docs.rs` |
| `LineBreak` | backslash form | `line one\\\nline two` | `syntactic_linebreak_backslash` |
| `LineBreak` | trailing-double-space form | `line one  \nline two` | (renders identically; covered indirectly via wrap suite) |
| `Code` | single backtick | `` `code` `` | `syntactic_code_single_backtick` |
| `Code` | double backtick (inner backtick) | (double-backtick form, used to embed a single backtick) | `syntactic_code_double_backtick_with_inner_backtick` |
| `Emph` | asterisk | `*emph*` | `syntactic_emph_asterisk` |
| `Strong` | asterisk | `**bold**` | `syntactic_strong_asterisk` |
| `Strong+Emph` | nested | `***both***` | `syntactic_strong_emph_nested` |
| `Strikethrough` | GFM | `~~deleted~~` | `syntactic_strikethrough_basic` |
| `Link` | inline (no title) | `[text](url)` | `syntactic_link_inline_no_title` |
| `Link` | inline (title) | `[text](url "t")` | `syntactic_link_inline_with_title` |
| `Link` | autolink `<url>` | `<https://example.com>` | `syntactic_link_autolink_angle_bracket` |
| `Link` | autolink (bare URL, GFM) | `https://example.com` | `syntactic_bare_url_autolink_no_angle_brackets` |
| `Link` | reference full | `[text][label]` | `syntactic_link_full_ref_preserved`; D18 |
| `Link` | reference full (title) | `[text][label]` + def | `syntactic_link_full_ref_with_title_preserved` |
| `Link` | reference collapsed | `[text][]` | `syntactic_link_collapsed_ref_preserved`; D18 |
| `Link` | reference shortcut (text == label) | `[text]` + def | `syntactic_link_shortcut_ref_normalized_to_collapsed`; D18 |
| `Link` | reference label case-insensitive | `[text][LINK]` + `[link]:` | `syntactic_link_full_ref_case_insensitive_label` |
| `Link` | reference label with spaces/apostrophe | `[School][St. John's School]` | `syntactic_link_label_with_spaces_and_punctuation`; D18 |
| `Link` | reference label normalized → lowercase | def emitted as lowercase | D20 (`test_d20_*`) |
| `Link` | reference missing definition | `[text][missing]` | `syntactic_link_missing_definition_pass_through` |
| `Image` | inline (no title) | `![alt](url)` | `syntactic_image_inline_no_title` |
| `Image` | inline (title) | `![alt](url "t")` | `syntactic_image_inline_with_double_quoted_title` |
| `Image` | inline (empty alt) | `![](url)` | `syntactic_image_inline_empty_alt` |
| `Image` | reference full → inlined | `![alt][img]` + def | `syntactic_image_full_ref`; D19 |
| `Image` | reference full (title) | `![alt][img]` + titled def | `syntactic_image_full_ref_with_title`; D19 |
| `Image` | reference full (case-insensitive label) | `![alt][IMG]` + `[Img]:` | `syntactic_image_full_ref_case_insensitive_label` |
| `Image` | reference collapsed | `![alt][]` + def | `syntactic_image_collapsed_ref`; D19 |
| `Image` | reference shortcut | `![alt]` + def | `syntactic_image_shortcut_ref`; D19 |
| `Image` | reference label with spaces | `![Logo][company logo]` | `syntactic_image_label_with_spaces_and_punctuation`; D19 |
| `Image` | reference missing definition | `![alt][missing]` | `syntactic_image_missing_definition_pass_through`; D19 |
| `Image` × `Link` | badge: inline image / full ref link | `[![alt](url)][label]` | `syntactic_badge_inline_image_in_full_ref_link` |
| `Image` × `Link` | badge: full-ref image / full-ref link | `[![alt][img]][label]` | `syntactic_badge_full_ref_image_in_full_ref_link`; D19 |
| `Image` × `Link` | badge: collapsed-ref image / full-ref link | `[![alt][]][label]` | `syntactic_badge_collapsed_ref_image_in_full_ref_link` |
| `Image` × `Link` | badge: shortcut-ref image / full-ref link | `[![alt]][label]` | `syntactic_badge_shortcut_ref_image_in_full_ref_link` |
| `Image` × `Link` | badge: inline image / inline link | `[![alt](url)](href)` | `syntactic_badge_inline_image_in_inline_link` |
| `FootnoteReference` | standalone | `[^1]` | `syntactic_footnote_reference_basic`; D5-D8 |
| `HtmlInline` | inline tag | `<span>x</span>` | `syntactic_html_inline_simple_tag` |
| `Math` | inline `$x$` (if extension) | — | **Gap:** no Rust test; comrak math extension off by default in Python parity surface — flag for next sync audit |
| `WikiLink` | `[[Page]]` (if extension) | — | **Gap:** flag for next sync audit; not used in upstream tests |
| `Escaped` | `\!`, `\[`, etc. | (32 chars covered) | `test_escape_handling.rs` (Rust) + W4 docs |

## Block nodes

| Node | Form | Source example | Covered by |
| --- | --- | --- | --- |
| `Document` | n/a | — | implicit |
| `Paragraph` | basic | text block | `test_ref_docs.rs`, wrapping suite |
| `Heading` | ATX levels 1–6 | `# H1` … `###### H6` | `test_heading_*`, ref-doc fixtures |
| `Heading` | Setext H1/H2 | `H1\n===` / `H2\n---` | `test_ref_docs.rs` (testdoc orig uses ATX); **Gap candidate:** verify Setext is normalized to ATX matching Python |
| `BlockQuote` | basic, nested, lazy continuation | `> q` | D6, D8 |
| `List` | bullet `-/*/+` | `- item` | wrapping/list suites |
| `List` | ordered `1.` / `1)` | `1. item` | wrapping/list suites |
| `List` | tight vs loose | spacing-sensitive | D4, list-spacing suite |
| `List` | task list `- [ ]` | `- [ ] todo` | `test_list_spacing` indirectly; **Gap candidate:** add explicit row |
| `CodeBlock` | fenced (```) | `` ```lang\ncode\n``` `` | D1, ref-doc fixtures |
| `CodeBlock` | indented 4-space | 4-space block | `test_fenced_code_blocks` indirectly |
| `ThematicBreak` | `---`, `***`, `___` | rule | D17 |
| `HtmlBlock` | type 1 (`<script>`/`<style>`) | block tag | `test_tag_formatting` |
| `HtmlBlock` | type 2 (`<!-- comment -->`) | HTML comment | W8 (HTML-comment spacing rules) |
| `HtmlBlock` | type 7 (generic) | other block tag | `test_tag_formatting` |
| `Table` | basic, alignment | GFM table | `test_ref_docs.rs` (wide-table fixture), D17 |
| `TableRow`/`TableCell` | n/a (children of `Table`) | — | covered with `Table` |
| `FootnoteDefinition` | single-line | `[^1]: body` | D5-D8, footnote suite |
| `FootnoteDefinition` | multi-line / list-bearing | `[^1]: body\n  - item` | D5, D7 |
| `Alert` | `[!NOTE]`/`[!TIP]`/etc. | `> [!NOTE]\n> body` | `test_alerts.rs` |

## Process: keeping the matrix complete

When syncing to a new upstream Python release:

1. Diff `repos/flowmark/src/flowmark/formats/flowmark_markdown.py` for any new
   `render_*` method or any modification to an existing one.
2. For each touched render method, find the matching `NodeValue::*` branch in
   [`src/formatter/filling.rs`](../src/formatter/filling.rs).
3. For each row in this matrix that maps to that node, re-verify by running the input
   through `uvx flowmark@<target-version> -` and updating the expected output if it
   changed (it usually has not — that’s the parity contract).
4. If the upstream change introduces a NEW form, add a row here and the matching test in
   `tests/test_syntactic_surface.rs`.
5. If a row is marked **Gap**, decide whether to close it now or carry it into the sync
   artifact’s known-variations list.

When adding or modifying a render branch in the Rust port:

1. Identify the AST node and the syntactic form(s) the change affects.
2. Add the matching row(s) to this matrix.
3. Add or update the test in `tests/test_syntactic_surface.rs`.
4. Cross-validate the expected output against Python before landing.

## Related

- [`tests/test_syntactic_surface.rs`](../tests/test_syntactic_surface.rs) — the test
  file backing this matrix
- [`tests/test_parity_discrepancies.rs`](../tests/test_parity_discrepancies.rs) — the
  regression suite for specific historical discrepancies (D1-D20)
- [`docs/port-status.md`](port-status.md) — current parity surface and tolerated
  variations
- [`docs/sync-artifacts/`](sync-artifacts/) — per-sync triage and validation
- **Planned (tracked as `fmr-i17c`):** CommonMark spec parity gate — runs both
  binaries over all 655 spec examples and reports the diff set against a baseline.
  Complements (does not replace) this targeted matrix: the matrix is curated and
  asserts exact-match per form, the spec gate is exhaustive and tracks divergences
  against a known-divergences list that shrinks over time.
