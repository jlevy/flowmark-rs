# Feature: Exact Cross-Language Parity (flowmark Python → Rust)

**Date:** 2026-02-17 (last updated 2026-02-17)

**Author:** Joshua Levy

**Status:** Draft

## Overview

Complete the flowmark-rs Rust port to achieve exact behavioral parity with the Python
flowmark v0.6.4.
Use the cross-language test mapping system (see
`plan-2026-02-17-test-mapping-meta-test.md`) as the source of truth for tracking
coverage.

"Exact parity" means: for every Python test function (except those explicitly excluded
as infrastructure-only), there is a passing Rust test that verifies the same behavior.

## Goals

- 100% of ported Python test functions have a `mapped` or `excluded` entry in
  `test-mapping.yaml`.
- `flowmark-dev check-mapping` passes with exit code 0 (no `missing` entries).
- All 178+ Rust tests pass.
- Golden/reference document tests produce identical output to Python.

## Non-Goals

- CLI feature parity (file discovery, config loading, `.flowmarkignore`). These are
  Python infrastructure features tracked separately.
- Performance benchmarking against the Python version.
- Adding features beyond what Python v0.6.4 supports.

## Background

### Current State

**Python flowmark v0.6.4**: 281 test functions across 20 files.
**Rust flowmark-rs**: 178 test functions (151 integration + 27 unit), all passing.

### Python Module → Rust Module Mapping

| Python Module | Rust Module | Status |
|---|---|---|
| `linewrapping/text_wrapping.py` | `wrapping/text_wrapping.rs` | Implemented |
| `linewrapping/text_filling.py` | `wrapping/text_filling.rs` | Implemented |
| `linewrapping/sentence_split_regex.py` | `wrapping/sentence.rs` | Implemented |
| `linewrapping/line_wrappers.py` | `wrapping/line_wrappers.rs` | Implemented |
| `linewrapping/tag_handling.py` | `wrapping/tag_handling.rs` | Implemented |
| `linewrapping/block_heuristics.py` | `wrapping/block_heuristics.rs` | Implemented |
| `linewrapping/atomic_patterns.py` | `wrapping/atomic_patterns.rs` | Implemented |
| `formats/flowmark_markdown.py` | `formatter/markdown.rs` | Implemented |
| `formats/frontmatter.py` | `parser/frontmatter.rs` | Implemented |
| `typography/ellipses.py` | `typography/ellipses.rs` | Implemented |
| `typography/smartquotes.py` | `typography/quotes.rs` | Implemented |
| `transforms/doc_cleanups.py` | `transform/cleanups.rs` | Implemented |
| `transforms/doc_transforms.py` | `transform/cleanups.rs` (partial) | Review needed |
| `reformat_api.py` | `lib.rs` (`reformat_text`, `reformat_file`) | Implemented |
| `config.py` | `config.rs` | Partial — no TOML loading |
| `cli.py` | `main.rs` | Basic — no file discovery |
| `file_resolver/` | Not ported | Excluded (infrastructure) |
| `skill.py` / `skills/` | Not ported | Excluded (Python-specific) |

### Mapping Summary (from `check-mapping` output)

| Status | Count | Description |
|---|---|---|
| **Mapped** | 137 | Direct Rust equivalent exists and verified |
| **Excluded** | 79 | Infrastructure-only, not applicable to Rust |
| **Missing** | 64 | Need to be ported |
| **Partial** | 1 | Rust test covers subset |

**27 extra Rust tests** (unit tests in `src/` not mapped to Python) — these are
Rust-native tests, not gaps.

### Missing Tests by File

| Python Test File | Missing | Total | Notes |
|---|---|---|---|
| `test_wrapping.py` | 32 | 48 | Largest gap — block heuristics, tag spacing, self-closing tags |
| `test_tag_formatting.py` | 15 | 30 | Multiline tags, selection fields, smart quotes in tags |
| `test_escape_handling.py` | 5 | 12 | List/quote/table escapes, mixed escapes |
| `test_smartquotes.py` | 5 | 28 | Blockquote/table integration tests |
| `test_alerts.py` | 2 | 15 | Empty alert type, quote with link-like content |
| `test_strikethrough.py` | 2 | 11 | Tilde space opener/closer |
| `test_fenced_code_blocks.py` | 1 | 12 | Minimum backticks computed from content |
| `test_heading_spacing.py` | 1 | 9 | Hard break in list heading |
| `test_width_options.py` | 1 | 7 | Negative width disables wrapping |

### Fully Mapped Files (no gaps)

| Python Test File | # Tests | Notes |
|---|---|---|
| `test_cleanups.py` | 1 | Complete |
| `test_ellipses.py` | 1 | 1:N split into 10 Rust functions |
| `test_filling.py` | 2 | Complete |
| `test_frontmatter.py` | 3 | 1:N split into 5+ Rust functions |
| `test_list_spacing.py` | 20 | Complete |
| `test_ref_docs.py` | 1 | Golden test, complete |
| `test_sentences.py` | 2 | Complete |

### Excluded Files (infrastructure)

| File | # Tests | Reason |
|---|---|---|
| `test_cli_file_discovery.py` | 18 | Python CLI arg handling, `--auto` mode, stdin |
| `test_config.py` | 18 | Python TOML config, pyproject.toml, config merging |
| `test_file_resolver.py` | 28 | Python file glob, gitignore, exclude patterns |
| `test_skill.py` | 9 | Python skill/plugin system for Claude Code |
| *(6 from other files)* | 6 | Individual tests within ported files that are infra-only |

## Design

### Approach

The work is organized into three phases:

1. **Complete the test mapping** — populate `test-mapping.yaml` with verified entries
   for all 281 Python tests so we have a clear picture.
2. **Port missing tests** — for each `missing` entry, write the Rust test equivalent.
3. **Verify parity** — run `check-mapping` to confirm zero missing entries, run all
   Rust tests to confirm they pass.

### Key Areas Requiring Work

Based on the test coverage summary above, the main gaps are:

#### A. Tag Formatting (~13 missing tests)

Python `test_tag_formatting.py` has 30 tests covering Jinja/Markdoc/Nunjucks template
tag handling.
Rust `test_tag_formatting.rs` currently has 17 tests.
Missing tests likely cover:
- Multiline tag handling (`test_line_wrapper_preserves_multiline_tags`,
  `test_multiline_opening_tag_closing_on_own_line`)
- Tag with embedded structures (`test_tag_with_array_spanning_lines`,
  `test_tag_with_object_spanning_lines`)
- Selection fields with task lists
- Inline tag spacing in lists
- HTML comment multiline closing
- Smart quotes with multiline tags

#### B. Wrapping (~32 missing tests)

Python `test_wrapping.py` has 48 tests.
Rust `test_wrapping.rs` currently has 16 tests.
Missing tests likely cover:
- Block heuristic tests (list items, table rows, mixed content, blank line
  normalization)
- Self-closing tag tests (Jinja, HTML comments, variable tags)
- Tag content spacing (paragraph vs block content, closing tag spacing)
- Newline preservation around tags
- Backslash in tag attributes
- Inline code edge cases (table cells, surrounding punctuation)
- Long tag preservation (HTML tags, Jinja comments, template tags with many
  attributes)
- Various tag types with tables

#### C. Escape Handling (5 missing tests)

Python `test_escape_handling.py` has 12 tests.
Rust has 7.
Missing: `test_escape_in_list_item`, `test_escape_in_quote`, `test_escape_in_table`,
`test_mixed_escapes`, `test_mixed_escapes_comprehensive`.

#### D. Remaining Gaps (scattered)

- `test_alerts.py`: 2 missing (`test_empty_alert_type_preserves_quote`,
  `test_quote_with_link_like_content`)
- `test_strikethrough.py`: 2 missing (`test_tilde_space_after_opener`,
  `test_tilde_space_before_closer`)
- `test_heading_spacing.py`: 1 missing (`test_heading_with_hard_break_in_list`)
- `test_list_spacing.py`: 2 missing
- `test_fenced_code_blocks.py`: 1 missing (`test_minimum_backticks_computed_from_content`)
- `test_width_options.py`: 1 missing (`test_negative_width_disables_wrapping`)
- `test_smartquotes.py`: review needed for complete parity

### Exclusions

The following Python test files are **excluded** from parity — they test Python-specific
infrastructure that does not apply to the Rust port:

| File | # Tests | Reason |
|---|---|---|
| `test_cli_file_discovery.py` | 18 | Python CLI arg handling, `--auto` mode, stdin |
| `test_config.py` | 18 | Python TOML config, pyproject.toml, config merging |
| `test_file_resolver.py` | 28 | Python file glob, gitignore, exclude patterns |
| `test_skill.py` | 9 | Python skill/plugin system for Claude Code |

Total excluded: **73 tests**. These are documented in `test-mapping.yaml` with
`status: excluded` and individual notes.

### Exceptions and Issues Log

This section tracks any behavioral differences, edge cases, or decisions encountered
during the parity process:

- *(None yet — update as issues are discovered)*

## Implementation Plan

### Phase 1: Complete Test Mapping — DONE

- [x] Populate `test-mapping.yaml` with all 281 entries (137 mapped, 79 excluded,
  64 missing, 1 partial)
- [x] Run `flowmark-dev check-mapping` and capture the baseline gap report
- [x] Review 1:N split cases: `test_ellipses` → 10 Rust fns,
  `test_split_frontmatter` → 5 Rust fns — verified correct
- [x] Document all exclusions with rationale (79 infrastructure tests across 4 files)

### Phase 2: Port Missing Tests and Fix Gaps (64 tests)

- [ ] Port 32 missing wrapping tests to `tests/test_wrapping.rs`
- [ ] Port 15 missing tag formatting tests to `tests/test_tag_formatting.rs`
- [ ] Port 5 missing escape handling tests to `tests/test_escape_handling.rs`
- [ ] Port 5 missing smartquotes integration tests to `tests/test_smartquotes.rs`
- [ ] Port 7 scattered missing tests (2 alerts, 2 strikethrough, 1 heading spacing,
  1 fenced code blocks, 1 width options)
- [ ] Fix any Rust implementation bugs discovered during test porting
- [ ] Update `test-mapping.yaml` as tests are added (change `missing` → `mapped`)

### Phase 3: Verify and Close

- [ ] `flowmark-dev check-mapping` exits with code 0
- [ ] All Rust tests pass (`cargo test`)
- [ ] Golden/reference document test produces identical output
- [ ] Update this spec status to "Implemented"
- [ ] Log any behavioral exceptions in the "Exceptions and Issues Log" section above

## Testing Strategy

- `flowmark-dev check-mapping` is the primary verification mechanism for mapping
  completeness.
- `cargo test` verifies all Rust tests pass.
- The golden test (`test_ref_docs.rs`) verifies full-pipeline output equivalence.
- For new tests being ported: read the Python test source, understand the assertion,
  write the equivalent Rust test, verify it passes.

## Open Questions

- **Behavioral differences**: Are there cases where the Rust comrak parser produces
  different Markdown output than Python's markdown-it?
  If so, these need to be documented as known exceptions.
- **`doc_transforms.py` coverage**: This module contains transforms beyond cleanups
  (e.g., strikethrough normalization).
  Need to verify all transform paths are covered in Rust.

## References

- Test mapping infrastructure spec:
  `docs/project/specs/active/plan-2026-02-17-test-mapping-meta-test.md`
- YAML artifacts: `port-coverage-mapping/`
- Original Python repo: https://github.com/jlevy/flowmark (pinned: `v0.6.4`)
- Porting plan: `docs/porting-plan.md`
