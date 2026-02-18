# Feature: Exact Cross-Language Parity (flowmark Python → Rust)

**Date:** 2026-02-17 (last updated 2026-02-17)

**Author:** Joshua Levy

**Status:** Complete

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
**Rust flowmark-rs**: 250 test functions (223 integration + 27 unit). 250 passing,
0 `#[ignore]`d, 0 failures, 0 `partial` mappings. 202 mapped + 79 excluded in test
mapping.

### Bugs Blocking Parity

**All 3 Rust implementation bugs have been fixed. All 4 previously ignored tests now
pass.**

| Bug ID | Tests Affected | Summary | Status |
|---|---|---|---|
| **fmr-2tll** | `test_escape_in_list_item_start_preserved`, `test_mixed_escapes` | `- 1\. text` loses backslash | **FIXED** — `postprocess_period_escapes` now strips list markers before checking for digit-period patterns |
| **fmr-4l1x** | `test_heading_with_hard_break_in_list` | Extra blank line before heading in list item | **FIXED** — added `child_is_hard_break_heading` check to `render_list_item` spacing logic |
| **fmr-5ojk** | `test_list_item_with_tag_on_continuation_line` | Extra blank line before HTML comment tag on list continuation | **FIXED** — approach B: detect tag-only HTML blocks and suppress blank line before them in list items |

### Partial Test Coverage Gap

| Mapping | Gap | Status |
|---|---|---|
| `test_other_escaped_chars` | Rust covers `\*`, `\#`, `\-` plus `\$`, `\_`, `\[`/`\]`, `` \` `` | **DONE** — all assertions added, mapping updated to `mapped` |

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
| **Mapped** | 202 | Direct Rust equivalent exists and verified |
| **Excluded** | 79 | Infrastructure-only, not applicable to Rust |
| **Missing** | 0 | All ported |
| **Partial** | 0 | All completed |

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
2. **Port missing tests** — DONE. 64 tests ported.
3. **Fix all bugs** — DONE. Fixed all 3 Rust bugs, un-ignored all 4 tests.
4. **Code quality & cleanup** — DONE. Completed partial test, fixed all 70 clippy warnings.
5. **Review previous implementation** — DONE. Current architecture validated. 7 edge
   case tests added. Comrak, proptest, WordSplitter, --json, clap_complete evaluated.
6. **Apply porting playbook best practices** — DONE. 32 items found (8 P1, 14 P2,
   10 P3). All P1 applied, key P2 items applied, remaining tracked as beads.
7. **Meta-playbook review** — DONE (Phases A+B). 13 observations recorded and triaged.
   Phase C (integrating changes into playbook) pending human review.
8. **Final verification** — DONE. 250 tests, 0 ignored, 0 partial, zero warnings,
   check-mapping PASS, golden test PASS.

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

### Phase 3: Fix Remaining Test Failures — DONE

All 3 bugs fixed, all 4 previously ignored tests un-ignored and passing.

- [x] **fmr-2tll**: Fixed escape at start of list item content
  - Fix: `postprocess_period_escapes()` now strips list markers (`- `, `* `, `+ `)
    before checking for digit-period patterns, matching blockquote marker stripping.
  - 2 tests un-ignored and passing.
- [x] **fmr-4l1x**: Fixed extra blank line before heading in list item with hard break
  - Fix: added `child_is_hard_break_heading` check to `render_list_item()` spacing
    logic, matching the existing check in `render_block_children()`.
  - 1 test un-ignored and passing.
- [x] **fmr-5ojk**: Fixed extra blank line before HTML comment tag on list continuation
  - Fix: approach B — detect tag-only HTML blocks (e.g., `<!-- #id -->`) and suppress
    blank line before them in list items.
  - 1 test un-ignored and passing.

### Phase 4: Code Quality & Cleanup — DONE

- [x] **test_other_escaped_chars**: Added `\$`, `\_`, `\[`/`\]`, `` \` `` assertions.
  Updated `test-mapping.yaml` status from `partial` to `mapped`.
- [x] **Clippy warnings**: Fixed all 70 clippy warnings. Zero warnings now.

### Phase 5: Review Previous Implementation (`attic/flowmark-rs-1`) — DONE

Comprehensive review complete. Current architecture validated as superior.

Evaluation results:

- [x] **Comrak 0.36 → 0.47**: SKIP — already effectively using 0.47 features.
  Stale comment removed from Cargo.toml.
- [x] **Property-based testing (proptest)**: DEFER — old impl declared but never used it.
  Future enhancement for idempotency/width/round-trip properties.
- [x] **Trait-based WordSplitter**: SKIP — current function-based approach is simpler
  and more composable.
- [x] **Post-processing edge cases**: DONE — 7 edge case tests added covering code
  fences, math LaTeX, bare dollars, footnotes. All pass without code changes.
- [x] **`--json` output mode**: SKIP — never implemented in old impl either.
- [x] **`clap_complete` shell completions**: DEFER — low effort but CLI still evolving.

### Phase 6: Apply Porting Playbook Best Practices — DONE

All 27 playbook documents reviewed. 32 actionable items identified and addressed:

**P1 fixes applied (8 items):**
- [x] SIGPIPE handling added (`libc::signal`)
- [x] `main()` returns `ExitCode` (not `process::exit()`)
- [x] `Box<dyn Error>` replaced with `anyhow::Result`
- [x] Unused `color-eyre`/`tracing`/`tracing-subscriber` deps removed
- [x] Atomic file writes via `tempfile::NamedTempFile::persist()`
- [x] CI workflow overhauled: 8 parallel jobs (fmt, clippy, test matrix, lib-only,
  MSRV, deny, docs, check-mapping)
- [x] `deny.toml` created with license allowlist and source restrictions
- [x] Error message format: lowercase "error:" with `{e:#}` chain display

**P2 fixes applied (6 items):**
- [x] `rustfmt.toml` created (edition 2024, max_width 100)
- [x] `pub(crate)` visibility for all internal APIs (~50 items changed)
- [x] Dead code removed (4 unused functions, 1 unused constant)
- [x] Cargo.toml metadata: keywords, categories
- [x] Release profile: `opt-level = 3`, `panic = "abort"`
- [x] Edge case tests: 7 tests from old impl review

**Remaining P2/P3 items tracked as open beads for future work:**
- justfile, release workflow, README, CHANGELOG, assert_cmd, ValueEnum, color flag,
  BufWriter, unwrap→expect, property tests.

### Phase 7: Meta-Playbook Review — DONE (Phases A+B)

13 structured observations recorded in
`attic/rust-porting-playbook/case-studies/flowmark/flowmark-port-observations-2.md`:

- [x] **Phase A**: 13 observations recorded (OBS-1 through OBS-13)
- [x] **Phase B**: Triaged: 1 FIX, 2 CLARIFY, 5 ADD, 5 VALIDATE
- [ ] **Phase C**: Integrate — requires human review before applying changes to
  playbook documents.

### Phase 8: Final Verification — DONE

- [x] Zero `#[ignore]` tests — `cargo test`: 250 passed, 0 failed, 0 ignored
- [x] Zero `partial` mappings in `test-mapping.yaml` — 202 mapped, 0 partial
- [x] `flowmark-dev check-mapping` exits with code 0 — PASS
- [x] All Rust tests pass — 250 tests across 20 test suites
- [x] Golden/reference document test produces identical output
- [x] All porting playbook best practices verified (32 items, P1 all applied)
- [x] Zero clippy warnings
- [x] No-default-features build succeeds

## Open Questions

These are decisions where multiple approaches exist and we need to choose:

1. ~~**fmr-5ojk fix approach**~~: **RESOLVED** — approach B was chosen: detect tag-only
   HTML blocks and suppress blank line before them in list items. This was simpler than
   AST transformation (approach A) and sufficient for the cases encountered.

2. ~~**Comrak version**~~: **RESOLVED** — SKIP. Already effectively using 0.47 features.
   Stale Cargo.toml comment removed.

3. ~~**Property-based testing**~~: **RESOLVED** — DEFER. Old impl declared proptest but
   never used it. Future enhancement for idempotency/width/round-trip properties.

4. ~~**Upstream Python bugs**~~: **RESOLVED** — all 3 bugs were confirmed as Rust
   implementation bugs, not upstream Python bugs. All fixed in the Rust codebase.

5. ~~**`doc_transforms.py` coverage**~~: **RESOLVED** — transform functionality is
   covered via integration tests that exercise cleanups. No untested paths found.

6. ~~**Trait-based vs function-based word splitting**~~: **RESOLVED** — SKIP. Current
   function-based approach is simpler, more composable, and has broader coverage than
   the old trait-based approach.

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
- Meta-playbook (improving the playbook):
  `attic/rust-porting-playbook/reference/meta-improving-this-playbook.md`

---

## Appendix A: Full Commit Log and Porting Synopsis

This appendix provides a complete record of the flowmark Python-to-Rust porting effort
as captured in the branch commit history (25 substantive commits, excluding tbd
bookkeeping). Each commit is annotated with what changed, the test state at that point,
and the architectural significance.

### Phase 0: Core Implementation (2 commits)

#### `f245a4b` — Initial Rust implementation of flowmark

The project scaffold: Cargo.toml, module structure, CLI with clap (feature-gated), and
all core library modules ported from Python. Established the module layout that persisted
through the entire port:

| Python Module | Rust Module |
|---|---|
| `linewrapping/text_wrapping.py` | `wrapping/text_wrapping.rs` |
| `linewrapping/text_filling.py` | `wrapping/text_filling.rs` |
| `linewrapping/sentence_split_regex.py` | `wrapping/sentence.rs` |
| `linewrapping/line_wrappers.py` | `wrapping/line_wrappers.rs` |
| `linewrapping/tag_handling.py` | `wrapping/tag_handling.rs` |
| `linewrapping/block_heuristics.py` | `wrapping/block_heuristics.rs` |
| `linewrapping/atomic_patterns.py` | `wrapping/atomic_patterns.rs` |
| `formats/flowmark_markdown.py` | `formatter/filling.rs` |
| `formats/frontmatter.py` | `parser/frontmatter.rs` |
| `typography/ellipses.py` | `typography/ellipses.rs` |
| `typography/smartquotes.py` | `typography/quotes.rs` |
| `transforms/doc_cleanups.py` | `transform/cleanups.rs` |
| `reformat_api.py` | `lib.rs` |
| `config.py` | `config.rs` |
| `cli.py` | `main.rs` |

**Tests: 27 unit tests passing.** No integration tests yet.

#### `0e45b63` — Complete Markdown formatting pipeline with 177 passing tests

The largest single commit (11,262 lines added). Rewrote `render_node` as a proper
block/inline AST renderer with blank line separation, blockquote/alert paragraph spacing,
and list item formatting. Key innovations:

- Unicode PUA placeholder system for preserving escape characters (`\*`, `\#`, `\-`,
  etc.) through comrak's AST, which strips backslash escapes.
- Code-span-aware period escape post-processing.
- Smart quotes applied at the paragraph level to work across inline elements.
- Code fence blank line preservation.
- Sentence-break detection after links/parens.

Added 16 integration test files with golden/reference document tests (4 modes: plain,
cleaned, semantic, auto) on a 1,416-line reference document.

**Tests: 177 passing (150 integration + 27 unit) across 16 integration test files.**

### Phase 1: Test Mapping Infrastructure (7 commits)

#### `af0784f` — Add plan spec for cross-language test mapping meta-test

Spec-only commit. Designed the systematic test provenance tracking system: Python
discovery scripts (AST-based), Rust discovery, hand-maintained YAML mapping, and
completeness checker.

#### `44a0143` — Add port coverage mapping: Python CLI, discovery scripts, and YAML

Implemented the `flowmark-dev` Python CLI with four subcommands: `discover-python` (281
tests found at v0.6.4), `discover-rust` (151 integration tests found via regex),
`init-mapping` (skeleton mapping), `check-mapping` (completeness validation). Generated
initial YAML artifacts. 14 files added.

#### `a1648d9` — Update test mapping spec: cargo-based discovery, idempotent merge

Spec update: switched Rust discovery to `cargo test -- --list` (compiler-authoritative,
finds all 178 tests including 27 unit tests). Documented idempotent additive merge
semantics for all commands.

#### `be6598c` — Implement cargo-based discovery, idempotent merge, and lint fixes

Implementation of the spec updates: `discover-rust` now uses cargo as primary strategy
(178 tests: 151 integration + 27 unit), with regex fallback. Both discovery commands
preserve hand-added YAML entries. All Python code passes ruff and basedpyright.

#### `1753a59` — Populate test-mapping.yaml with exact gap counts

All 281 Python tests reviewed and mapped: 137 mapped, 79 excluded, 64 missing, 1
partial. This commit established the precise gap that needed closing.

#### `a5b1b3f` — Add exact parity spec

Created this spec document, outlining the full roadmap from "64 missing tests" to "exact
behavioral parity."

#### `4f05cbe` — Finalize both specs: mark completed phases

Updated both specs to reflect completed mapping work. The 64 missing tests enumerated by
file with exact counts.

### Phase 2: Test Infrastructure and CI (3 commits)

#### `25e8c1a` — TDD smoke tests for cross-language test mapping

Added 9 Python smoke tests validating the dev-tools pipeline end-to-end: YAML round-trip
serialization, discovery counts, and mapping completeness checks.

#### `a4a13b6` — Enforce deterministic YAML serialization

Moved record sorting into `write_*_yaml()` functions for canonical ordering. Added
`TestYamlDeterminism` suite verifying stable output and checked-in files match canonical
form.

#### `616859a` — CI: GitHub Actions workflow

Initial CI with Rust tests (cargo test with caching) and check-mapping (Python smoke
tests as hard gate, completeness check informational).

### Phase 3: Porting the 64 Missing Tests (3 commits)

#### `7c2b3bf` — Port 64 missing Python tests to Rust (61 pass, 4 known bugs)

The second-largest commit. All 64 missing tests ported:

- 32 wrapping tests (all pass)
- 15 tag formatting tests (1 ignored: fmr-5ojk)
- 5 escape handling tests (2 ignored: fmr-2tll)
- 5 smartquotes integration tests (all pass)
- 2 alert, 2 strikethrough, 1 heading spacing, 1 code block, 1 width options

3 Rust implementation bugs identified and tracked: fmr-2tll (escape at list item start),
fmr-4l1x (extra blank line before heading in list), fmr-5ojk (extra blank line before
HTML comment tag on list continuation).

**Tests: 243 passing, 4 ignored (known bugs).**

#### `881a30a` — Achieve full test mapping (0 missing)

Updated test-mapping.yaml: all 64 entries changed from `missing` to `mapped`.
Regenerated rust-tests.yaml. Mapping: 201 mapped, 79 excluded, 1 partial, 0 missing.

#### `b74c856` / `7fdc5bd` — Spec status corrections

Spec initially marked "Implemented" prematurely, then corrected to "In Progress" with
addition of Phases 3-8 covering bug fixes, code quality, previous impl review, playbook
audit, meta-playbook review, and final verification. 18 new beads created.

### Phase 4: Bug Fixes and Code Quality (3 commits)

#### `5b64d8c` — Fix 3 bugs and complete partial test

All 3 blocking bugs fixed:

- **fmr-2tll**: `postprocess_period_escapes` now strips list markers before checking for
  digit-period patterns.
- **fmr-4l1x**: Added `child_is_hard_break_heading` check to `render_list_item` spacing.
- **fmr-5ojk**: Detect tag-only HTML blocks and suppress blank line before them in list
  items.
- **fmr-p2pr**: Completed `test_other_escaped_chars` with full escape assertions.

Updated golden test files. All 4 previously ignored tests now passing.

**Tests: 247 passing, 0 ignored, 0 failures.**

#### `e544c7e` — Fix all 70 clippy warnings across 15 files

Zero-warning build achieved. Key changes: `push_str(&format!())` to `write!`/`writeln!`,
doc-comment backticking, `repeat_n()`, `is_some_and()`, collapsed nested `if`s, raw
string literals. Updated mapping: `test_other_escaped_chars` from `partial` to `mapped`.

**Tests: 250 passing, 0 ignored. Mapping: 202 mapped, 0 partial.**

#### `06a43b3` — Update parity spec (Phases 3-4 DONE) and refresh rust-tests.yaml

Spec and YAML artifacts updated to reflect bug fixes and code quality work.

### Phase 5: Porting Playbook Best Practices (4 commits)

#### `a9762b3` — P1: Refactor main() with anyhow, ExitCode, SIGPIPE; atomic writes

- `anyhow::Result` replacing `Box<dyn Error>`
- `ExitCode` from `main()` (runs destructors properly)
- SIGPIPE reset on Unix via `libc` (piping to `head`/`grep` works)
- `tempfile::NamedTempFile::persist()` for atomic file writes
- Removed unused `color-eyre`, `tracing`, `tracing-subscriber` deps
- Standard `eprintln!("error: {e:#}")` format

#### `a8567e1` — P1: Overhaul CI workflow, add deny.toml and project config

CI expanded from 2 to 8 parallel jobs: fmt, clippy, test (ubuntu + macOS matrix),
test-lib-only, MSRV (1.85), cargo-deny, docs, check-mapping. Added `deny.toml` (license
allowlist, source restrictions), `rustfmt.toml` (edition 2024, max_width 100), Cargo.toml
metadata (keywords, categories).

#### `c4905d2` — P2: Add edge case tests from previous implementation review

7 tests covering edge cases from reviewing `attic/flowmark-rs-1`: code fence with
indented YAML/list content (comrak parse edge case), inline and display math with LaTeX
backslashes, bare dollar signs, code block content preservation, footnote
references/definitions. All passed without code changes, validating the current renderer.

**Tests: 250 passing.**

#### `8b6e33b` — P2: Restrict visibility with pub(crate) and remove dead code

~50 items changed from `pub` to `pub(crate)`. Removed 4 unused functions and 1 unused
constant. Public API (re-exported from lib.rs) unchanged.

### Phase 6: Finalization (2 commits)

#### `111ca3a` — Mark exact parity spec as Complete — all 8 phases done

Final spec update: 250 tests, 0 ignored, 0 partial, check-mapping PASS, golden test
PASS, zero clippy warnings. All open questions resolved.

#### `100b2cd` — P3: CLI polish and replace unwrap() with expect()

Final polish: `ValueEnum` derive for `ListSpacing` (rich `--help`), `BufWriter` for
stdout, `--verbose` flag, all 33 `unwrap()` in library code replaced with descriptive
`expect()` messages.

### Test Count Evolution

| Commit | Tests | Ignored | Status |
|---|---|---|---|
| `f245a4b` Initial impl | 27 | 0 | Unit tests only |
| `0e45b63` Pipeline complete | 177 | 0 | +16 integration test files |
| `7c2b3bf` Port 64 missing | 243 | 4 | 3 bugs found |
| `5b64d8c` Fix 3 bugs | 247 | 0 | All bugs fixed |
| `e544c7e` Clippy cleanup | 250 | 0 | +3 during cleanup |
| `c4905d2` Edge case tests | 250 | 0 | +7 (replaced 7 unused) |
| `100b2cd` Final polish | **250** | **0** | **Final state** |

### Mapping Status Evolution

| Commit | Mapped | Excluded | Missing | Partial |
|---|---|---|---|---|
| `1753a59` Initial mapping | 137 | 79 | 64 | 1 |
| `881a30a` Full mapping | 201 | 79 | 0 | 1 |
| `e544c7e` Partial resolved | **202** | **79** | **0** | **0** |

---

## Appendix B: Current Test Suite Catalog

### Codebase Size

**Library source (`src/`):** 3,485 lines across 21 files.
**Integration tests (`tests/`):** 3,424 lines across 17 test files + golden test docs.

#### Source Files by Size

| Source File | Lines | Description |
|---|---|---|
| `formatter/filling.rs` | 1,270 | Core Markdown filling/normalization pipeline |
| `wrapping/tag_handling.rs` | 387 | Jinja/Markdoc/HTML tag handling |
| `wrapping/text_wrapping.rs` | 290 | Word splitting and paragraph wrapping |
| `typography/quotes.rs` | 189 | Smart quote conversion |
| `wrapping/line_wrappers.rs` | 170 | Line wrapper factory functions |
| `main.rs` | 158 | CLI entry point (feature-gated) |
| `wrapping/text_filling.rs` | 143 | Multi-paragraph text filling |
| `lib.rs` | 127 | Public API: `reformat_text`, `reformat_file` |
| `wrapping/atomic_patterns.rs` | 120 | Atomic construct regex patterns |
| `wrapping/block_heuristics.rs` | 111 | Block content detection heuristics |
| `wrapping/sentence.rs` | 111 | Sentence splitting regex |
| `typography/ellipses.rs` | 109 | Ellipsis conversion |
| `parser/frontmatter.rs` | 80 | YAML frontmatter parsing |
| `transform/cleanups.rs` | 69 | Document transforms (unbold headings) |
| `formatter/markdown.rs` | 57 | Comrak options and AST rendering helpers |
| `config.rs` | 48 | Configuration types |
| `error.rs` | 17 | Error type definitions |

### Integration Tests (223 tests across 17 files)

| Test File | Tests | Lines | Coverage Area |
|---|---|---|---|
| `test_wrapping.rs` | 47 | 944 | Word splitting, paragraph wrapping, tag handling, block heuristics |
| `test_tag_formatting.rs` | 32 | 319 | Jinja/HTML tag normalization, multiline tags, smart quotes with tags |
| `test_smartquotes.rs` | 28 | 409 | Smart quote conversion, apostrophes, edge cases |
| `test_list_spacing.rs` | 19 | 256 | Tight/loose/preserve list spacing modes |
| `test_alerts.rs` | 15 | 160 | GitHub-flavored Markdown alert blocks |
| `test_escape_handling.rs` | 13 | 144 | Backslash escape preservation across contexts |
| `test_fenced_code_blocks.rs` | 12 | 130 | Code fence preservation, nesting, tilde fences |
| `test_strikethrough.rs` | 11 | 80 | Strikethrough formatting |
| `test_ellipses.rs` | 10 | 128 | Ellipsis conversion |
| `test_heading_spacing.rs` | 9 | 79 | Blank lines around headings, hard breaks |
| `test_edge_cases.rs` | 7 | 148 | Math LaTeX, footnotes, code fence edge cases |
| `test_frontmatter.rs` | 7 | 74 | YAML frontmatter split/preserve |
| `test_width_options.rs` | 7 | 85 | Width=0, semantic+width=0, normal width |
| `test_filling.rs` | 2 | 268 | Multi-paragraph list items, normalize pipeline |
| `test_sentences.rs` | 2 | 26 | Sentence splitting, first sentence extraction |
| `test_ref_docs.rs` | 1 | 108 | Golden test: 4 formatting modes on reference doc |
| `test_cleanups.rs` | 1 | 66 | Unbold headings transform |

### Unit Tests (27 tests across 7 modules in `src/`)

| Module | Tests | Coverage |
|---|---|---|
| `typography::quotes` | 6 | Double/single quotes, apostrophes, possessives, code-like, tags |
| `wrapping::text_wrapping` | 5 | Markdown escaping, HTML/MD word split, simple wrapping, no-wrap |
| `parser::frontmatter` | 4 | `has_frontmatter`, `split_frontmatter` (present/absent/unclosed) |
| `wrapping::block_heuristics` | 4 | Table rows, ordered/unordered list items, block content |
| `typography::ellipses` | 4 | Basic, end-of-line, with space, with punctuation |
| `wrapping::sentence` | 3 | End-of-sentence heuristic, splitting, first sentence |
| `wrapping::text_filling` | 1 | Paragraph splitting |

### Golden/Reference Document Test

`test_ref_docs.rs` runs the full formatting pipeline on a substantial reference document
(`tests/testdocs/testdoc.orig.md`, 1,416 lines) in 4 modes and compares against expected
output files:

| Mode | Expected File | Options |
|---|---|---|
| Plain text | `testdoc.expected.plain.md` | `plaintext=true` |
| Cleaned | `testdoc.expected.cleaned.md` | `cleanups=true` |
| Semantic | `testdoc.expected.semantic.md` | `semantic=true, cleanups=true` |
| Auto | `testdoc.expected.auto.md` | `semantic+cleanups+smartquotes+ellipses` |

### CI Pipeline (8 parallel jobs)

| Job | What It Checks |
|---|---|
| **fmt** | `cargo fmt --check` — code formatting |
| **clippy** | `cargo clippy -D warnings` — lint (pedantic enabled) |
| **test** | `cargo test --locked` on ubuntu + macOS |
| **test-lib-only** | `cargo test --no-default-features` — library without CLI |
| **msrv** | `cargo check` with Rust 1.85 — minimum supported version |
| **deny** | `cargo-deny` — license allowlist, source restrictions |
| **docs** | `cargo doc -D warnings` — documentation builds clean |
| **check-mapping** | Python smoke tests + cross-language mapping completeness |

### Cross-Language Mapping Final Summary

| Status | Count | Description |
|---|---|---|
| **Mapped** | 202 | Python test has a verified Rust equivalent |
| **Excluded** | 79 | Infrastructure-only (CLI, config, file resolver, skill system) |
| **Missing** | 0 | All gaps closed |
| **Partial** | 0 | All partial coverage completed |
| **Extra Rust** | 27 | Unit tests in `src/` not mapped to Python (Rust-native) |

**Total: 250 Rust tests covering 202 Python test behaviors, 27 Rust-specific unit tests,
and 7 edge case tests from previous implementation review. All passing, zero ignored.**

---

## Appendix C: Lines of Code — Python vs Rust

Comprehensive line counts for the original Python flowmark v0.6.4 and the Rust port.
"Code lines" excludes blank lines, comments, and docstrings/doc comments.

### Python (Original)

| Category | Total | Blank | Comments | Docstrings | Code |
|---|---:|---:|---:|---:|---:|
| Library (`src/flowmark/`, 26 files) | 4,433 | 626 | 355 | 921 | 2,531 |
| Tests (`tests/`, 20 files) | 5,619 | 1,085 | 346 | 1,440 | 2,748 |
| **Combined** | **10,052** | **1,711** | **701** | **2,361** | **5,279** |

Largest Python library files: `flowmark_markdown.py` (727 total / 421 code),
`tag_handling.py` (531 / 255), `cli.py` (526 / 385), `text_wrapping.py` (258 / 134).

### Rust (Port)

| Category | Total | Blank | Comments | Doc comments | Code |
|---|---:|---:|---:|---:|---:|
| Library (`src/`, 22 files, excl. unit tests) | 3,221 | — | — | — | — |
| Unit tests in `src/` (`#[cfg(test)]`, 7 files) | 264 | — | — | — | — |
| All of `src/` | 3,485 | 450 | 171 | 254 | 2,610 |
| Integration tests (`tests/`, 17 files) | 3,424 | 599 | 146 | 5 | 2,674 |
| **Combined** | **6,909** | **1,049** | **317** | **259** | **5,284** |

Largest Rust library files: `filling.rs` (1,270 total), `tag_handling.rs` (387),
`text_wrapping.rs` (290), `quotes.rs` (189), `line_wrappers.rs` (170).

### Comparison

| Metric | Python | Rust | Ratio |
|---|---:|---:|---|
| **Library total lines** | 4,433 | 3,485 | 0.79x |
| **Library code lines** | 2,531 | 2,610 | 1.03x |
| **Test total lines** | 5,619 | 3,688 | 0.66x |
| **Test code lines** | 2,748 | 2,674 | 0.97x |
| **Combined total lines** | 10,052 | 6,909 | 0.69x |
| **Combined code lines** | 5,279 | 5,284 | 1.00x |
| Library files | 26 | 22 | — |
| Test files | 20 | 17 | — |
| Test functions | 281 | 250 | — |
| Tests ported (mapped) | — | 202 | 72% of Python |
| Tests excluded (infra) | — | 79 | 28% of Python |

### Observations

- **Code lines are essentially identical** (5,279 Python vs 5,284 Rust) — a 1:1 ratio.
  The port neither expanded nor compressed the logic.
- **Total lines are 31% smaller in Rust** (6,909 vs 10,052) because Python has
  significantly more docstrings (2,361 lines, 23% of total) and comments (701 lines)
  compared to Rust's doc comments (259 lines) and comments (317 lines).
- **Test suite is 34% smaller by total lines** but nearly identical by code lines,
  again due to Python's heavy use of triple-quoted docstrings for test fixtures vs
  Rust's raw string literals which are more compact.
- **79 Python tests (28%) were excluded** as infrastructure-only (CLI file discovery,
  config, file resolver, skill system). The Rust port covers all behavioral tests.
- The Rust codebase has **zero `#[ignore]` tests, zero clippy warnings, and zero
  `unwrap()` calls** in library code.
