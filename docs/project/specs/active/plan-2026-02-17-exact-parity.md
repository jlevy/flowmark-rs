# Feature: Exact Cross-Language Parity (flowmark Python → Rust)

**Date:** 2026-02-17 (last updated 2026-02-17)

**Author:** Joshua Levy

**Status:** In Progress

**Epic bead:** fmr-kd36

## Overview

Complete the flowmark-rs Rust port to achieve **exact behavioral parity** with the
Python flowmark v0.6.4. Use the cross-language test mapping system (see
`plan-2026-02-17-test-mapping-meta-test.md`) as the source of truth for tracking
coverage.

**"Exact parity" means: every ported Python test has a passing (not ignored) Rust test
that verifies the same behavior. No test may be `#[ignore]`d. No `partial` mappings.
Every discrepancy is either a Rust bug to fix or an upstream Python bug to fix (and then
match in Rust).**

## Goals

- Zero `#[ignore]` tests. Every test passes or is explicitly excluded as infrastructure.
- Zero `partial` test mappings. Every mapped test covers the full Python behavior.
- `flowmark-dev check-mapping` passes with exit code 0 (no `missing` entries).
- All Rust tests pass (`cargo test` with no ignored tests).
- Golden/reference document tests produce identical output to Python.

## Non-Goals

- CLI feature parity (file discovery, config loading, `.flowmarkignore`). These are
  Python infrastructure features tracked separately.
- Performance benchmarking against the Python version.
- Adding features beyond what Python v0.6.4 supports.

## Background

### Current State

**Python flowmark v0.6.4**: 281 test functions across 20 files.
**Rust flowmark-rs**: 243 test functions (216 integration + 27 unit). 232 passing,
4 `#[ignore]`d (3 bugs), 1 `partial` mapping.

### Bugs Blocking Parity

**There are currently 3 Rust implementation bugs causing 4 ignored tests. Each must be
fixed (or if the upstream Python behavior is itself wrong, fix upstream first and then
match).**

| Bug ID | Tests Affected | Summary | Root Cause |
|---|---|---|---|
| **fmr-2tll** | `test_escape_in_list_item_start_preserved`, `test_mixed_escapes` | `- 1\. text` loses backslash | `postprocess_period_escapes` doesn't strip list markers before checking for digit-period patterns |
| **fmr-4l1x** | `test_heading_with_hard_break_in_list` | Extra blank line before heading in list item | `render_list_item` inserts blank line before heading without checking for hard-break headings (logic exists in `render_block_children` but not in list items) |
| **fmr-5ojk** | `test_list_item_with_tag_on_continuation_line` | Extra blank line before HTML comment tag on list continuation | Comrak parses indented HTML comment as `HtmlBlock` (separate child), and `render_list_item` unconditionally adds blank line between list item children |

### Partial Test Coverage Gap

| Mapping | Gap | Status |
|---|---|---|
| `test_other_escaped_chars` (partial) | Rust covers `\*`, `\#`, `\-` but omits `\$`, `\_`, `\[`/`\]`, `` \` `` | All 4 missing cases verified to pass at runtime; test assertions need to be added |

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
| **Mapped** | 201 | Direct Rust equivalent exists and verified |
| **Excluded** | 79 | Infrastructure-only, not applicable to Rust |
| **Missing** | 0 | All ported |
| **Partial** | 1 | `test_other_escaped_chars` — must be completed |

**27 extra Rust tests** (unit tests in `src/` not mapped to Python) — these are
Rust-native tests, not gaps.

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

The work is organized into phases:

1. **Test mapping** — DONE. All 281 Python tests have entries in `test-mapping.yaml`.
2. **Port missing tests** — DONE. 64 tests ported. But 4 are `#[ignore]`d and
   1 is `partial`.
3. **Fix all bugs** — Fix the 3 Rust bugs blocking the 4 ignored tests.
4. **Complete partial test** — Add missing assertions to `test_other_escaped_chars`.
5. **Review previous implementation** — Review `attic/flowmark-rs-1` for architectural
   choices that may be superior to the current approach.
6. **Apply porting playbook best practices** — Review each document in
   `attic/rust-porting-playbook/` and verify all best practices are applied.
7. **Meta-playbook review** — Fold learnings from this port back into the porting
   playbook to improve it for future ports.
8. **Final verification** — Zero ignored tests, zero partial mappings, all tests pass,
   golden test passes, `check-mapping` passes.

## Implementation Plan

### Phase 1: Complete Test Mapping — DONE

- [x] Populate `test-mapping.yaml` with all 281 entries
- [x] Run `flowmark-dev check-mapping` and capture the baseline gap report
- [x] Review 1:N split cases (verified correct)
- [x] Document all exclusions with rationale (79 infrastructure tests)

### Phase 2: Port Missing Tests (64 tests) — DONE

- [x] Port 32 missing wrapping tests to `tests/test_wrapping.rs`
- [x] Port 15 missing tag formatting tests to `tests/test_tag_formatting.rs`
- [x] Port 5 missing escape handling tests to `tests/test_escape_handling.rs`
- [x] Port 5 missing smartquotes integration tests to `tests/test_smartquotes.rs`
- [x] Port 7 scattered missing tests (alerts, strikethrough, heading, code blocks, width)
- [x] Updated `test-mapping.yaml` — all 64 entries changed `missing` → `mapped`

### Phase 3: Fix All Bugs — NOT DONE

Every discrepancy is tracked as a bead. For each: determine whether it's a Rust bug or
an upstream Python bug, fix it at the source, and un-ignore the test.

- [ ] **fmr-2tll**: Fix escape at start of list item content
  - Root cause: `postprocess_period_escapes()` in `src/formatter/filling.rs` strips
    blockquote markers but not list markers (`- `, `* `, `+ `) before checking for
    digit-period patterns. Fix: also strip list item prefixes before the digit check.
  - Python behavior confirmed correct (preserves escape).
  - 2 tests to un-ignore.
- [ ] **fmr-4l1x**: Fix extra blank line before heading in list item with hard break
  - Root cause: `render_list_item()` in `src/formatter/filling.rs` adds blank lines
    between children without checking whether the current child is a hard-break heading.
    `render_block_children()` has this logic but `render_list_item()` doesn't.
    Fix: add `child_is_hard_break_heading` check to the list item spacing logic.
  - Python behavior confirmed correct (no extra blank line).
  - 1 test to un-ignore.
- [ ] **fmr-5ojk**: Fix extra blank line before HTML comment tag on list continuation
  - Root cause: Comrak parses `<!-- #id -->` on an indented line as a block-level
    `HtmlBlock` (separate child of the list item), while Python's Marko disables block
    HTML parsing entirely and treats it as inline HTML within the paragraph.
    The Rust formatter then adds a blank line between the paragraph and HtmlBlock children.
  - Python behavior confirmed correct (no extra blank line).
  - Fix approach TBD (see Open Questions).
  - 1 test to un-ignore.

### Phase 4: Complete Partial Test — NOT DONE

- [ ] **test_other_escaped_chars**: Add `\$`, `\_`, `\[`/`\]`, `` \` `` assertions
  - All 4 cases verified passing at runtime. Just need test assertions added.
  - Update `test-mapping.yaml` status from `partial` to `mapped`.

### Phase 5: Review Previous Implementation (`attic/flowmark-rs-1`) — NOT DONE

Review the earlier Rust implementation to evaluate whether any architectural choices
there are superior to the current approach.

Key differences identified:

| Aspect | Previous (flowmark-rs-1) | Current |
|---|---|---|
| **Rendering** | comrak `format_commonmark()` + ~25 post-processing passes | Custom AST renderer with minimal post-processing |
| **Escape handling** | Post-processing regex fixes (fragile) | Pre-processing PUA placeholder protection |
| **Word splitting** | Trait-based (`WordSplitter` trait) | Function-based (closures) |
| **Tag handling** | None | Full Jinja/Markdoc/HTML comment support |
| **Typography** | Implemented but NOT integrated | Fully integrated at AST level |
| **List spacing** | Post-processing normalization | Configurable `ListSpacing` enum |
| **comrak version** | 0.47 | 0.36 |
| **Extra deps** | serde_yaml, serde_json, clap_complete, supports-color, indicatif, ctrlc, proptest | toml (minimal) |

Preliminary assessment: the current implementation's approach is architecturally
superior (custom renderer avoids the ~25 post-processing workarounds). However, specific
items to evaluate:

- [ ] Should we upgrade comrak from 0.36 to 0.47? (may fix some bugs, may introduce
  others)
- [ ] Should we adopt property-based testing (`proptest`) from the old impl?
- [ ] Should we adopt a trait-based `WordSplitter` instead of function-based splitting?
- [ ] Review whether any of the ~25 post-processing functions in the old impl address
  edge cases the current renderer misses.
- [ ] Evaluate `serde_json` / `--json` output mode for CLI.
- [ ] Evaluate `clap_complete` for shell completion generation.

### Phase 6: Apply Porting Playbook Best Practices — NOT DONE

Review each document in `attic/rust-porting-playbook/` and verify all best practices
have been applied to the current codebase.

#### Guidelines (6 documents)

- [ ] `python-to-rust-cli-porting.md` — Verify CLI patterns: SIGPIPE handling, exit
  codes, color detection, argument structure
- [ ] `python-to-rust-porting-rules.md` — Verify type mappings, dependency choices,
  error handling patterns, acceptance criteria
- [ ] `rust-cli-app-patterns.md` — Verify error handling (anyhow vs color-eyre),
  logging setup, version strings, atomic file writes
- [ ] `rust-general-rules.md` — Verify Edition 2024, `LazyLock` (not `once_cell`),
  `resolver = "3"`, testing organization
- [ ] `rust-project-setup.md` — Verify Cargo.toml settings, CI configuration, release
  profiles, clippy configuration, cargo-deny
- [ ] `test-coverage-for-porting.md` — Verify test coverage level, coverage tools,
  fixture management, cross-validation approach

#### Reference Documents (11 documents)

- [ ] `python-to-rust-playbook.md` — Verify 8-phase methodology was followed
- [ ] `python-to-rust-mapping-reference.md` — Verify type mappings, error handling,
  string handling
- [ ] `python-to-rust-porting-guide.md` — Verify version tracking, automation scripts
- [ ] `rust-cli-best-practices.md` — Verify CI/CD, clippy, release workflow,
  cross-compilation
- [ ] `rust-code-review-checklist.md` — Apply review checklist to current codebase
  (correctness, safety, performance, style, testing, documentation)
- [ ] `python-to-rust-test-coverage-playbook.md` — Run coverage tools, compare against
  90%+ target
- [ ] `port-checklist-initial-template.md` — Verify completion gates were met
- [ ] `port-checklist-update-template.md` — Verify sync tracking is in place
- [ ] `case-study-observations-template.md` — Record observations from this port
- [ ] `case-study-improvement-triage-template.md` — Triage observations into fixes

#### Case Study Documents (7 documents)

- [ ] `flowmark-port-analysis.md` — Verify metrics (test count, LOC, performance)
- [ ] `flowmark-port-comrak-bug.md` — Check if comrak fence bug still applies;
  verify workaround status
- [ ] `flowmark-port-cross-validation.md` — Verify workaround functions documented
  with `XXX:` comments
- [ ] `flowmark-port-decision-log.md` — Verify all 10 decisions (D1-D10) against
  current implementation
- [ ] `flowmark-port-library-choices.md` — Verify evaluation methodology was followed;
  check all 14 workarounds
- [ ] `flowmark-port-migration-plan.md` — Verify dependencies, comrak config, API
  surface, test organization
- [ ] `flowmark-port-wrapping-solution.md` — Verify wrapping approach in use

#### Playbook Review Fixes

- [ ] `plan-2026-02-08-playbook-review-fixes.md` — 53+ specific fixes identified.
  Verify critical ones against the codebase: `serde_yaml` vs `serde_yaml_ng`,
  `resolver` version, `LazyLock` vs `once_cell`, GitHub Actions versions.

### Phase 7: Meta-Playbook Review — NOT DONE

Follow the process in `attic/rust-porting-playbook/reference/meta-improving-this-playbook.md`:

- [ ] **Phase A: Conduct** — Record structured observations (OBS-N format) from the
  flowmark port experience, covering what worked, what didn't, and what the playbook
  missed or got wrong.
- [ ] **Phase B: Extract** — Triage observations into FIX/ADD/CLARIFY/GENERALIZE/VALIDATE
  categories using the improvement triage template.
- [ ] **Phase C: Integrate** — Apply approved changes back into the porting playbook
  documents so the next port benefits from these learnings.

### Phase 8: Final Verification — NOT DONE

- [ ] Zero `#[ignore]` tests (`cargo test` shows 0 ignored)
- [ ] Zero `partial` mappings in `test-mapping.yaml`
- [ ] `flowmark-dev check-mapping` exits with code 0
- [ ] All Rust tests pass
- [ ] Golden/reference document test produces identical output
- [ ] All porting playbook best practices verified or documented as not applicable
- [ ] Update this spec status to "Complete"

## Open Questions

These are decisions where multiple approaches exist and we need to choose:

1. **fmr-5ojk fix approach**: Comrak parses indented HTML comments as block-level
   `HtmlBlock` nodes. Python avoids this by disabling block HTML parsing entirely.
   Three possible approaches for Rust:
   - **(A) AST transformation**: After parsing, walk the AST and convert `HtmlBlock`
     nodes within list items into `HtmlInline` appended to the preceding paragraph.
     Most closely matches Python behavior.
   - **(B) Suppress spacing**: Modify `render_list_item` to not add blank lines before
     short `HtmlBlock` children (type 2 = HTML comment) in list items. Simpler but
     may not cover all cases.
   - **(C) Comrak configuration**: Check if comrak has options to disable block HTML
     parsing (likely not available).
   Which approach is most robust?

2. **Comrak version**: Should we upgrade from 0.36 to 0.47? The previous implementation
   used 0.47. Benefits: potential bug fixes, newer CommonMark spec. Risks: may change
   rendering behavior and require new workarounds. Need to evaluate.

3. **Property-based testing**: The previous implementation used `proptest`. Should we
   adopt it for fuzzing edge cases? Would help catch regressions but adds complexity.

4. **Upstream Python bugs**: If any of the 3 bugs turn out to reflect Python bugs
   (e.g., Python preserving escapes where the CommonMark spec says they should be
   removed), should we match the Python bug for parity and track the upstream fix
   separately, or fix both simultaneously?

5. **`doc_transforms.py` coverage**: This module's status is "Review needed" in the
   module mapping. Need to determine if there are untested transform paths.

6. **Trait-based vs function-based word splitting**: The previous implementation used a
   `WordSplitter` trait; the current uses closures. Is the trait approach better for
   extensibility, or is the closure approach sufficient?

## Exclusions

The following Python test files are **excluded** from parity — they test Python-specific
infrastructure that does not apply to the Rust port:

| File | # Tests | Reason |
|---|---|---|
| `test_cli_file_discovery.py` | 18 | Python CLI arg handling, `--auto` mode, stdin |
| `test_config.py` | 18 | Python TOML config, pyproject.toml, config merging |
| `test_file_resolver.py` | 28 | Python file glob, gitignore, exclude patterns |
| `test_skill.py` | 9 | Python skill/plugin system for Claude Code |

Total excluded: **73 tests** (+ 6 infra-only tests in other files). These are documented
in `test-mapping.yaml` with `status: excluded` and individual notes.

## References

- Test mapping infrastructure spec:
  `docs/project/specs/active/plan-2026-02-17-test-mapping-meta-test.md`
- YAML artifacts: `port-coverage-mapping/`
- Original Python repo: https://github.com/jlevy/flowmark (pinned: `v0.6.4`)
- Porting plan: `docs/porting-plan.md`
- Previous Rust implementation: `attic/flowmark-rs-1/`
- Porting playbook: `attic/rust-porting-playbook/`
- Meta-playbook (improving the playbook): `attic/rust-porting-playbook/reference/meta-improving-this-playbook.md`
