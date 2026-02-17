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
- Meta-playbook (improving the playbook): `attic/rust-porting-playbook/reference/meta-improving-this-playbook.md`
