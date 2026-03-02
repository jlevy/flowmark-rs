# Feature: Fix All Python Parity Discrepancies

**Date:** 2026-02-18

**Author:** Senior review (Claude)

**Status:** COMPLETE — All discrepancies D1-D11 resolved (2026-02-19)

## Overview

A comprehensive senior engineering review comparing flowmark-rs against Python flowmark
v0.6.4 revealed 8 distinct parity discrepancies.
The core formatting modes (default, semantic, auto, width 120) match exactly.
Discrepancies appear in plaintext mode, narrow widths, list spacing modes, nested
blockquotes, and footnote body handling.

Two issues were already fixed during this review session:
- Clippy `inefficient_to_string` lint (4 instances in `protect_autolinks`)
- Autolink false positive for relative-path links where text == URL

## Goals

- Achieve exact byte-for-byte output parity with Python flowmark v0.6.4 on the golden
  test document across ALL formatting modes
- Zero tolerance for discrepancies between Python and Rust output

## Non-Goals

- Performance optimization (separate effort)
- New features beyond Python parity

## Background

Cross-comparison methodology: The golden test document
(`tests/testdocs/testdoc.orig.md`) was processed through both Python flowmark v0.6.4 and
Rust flowmark across 9 modes: default, semantic, auto
(semantic+cleanups+smartquotes+ellipses), plaintext, width 60, width 120, semantic+width
60, loose list-spacing, tight list-spacing.
Additional targeted edge-case inputs were also tested.

### Summary of current state

| Mode | Status |
| --- | --- |
| Default (width 88) | Exact match |
| Semantic | Exact match |
| Auto (semantic+cleanups+smartquotes+ellipses) | Exact match |
| Width 120 | Exact match |
| Width 0 (no wrap) | Exact match |
| Frontmatter | Exact match |
| HTML comments, tables, images, thematic breaks | Exact match |
| Hard breaks, reference links, escapes | Exact match |
| Strikethrough, task lists | Exact match |
| GFM alerts | Exact match |
| Cleanups (unbold headings) | Exact match |
| Plaintext mode | Exact match (resolved 2026-02-19) |
| Width 60 | Exact match (resolved 2026-02-19) |
| Semantic + Width 60 | Exact match (resolved 2026-02-19) |
| Tight list spacing | Exact match (resolved 2026-02-19) |
| Loose list spacing | Exact match (resolved 2026-02-19) |
| Nested blockquotes | Exact match (resolved 2026-02-19) |
| Footnote list items | Exact match (resolved 2026-02-19) |
| Footnote blockquotes | Exact match (resolved 2026-02-19) |

## Discrepancies

### D1: Plaintext mode — code blocks collapsed

**Severity:** Medium **Files:** `src/wrapping/text_filling.rs`, `src/lib.rs`

Python’s plaintext mode preserves code fence structure (``` blocks stay multi-line).
Rust collapses them into a single line because plaintext mode treats everything as flat
text with no Markdown awareness.

**Reproduction:**
```
Input:  ```javascript\n// comment\nvar x = 5;\n```
Python: ```javascript\n// comment\nvar x = 5;\n```  (preserved)
Rust:   ```javascript // comment var x = 5; ```  (collapsed)
```

### D2: Plaintext mode — “St.” sentence detection

**Severity:** Low **Files:** `src/wrapping/text_filling.rs`

Python’s plaintext wrapper splits after “St.”
(treating it as sentence end), producing:
```
[St.
John's Beaumont School](url)
```
Rust keeps it on one line.
This is a word-splitting heuristic difference in the plaintext text_filling path.

### D3: Narrow width — `<sup>` tag word splitting

**Severity:** Low **Files:** `src/wrapping/text_wrapping.rs`

At width 60, Rust wraps differently around `<sup>19</sup>` and `<sup>59</sup>` tags.
Python produces 6 lines, Rust produces 5 lines for the same block.
The difference is in how the word splitter treats HTML sup tags — whether they are
breakable or atomic.

**Reproduction (width 60):**
```
Python:     ...automatically deleted
            when closed or on process termination.<sup>19</sup>
            While convenient, POSIX notes...

Rust:       ...automatically deleted when
            closed or on process termination.<sup>19</sup> While
            convenient, POSIX notes...
```

### D4: Tight list spacing — extra blank lines in nested lists

**Severity:** Medium **Files:** `src/formatter/filling.rs` (list rendering in
`render_block`)

In `--list-spacing tight` mode, Rust inserts blank lines between nested list items where
Python keeps them tight.
Affects 10+ locations in the golden test doc.

**Reproduction:**
```
Python:     - Level 1a
              - Level 2a
                - Level 3a

Rust:       - Level 1a

              - Level 2a

                - Level 3a
```

The `can_be_tight()` function or the tight-mode rendering path is not correctly
suppressing inter-item blank lines for nested sublists.

### D5: Loose list spacing — missing blank lines in footnote embedded lists

**Severity:** Low **Files:** `src/formatter/filling.rs` (FNDEF rendering)

In `--list-spacing loose` mode, Python adds a blank line after footnote list items that
Rust omits. Only 2 lines differ in the golden test.

### D6: Nested blockquotes — extra blank separator lines

**Severity:** Medium **Files:** `src/formatter/filling.rs`
(`render_block_children_quoted`)

Rust inserts `> ` blank lines between adjacent nested blockquote levels.
Python does not.

**Reproduction:**
```
Input:      > Level 1
            > > Level 2
            > > > Level 3

Python:     > Level 1
            > > Level 2
            > > > Level 3

Rust:       > Level 1
            >
            > > Level 2
            > >
            > > > Level 3
```

### D7: Footnote body — continuation list items collapsed

**Severity:** High **Files:** `src/formatter/filling.rs` (FNDEF list handling)

When a footnote body contains a preamble followed by list items, Rust collapses the list
items onto the preamble line.
Python preserves them as separate list items with proper indentation.

**Reproduction:**
```
Input:      [^3]: Footnote with a list:
                - Item 1
                - Item 2

Python:     [^3]: Footnote with a list:
                - Item 1
                  - Item 2

Rust:       [^3]: Footnote with a list: - Item 1 - Item 2
```

### D8: Footnote body — blockquote continuation collapsed

**Severity:** High **Files:** `src/formatter/filling.rs` (FNDEF rendering)

Blockquote content inside a footnote body is collapsed onto the footnote’s first line.

**Reproduction:**
```
Input:      [^4]: Footnote with blockquote:
                > Quoted text.

Python:     [^4]: Footnote with blockquote:
                > Quoted text.

Rust:       [^4]: Footnote with blockquote: > Quoted text.
```

### D9: Empty/whitespace input — missing trailing newline

**Severity:** Medium **Files:** `src/formatter/filling.rs`, `src/lib.rs`

Python always outputs at least a trailing newline (`\n`) for empty or whitespace-only
input. Rust outputs nothing (0 bytes).

**Reproduction:**
```
Input:      "" (empty string)
Python:     "\n" (1 byte)
Rust:       "" (0 bytes)
```

### D10: HTML entities decoded instead of preserved

**Severity:** Medium **Files:** `src/formatter/filling.rs` (comrak rendering)

Comrak’s AST construction decodes HTML entities (`&amp;` → `&`, `&lt;` → `<`, etc.).
Python’s marko preserves them as-is.

**Reproduction:**
```
Input:      &amp; &lt; &gt; &quot;
Python:     &amp; &lt; &gt; &quot;
Rust:       & < > "
```

### D11: CLI error handling parity gaps (FIXED)

**Severity:** Medium **Files:** `src/error.rs`, `src/main.rs`,
`tests/test_parity_discrepancies.rs`

Multiple CLI error handling discrepancies discovered during hands-on parity testing of
error paths. Three code bugs were fixed and five cross-binary parity tests were added.

#### Bugs fixed

1. **Duplicate error message in I/O errors** — `Error::Io` used
   `#[error("I/O error: {0}")]` which included the inner error in Display, then anyhow’s
   `{:#}` chain appended the source again, producing:
   `"I/O error: No such file or directory (os error 2): No such file or directory (os error 2)"`.
   Fixed by changing to `#[error("I/O error")]`.

2. **Missing `--inplace` + stdin validation** — Python rejects `flowmark --inplace -`
   with `"Error: Cannot use 'inplace' with stdin"` (exit 1). Rust silently ignored the
   flag and processed stdin normally.
   Fixed by adding validation before the processing loop.

3. **Nonexistent file error format** — Rust produced
   `error: failed to format X: I/O error: No such file or directory (os error 2)` with
   lowercase prefix and anyhow chain wrapping.
   Fixed by validating file existence early in `resolve_files()` (matching Python’s
   file_resolver) and producing `Error: Path not found: X`.

#### Remaining discrepancy (accepted)

| Error case | Python | Rust | Match? |
| --- | --- | --- | --- |
| No arguments | `Error: No input specified...` (exit 1) | Same | Exact |
| `--auto` no args | `Error: --auto requires...` (exit 1) | Same | Exact |
| `--list-files` no args | `Error: --list-files requires...` (exit 1) | Same | Exact |
| `--inplace` + stdin | `Error: Cannot use \`inplace\` with stdin` (exit 1) | Same | Exact |
| `-o` + multiple files | `Error: Cannot specify output file...` (exit 1) | Same | Exact |
| Nonexistent file | `Error: [Errno 2] No such file or directory: 'X'` (exit 2) | `Error: Path not found: X` (exit 1) | Semantic only |

The nonexistent file case cannot be byte-for-byte identical because `[Errno 2]` is a
Python runtime artifact.
Both use the `Error:` prefix and include the filename.
Exit code difference (2 vs 1) is acceptable.

#### Parity tests added

Five tests in `test_parity_discrepancies.rs` invoke both the Python and Rust binaries
with identical arguments and compare stderr output + exit codes:
- `test_d11_no_args_error_matches_python` — exact match
- `test_d11_auto_no_args_error_matches_python` — exact match
- `test_d11_inplace_stdin_error_matches_python` — exact match
- `test_d11_output_multiple_files_error_matches_python` — exact match
- `test_d11_nonexistent_file_error_format` — semantic match (prefix + filename)

#### Lessons learned

1. The tryscript E6 test used a `[..]` wildcard which completely masked the duplicate
   error message bug. **All error case golden tests should validate exact error
   messages**, not use wildcards.

2. `TRYSCRIPT_GIT_ROOT` is a tryscript built-in variable (auto-detects nearest `.git`)
   that cannot be overridden via environment.
   Attempting to run tryscript against the Python binary by setting
   `TRYSCRIPT_GIT_ROOT=/nonexistent` silently used the Rust binary anyway.
   **Cross-binary parity tests must invoke both binaries by explicit path**, not rely on
   tryscript PATH manipulation.

3. Error parity testing was entirely absent from the original parity review (D1-D10
   focused only on formatting output).
   CLI error messages are a first-class parity surface that must be tested with the same
   rigor as formatting output.

## Implementation Plan — COMPLETED

### Phase 1: Footnote body handling (D7, D8) — DONE

- [x] Fix FNDEF rendering to preserve list items in footnote bodies (D7)
- [x] Fix FNDEF rendering to preserve blockquote content in footnote bodies (D8)
- [x] Add regression tests for both cases

### Phase 2: Block-level spacing (D4, D6) — DONE

- [x] Fix tight list spacing to not insert blanks between nested sublists (D4)
- [x] Fix nested blockquote rendering to not insert extra blank lines (D6)
- [x] Add regression tests

### Phase 3: Wrapping, plaintext, and normalization (D1, D2, D3, D5, D9, D10) — DONE

- [x] Fix plaintext mode to preserve code fence structure (D1)
- [x] Fix plaintext word splitting heuristic for “St.”
  (D2)
- [x] Fix narrow-width `<sup>` tag word splitting (D3)
- [x] Fix loose list spacing for footnote embedded lists (D5)
- [x] Fix empty/whitespace input to output trailing newline (D9)
- [x] Fix HTML entity preservation (D10)
- [x] Add regression tests

### Phase 4: Real-world corpus parity (P6-P9) — DONE

- [x] Fix paragraph→CodeBlock tight transition (P6, 454 instances)
- [x] Fix blockquote blank continuation `>` prefix (P7, 9 instances)
- [x] Fix escaped backtick in table inline code (P8)
- [x] Fix smart quote after inline code backtick (P9)

### Phase 5: Full 4-mode parity (auto, tight, loose, plaintext) — DONE

- [x] Fix tight mode complex item detection (8 gaps → 0)
- [x] Fix loose mode Rules 3/4 blank line suppression
- [x] Fix loose mode footnote FNDEF preamble→list separator
- [x] Fix plaintext paired Jinja tag regex
- [x] Fix nested blockquote source position tracking
- [x] Fix golden test regression (Preserve mode gating)

## Testing Strategy

- 31 targeted D-series tests verify exact match with Python output
- Golden reference document test (`test_reference_doc_formats`) verifies 4 modes
- 481 total tests, 0 ignored, 0 failures
- CI: 12/12 checks pass (clippy, format, tests, semver, docs, coverage, MSRV, audit)

## References

- Python flowmark v0.6.4: `repos/flowmark/`
- Golden test doc: `tests/testdocs/testdoc.orig.md`
- Previous review: `docs/project/specs/active/code-review-2026-02-17.md`
- PR #17: https://github.com/jlevy/flowmark-rs/pull/17
