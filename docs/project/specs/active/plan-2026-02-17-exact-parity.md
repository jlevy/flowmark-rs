# Feature: Exact Cross-Language Parity (flowmark Python → Rust)

**Date:** 2026-02-17 (last updated 2026-02-19)

**Author:** Joshua Levy

**Status:** COMPLETE — All formatting parity gaps P1-P9 and D1-D15 are resolved. Exact
byte-for-byte parity achieved across all 4 modes (auto, tight, loose, plaintext). 481
tests pass, 0 ignored. See Appendix E for full resolution details.

**Epic bead:** fmr-kd36

## Overview

**flowmark-rs is a drop-in replacement for Python flowmark.** Any human or agent using
`flowmark` (Python) must be able to switch to `flowmark` (Rust) with zero changes to
their workflow, flags, config files, or expectations.
The Rust binary produces identical output, accepts identical flags, reads the same
config files, discovers the same files, and installs the same Claude Code skill.

Use the cross-language test mapping system (see
`plan-2026-02-17-test-mapping-meta-test.md`) as the source of truth for tracking
coverage.

**“Exact parity” means: every Python test has a passing Rust test that verifies the same
behavior. No test may be `#[ignore]`d. No `partial` mappings.
No exclusions.
Every discrepancy is either a Rust bug to fix or an upstream Python bug to
fix (and then match in Rust).**

## Key Principle: Every Parity Gap Must Have a Failing Test

**A hidden gap is worse than a known failure.** Every known behavioral difference
between the Rust and Python binaries MUST be surfaced as a failing test.
Tests that use `head`, `tail`, `[..]`, `basename | sort`, `grep -c`, or other
output-masking patterns to hide real differences are not acceptable.

If a parity gap exists and cannot be immediately fixed, the correct approach is:

1. **Write a test that fails** — the test should assert the correct (Python-matching)
   behavior.
2. **Mark the test as `#[ignore]`** with a comment linking to the tracking bead — but
   only if the fix is blocked.
   Prefer fixing immediately.
3. **Track the gap in Appendix E** of this spec with a bead, root cause analysis, and
   fix plan.

**Never mask a difference to make a test pass.** A green test suite with hidden gaps is
worse than a red test suite with documented failures, because hidden gaps erode trust
and make the “drop-in replacement” claim false.

## Goals

- **Drop-in replacement**: `flowmark` (Rust) is fully interchangeable with `flowmark`
  (Python) — identical CLI interface, identical behavior, identical output.
- Zero `#[ignore]` tests.
  Every test passes.
- Zero `partial` test mappings.
  Every mapped test covers the full Python behavior.
- Zero `excluded` test mappings.
  Every Python test has a Rust equivalent.
- `flowmark-dev check-mapping` passes with exit code 0 (281 mapped, 0 excluded, 0
  missing, 0 partial).
- All Rust tests pass (`cargo test` with no ignored tests).
- Golden/reference document tests produce identical output to Python.
- Tryscript CLI golden tests pass for both Python and Rust binaries.
- **Any deviation in drop-in behavior is a bug** and must be surfaced as a CI failure.
- **Every known parity gap has a failing test** — no masking, no `head | tail` tricks,
  no approximate assertions that hide real differences.

## Non-Goals

- Performance benchmarking against the Python version.
- Adding features beyond what Python v0.6.4 supports.

**Scope clarification:** “Exact parity” means exact behavioral compatibility across
**every** feature of Python flowmark v0.6.4, with **no exceptions**:

- **Formatting behavior**: identical output for all formatting modes
- **CLI flags**: every Python flag has an equivalent Rust flag with identical behavior
- **File discovery**: directory recursion, glob expansion, gitignore, `.flowmarkignore`
- **Config loading**: `.flowmark.toml`, `flowmark.toml`,
  `pyproject.toml [tool.flowmark]`
- **Multi-file batch processing**: same files discovered, same output produced
- **Skill system**: `--skill`, `--install-skill`, `--docs` — Claude Code skill
  installation
- **Error handling**: same error messages and exit codes for all error paths
- **Logging and informational output**: all status messages, warnings, and verbose
  output must match

Every Python test — all 281 of them — must have a passing Rust equivalent.
Zero exclusions. Zero exceptions.

The only tolerated differences are trivial formatting details that don’t affect
substantive behavior and are hard to match exactly (e.g., auto-generated `--help` screen
layout differences between argparse and clap, or `usize` vs negative int for width).
All other behavior — including file paths, error messages, exit codes, and informational
output — must be identical.

**Test mapping rule:** Unit tests of internal infrastructure (e.g., config parsing
internals, file resolver implementation details) may be mapped to idiomatic Rust
equivalents — they don’t need to be literal translations.
But every Python test must have a corresponding Rust test in the test mapping.
No test may be left unmapped.

## Background

### Current State

**Python flowmark v0.6.4**: 281 test functions across 20 files.
**Rust flowmark-rs**: 251 test functions (223 integration + 27 unit + 1 doc-test).
251 passing, 0 `#[ignore]`d, 0 failures, 0 `partial` mappings.
202 mapped + 79 excluded in test mapping.

### Bugs Blocking Parity

**All 3 Rust implementation bugs have been fixed.
All 4 previously ignored tests now pass.**

| Bug ID | Tests Affected | Summary | Status |
| --- | --- | --- | --- |
| **fmr-2tll** | `test_escape_in_list_item_start_preserved`, `test_mixed_escapes` | `- 1\. text` loses backslash | **FIXED** — `postprocess_period_escapes` now strips list markers before checking for digit-period patterns |
| **fmr-4l1x** | `test_heading_with_hard_break_in_list` | Extra blank line before heading in list item | **FIXED** — added `child_is_hard_break_heading` check to `render_list_item` spacing logic |
| **fmr-5ojk** | `test_list_item_with_tag_on_continuation_line` | Extra blank line before HTML comment tag on list continuation | **FIXED** — approach B: detect tag-only HTML blocks and suppress blank line before them in list items |

### Partial Test Coverage Gap

| Mapping | Gap | Status |
| --- | --- | --- |
| `test_other_escaped_chars` | Rust covers `\*`, `\#`, `\-` plus `\$`, `\_`, `\[`/`\]`, `` \` `` | **DONE** — all assertions added, mapping updated to `mapped` |

### Python Module → Rust Module Mapping

| Python Module | Rust Module | Status |
| --- | --- | --- |
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
| `transforms/doc_transforms.py` | `transform/cleanups.rs` | Implemented (covered via integration tests) |
| `reformat_api.py` | `lib.rs` (`reformat_text`, `reformat_file`) | Implemented |
| `config.py` | `config.rs` | Partial — no TOML loading |
| `cli.py` | `main.rs` | Partial — missing file discovery flags |
| `file_resolver/` | Not yet ported | **Must port** — file discovery, gitignore, globs |
| `reformat_api.py` (`reformat_files`) | `lib.rs` | Partial — no multi-file batch |
| `skill.py` / `skills/` | Not yet ported | **Must port** — Claude Code skill installation (`--skill`, `--install-skill`, `--docs`) |

### Mapping Summary (from `check-mapping` output)

**Current (Phases 1-9):**

| Status | Count | Description |
| --- | --- | --- |
| **Mapped** | 202 | Direct Rust equivalent exists and verified |
| **Excluded** | 79 | Previously excluded — **all now in scope for Phase 10** |
| **Missing** | 0 | All behavioral tests ported |
| **Partial** | 0 | All completed |

**Target (after Phase 10):**

| Status | Count | Description |
| --- | --- | --- |
| **Mapped** | 281 | Every Python test has a verified Rust equivalent |
| **Excluded** | 0 | No exclusions — exact parity means every feature is ported |
| **Missing** | 0 | — |
| **Partial** | 0 | — |

**34 extra Rust tests** (unit tests in `src/` and Rust-specific edge case tests not
mapped to Python) — these are Rust-native tests, not gaps.

### Previously Excluded Files — Now In Scope (Phase 10)

All of these files must be ported for exact parity:

| File | # Tests | Feature |
| --- | --- | --- |
| `test_cli_file_discovery.py` | 19 | CLI arg handling, `--auto` mode, file discovery, error messages |
| `test_config.py` | 20 | TOML config loading, pyproject.toml, three-way merge |
| `test_file_resolver.py` | 31 | Directory recursion, glob expansion, gitignore, exclude patterns |
| `test_skill.py` | 9 | Claude Code skill installation (`--skill`, `--install-skill`, `--docs`) |

## Design

### Approach

The work is organized into phases:

1. **Test mapping** — DONE. All 281 Python tests have entries in `test-mapping.yaml`.
2. **Port missing tests** — DONE. 64 tests ported.
3. **Fix all bugs** — DONE. Fixed all 3 Rust bugs, un-ignored all 4 tests.
4. **Code quality & cleanup** — DONE. Completed partial test, fixed all 70 clippy
   warnings.
5. **Review previous implementation** — DONE. Current architecture validated.
   7 edge case tests added.
   Comrak, proptest, WordSplitter, --json, clap_complete evaluated.
6. **Apply porting playbook best practices** — DONE. 32 items found (8 P1, 14 P2, 10
   P3). All P1 applied, key P2 items applied, remaining tracked as beads.
7. **Meta-playbook review** — DONE (Phases A+B). 13 observations recorded and triaged.
   Phase C (integrating changes into playbook) pending human review.
8. **Final verification** — DONE. 250 tests, 0 ignored, 0 partial, zero warnings,
   check-mapping PASS, golden test PASS.
9. **CI hardening** — DONE. All mapping checks promoted from informational to hard
   gates. `rust-tests.yaml` refreshed to 250 entries.
   All 13 smoke tests enforced.

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
- [x] Port 7 scattered missing tests (alerts, strikethrough, heading, code blocks,
  width)
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
- [x] **Clippy warnings**: Fixed all 70 clippy warnings.
  Zero warnings now.

### Phase 5: Review Previous Implementation (`attic/flowmark-rs-1`) — DONE

Comprehensive review complete.
Current architecture validated as superior.

Evaluation results:

- [x] **Comrak 0.36 → 0.47**: SKIP — already effectively using 0.47 features.
  Stale comment removed from Cargo.toml.
- [x] **Property-based testing (proptest)**: DEFER — old impl declared but never used
  it. Future enhancement for idempotency/width/round-trip properties.
- [x] **Trait-based WordSplitter**: SKIP — current function-based approach is simpler
  and more composable.
- [x] **Post-processing edge cases**: DONE — 7 edge case tests added covering code
  fences, math LaTeX, bare dollars, footnotes.
  All pass without code changes.
- [x] **`--json` output mode**: SKIP — never implemented in old impl either.
- [x] **`clap_complete` shell completions**: DEFER — low effort but CLI still evolving.

### Phase 6: Apply Porting Playbook Best Practices — DONE

All 27 playbook documents reviewed.
32 actionable items identified and addressed:

**P1 fixes applied (8 items):**
- [x] SIGPIPE handling added (`libc::signal`)
- [x] `main()` returns `ExitCode` (not `process::exit()`)
- [x] `Box<dyn Error>` replaced with `anyhow::Result`
- [x] Unused `color-eyre`/`tracing`/`tracing-subscriber` deps removed
- [x] Atomic file writes via `tempfile::NamedTempFile::persist()`
- [x] CI workflow overhauled: 8 parallel jobs (fmt, clippy, test matrix, lib-only, MSRV,
  deny, docs, check-mapping)
- [x] `deny.toml` created with license allowlist and source restrictions
- [x] Error message format: lowercase “error:” with `{e:#}` chain display

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
- [ ] **Phase C**: Integrate — requires human review before applying changes to playbook
  documents.

### Phase 8: Final Verification — DONE

- [x] Zero `#[ignore]` tests — `cargo test`: 250 passed, 0 failed, 0 ignored
- [x] Zero `partial` mappings in `test-mapping.yaml` — 202 mapped, 0 partial
- [x] `flowmark-dev check-mapping` exits with code 0 — PASS
- [x] All Rust tests pass — 250 tests across 20 test suites
- [x] Golden/reference document test produces identical output
- [x] All porting playbook best practices verified (32 items, P1 all applied)
- [x] Zero clippy warnings
- [x] No-default-features build succeeds

### Phase 9: CI Hardening — DONE

- [x] Refresh `rust-tests.yaml` (243 → 250 entries) via `flowmark-dev discover-rust`
- [x] Verify `flowmark-dev check-mapping` passes with exit code 0 (all 13 smoke tests)
- [x] Promote check-mapping from informational (`|| true`) to hard CI gate
- [x] Run all `TestMappingCompleteness` tests in CI (removed
  `-k "not MappingCompleteness"`)
- [x] Update Rust test count assertion from `>= 178` to exact `== 250`

### Phase 9b: Code Review — DONE

**Epic bead:** fmr-ow2t **PR:** [#2](https://github.com/jlevy/flowmark-rs/pull/2)
(branch: `code-review-fixes`)

Comprehensive code review of the Rust codebase after Phase 9 completion.
16 findings addressed across P0-P3, all implemented and CI-passing (251 tests).

**P0 fixes (blocking CI):**
- [x] Fix 9 clippy `inefficient_to_string` errors in `filling.rs` and `tag_handling.rs`
  (fmr-n1w9)
- [x] Verify formatting clean (fmr-5gkb)
- [x] Tighten lints: `warnings=deny`, clippy `pedantic=deny`, `unwrap_used=deny` in
  Cargo.toml (fmr-5255)

**P1 fixes (correctness/hygiene):**
- [x] Remove dead dependencies: `toml`, `serde`, `unicode-segmentation` from Cargo.toml
  (fmr-y3zv)
- [x] Remove dead error variants `Error::Config` and `Error::Other` from `error.rs`
  (fmr-oodm)
- [x] Add `RUSTFLAGS=-D warnings` to CI test jobs (fmr-wcjh)

**P2 fixes (quality/performance):**
- [x] Extract fence-tracking helper to eliminate 3x code duplication in `filling.rs`
  (fmr-q9og)
- [x] Replace `nc.to_string()` regex check with `is_word_char()` in `ellipses.rs`
  (fmr-0a97)
- [x] Introduce `FormatOptions` struct to replace boolean parameter lists in public API
  (fmr-yhmk)
- [x] Remove unused `_name` field from `AtomicPattern` struct (fmr-avjf)
- [x] Remove unnecessary `info.clone()` in filling.rs
- [x] Extract repeated `.expect()` calls to locals

**P3 fixes (polish):**
- [x] Fix `DEFAULT_WRAP_WIDTH` comment
- [x] Add doc-test for `FormatOptions::reformat_text` (fmr-nbp4) — test count 250 → 251
- [x] Remove `|| true` from mapping check in CI to enforce completeness (fmr-nswa)

### Phase 10: CLI & Feature Parity

**Epic bead:** fmr-7mmt

Port all remaining Python CLI features to achieve exact behavioral parity at the CLI
level. Every Python CLI flag must have an equivalent Rust flag with identical behavior.
Use tryscript-based golden tests for end-to-end CLI validation.

**Beads (dependency order — implement top-down):**

| Bead | Task | Tests | Depends On |
| --- | --- | --- | --- |
| **fmr-t834** | 10.1: Port file resolver module | 31 | — |
| **fmr-z8j5** | 10.2: Port config loading (TOML, three-way merge) | 20 | — |
| **fmr-4sc5** | 10.3: CLI flag parity (add 11 missing flags) | 19 | fmr-t834, fmr-z8j5 |
| **fmr-qa6p** | 10.3b: Port skill system (`--skill`, `--install-skill`, `--docs`) | 9 | fmr-4sc5 |
| **fmr-t3va** | 10.4: Tryscript CLI golden tests | — | fmr-4sc5, fmr-qa6p |
| **fmr-v2de** | 10.5: Update test mapping and CI (281 mapped, 0 excluded) | — | fmr-t3va |
| **fmr-03xy** | 10.6: Upstream contributions (PR tests to Python repo) | — | fmr-t3va |
| **fmr-h01s** | 10.7: Final acceptance (review all mappings, sign off) | — | fmr-v2de |

#### 10.1: File Resolver Module

**Bead:** fmr-t834 | **Python source:** 402 lines across 4 files | **Tests:** 31 |
**Estimated Rust LOC:** 400-500

Port `file_resolver/` (4 Python files → Rust module).
This is the largest gap.

**Python source files (all in `attic/flowmark/src/flowmark/file_resolver/`):**
- `resolver.py` (222 lines) — `FileResolver` class: `resolve()`, `_walk_directory()`,
  `_should_include_explicit()`, `_is_dir_excluded()`, `_expand_glob()`,
  `_exceeds_max_size()`, `_get_gitignore()`, `_get_gitignore_chain()`,
  `_get_tool_ignore()`
- `types.py` (39 lines) — `FileResolverConfig` dataclass with `effective_include` and
  `effective_exclude` properties
- `defaults.py` (57 lines) — `DEFAULT_INCLUDES` and `DEFAULT_EXCLUDES` constants
- `gitignore.py` (54 lines) — `_read_ignore_file()`, `load_gitignore()`,
  `load_tool_ignore()`

**Python data structures:**

`FileResolverConfig` fields (all with defaults):
- `tool_name: str = "flowmark"` — determines ignore file name (`.flowmarkignore`)
- `include: list[str] = ["*.md"]` — base include patterns
- `extend_include: list[str] = []` — additional include patterns
- `exclude: list[str] | None = None` — `None` = use `DEFAULT_EXCLUDES`; list replaces
  them
- `extend_exclude: list[str] = []` — added to effective excludes
- `respect_gitignore: bool = True`
- `force_exclude: bool = False` — apply exclusions to explicitly named files
- `files_max_size: int = 1_048_576` — 1 MiB; 0 = no limit

Properties:
- `effective_include` → `include + extend_include`
- `effective_exclude` → `(exclude ?? DEFAULT_EXCLUDES) + extend_exclude`

`DEFAULT_EXCLUDES` (30 patterns — must match exactly):

```
.git/, .hg/, .svn/, .bzr/, _darcs/,
.venv/, venv/, __pycache__/, .tox/, .nox/, .mypy_cache/, .ruff_cache/, .pytest_cache/,
.eggs/, *.egg-info/,
build/, dist/,
node_modules/, .next/, .nuxt/, .output/, .cache/, .parcel-cache/, .turbo/,
.idea/, .vscode/, .vs/, .fleet/,
coverage/, htmlcov/, .coverage/,
vendor/, third_party/, Pods/, target/, .terraform/
```

**Python behavior (exact algorithm):**

`FileResolver.resolve(paths)` dispatches each input path:
- **Existing file** → pass through (apply `force_exclude` and `max_size` checks via
  `_should_include_explicit`)
- **Existing directory** → recursive walk with all filters (`_walk_directory`)
- **Contains glob chars** (`*`, `?`, `[`) → expand via `Path.glob()`, filter results
  (`_expand_glob`)
- **Otherwise** → raise `FileNotFoundError(f"Path not found: {raw_path}")`

Result: deduplicated by `Path.resolve()` into `seen: set[Path]`, sorted
lexicographically.

`_walk_directory(root)`:
1. Load tool ignore once per walk root: `_get_tool_ignore(root)` — walks up from root
   looking for `.flowmarkignore`, caches per `resolved` directory
2. `os.walk(root)` loop: a. Compute `rel_to_root = current.relative_to(root)` b. Prune
   directories in-place:
   `dirnames[:] = [d for d if not _is_dir_excluded(d, rel/d, current, tool_ignore, root)]`
   c. Collect gitignore specs: `_get_gitignore_chain(current, root)` if
   `respect_gitignore` d. For each file: check `include_spec` → check `max_size` → check
   gitignore chain → check tool ignore → yield

`_is_dir_excluded(dirname, rel_path, current_dir, tool_ignore, walk_root)`:
1. Check `exclude_spec.match_file(dirname + "/")` — bare directory name
2. Check `exclude_spec.match_file(str(rel_path) + "/")` — relative path
3. If `respect_gitignore`: check all specs from
   `_get_gitignore_chain(current_dir, root)` against `dirname + "/"`
4. Check `tool_ignore.match_file(dirname + "/")` and
   `tool_ignore.match_file(str(rel) + "/")` if tool_ignore exists

`_should_include_explicit(path)`:
1. If `force_exclude`: check `filename` and each `parent.parts[:-1]` component + “/”
   against `exclude_spec`
2. Check `_exceeds_max_size(path)` (0 = no limit; `OSError` on `stat()` → return False,
   i.e., include)

`_get_gitignore_chain(directory, walk_root)`:
- Walk from `walk_root.resolve()` down to `directory.resolve()`, collecting
  `_get_gitignore(dir)` for each ancestor directory (inclusive)
- Returns `list[PathSpec]`

`_read_ignore_file(path)`:
- Read text; return `None` on `OSError` or `UnicodeDecodeError`
- Strip comments (`#`) and blank lines
- Return `PathSpec.from_lines("gitignore", lines)` or `None` if no active rules

`load_tool_ignore(tool_name, start_dir)`:
- Walk up from `start_dir.resolve()` to filesystem root
- Look for `.{tool_name}ignore` (e.g., `.flowmarkignore`) in each directory
- Return first found via `_read_ignore_file`, or `None`

**Rust implementation plan:**

New dependency: `ignore` crate (gitignore parsing; may also use `glob` crate for pattern
expansion).

Module structure:
- [ ] `src/file_resolver/mod.rs` — public API re-exports
- [ ] `src/file_resolver/config.rs` — `FileResolverConfig` struct with
  `effective_include()` and `effective_exclude()` methods
- [ ] `src/file_resolver/defaults.rs` — `DEFAULT_INCLUDES` and `DEFAULT_EXCLUDES`
  constants (exact match with Python)
- [ ] `src/file_resolver/gitignore.rs` — `read_ignore_file()`, `load_gitignore()`,
  `load_tool_ignore()` using `ignore::gitignore::GitignoreBuilder` or custom parsing
- [ ] `src/file_resolver/resolver.rs` — `FileResolver` struct with `resolve()`,
  `walk_directory()`, `should_include_explicit()`, `is_dir_excluded()`, `expand_glob()`,
  `exceeds_max_size()`, `get_gitignore_chain()`, `get_tool_ignore()`
- [ ] Register module in `src/lib.rs`: `pub mod file_resolver;`
- [ ] Port all 31 tests to `tests/test_file_resolver.rs` (use `tempfile` crate for temp
  directories)
- [ ] Update `test-mapping.yaml`: change 31 entries from `excluded` → `mapped`

**Rust test mapping (31 tests from `test_file_resolver.py`):**

Config tests (4):
- `test_config_effective_include` — include pattern merging
- `test_config_effective_include_custom_base` — custom base patterns
- `test_config_effective_exclude_replaced` — exclude replacement
- `test_config_effective_exclude_extended` — exclude extension

Resolver core (7):
- `test_resolver_single_file` — explicit file pass-through
- `test_resolver_directory_recursion` — recursive walk, only `*.md` found
- `test_resolver_excludes_default_dirs` — `node_modules/`, `.venv/` excluded
- `test_resolver_respects_gitignore` — `.gitignore build/` respected
- `test_resolver_no_respect_gitignore` — `respect_gitignore=False` override
- `test_resolver_force_exclude_filters_explicit_files` — `force_exclude=True` filters
  `node_modules/README.md`
- `test_resolver_explicit_files_bypass_exclusions_by_default` — `force_exclude=False`
  passes through

Filter options (5):
- `test_resolver_extend_include` — `extend_include=["*.mdx"]` adds patterns
- `test_resolver_exclude_replaces_defaults` — `exclude=["custom_dir/"]` replaces
  defaults
- `test_resolver_extend_exclude` — `extend_exclude=["drafts/"]` adds to defaults
- `test_resolver_files_max_size` — 2MB file excluded with default 1MB limit
- `test_resolver_files_max_size_zero_disables` — `max_size=0` disables limit

Glob and deduplication (4):
- `test_resolver_glob_pattern` — `docs/*.md` glob expansion
- `test_resolver_mixed_inputs` — explicit file + directory together
- `test_resolver_deduplication` — same file listed twice → one result
- `test_resolver_sorted_output` — results are sorted

Error handling (1):
- `test_resolver_file_not_found` — nonexistent path → `FileNotFoundError`

Tool ignore (2):
- `test_resolver_flowmarkignore` — `.flowmarkignore` with `drafts/`
- `test_resolver_tool_ignore_per_walk_root` — separate `.flowmarkignore` per walk root

Gitignore specifics (5):
- `test_resolver_nested_gitignore` — nested `.gitignore` in subdirectory
- `test_resolver_nested_gitignore_combines_parent_rules` — parent `*.log` rule applies
  in child
- `test_resolver_gitignore_file_patterns` — `draft.md` file pattern (not just
  directories)
- `test_resolver_gitignore_wildcard_file_pattern` — `temp.*` wildcard pattern

Ignore file internals (3):
- `test_read_ignore_file_missing` — missing file → `None`
- `test_read_ignore_file_unreadable` — unreadable file (chmod 000) → `None`
- `test_read_ignore_file_non_utf8` — non-UTF-8 bytes → `None`

Flowmarkignore positive (1):
- `test_resolver_flowmarkignore_positive_assertion` — verify exactly which files kept

#### 10.2: Config Loading

**Bead:** fmr-z8j5 | **Python source:** 184 lines | **Tests:** 20 | **Estimated Rust
LOC:** 250-350

Port `config.py` (TOML-based config file loading with three-way merge).
The existing `src/config.rs` (83 lines) has `ListSpacing` and `FormatOptions` but no
TOML loading — extend it or create `src/config/` module.

**Python source:** `attic/flowmark/src/flowmark/config.py` (184 lines)

**Python data structures:**

`FlowmarkConfig` — all fields are `Option<T>` (Python `None`) to distinguish “not
configured” from “explicitly set to default”:

```
# Formatting
width: int | None = None
semantic: bool | None = None
cleanups: bool | None = None
smartquotes: bool | None = None
ellipses: bool | None = None
list_spacing: str | None = None

# File discovery
include: list[str] | None = None
extend_include: list[str] | None = None
exclude: list[str] | None = None
extend_exclude: list[str] | None = None
files_max_size: int | None = None
respect_gitignore: bool | None = None
force_exclude: bool | None = None
```

`_CONFIG_FILENAMES = [".flowmark.toml", "flowmark.toml", "pyproject.toml"]`

Kebab-to-snake mapping table (6 entries):

```
"list-spacing" → "list_spacing"
"extend-include" → "extend_include"
"extend-exclude" → "extend_exclude"
"files-max-size" → "files_max_size"
"respect-gitignore" → "respect_gitignore"
"force-exclude" → "force_exclude"
```

Auto-locked fields (not overridable by config in `--auto` mode):
`{"semantic", "cleanups", "smartquotes", "ellipses", "inplace", "nobackup"}`

**Python behavior (exact algorithm):**

`find_config_file(start_dir)`:
1. `current = start_dir.resolve()`
2. Loop: for each of `.flowmark.toml`, `flowmark.toml`, `pyproject.toml`:
   - If file exists: for `pyproject.toml`, check
     `_pyproject_has_flowmark_section(candidate)` first; for others, return immediately
3. `parent = current.parent`; if `parent == current`, break (filesystem root)
4. `current = parent`; repeat
5. Return `None`

`_pyproject_has_flowmark_section(path)`:
- Parse TOML; check `"flowmark" in data.get("tool", {})`
- Catch `TOMLDecodeError` and `OSError` → return `False`

`load_config(config_path)`:
1. Parse TOML text; on `TOMLDecodeError`/`OSError`:
   `eprintln!("Warning: could not parse config file {config_path}")` and return empty
   config
2. If `config_path.name == "pyproject.toml"`: extract `data["tool"]["flowmark"]`
   subsection
3. Call `_parse_config_data(data)`: a. Flatten nested sections: any `dict` value’s
   sub-keys merge to top level b. Map kebab-case → snake_case via lookup table
   (fallback: `key.replace("-", "_")`) c. Validate against `FlowmarkConfig` field names
   d. Unrecognized keys: `eprintln!("Warning: unrecognized config key '{key}'")` e.
   Construct `FlowmarkConfig(**mapped)`

`merge_cli_with_config(cli_opts, config, is_auto, explicit_flags)`:
1. If `config` is `None`, return `cli_opts` unchanged
2. For each field in `FlowmarkConfig`: a. If field value is `None` → skip (not set in
   config) b. If field name in `explicit_flags` → skip (CLI takes precedence) c. If
   `is_auto` and field name in `auto_locked` → skip d. Set
   `cli_opts.{field} = config_value`
3. Return `cli_opts`

**Rust implementation plan:**

New dependency: `toml` crate (add to `[dependencies]` with `optional = true` under `cli`
feature, or unconditional if config loading belongs in lib).

- [ ] Add `serde` and `toml` crates as dependencies
- [ ] Create `FlowmarkConfig` struct with all-`Option<T>` fields (use
  `#[derive(Default, Deserialize)]`)
- [ ] Implement `find_config_file(start_dir: &Path) -> Option<PathBuf>` — directory walk
  with per-directory search order
- [ ] Implement `pyproject_has_flowmark_section(path: &Path) -> bool`
- [ ] Implement `load_config(config_path: &Path) -> FlowmarkConfig` — TOML parsing with
  section flattening, kebab→snake mapping, and warning messages to stderr
- [ ] Implement `merge_cli_with_config(cli_opts, config, is_auto, explicit_flags)` —
  three-way merge
- [ ] Decide: extend existing `src/config.rs` or create `src/config/` module with
  `mod.rs` + `toml_config.rs`
- [ ] Port 20 tests to `tests/test_config.rs` (use `tempfile` for temp directories)
- [ ] Update `test-mapping.yaml`: change 20 entries from `excluded` → `mapped`

**Rust test mapping (20 tests from `test_config.py`):**

Config file discovery (6):
- `test_find_config_flowmark_toml` — finds `flowmark.toml`
- `test_find_config_dot_flowmark_toml_takes_precedence` — `.flowmark.toml` >
  `flowmark.toml`
- `test_find_config_pyproject_toml` — finds `pyproject.toml` with `[tool.flowmark]`
- `test_find_config_pyproject_without_section_skipped` — skips `pyproject.toml` without
  section
- `test_find_config_walks_up` — finds config in parent directory
- `test_find_config_none_when_missing` — returns `None` when no config exists

Config loading (5):
- `test_load_config_flowmark_toml` — loads formatting options, unset fields are `None`
- `test_load_config_pyproject_toml` — extracts `[tool.flowmark]` section
- `test_load_config_kebab_case` — kebab-case keys mapped correctly (all 6 mappings)
- `test_load_config_file_discovery_section` — `[file-discovery]` section parsed
- `test_load_config_partial` — partial config, unset fields remain `None`

Config merge (7):
- `test_merge_no_config` — `None` config returns defaults unchanged
- `test_merge_config_overrides_defaults` — config values override defaults
- `test_merge_explicit_cli_overrides_config` — explicit CLI flag beats config
- `test_merge_auto_mode_overrides_formatting` — `--auto` locks formatting on
- `test_merge_auto_mode_width_from_config` — width comes from config even in auto mode
- `test_merge_file_discovery_from_config` — file discovery settings from config
- `test_merge_extend_include_from_config` — `extend_include` from config applied

Error handling (2):
- `test_load_config_malformed_toml` — malformed TOML returns empty config (no crash)
- `test_parse_config_warns_unknown_keys` — unknown keys produce
  `"unrecognized config key"` warning to stderr

#### 10.3: CLI Flag Parity

**Bead:** fmr-4sc5 | **Python source:** 527 lines (`cli.py`) | **Tests:** 19 |
**Estimated Rust LOC:** 200-300 | **Depends on:** fmr-t834, fmr-z8j5

Add all missing Python CLI flags to `main.rs`, integrate file resolver and config
loading, port argument validation and error messages.

**Current Rust CLI state (`src/main.rs`, 143 lines):**
- `Args` struct has 13 fields (files, output, width, plaintext, semantic, cleanups,
  smartquotes, ellipses, list_spacing, inplace, nobackup, auto, verbose)
- `run()` function: parse args → auto expansion → build `FormatOptions` → simple file
  loop (stdin or file, no directory support)
- No file discovery, no config loading, no `--list-files`, no skill flags
- Default `files = ["-"]` (stdin) — Python changed to `files = []` (empty, requires
  explicit input)

**Missing flags (11 — Python has, Rust doesn’t):**

| Flag | clap Type | Default | Purpose |
| --- | --- | --- | --- |
| `--extend-include PATTERN` | `Vec<String>` (append) | `[]` | Additional file include patterns |
| `--exclude PATTERN` | `Option<Vec<String>>` (append) | `None` | Replace default exclusions |
| `--extend-exclude PATTERN` | `Vec<String>` (append) | `[]` | Add to default exclusions |
| `--no-respect-gitignore` | `bool` (flag) | `false` | Disable .gitignore integration |
| `--force-exclude` | `bool` (flag) | `false` | Apply exclusions to explicit files |
| `--list-files` | `bool` (flag) | `false` | Print resolved paths, don’t format |
| `--files-max-size BYTES` | `usize` | `1_048_576` | Skip files larger than N bytes |
| `--skill` | `bool` (flag) | `false` | Print SKILL.md content to stdout |
| `--install-skill` | `bool` (flag) | `false` | Install skill to `~/.claude/` |
| `--agent-base DIR` | `Option<String>` | `None` | Custom agent config dir |
| `--docs` | `bool` (flag) | `false` | Print documentation to stdout |

**Already present in Rust (no action needed):**
- `--version` — clap `version` derive (auto-generated from Cargo.toml)
- `--verbose` (`-v`) — Rust-only addition (doesn’t break drop-in compatibility)

**Critical behavior change: default files argument**

Python changed from `default=["-"]` (stdin) to `default=[]` (empty, requires explicit
input). Rust must match:
- Change `#[arg(default_value = "-")]` to no default
- Add validation: if `files.is_empty()`, print error and exit 1

**Explicit-flag tracking (for config merge precedence):**

Python uses a sentinel parser to detect which flags the user explicitly passed (even if
the value matches the default).
Rust approach options:
- Use `clap`’s `value_source()` method to check if a value came from CLI vs default
- Or: parse with `Option<T>` for tracked fields, then apply defaults after detection

Tracked flags (12): `width`, `semantic`, `cleanups`, `smartquotes`, `ellipses`,
`list_spacing`, `extend_include`, `exclude`, `extend_exclude`, `respect_gitignore`,
`force_exclude`, `files_max_size`

**Error messages (must match Python exactly):**

1. `--auto` without files (exit 1):

   ```
   Error: --auto requires at least one file or directory argument (use '.' for current directory, --help for more options)
   ```

2. `--list-files` without files (exit 1):

   ```
   Error: --list-files requires at least one file or directory argument (use '.' for current directory, --help for more options)
   ```

3. No input at all (exit 1):

   ```
   Error: No input specified. Provide files, directories (use '.' for current directory), or '-' for stdin. Use --help for more options.
   ```

4. `--auto --list-files` without files → `--auto` message takes priority (exit 1)

**`_needs_file_resolution` logic:**
- Skip stdin marker `"-"`
- Return true if any path `is_dir()` or contains glob chars `*?[`
- If returns false and not `--list-files`, pass files through unchanged

**`_resolve_files` logic:**
- Filter out `"-"` before passing to `FileResolver`
- Track whether stdin was present; re-insert at position 0 if so
- Create `FileResolverConfig` from CLI options
- Call `resolver.resolve(resolvable)` and convert results to strings

**Multi-file processing (`reformat_files` behavior):**
- Single stdin: pass through to `reformat_file`
- Multiple files with `--output` (not `-`): error
  `"Cannot specify output file when processing multiple files (use --inplace instead)"`
- Multiple files with `--inplace`: process each in-place
- Multiple files without `--inplace`: process each to stdout

**Main function flow (must match Python `main()`):**
1. Parse args, detect explicit flags, detect `is_auto`
2. Handle `--auto` expansion (set inplace, nobackup, semantic, cleanups, smartquotes,
   ellipses)
3. Early exits: `--version` → print and exit 0; `--install-skill` → install and exit 0;
   `--skill` → print SKILL.md and exit 0; `--docs` → print docs and exit 0
4. Validate: files required (with mode-specific error messages)
5. Load config: `find_config_file(cwd)` → `load_config()` → `merge_cli_with_config()`
6. Resolve files: `_resolve_files()` (conditionally invoke `FileResolver`)
7. Handle `--list-files`: print resolved paths and exit 0
8. Format files via loop (existing behavior, extended for multi-file)

**Rust implementation plan:**
- [ ] Change `files` default from `"-"` to empty (no default)
- [ ] Add all 11 missing flags to `Args` struct with clap derive attributes
- [ ] Implement explicit-flag tracking via `clap::ArgMatches::value_source()`
- [ ] Implement file resolution integration (`_needs_file_resolution` +
  `_resolve_files`)
- [ ] Implement config loading integration (`find_config_file` + `load_config` +
  `merge_cli_with_config`)
- [ ] Implement `--list-files` mode (resolve → print → exit)
- [ ] Port all 4 error messages with exact wording
- [ ] Port multi-file processing with `--output` validation
- [ ] Port early exit handlers (`--version`, `--skill`, `--install-skill`, `--docs`)
- [ ] Port 19 tests to `tests/test_cli_file_discovery.rs`
- [ ] Update `test-mapping.yaml`: change 19 entries from `excluded` → `mapped`

**Rust test mapping (19 tests from `test_cli_file_discovery.py`):**

File discovery via `--list-files` (7):
- `test_list_files_directory` — `--list-files .` lists `README.md`, `api.md`, `guide.md`
- `test_list_files_skips_excluded_dirs` — `node_modules/`, `.venv/` not in output
- `test_list_files_extend_include` — `--extend-include *.mdx` finds `.mdx` files
- `test_list_files_extend_exclude` — `--extend-exclude drafts/` excludes directory
- `test_list_files_no_respect_gitignore` — `--no-respect-gitignore` overrides
- `test_list_files_force_exclude` — `--force-exclude` filters explicit
  `node_modules/README.md`
- `test_list_files_max_size` — `--files-max-size 100` skips 2MB file

Error cases (4):
- `test_auto_no_args_errors` — `--auto` with no files → exit 1,
  `"--auto requires at least one file or directory argument"`
- `test_list_files_no_args_errors` — `--list-files` with no files → exit 1
- `test_no_args_errors` — bare `flowmark` → exit 1, `"No input specified"` +
  `"'-' for stdin"` + `"'.' for current directory"` + `"--help"`
- `test_auto_list_files_no_args_errors` — `--auto --list-files` → exit 1, auto message
  takes priority

Formatting integration (4):
- `test_auto_with_dot_formats_cwd` — `--auto .` formats files in cwd
- `test_explicit_file_still_works` — explicit file path → stdout
- `test_stdin_still_works` — `-` reads stdin → stdout
- `test_auto_with_explicit_file` — `--auto file.md` formats single file in-place

Tool ignore (1):
- `test_flowmarkignore` — `.flowmarkignore` respected in `--list-files` mode

Edge cases (3):
- `test_list_files_stdin_does_not_crash` — `--list-files - /dir` doesn’t crash
- `test_stdin_explicit_dash` — explicit `-` reads stdin
- `test_explicit_flag_detection_with_default_value` — `--width 88` (default value) still
  detected as explicit flag

#### 10.3b: Skill System

**Bead:** fmr-qa6p | **Python source:** 158 lines (`skill.py`) | **Tests:** 9 |
**Estimated Rust LOC:** 150-200 | **Depends on:** fmr-4sc5

Port `skill.py` and `skills/` — Claude Code skill installation.

**Python source:** `attic/flowmark/src/flowmark/skill.py` (158 lines)

**Python functions:**

`get_skill_content() -> str`:
- Loads `SKILL.md` from package data via `importlib.resources.files("flowmark")`
- Returns content as string
- Raises `ImportError` / `FileNotFoundError` if unavailable

`get_docs_content() -> str`:
- Finds `README.md` relative to `skill.py` (up 3 levels to repo root)
- Falls back to basic help text with link to GitHub if not found

`install_skill(agent_base: str | None = None)`:
- Default (`None`): install to `~/.claude/skills/flowmark/SKILL.md`
- Custom: install to `{agent_base}/skills/flowmark/SKILL.md`
- Creates directories with `mkdir(parents=True, exist_ok=True)`
- Writes SKILL.md content
- Prints success message with location
- If custom base: prints tip “Commit .claude/skills/ to share with team”
- On `PermissionError`: `"Permission denied: {e}"` to stderr, exit 1
- On `OSError`: `"Installation failed: {e}"` to stderr, exit 1

**SKILL.md content** (118 lines, in `attic/flowmark/src/flowmark/skills/SKILL.md`):
- YAML-style frontmatter: `name: flowmark`, `description:`, `allowed-tools:`
- Usage instructions with `uvx flowmark@latest`
- Key options table, common workflows, semantic line breaks explanation

**Rust implementation plan:**

Resources to embed at compile time:
- `SKILL.md` — copy from Python’s `skills/SKILL.md` into a Rust-accessible location
  (e.g., `src/skills/SKILL.md`), embed via `include_str!("skills/SKILL.md")`
- Documentation content — embed README.md or equivalent via `include_str!()`

Note: Python’s `SKILL.md` references `uvx flowmark@latest` (Python distribution).
The Rust binary will need its own SKILL.md that references the Rust binary installation
method (e.g., `cargo install flowmark`). This is an acceptable adaptation, not a parity
violation.

Module structure:
- [ ] Create `src/skill.rs` with `get_skill_content()`, `get_docs_content()`,
  `install_skill(agent_base: Option<&str>)`
- [ ] Create `src/skills/SKILL.md` — adapted from Python version (update install
  instructions for Rust binary)
- [ ] Embed SKILL.md via `include_str!()` in `skill.rs`
- [ ] Embed documentation content via `include_str!()` (either README.md or dedicated
  docs file)
- [ ] Register module in `src/lib.rs`: `pub mod skill;`
- [ ] Wire in CLI (`main.rs`): `--skill` → `print!(get_skill_content())`;
  `--install-skill` → `install_skill(args.agent_base)`; `--docs` →
  `print!(get_docs_content())`
- [ ] Handle errors: permission denied, OS errors
- [ ] Add `dirs` crate for `home_dir()` or use `std::env::var("HOME")` on Unix
- [ ] Port 9 tests to `tests/test_skill.rs`
- [ ] Update `test-mapping.yaml`: change 9 entries from `excluded` → `mapped`

**Rust test mapping (9 tests from `test_skill.py`):**

Skill content loading (3):
- `test_skill_content_loads` — `get_skill_content()` returns non-empty string
- `test_skill_content_has_metadata` — contains `name: flowmark`, `description:`,
  `allowed-tools:`
- `test_skill_content_has_usage` — contains `# Flowmark` and install command

Docs content loading (2):
- `test_docs_content_loads` — `get_docs_content()` returns non-empty string
- `test_docs_content_is_readme` — contains distinctive sections (`# flowmark`,
  `## Installation`, `## Semantic Line Breaks`)

Skill installation (4):
- `test_install_skill_default` — installs to `~/.claude/skills/flowmark/SKILL.md` (mock
  home dir)
- `test_install_skill_custom_base` — installs to
  `{custom_base}/skills/flowmark/SKILL.md`
- `test_install_skill_creates_directories` — creates nested
  `deep/nested/path/skills/...`
- `test_install_skill_overwrites_existing` — overwrites old SKILL.md content

#### 10.4: Tryscript CLI Golden Tests

**Bead:** fmr-t3va | **Depends on:** fmr-4sc5, fmr-qa6p

Establish tryscript-based end-to-end golden tests as the authoritative cross-language
CLI validation.

**Workflow:**

1. **Audit**: Enumerate every CLI feature/flag (done — see 10.3 flag table)
2. **Baseline**: Write tryscript tests against the Python `flowmark` CLI to capture
   expected behavior as golden output
3. **Replicate**: Run the same tryscript tests against the Rust `flowmark` binary and
   verify identical output
4. **Map**: Add all tryscript test scenarios to the test mapping system
5. **Review**: Manually review every golden file for accuracy

**Prerequisites — review before implementing:**
- [ ] Read `tbd guidelines golden-testing-guidelines` for the full golden testing
  methodology, session modeling, and tryscript integration patterns
- [ ] Run `npx tryscript@latest readme` for tryscript overview
- [ ] Run `npx tryscript@latest docs` for tryscript syntax quick reference (patterns,
  elisions, config, YAML frontmatter)
- [ ] Run `npx tryscript@latest --help` for CLI options

**Tryscript setup:**
- [ ] Install tryscript: `npx tryscript@latest`
- [ ] Create `tests/tryscript/` directory structure
- [ ] Add tryscript CI job (runs after build, validates CLI golden output)
- [ ] Use `[..]` for variable output (paths, timestamps), `...` for multi-line elision
- [ ] Define `[PATTERN]` regex patterns in YAML frontmatter for platform-specific paths

**Concrete tryscript test matrix (24 scenarios):**

Each scenario below becomes a tryscript test file in `tests/tryscript/`. First run
against Python `flowmark` to capture golden baseline, then verify Rust binary produces
identical output.

| # | Scenario | Test File | What It Validates |
| --- | --- | --- | --- |
| 1 | `flowmark file.md` (single file → stdout) | `basic-file.try` | Basic file formatting |
| 2 | `echo "..." \| flowmark -` (stdin → stdout) | `basic-stdin.try` | Stdin processing |
| 3 | `flowmark --inplace file.md` (backup created) | `inplace-backup.try` | In-place with `.bak` backup |
| 4 | `flowmark --inplace --nobackup file.md` | `inplace-nobackup.try` | In-place without backup |
| 5 | `flowmark --auto .` (dir with mixed files) | `auto-directory.try` | Auto mode on directory |
| 6 | `flowmark --auto file.md` (single file) | `auto-single-file.try` | Auto mode on explicit file |
| 7 | `flowmark --width 60 file.md` | `width-custom.try` | Custom width |
| 8 | `flowmark --width 0 file.md` | `width-zero.try` | Width 0 (no wrapping) |
| 9 | `flowmark --plaintext file.txt` | `plaintext.try` | Plaintext mode |
| 10 | `flowmark --semantic file.md` | `semantic.try` | Semantic line breaks |
| 11 | `flowmark --smartquotes --ellipses file.md` | `typography.try` | Smart quotes + ellipses |
| 12 | `flowmark --list-spacing loose file.md` | `list-spacing.try` | List spacing modes |
| 13 | `flowmark --list-files .` | `list-files-dir.try` | File discovery + listing |
| 14 | `flowmark --list-files --extend-include "*.mdx" .` | `list-files-extend.try` | Extended include patterns |
| 15 | `flowmark --list-files --extend-exclude "drafts/" .` | `list-files-exclude.try` | Extended exclude patterns |
| 16 | `flowmark --list-files --no-respect-gitignore .` | `list-files-gitignore.try` | Gitignore override |
| 17 | `flowmark --list-files --force-exclude nm/README.md` | `list-files-force.try` | Force exclude on explicit files |
| 18 | `flowmark --list-files --files-max-size 100 .` | `list-files-maxsize.try` | Max file size filtering |
| 19 | `flowmark` (no args) | `error-no-args.try` | Error: no input specified |
| 20 | `flowmark --auto` (no files) | `error-auto-no-args.try` | Error: --auto requires files |
| 21 | `flowmark nonexistent.md` | `error-not-found.try` | Error: file not found |
| 22 | `flowmark --version` | `version.try` | Version output format |
| 23 | `flowmark --skill` | `skill-print.try` | Print SKILL.md content |
| 24 | `flowmark --docs` | `docs-print.try` | Print documentation |

**Test fixture directory (`tests/tryscript/fixtures/`):**
- [ ] `simple.md` — basic Markdown (heading + paragraphs)
- [ ] `with-frontmatter.md` — YAML frontmatter + content
- [ ] `with-code.md` — code blocks, inline code
- [ ] `large.md` — >1MB file (for max-size testing)
- [ ] `page.mdx` — MDX file (for extend-include testing)
- [ ] `nested/` directory structure:
  - `docs/guide.md`, `docs/api.md`
  - `node_modules/pkg/README.md` (should be excluded)
  - `.venv/lib/README.md` (should be excluded)
  - `drafts/wip.md` (for extend-exclude testing)
- [ ] `.gitignore` with `ignored/` pattern
- [ ] `ignored/found.md` (for gitignore testing)
- [ ] `.flowmarkignore` with `skip/` pattern
- [ ] `skip/nope.md` (for flowmarkignore testing)
- [ ] `.flowmark.toml` with `[formatting]` and `[file-discovery]` sections
- [ ] `pyproject.toml` with `[tool.flowmark]` section

**Config-related tryscript tests (optional — may be better as unit tests):**

Config interactions are complex (three-way merge, auto locking).
These may be more reliably tested as Rust integration tests (already covered by the 20
config tests) rather than tryscript golden tests, since config behavior depends on which
directory you’re in.

**Implementation steps:**
- [ ] Create fixture directory and files
- [ ] Write all 24 tryscript test files against Python `flowmark`
- [ ] Run `npx tryscript@latest tests/tryscript/` to capture golden output
- [ ] Build Rust binary and run same tryscript tests against it
- [ ] Diff output — fix any discrepancies in Rust implementation
- [ ] Add tryscript CI job to `.github/workflows/`
- [ ] Iterate until all 24 scenarios pass for both Python and Rust

#### 10.4b: New Dependencies for Phase 10

| Rust Crate | Replaces (Python) | Purpose |
| --- | --- | --- |
| `ignore` | `pathspec` + `os.walk` | Gitignore-aware directory walking and glob matching |
| `toml` | `tomllib` / `tomli` | TOML config file parsing |
| `serde` | — | Deserialization for TOML config struct |
| `glob` | `pathlib.Path.glob()` | Glob pattern expansion (if `ignore` doesn’t cover all cases) |
| `dirs` | — | Home directory resolution (for skill installation) |

These are additions to the existing dependency table in the porting plan.
All should be feature-gated under `cli` except `serde`/`toml` if config loading is in
the library.

#### 10.5: Update Test Mapping and CI

**Bead:** fmr-v2de | **Depends on:** fmr-t3va

Update the test mapping system and CI gates to reflect the new scope.

- [ ] Update `test-mapping.yaml`: all 79 previously excluded entries change from
  `excluded` → `mapped`
- [ ] Update `check-mapping` expected counts: 281 mapped, 0 excluded, 0 missing, 0
  partial
- [ ] Update Rust test count assertion in CI (will increase as Phase 10 tests are added)
- [ ] Add tryscript CI job to `.github/workflows/`
- [ ] Run `flowmark-dev discover-rust` to refresh `rust-tests.yaml`
- [ ] Run `flowmark-dev check-mapping` — verify exit code 0

#### 10.6: Upstream Contributions

**Bead:** fmr-03xy | **Priority:** P2 | **Depends on:** fmr-t3va

We have flexibility to PR tryscript tests and any needed end-to-end tests to the Python
flowmark repo (`github.com/jlevy/flowmark`) to ensure parity.
This is valuable because:

- Tryscript tests written against Python establish the golden baseline
- These tests can live in the Python repo as part of its own test suite
- The same tryscript scripts can then be run against the Rust binary for
  cross-validation
- Any gaps in Python test coverage discovered during the audit benefit both repos

**Plan:**
- [ ] PR tryscript tests to the Python flowmark repo (if not already present)
- [ ] PR any missing CLI test coverage discovered during the Phase 10 audit
- [ ] Bump the Python source pin from `v0.6.4` to the version that includes the new
  tests (once merged)
- [ ] Update `flowmark-dev discover-python` to pick up new test functions

#### 10.7: Acceptance Criteria

**Bead:** fmr-h01s | **Depends on:** fmr-v2de

- [ ] **Every** Python CLI flag has a Rust equivalent with identical behavior — no
  exceptions
- [ ] `flowmark --auto .` works identically in both Python and Rust (same files
  discovered, same output produced)
- [ ] `flowmark --list-files .` produces identical sorted file lists
- [ ] Config loading from `.flowmark.toml` and `pyproject.toml [tool.flowmark]` works
- [ ] `.flowmarkignore` patterns are respected
- [ ] Gitignore integration works (and `--no-respect-gitignore` disables it)
- [ ] Skill system works: `--skill`, `--install-skill`, `--agent-base`, `--docs`
- [ ] All 79 previously-excluded tests are ported and passing
- [ ] Tryscript golden tests pass in CI for both Python and Rust
- [ ] `check-mapping` passes: **281 mapped, 0 excluded, 0 missing, 0 partial**
- [ ] Every mapping entry manually reviewed for accuracy
- [ ] Tryscript tests contributed upstream to the Python repo
- [ ] All existing 251+ tests continue to pass (no regressions)

### Remaining Steps and Current Status

This section provides a comprehensive accounting of everything that is done, everything
enforced in CI, and everything remaining.

#### Fully Complete and CI-Enforced

| Item | CI Job | Status |
| --- | --- | --- |
| All 251 Rust tests pass | `test` (ubuntu + macOS) | Hard gate |
| Library builds without CLI feature | `test-lib-only` | Hard gate |
| Zero clippy warnings (pedantic) | `clippy` | Hard gate (`-D warnings`) |
| `unwrap_used` denied in library code | `clippy` | Hard gate (Cargo.toml lint) |
| `RUSTFLAGS=-D warnings` on test jobs | `test` | Hard gate |
| Code formatting | `fmt` | Hard gate |
| MSRV 1.85 compiles | `msrv` | Hard gate |
| Dependency audit (licenses, sources) | `deny` | Hard gate |
| Documentation builds clean | `docs` | Hard gate (`-D warnings`) |
| YAML round-trip serialization | `check-mapping` | Hard gate |
| YAML deterministic output | `check-mapping` | Hard gate |
| Checked-in YAML matches canonical form | `check-mapping` | Hard gate |
| Python test count == 281 | `check-mapping` | Hard gate |
| Rust test count == 251 | `check-mapping` | Hard gate |
| Mapping count == 281 | `check-mapping` | Hard gate |
| Zero `missing` mapping entries | `check-mapping` | Hard gate |
| All mapped Rust refs exist | `check-mapping` | Hard gate |
| `check-mapping` end-to-end exit code 0 | `check-mapping` | Hard gate |

**Total: 9 CI jobs, all hard gates.
Zero informational-only steps.**

#### Fully Complete — Not CI-Enforced (verified manually)

| Item | Verified How | Status |
| --- | --- | --- |
| Zero `#[ignore]` tests | `cargo test` output shows 0 ignored | Done |
| Zero `unwrap()` in library code | `grep -r '\.unwrap()' src/` → 0 matches | Done |
| Zero `partial` mapping entries | `check-mapping` report | Done |
| `pub(crate)` visibility for internal APIs | Manual review, ~50 items changed | Done |
| 202 mapped + 79 excluded = 281 total | `check-mapping` report | Done |
| Golden test produces identical output | `test_ref_docs` in 4 modes | Done (CI-tested) |

#### Phase 10 Work (CLI & Feature Parity)

| Bead | Item | Priority | Notes |
| --- | --- | --- | --- |
| fmr-t834 | **File resolver module** | P1 | Port `file_resolver/` — 31 tests |
| fmr-z8j5 | **Config loading** | P1 | Port `config.py` — 20 tests |
| fmr-4sc5 | **CLI flag parity** | P1 | Add 11 missing flags — 19 tests |
| fmr-qa6p | **Skill system** | P1 | Port `skill.py` — 9 tests |
| fmr-t3va | **Tryscript CLI golden tests** | P1 | End-to-end CLI validation |
| fmr-v2de | **Update test mapping and CI** | P1 | 281 mapped, 0 excluded, tryscript CI |
| fmr-03xy | **Upstream contributions** | P2 | PR tryscript tests to Python repo |
| fmr-h01s | **Final acceptance** | P1 | Review all mappings, sign off |

#### Future Work (tracked as separate beads or deferred)

| Item | Priority | Bead | Notes |
| --- | --- | --- | --- |
| **Performance optimization + benchmarks** | P1 | fmr-aq8o | File resolver `--list-files` 4× slower than Python due to excessive syscalls; fix with `ignore::WalkBuilder`, then benchmark |
| **Phase 7C**: Integrate meta-playbook observations into playbook docs | P3 | — | Requires human review of 13 observations |
| **CI drift detection**: Re-run discovery in CI and diff against committed YAML | P4 | — | Optional — current canonical-form test catches most drift |
| **Property-based testing** (proptest) | P3 | — | Idempotency, width invariants, round-trip properties |
| **justfile** for common dev workflows | P3 | — | `just test`, `just lint`, `just check-mapping` |
| **Release workflow** (GitHub Actions) | P3 | — | Automated binary builds + crates.io publish (see build-publishing spec) |
| **README and CHANGELOG** | P3 | — | Public-facing documentation (see build-publishing spec) |
| **`clap_complete` shell completions** | P4 | — | Generate bash/zsh/fish completions |
| **Color flag** (`--color auto/always/never`) | P4 | — | Standard CLI convention |

#### Performance Benchmarks and Optimization (fmr-aq8o)

Benchmark the Rust binary against Python flowmark on the same inputs to quantify the
speedup. This is a key selling point for the Rust port.

**Observed regression — file resolver (`--list-files`):**

Initial benchmarking on a medium-sized repository (~ai-trade-arena) revealed the Rust
binary is **~4× slower** than Python for `--list-files`:

|  | real | user | sys |
| --- | --- | --- | --- |
| Rust (`flowmark-rs`) | 0.963s | 0.335s | **0.623s** |
| Python (`flowmark`) | 0.261s | 0.151s | **0.070s** |

The 9× higher `sys` time indicates excessive syscalls in the Rust file resolver.
Root causes identified in `src/file_resolver/resolver.rs`:

1. **`canonicalize()` on every discovered file** (line 364) — `realpath()` does multiple
   stat() calls per path component to resolve symlinks.
   Called for every file.
2. **Extra stat() per directory entry** (lines 132-139) — `path.is_dir()` /
   `path.is_file()` each issue a fresh stat(). Should use `entry.file_type()` (free on
   macOS/Linux via `d_type` from readdir).
3. **Gitignore chain rebuilt per directory** (lines 298-321) — `get_gitignore_chain()`
   calls `canonicalize()` on both root and directory, then walks from root to current
   for every directory visited.
4. **Glob patterns recompiled per-check** (lines 340-361) — `glob::Pattern::new()`
   called for every pattern on every file instead of pre-compiling once.
5. **Manual walker instead of `ignore::WalkBuilder`** — The `ignore` crate (already a
   dependency) provides the same optimized parallel walker that ripgrep uses.
   It handles gitignore, efficient stat batching, and directory pruning natively.

**Optimization plan — replace manual walker with `ignore::WalkBuilder`:**

The highest-impact fix is replacing the manual `walk_recursive` implementation with
`ignore::WalkBuilder`, which eliminates issues 1-3 in one shot:

- `WalkBuilder::new(root)` with `.git_ignore(bool)` replaces manual gitignore chain
- `.add_custom_ignore_filename(".flowmarkignore")` replaces `load_tool_ignore_patterns`
- `OverrideBuilder` handles include/exclude patterns (gitignore syntax, already
  compatible with our patterns)
- `WalkBuilder` uses `DirEntry::file_type()` internally (no extra stat)
- No `canonicalize()` needed — `WalkBuilder` returns canonical-ish paths already
- Pre-compile glob patterns for any residual matching (explicit file filtering)

File-level changes:
- `src/file_resolver/resolver.rs` — Replace `walk_directory`/`walk_recursive` with
  `WalkBuilder`-based implementation.
  Pre-compile glob patterns in `new()`. Remove `canonicalize_or_absolute()` from the hot
  path (keep for explicit file dedup only).
  Simplify `is_dir_excluded` (handled by WalkBuilder overrides).
- `src/file_resolver/gitignore.rs` — `load_gitignore` and `get_gitignore_chain` become
  unused for walking (WalkBuilder handles this).
  Keep `read_ignore_patterns` for any non-walk use cases.
  `load_tool_ignore_patterns` no longer needed for walking.
- `tests/test_file_resolver.rs` — All 28 existing tests must continue to pass
  (behavioral parity).
  No new tests needed unless behavior changes.

**Benchmarking approach:**
- Use `hyperfine` (standard CLI benchmarking tool) to compare:
  - `flowmark-rs` (Rust) vs `flowmark` (Python) on the reference doc
    (`tests/testdocs/testdoc.orig.md`, 1,416 lines)
  - Both in `--auto` mode and plain mode
  - `--list-files .` on medium and large repositories
- Optionally add `criterion` benchmarks for library-level performance (internal only,
  not for README)
- Record results in a `benchmarks/` directory with reproduction instructions

**Results for README** (in the build-publishing spec):
- Include a simple comparison table in the Rust README showing wall-clock times
- Example format: “flowmark (Rust) formats a 1,400-line Markdown file in Xms vs Yms for
  Python — Z× faster”

## Open Questions

These are decisions where multiple approaches exist and we need to choose:

1. ~~**fmr-5ojk fix approach**~~: **RESOLVED** — approach B was chosen: detect tag-only
   HTML blocks and suppress blank line before them in list items.
   This was simpler than AST transformation (approach A) and sufficient for the cases
   encountered.

2. ~~**Comrak version**~~: **RESOLVED** — SKIP. Already effectively using 0.47 features.
   Stale Cargo.toml comment removed.

3. ~~**Property-based testing**~~: **RESOLVED** — DEFER. Old impl declared proptest but
   never used it. Future enhancement for idempotency/width/round-trip properties.

4. ~~**Upstream Python bugs**~~: **RESOLVED** — all 3 bugs were confirmed as Rust
   implementation bugs, not upstream Python bugs.
   All fixed in the Rust codebase.

5. ~~**`doc_transforms.py` coverage**~~: **RESOLVED** — transform functionality is
   covered via integration tests that exercise cleanups.
   No untested paths found.

6. ~~**Trait-based vs function-based word splitting**~~: **RESOLVED** — SKIP. Current
   function-based approach is simpler, more composable, and has broader coverage than
   the old trait-based approach.

## Exclusions

**No exclusions.** Every Python feature and every Python test must have an exact Rust
equivalent. There are zero permanently excluded tests.

### Previously Excluded — Now In Scope (Phase 10)

All 79 previously excluded tests are now in scope.
Every one must be ported:

| File | # Tests | Feature |
| --- | --- | --- |
| `test_cli_file_discovery.py` | 19 | CLI arg handling, `--auto` mode, `--list-files`, error messages |
| `test_config.py` | 20 | TOML config loading, pyproject.toml, three-way merge |
| `test_file_resolver.py` | 31 | Directory recursion, glob expansion, gitignore, exclude patterns |
| `test_skill.py` | 9 | Claude Code skill installation (`--skill`, `--install-skill`, `--docs`) |

Total: **79 tests** moving from `excluded` → `mapped` in `test-mapping.yaml`.

After Phase 10, the mapping should be: **281 mapped, 0 excluded, 0 missing, 0 partial.**

## References

- **Code review PR:** [#2](https://github.com/jlevy/flowmark-rs/pull/2) (Phase 9b — all
  code review findings, branch `code-review-fixes`)
- Test mapping infrastructure spec:
  `docs/project/specs/active/plan-2026-02-17-test-mapping-meta-test.md`
- YAML artifacts: `port-coverage-mapping/`
- Original Python repo: https://github.com/jlevy/flowmark (pinned: `v0.6.4`)
- Local Python checkout: `attic/flowmark/` (gitignored — clone with
  `git clone --branch v0.6.4 --depth 1 https://github.com/jlevy/flowmark.git attic/flowmark`)
- Porting plan: `docs/project/specs/done/porting-plan.md`
- Previous Rust implementation: `attic/flowmark-rs-1/`
- Porting playbook: `attic/rust-porting-playbook/`
- Meta-playbook (improving the playbook):
  `attic/rust-porting-playbook/reference/meta-improving-this-playbook.md`

### Phase 10 References

- **Golden testing methodology**: `tbd guidelines golden-testing-guidelines` — read
  before implementing tryscript tests.
  Covers session modeling, stable vs unstable fields, tryscript integration patterns,
  and CI best practices.
- **Tryscript documentation**: `npx tryscript@latest readme` (overview),
  `npx tryscript@latest docs` (syntax reference — patterns, elisions, YAML frontmatter),
  `npx tryscript@latest --help` (CLI options)
- **Tryscript repo**: https://github.com/jlevy/tryscript
- **Python CLI source**: `attic/flowmark/src/flowmark/cli.py` — all argparse definitions
- **Python config source**: `attic/flowmark/src/flowmark/config.py` — TOML config
  loading
- **Python file resolver**: `attic/flowmark/src/flowmark/file_resolver/` — 4 files
- **Python test files (in scope)**: `attic/flowmark/tests/test_cli_file_discovery.py`,
  `attic/flowmark/tests/test_config.py`, `attic/flowmark/tests/test_file_resolver.py`

* * *

## Appendix A: Full Commit Log and Porting Synopsis

This appendix provides a complete record of the flowmark Python-to-Rust porting effort
as captured in the branch commit history (25 substantive commits, excluding tbd
bookkeeping). Each commit is annotated with what changed, the test state at that point,
and the architectural significance.

### Phase 0: Core Implementation (2 commits)

#### `f245a4b` — Initial Rust implementation of flowmark

The project scaffold: Cargo.toml, module structure, CLI with clap (feature-gated), and
all core library modules ported from Python.
Established the module layout that persisted through the entire port:

| Python Module | Rust Module |
| --- | --- |
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

The largest single commit (11,262 lines added).
Rewrote `render_node` as a proper block/inline AST renderer with blank line separation,
blockquote/alert paragraph spacing, and list item formatting.
Key innovations:

- Unicode PUA placeholder system for preserving escape characters (`\*`, `\#`, `\-`,
  etc.) through comrak’s AST, which strips backslash escapes.
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
`init-mapping` (skeleton mapping), `check-mapping` (completeness validation).
Generated initial YAML artifacts.
14 files added.

#### `a1648d9` — Update test mapping spec: cargo-based discovery, idempotent merge

Spec update: switched Rust discovery to `cargo test -- --list` (compiler-authoritative,
finds all 178 tests including 27 unit tests).
Documented idempotent additive merge semantics for all commands.

#### `be6598c` — Implement cargo-based discovery, idempotent merge, and lint fixes

Implementation of the spec updates: `discover-rust` now uses cargo as primary strategy
(178 tests: 151 integration + 27 unit), with regex fallback.
Both discovery commands preserve hand-added YAML entries.
All Python code passes ruff and basedpyright.

#### `1753a59` — Populate test-mapping.yaml with exact gap counts

All 281 Python tests reviewed and mapped: 137 mapped, 79 excluded, 64 missing, 1
partial. This commit established the precise gap that needed closing.

#### `a5b1b3f` — Add exact parity spec

Created this spec document, outlining the full roadmap from “64 missing tests” to “exact
behavioral parity.”

#### `4f05cbe` — Finalize both specs: mark completed phases

Updated both specs to reflect completed mapping work.
The 64 missing tests enumerated by file with exact counts.

### Phase 2: Test Infrastructure and CI (3 commits)

#### `25e8c1a` — TDD smoke tests for cross-language test mapping

Added 9 Python smoke tests validating the dev-tools pipeline end-to-end: YAML round-trip
serialization, discovery counts, and mapping completeness checks.

#### `a4a13b6` — Enforce deterministic YAML serialization

Moved record sorting into `write_*_yaml()` functions for canonical ordering.
Added `TestYamlDeterminism` suite verifying stable output and checked-in files match
canonical form.

#### `616859a` — CI: GitHub Actions workflow

Initial CI with Rust tests (cargo test with caching) and check-mapping (Python smoke
tests as hard gate, completeness check informational).

### Phase 3: Porting the 64 Missing Tests (3 commits)

#### `7c2b3bf` — Port 64 missing Python tests to Rust (61 pass, 4 known bugs)

The second-largest commit.
All 64 missing tests ported:

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
Regenerated rust-tests.yaml.
Mapping: 201 mapped, 79 excluded, 1 partial, 0 missing.

#### `b74c856` / `7fdc5bd` — Spec status corrections

Spec initially marked “Implemented” prematurely, then corrected to “In Progress” with
addition of Phases 3-8 covering bug fixes, code quality, previous impl review, playbook
audit, meta-playbook review, and final verification.
18 new beads created.

### Phase 4: Bug Fixes and Code Quality (3 commits)

#### `5b64d8c` — Fix 3 bugs and complete partial test

All 3 blocking bugs fixed:

- **fmr-2tll**: `postprocess_period_escapes` now strips list markers before checking for
  digit-period patterns.
- **fmr-4l1x**: Added `child_is_hard_break_heading` check to `render_list_item` spacing.
- **fmr-5ojk**: Detect tag-only HTML blocks and suppress blank line before them in list
  items.
- **fmr-p2pr**: Completed `test_other_escaped_chars` with full escape assertions.

Updated golden test files.
All 4 previously ignored tests now passing.

**Tests: 247 passing, 0 ignored, 0 failures.**

#### `e544c7e` — Fix all 70 clippy warnings across 15 files

Zero-warning build achieved.
Key changes: `push_str(&format!())` to `write!`/`writeln!`, doc-comment backticking,
`repeat_n()`, `is_some_and()`, collapsed nested `if`s, raw string literals.
Updated mapping: `test_other_escaped_chars` from `partial` to `mapped`.

**Tests: 250 passing, 0 ignored.
Mapping: 202 mapped, 0 partial.**

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
test-lib-only, MSRV (1.85), cargo-deny, docs, check-mapping.
Added `deny.toml` (license allowlist, source restrictions), `rustfmt.toml` (edition
2024, max_width 100), Cargo.toml metadata (keywords, categories).

#### `c4905d2` — P2: Add edge case tests from previous implementation review

7 tests covering edge cases from reviewing `attic/flowmark-rs-1`: code fence with
indented YAML/list content (comrak parse edge case), inline and display math with LaTeX
backslashes, bare dollar signs, code block content preservation, footnote
references/definitions.
All passed without code changes, validating the current renderer.

**Tests: 250 passing.**

#### `8b6e33b` — P2: Restrict visibility with pub(crate) and remove dead code

~50 items changed from `pub` to `pub(crate)`. Removed 4 unused functions and 1 unused
constant. Public API (re-exported from lib.rs) unchanged.

### Phase 6: Finalization (2 commits)

#### `111ca3a` — Mark exact parity spec as Complete — all 8 phases done

Final spec update: 250 tests, 0 ignored, 0 partial, check-mapping PASS, golden test
PASS, zero clippy warnings.
All open questions resolved.

#### `100b2cd` — P3: CLI polish and replace unwrap() with expect()

Final polish: `ValueEnum` derive for `ListSpacing` (rich `--help`), `BufWriter` for
stdout, `--verbose` flag, all 33 `unwrap()` in library code replaced with descriptive
`expect()` messages.

### Test Count Evolution

| Commit | Tests | Ignored | Status |
| --- | --- | --- | --- |
| `f245a4b` Initial impl | 27 | 0 | Unit tests only |
| `0e45b63` Pipeline complete | 177 | 0 | +16 integration test files |
| `7c2b3bf` Port 64 missing | 243 | 4 | 3 bugs found |
| `5b64d8c` Fix 3 bugs | 247 | 0 | All bugs fixed |
| `e544c7e` Clippy cleanup | 250 | 0 | +3 during cleanup |
| `c4905d2` Edge case tests | 250 | 0 | +7 (replaced 7 unused) |
| `100b2cd` Final polish | 250 | 0 | — |
| PR #2 Code review fixes | **251** | **0** | +1 doc-test (`FormatOptions`) |

### Mapping Status Evolution

| Commit | Mapped | Excluded | Missing | Partial |
| --- | --- | --- | --- | --- |
| `1753a59` Initial mapping | 137 | 79 | 64 | 1 |
| `881a30a` Full mapping | 201 | 79 | 0 | 1 |
| `e544c7e` Partial resolved | **202** | **79** | **0** | **0** |

* * *

## Appendix B: Current Test Suite Catalog

### Codebase Size

**Library source (`src/`):** 3,485 lines across 21 files.
**Integration tests (`tests/`):** 3,424 lines across 17 test files + golden test docs.

#### Source Files by Size

| Source File | Lines | Description |
| --- | --- | --- |
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
| --- | --- | --- | --- |
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
| --- | --- | --- |
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
| --- | --- | --- |
| Plain text | `testdoc.expected.plain.md` | `plaintext=true` |
| Cleaned | `testdoc.expected.cleaned.md` | `cleanups=true` |
| Semantic | `testdoc.expected.semantic.md` | `semantic=true, cleanups=true` |
| Auto | `testdoc.expected.auto.md` | `semantic+cleanups+smartquotes+ellipses` |

### CI Pipeline (8 parallel jobs)

| Job | What It Checks |
| --- | --- |
| **fmt** | `cargo fmt --check` — code formatting |
| **clippy** | `cargo clippy -D warnings` — lint (pedantic enabled) |
| **test** | `cargo test --locked` on ubuntu + macOS |
| **test-lib-only** | `cargo test --no-default-features` — library without CLI |
| **msrv** | `cargo check` with Rust 1.85 — minimum supported version |
| **deny** | `cargo-deny` — license allowlist, source restrictions |
| **docs** | `cargo doc -D warnings` — documentation builds clean |
| **check-mapping** | Python smoke tests + cross-language mapping completeness |

### Cross-Language Mapping Summary (end of Phase 9)

| Status | Count | Description |
| --- | --- | --- |
| **Mapped** | 202 | Python test has a verified Rust equivalent |
| **Excluded** | 79 | Infrastructure-only (CLI, config, file resolver, skill system) |
| **Missing** | 0 | All gaps closed |
| **Partial** | 0 | All partial coverage completed |
| **Extra Rust** | 28 | Unit tests in `src/` + doc-tests, not mapped to Python (Rust-native) |

**Total: 251 Rust tests covering 202 Python test behaviors, 27 Rust-specific unit tests,
1 doc-test, and 7 edge case tests from previous implementation review.
All passing, zero ignored.**

**Note:** All 79 excluded tests are now in scope for Phase 10. Target after Phase 10:
281 mapped, 0 excluded, 0 missing, 0 partial.

* * *

## Appendix C: Lines of Code — Python vs Rust

Comprehensive line counts for the original Python flowmark v0.6.4 and the Rust port.
“Code lines” excludes blank lines, comments, and docstrings/doc comments.

### Python (Original)

| Category | Total | Blank | Comments | Docstrings | Code |
| --- | ---: | ---: | ---: | ---: | ---: |
| Library (`src/flowmark/`, 26 files) | 4,433 | 626 | 355 | 921 | 2,531 |
| Tests (`tests/`, 20 files) | 5,619 | 1,085 | 346 | 1,440 | 2,748 |
| **Combined** | **10,052** | **1,711** | **701** | **2,361** | **5,279** |

Largest Python library files: `flowmark_markdown.py` (727 total / 421 code),
`tag_handling.py` (531 / 255), `cli.py` (526 / 385), `text_wrapping.py` (258 / 134).

### Rust (Port)

| Category | Total | Blank | Comments | Doc comments | Code |
| --- | ---: | ---: | ---: | ---: | ---: |
| Library (`src/`, 22 files, excl. unit tests) | 3,221 | — | — | — | — |
| Unit tests in `src/` (`#[cfg(test)]`, 7 files) | 264 | — | — | — | — |
| All of `src/` | 3,485 | 450 | 171 | 254 | 2,610 |
| Integration tests (`tests/`, 17 files) | 3,424 | 599 | 146 | 5 | 2,674 |
| **Combined** | **6,909** | **1,049** | **317** | **259** | **5,284** |

Largest Rust library files: `filling.rs` (1,270 total), `tag_handling.rs` (387),
`text_wrapping.rs` (290), `quotes.rs` (189), `line_wrappers.rs` (170).

### Comparison

| Metric | Python | Rust | Ratio |
| --- | ---: | ---: | --- |
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
  compared to Rust’s doc comments (259 lines) and comments (317 lines).
- **Test suite is 34% smaller by total lines** but nearly identical by code lines, again
  due to Python’s heavy use of triple-quoted docstrings for test fixtures vs Rust’s raw
  string literals which are more compact.
- **79 Python tests (28%) were excluded** as infrastructure-only (CLI file discovery,
  config, file resolver, skill system) during Phases 1-9. **All 79 are now in scope for
  Phase 10** — see Phase 10 plan above.
- The Rust codebase has **zero `#[ignore]` tests, zero clippy warnings, and zero
  `unwrap()` calls** in library code.

## Appendix D: Senior Engineering Review — Bugs & Issues (2026-02-17)

Fresh code review findings from an end-to-end build + test + source review.
All 339 tests pass, CI clippy clean, release build succeeds.

### Critical

| # | Issue | Location | Description |
| --- | --- | --- | --- |
| C1 | Regex compiled in loop | `filling.rs:961-973` | `min_fence_length` compiles a new `Regex` per code block. Only 2 possible patterns (`` ` `` or `~`), should be `LazyLock<Regex>` statics. |
| C2 | Ellipsis+smartquotes interaction bug | `ellipses.rs:11`, `filling.rs:1046-1051` | When both `--smartquotes` and `--ellipses` are enabled (including `--auto`), `...` followed by a curly double quote `\u{201d}` is NOT converted. Root cause: `ELLIPSIS_PATTERN` group 4 doesn’t include `\u{201c}`/`\u{201d}`, so boundary check fails. **Affects `--auto` mode.** |
| C3 | Inplace mode loses file permissions | `lib.rs:139-146` | `atomic_write` uses `tempfile::NamedTempFile` then `persist()`, which creates the new file with `0600` permissions regardless of original mode. Confirmed: `755` → `600`. |

### High

| # | Issue | Location | Description |
| --- | --- | --- | --- |
| H1 | `usize` underflow in `fill_text` | `text_filling.rs:103` | `width - subsequent_indent.chars().count()` panics if indent wider than width (e.g., `width=2` with 4-space indent). |
| H2 | Glob expansion skips exclusion filters | `resolver.rs:230-248` | `expand_glob` doesn’t apply exclude/gitignore/tool-ignore patterns. `flowmark "**/*.md"` would include `node_modules/`, `.git/`, etc. |
| H3 | Gitignore matching uses filename only | `resolver.rs:163` | `matched(filename, false)` passes bare filename, not relative path. Patterns like `docs/*.md` never match. |
| H4 | Smart quotes char-boundary redistribution fragile | `filling.rs:1107-1149` | Redistribution across AST nodes assumes `smart_quotes` preserves character count. Invariant is implicit and undocumented. |

### Medium

| # | Issue | Location | Description |
| --- | --- | --- | --- |
| M1 | Column off-by-one in sentence wrapper | `line_wrappers.rs:103-107` | `current_column` doesn’t account for the joining space, can overshoot width by 1. |
| M2 | PUA placeholder collision | `filling.rs:1028-1029` | U+E000–U+E07A used as escape placeholders. Input containing these PUA chars would be corrupted. |
| M3 | O(n*m) placeholder restoration | `text_wrapping.rs:40-52` | Every token tested against every construct via `String::replace`. Could use HashMap. |
| M4 | CRLF line endings not preserved | `frontmatter.rs:30-32` | `text.lines()` strips `\r\n` but rejoins with `\n`. Windows CRLF files silently converted. |
| M5 | `install_skill` path traversal | `skills/mod.rs:51-75` | `--agent-base` accepts arbitrary paths with no validation. |
| M6 | `read_ignore_file` silently drops all patterns on one bad line | `gitignore.rs:34` | `builder.add_line(...).ok()?` returns `None` if any line invalid, discarding entire file. |
| M7 | `should_include_explicit` skips same-named directory component | `resolver.rs:96-98` | Path `foo/foo` skips the directory `foo` because it matches the filename. |

### Low

| # | Issue | Location | Description |
| --- | --- | --- | --- |
| L1 | `has_frontmatter` parses entire document | `frontmatter.rs:42-45` | Allocates and splits full document just to check existence. Could check first chars. |
| L2 | `simple_word_split` appears unused in production | `text_wrapping.rs:75-77` | `pub` but only used in tests. Should be `pub(crate)` or `#[cfg(test)]`. |
| L3 | `first_sentence`/`first_sentences` unused | `sentence.rs:61-71` | Public functions not called from anywhere in the codebase. |
| L4 | Misleading error message in `install_skill` | `skills/mod.rs:64` | Always says “Permission denied” but `create_dir_all` can fail for many reasons. |
| L5 | `in_heading` threaded as `&mut bool` through deep call chain | `filling.rs:328+` | Fragile mutable shared state. A rendering context struct would be cleaner. |
| L6 | `byte_indexing` in `markdown_escape_word` | `text_wrapping.rs:84-85` | Relies on last char being ASCII. Safe today but fragile if regex broadened. |

### Test Coverage Gap Analysis

Every bug above was missed because no existing test covers the specific condition.
Below is the gap analysis and required test for each.

#### C1 — Regex compiled in loop

- **Existing coverage:** 6 tests in `test_fenced_code_blocks.rs` test correctness of
  fence length computation, all pass because the function returns correct values
  regardless of compilation cost.
- **Gap:** No performance test exists.
  The fix is to use `LazyLock<Regex>` statics (only 2 patterns), at which point existing
  correctness tests suffice.
- **Required test:** None beyond the fix itself.
  Existing tests validate correctness.

#### C2 — Ellipsis + smartquotes interaction (PARITY BUG)

- **Existing coverage:** `test_ellipses_quotes` tests `ellipses()` with straight quotes
  only. `test_ref_docs` “auto” mode enables both features on the reference doc, but the
  doc’s `...` + quote patterns have a space between `...` and the closing quote, so the
  bug isn’t triggered.
- **Gap:** No test calls `fill_markdown` with both `smartquotes: true` and
  `ellipses: true` on input where `...` is directly adjacent to a closing quote
  (`word..."`).
- **Required test (integration, `test_ellipses.rs`):**
  - Input: `He said "well..."` with both features → expect
    `He said \u{201c}well \u{2026}\u{201d}`
  - Input: `"Hello..." she said` → expect `\u{201c}Hello \u{2026}\u{201d} she said`
  - Input: `'Maybe...'` → single-quote variant

#### C3 — Inplace mode loses file permissions

- **Existing coverage:** `test_auto_with_dot_formats_cwd` and
  `test_auto_with_explicit_file` verify file content after `--auto` but never check
  permissions.
- **Gap:** Zero permission tests in the entire test suite.
- **Required test (integration, `test_cli_file_discovery.rs`, `#[cfg(unix)]`):**
  - Create file with `0o644` permissions, run `--auto`, assert permissions are `0o644`
    after.
  - Create file with `0o755` permissions, run `--auto`, assert permissions are `0o755`
    after.

#### H1 — `usize` underflow in `fill_text`

- **Existing coverage:** `test_width_options.rs` tests widths 0, 40, 88, 200.
  `test_wrapping.rs` tests `wrap_paragraph_lines` directly with widths 5–80. No test
  calls `fill_text` with `width < indent_length`.
- **Gap:** No small-width test with indented content (e.g., `width=2` with
  `Wrap::WrapIndent` which uses 4-char indent).
- **Required test (unit, `test_width_options.rs`):**
  - Call `fill_text` with `width=2` and `Wrap::WrapIndent` — must not panic, should
    degrade gracefully.
  - Call `fill_markdown` with `width=3` on a nested list item — must not panic.

#### H2 — `expand_glob` skips exclusion filters

- **Existing coverage:** `test_resolver_glob_pattern` tests `"docs/*.md"` but only
  checks include filtering (`.txt` excluded), not directory exclusion.
  `test_resolver_excludes_default_dirs` tests directory exclusion via `walk_directory`,
  not glob expansion.
- **Gap:** No test resolves a glob like `"**/*.md"` and verifies excluded directories
  (`node_modules/`, etc.)
  are filtered.
- **Required test (integration, `test_file_resolver.rs`):**
  - Create `docs/api.md`, `node_modules/pkg/README.md`. Resolve with `"**/*.md"`. Assert
    `node_modules/` file is excluded.
  - Create `.gitignore` with `build/`, create `build/output.md`. Resolve with
    `"**/*.md"`. Assert `build/output.md` is excluded.

#### H3 — Gitignore matching uses filename only

- **Existing coverage:** 5 gitignore tests all use bare-filename patterns (`draft.md`,
  `temp.*`, `*.log`) or directory patterns (`build/`). All pass because they don’t need
  path-based matching.
- **Gap:** No test uses a path-based gitignore pattern like `docs/draft.md` or
  `sub/specific.md`.
- **Required test (unit, `test_file_resolver.rs`):**
  - Create `sub/keep.md`, `sub/ignore-me.md`, `.gitignore` with `sub/ignore-me.md`.
    Assert `ignore-me.md` is excluded via directory walk.

#### H4 — Smart quotes char-boundary redistribution fragile

- **Existing coverage:** 6 spanning-quote tests in `test_smartquotes.rs` verify quotes
  across inline elements (code spans, emphasis, links).
  All work because `"` → curly quote is 1:1 char replacement.
- **Gap:** No test with many interleaved inline elements or text nodes where boundary
  falls exactly on a quote character.
- **Required test (integration, `test_smartquotes.rs`):**
  - Input: `He said "this *is* **really** 'quite' important" to her.` Verify all quotes
    converted and no text lost.
  - Input with boundary on quote: `"*bold*" and "*italic*"` — verify correct
    redistribution.

#### H5/M6 — `read_ignore_file` drops all patterns on bad line

- **Existing coverage:** 3 tests for complete failure modes (missing, unreadable,
  non-UTF-8).
- **Gap:** No test with a mix of valid and invalid patterns.
- **Required test (unit, `test_file_resolver.rs`):**
  - Create `.gitignore` with valid patterns and one potentially invalid line.
    Verify valid patterns still apply.
  - Note: `GitignoreBuilder::add_line` is lenient, so may need to check what actually
    triggers an error and test that specific case.

#### M1 — Column off-by-one in sentence wrapper

- **Existing coverage:** `test_wrap_width` tests `wrap_paragraph_lines` at width 80 (not
  the sentence combiner).
  Golden test uses width 88, unlikely to hit the exact boundary.
- **Gap:** No test targets boundary-width cases in `line_wrap_by_sentence`.
- **Required test (unit, `test_wrapping.rs`):**
  - Construct input where a short sentence + joining space + next word = exactly
    `width`. Call `line_wrap_by_sentence` and assert no line exceeds width.

#### M2 — PUA placeholder collision

- **Existing coverage:** Escape tests cover backslash escapes but zero tests include PUA
  characters (U+E000–U+E07A) in input.
- **Gap:** No test with PUA chars in input text.
- **Required test (unit, `test_escape_handling.rs`):**
  - Input: `"Text with \u{E000} and \u{E05C} characters."` — assert PUA chars preserved
    unchanged.
  - Input with PUA adjacent to backslash escape: `"\*test\u{E000}"` — assert both
    preserved.

#### M3 — O(n*m) placeholder restoration

- **Gap:** No performance benchmarks exist at all (`benches/` directory missing).
- **Required:** Consider adding a `benches/` directory with `criterion` benchmarks for
  `html_md_word_split` and wrapping on large documents.
  Not blocking.

#### M4 — CRLF not preserved

- **Existing coverage:** 9 frontmatter tests, all use `\n` exclusively.
- **Gap:** Zero CRLF tests.
- **Required test (unit, `test_frontmatter.rs`):**
  - Input: `"---\r\ntitle: Test\r\n---\r\n\r\n# Content\r\n"` — verify frontmatter
    section preserves `\r\n` line endings (or at minimum that the content is not
    corrupted).

#### M5 — `install_skill` path traversal

- **Existing coverage:** 4 install tests, all use safe tempdir paths.
- **Gap:** No test with `..` path components.
- **Required test (unit, `test_skill.rs`):**
  - Call `install_skill(Some("../../tmp/evil"))` and verify it either rejects the path
    or canonicalizes it safely.

#### M7 — `should_include_explicit` skips same-named directory

- **Existing coverage:** `force_exclude` test uses `node_modules/README.md` where names
  differ.
- **Gap:** No test where directory and file share the same name.
- **Required test (unit, `test_file_resolver.rs`):**
  - Create `excluded/excluded` (file named `excluded` inside directory `excluded`). Add
    `excluded/` to exclude patterns with `force_exclude`. Assert the file is excluded.

### Appendix D Learnings: Testing Gaps and Backfill Tracker

All bugs in this review were missed because no test in either Python or Rust covers the
specific condition. The test mapping system (`test-mapping.yaml`) is Python-centric — it
tracks whether every Python test has a Rust equivalent.
New tests for these bugs will be Rust-only (`extra_rust` in `check-mapping` output).
Consider upstreaming tests to Python flowmark for cross-language gaps (marked below).

| # | Bug | Sev | Python test? | Rust test? | Required test file | Test type | Upstream candidate? | Bead |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| C1 | Regex in loop | Crit | No | No (fix only) | N/A — existing tests suffice after fix | Code fix | No | fmr-he1d |
| C2 | Ellipsis+smartquotes | Crit | No | No | `test_ellipses.rs` | Integration | **Yes** | fmr-wbsk |
| C3 | Permissions lost | Crit | No | No | `test_cli_file_discovery.rs` | Integration (`#[cfg(unix)]`) | **Yes** | fmr-7o1d |
| H1 | `usize` underflow | High | No | No | `test_width_options.rs` | Unit | No | fmr-86se |
| H2 | Glob skips excludes | High | No | No | `test_file_resolver.rs` | Integration | **Yes** | fmr-a4on |
| H3 | Gitignore filename only | High | No | No | `test_file_resolver.rs` | Unit | **Yes** | fmr-albo |
| H4 | Char redistribution | High | No | No | `test_smartquotes.rs` | Integration | No | fmr-afg0 |
| H5 | Ignore file drops all | Med | No | No | `test_file_resolver.rs` | Unit | No | fmr-gpi6 |
| M1 | Column off-by-one | Med | No | No | `test_wrapping.rs` | Unit | No | fmr-myhs |
| M2 | PUA collision | Med | No | No | `test_escape_handling.rs` | Unit | No | fmr-draa |
| M3 | O(n*m) restore | Med | No | No | `benches/` (new) | Benchmark | No | fmr-wjjm |
| M4 | CRLF not preserved | Med | No | No | `test_frontmatter.rs` | Unit | **Yes** | fmr-9s7o |
| M5 | Path traversal | Med | No | No | `test_skill.rs` | Unit | No | fmr-ol08 |
| M7 | Same-name dir/file | Med | No | No | `test_file_resolver.rs` | Unit | No | fmr-n6ve |
| L1 | `has_frontmatter` inefficient | Low | No | No | N/A — perf optimization | Code fix | No | fmr-xado |
| L2 | `simple_word_split` unused | Low | No | No | N/A — visibility fix | Code fix | No | fmr-jcsj |
| L3 | `first_sentence` unused | Low | No | No | N/A — dead code removal | Code fix | No | fmr-wonk |
| L4 | Misleading error msg | Low | No | No | N/A — string fix | Code fix | No | fmr-zygd |
| L5 | `in_heading` mut bool | Low | No | No | N/A — refactor | Code fix | No | fmr-kqxb |
| L6 | Byte indexing fragile | Low | No | No | N/A — defensive fix | Code fix | No | fmr-m7o8 |

**After implementation:**

1. Write each test (test should fail before fix, pass after).
2. Run `flowmark-dev discover-rust` to update `rust-tests.yaml`.
3. New tests appear in `extra_rust` section of `flowmark-dev check-mapping`.
4. For upstream candidates (marked **Yes**): consider filing issues or PRs against
   Python flowmark to add equivalent tests.

## Appendix E: Parity Gap Resolution (2026-02-19)

**ALL GAPS RESOLVED.** Every known behavioral difference between the Rust and Python
flowmark binaries has been fixed and verified with passing tests. Exact byte-for-byte
parity achieved across all 4 formatting modes (auto, tight, loose, plaintext).

Gaps P1-P5 were discovered during comprehensive tryscript golden test development
(2026-02-18). Gaps P6-P9 were discovered by running `flowmark-rs --auto` on a real-world
corpus (ai-trade-arena `docs/`) already formatted by Python flowmark, producing 81 files
changed with 456 insertions (2026-02-19). All gaps were resolved on 2026-02-19.

### Gap Summary

| # | Gap | Status | Bead | Tests |
| --- | --- | --- | --- | --- |
| P1 | Reference links converted to inline | **RESOLVED** | (pre-parse extraction) | `test_reference_links_preserved` |
| P2 | Footnotes moved to end of document | **RESOLVED** | (position tracking) | `test_footnote_position_preserved` |
| P3 | `\"` escape stripped | **RESOLVED** | (ESCAPE_CHARS) | `test_escaped_double_quote_preserved` |
| P4 | Nested list extra blank line | **RESOLVED** | fmr-r9k6 | `test_d4_*` (3 tests) |
| P5 | `--verbose` flag (Rust-only) | **Accepted** | N/A | N/A |
| P6 | Extra blank line before code fence | **RESOLVED** | fmr-0u55 | `test_d12_*` (3 tests) |
| P7 | Blockquote blank continuation loses `>` prefix | **RESOLVED** | fmr-e38z | `test_d13_*` (2 tests) |
| P8 | Escaped backtick stripped in table inline code | **RESOLVED** | fmr-9kth | `test_d14_*` (1 test) |
| P9 | Smart quote after inline code backtick | **RESOLVED** | fmr-el2i | `test_d15_*` (1 test) |

### Additional Gaps Discovered and Resolved (D-series)

| # | Gap | Status | Bead | Tests |
| --- | --- | --- | --- | --- |
| D1 | Plaintext mode collapses code blocks | **RESOLVED** | fmr-n69j | `test_d1_*` (2 tests) |
| D2 | Plaintext "St." sentence detection | **RESOLVED** | fmr-fzth | `test_d2_*` (1 test) |
| D3 | Narrow width `<sup>` tag wrapping | **RESOLVED** | fmr-bzra | `test_d3_*` (1 test) |
| D4 | Tight list spacing nested sublists | **RESOLVED** | fmr-r9k6 | `test_d4_*` (3 tests) |
| D5 | Loose footnote embedded list items | **RESOLVED** | fmr-vpg4 | `test_d5_*` (1 test) |
| D6 | Nested blockquote extra blank lines | **RESOLVED** | fmr-3i50 | `test_d6_*` (3 tests) |
| D7 | Footnote list items collapsed | **RESOLVED** | fmr-81j7 | `test_d7_*` (2 tests) |
| D8 | Footnote blockquote collapsed | **RESOLVED** | fmr-xcr9 | `test_d8_*` (1 test) |
| D9 | Empty input trailing newline | **RESOLVED** | (trailing newline) | `test_d9_*` (3 tests) |
| D10 | HTML entities decoded | **RESOLVED** | fmr-gocw | `test_d10_*` (2 tests) |
| D11 | CLI error handling | **RESOLVED** | fmr-8ixa | `test_d11_*` (5 tests) |
| D12 | Paragraph→code fence blank line | **RESOLVED** | fmr-0u55 | `test_d12_*` (3 tests) |
| D13 | Blockquote blank continuation `>` | **RESOLVED** | fmr-e38z | `test_d13_*` (2 tests) |
| D14 | Escaped backtick in table | **RESOLVED** | fmr-9kth | `test_d14_*` (1 test) |
| D15 | Smart quote after inline code | **RESOLVED** | fmr-el2i | `test_d15_*` (1 test) |

### Session 2026-02-19 Fixes (This PR)

Additional issues discovered and fixed during final parity push:

| Issue | Bead | Root Cause | Fix |
| --- | --- | --- | --- |
| Tight mode: 8 spacing gaps | fmr-afof | `any_item_is_complex` not checking sublists/code/multi-para; `item_needs_child_spacing` not mode-aware; `parent_is_tight` inconsistent | Complete rewrite of tight mode logic: `any_item_is_complex`, `item_needs_child_spacing` with per-mode paths, `parent_is_tight` mirroring |
| Loose mode: Rules 3/4 suppression | fmr-desq | `render_block_children` Rules 3 (para→list) and 4 (para→code) suppressed blank lines even in loose mode | Added `list_spacing != ListSpacing::Loose` guard |
| Loose mode: FNDEF separator | fmr-8pya | COMRAK-WORKAROUND9b FNDEF rendering didn't check `list_spacing` for preamble→list separator | Added `if list_spacing == ListSpacing::Loose` double-newline |
| Plaintext: paired tag regex | fmr-dpjh | Paired Jinja/HTML regex matched two closing tags as atomic pair | Opening tag requires `[a-zA-Z]` start (not `/`) |
| Blockquote: blank separator | fmr-xkh3 | `render_block_children_quoted` always inserted blank before nested blockquotes | Added source position tracking (`originally_tight`) |
| Golden test regression | fmr-gydk | `has_complex_sublist` applied in Preserve mode, breaking golden test | Gated check to `ListSpacing::Tight` only |

### P1: Reference Links Converted to Inline (Critical)

**Behavior difference:**
- **Input:** `[reference link][ref1]` with `[ref1]: https://example.com "Title"`
- **Python output:** Preserves reference syntax: `[reference link][ref1]` and keeps
  `[ref1]: https://example.com "Title"` as a separate block
- **Rust output:** Converts to inline: `[reference link](https://example.com "Title")`
  and drops the link reference definition entirely

**Root cause:** Comrak (the Rust Markdown parser) resolves reference links during AST
construction. By the time the AST is available, a `[text][ref]` has become a
`NodeValue::Link` node with the URL filled in.
The link reference definition is consumed and does not appear in the AST. The Rust
renderer (`filling.rs:854-862`) always outputs `[text](url "title")` because it has no
information about the original link syntax.

Python’s Marko parser keeps `LinkRefDef` as a block-level AST node and checks
`root_node.link_ref_defs` in its link renderer to reconstruct reference syntax.

**Impact:** This is a **lossy transformation**. Documents using reference-style links
for readability lose that structure.
The same URL referenced multiple times gets duplicated inline.
This violates the “identical output” requirement.

**Files affected:**
- `tests/tryscript/fixtures/content/comprehensive.md` (line 68-70)
- `tests/tryscript/fixtures/content/links-emphasis.md` (line 5-7)
- Any user document using reference-style links

**Demonstrated diff (comprehensive.md):**

```diff
- An [inline link](https://example.com) and a
- [reference link](https://example.com "Example Reference").
+ An [inline link](https://example.com) and a [reference link][ref1].
+
+ [ref1]: https://example.com "Example Reference"
```

**Fix approach:**
1. **Pre-parse extraction**: Before passing to comrak, scan the input for link reference
   definitions (`[label]: url "title"`) and record them with their positions.
   After comrak renders the AST, post-process the output to reconstruct reference syntax
   where a link’s URL+title matches a known reference definition.
   This mirrors the approach already used for escape character preservation (PUA
   placeholder system).
2. **Alternative**: Investigate whether comrak can be configured to preserve link
   reference information in the AST (check comrak options and extensions).
3. **Alternative**: Use comrak’s sourcepos to detect which links were originally
   reference-style.

**Failing tests:**
- Tryscript: `formatting.tryscript.md` scenario F10 — asserts full Python-matching
  output (no `head -20` truncation)
- Rust unit test: `test_reference_links_preserved` (to be written in
  `test_escape_handling.rs` or `test_link_handling.rs`)

### P2: Footnotes Moved to End of Document (Critical)

**Behavior difference:**
- **Input:** Footnote `[^1]: definition` placed after the paragraph that references it
- **Python output:** Footnote definition stays in its original position
- **Rust output:** Footnote definition moved to the very end of the document

**Root cause:** Comrak moves all `FootnoteDefinition` nodes to the end of the document
AST during parsing. The Rust renderer walks the AST in order, so footnote definitions
always appear at the bottom regardless of their original position.

**Demonstrated diff:**

```diff
 This has a footnote[^1] reference.

-Inline math $x^2 + y^2 = z^2$ and display math:
+[^1]: Footnote definition here.

-$$ \sum_{i=1}^{n} i = \frac{n(n+1)}{2} $$
+Inline math $x^2 + y^2 = z^2$ and display math:

-Final paragraph of the comprehensive document.
+$$ \sum_{i=1}^{n} i = \frac{n(n+1)}{2} $$

-[^1]: Footnote definition here.
+Final paragraph of the comprehensive document.
```

**Fix approach:**
1. **Pre-parse position tracking**: Before passing to comrak, record the position of
   each footnote definition in the source.
   After rendering, reorder footnote definitions back to their original positions
   relative to surrounding content.
2. **Alternative**: Accept end-of-document placement as the normalized form.
   However, this violates the “identical output” requirement and would need to be a
   documented exception with explicit approval.

**Failing tests:**
- Tryscript: `formatting.tryscript.md` scenario F10 — asserts full Python-matching
  output
- Rust unit test: `test_footnote_position_preserved` (to be written)

### P3: Backslash-Escaped Double Quote Stripped (High)

**Behavior difference:**
- **Input:** `\"literal quotes\"`
- **Python output:** `\"literal quotes\"` (backslash preserved)
- **Rust output:** `"literal quotes"` (backslash stripped)

**Root cause:** The `ESCAPE_CHARS` list in `filling.rs:1004-1007` contains 19 characters
but is missing `"` (double quote) and 12 other CommonMark-spec-escapable ASCII
punctuation characters.
The full CommonMark spec allows backslash-escaping of all 31 ASCII punctuation
characters: `!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~`

Missing from `ESCAPE_CHARS`:

```
"  %  &  '  ,  /  :  ;  <  =  ?  @  ^
```

The PUA placeholder system only protects characters in the `ESCAPE_CHARS` list.
When comrak encounters `\"`, the backslash is stripped during parsing because `"` is not
protected by a PUA placeholder.

**Fix approach:** Add all 13 missing characters to `ESCAPE_CHARS`. At minimum, add `"`
immediately since it’s the most common and is explicitly tested in `escapes.md`.

```rust
const ESCAPE_CHARS: &[char] = &[
    '\\', '~', '*', '#', '-', '+', '>', '.', '!', '[', ']', '(', ')', '{', '}', '$',
    '_', '|', '`',
    // Previously missing CommonMark-escapable characters:
    '"', '%', '&', '\'', ',', '/', ':', ';', '<', '=', '?', '@', '^',
];
```

**Failing tests:**
- Tryscript: `typography-tests.tryscript.md` scenario T6 — asserts Python-matching
  output `\"literal quotes\"`
- Rust unit test: `test_escaped_double_quote_preserved` (to be written in
  `test_escape_handling.rs`)

### P4: Nested List Extra Blank Line (Medium)

**Behavior difference:**
- **Python:** Tight nested list has no blank line between parent item and child sublist
- **Rust:** Inserts extra blank line after the parent item text before the nested
  sublist begins

**Demonstrated diff (from comprehensive.md):**

```diff
 - First level
-  - Second level
+
+  - Second level
     - Third level deep
   - Back to second level
```

**Root cause:** The Rust list renderer’s spacing logic inserts a blank line separator
between the parent item content and its child sublist.
Python’s renderer keeps them tight when the original was tight.

**Fix approach:** Adjust the blank-line-before-sublist logic in the Rust list renderer
(`filling.rs` list item rendering) to not insert a blank line when the parent list is
tight.

**Failing tests:**
- Tryscript: `formatting.tryscript.md` scenario F10 — asserts full Python-matching
  output
- Rust unit test: `test_nested_list_no_extra_blank_line` (to be written in
  `test_wrapping.rs` or `test_lists.rs`)

### P5: `--verbose` Flag (Rust-Only Addition — Acceptable)

**Behavior difference:** Rust has `--verbose` / `-v` flag that prints
`formatting <path>` to stderr for each file processed.
Python has no equivalent.

**Assessment:** This is an **intentional feature addition**, not a gap.
It prints to stderr only (never affects stdout), has no effect on formatting output, and
does not break drop-in compatibility.
A Python user switching to Rust will never notice `--verbose` unless they explicitly
pass the flag.

**Status:** Accepted.
No fix needed. Excluded from binary-agnostic tryscript tests.

### Test Masking Patterns Removed

The following tryscript test patterns were identified as hiding real parity differences.
They have been updated to assert the full correct output:

| Test | Old Pattern | What It Hid | New Behavior |
| --- | --- | --- | --- |
| F10 (`formatting.tryscript.md`) | `head -20` | Reference link inlining, footnote relocation, nested list spacing | Asserts full output (all lines) |
| T6 (`typography-tests.tryscript.md`) | Golden output matched Rust (wrong) | `\"` stripped to `"` | Golden output matches Python (`\"` preserved) |
| T4 (`typography-tests.tryscript.md`) | `tail -1` | All output except last line | Asserts full output |
| T5 (`typography-tests.tryscript.md`) | `tail -1` | All output except last line | Asserts full output |

### How Tests Were Passing Despite Real Gaps

The tryscript tests passed because they were designed with output-masking patterns that
hid the behavioral differences:

1. **F10 (comprehensive formatting)**: Used `flowmark comprehensive.md | head -20` to
   only check the first 20 lines.
   The reference link inlining (line 75), footnote relocation (lines 97-105), and nested
   list spacing (line 41) were all beyond line 20.
2. **T6 (escapes with smart quotes)**: The golden output was written against the Rust
   binary, encoding `"literal quotes"` (with backslash stripped) as the expected output.
   The correct Python output is `\"literal quotes\"`.
3. **T4/T5 (typography in code blocks)**: Used `tail -1` to only check the last line,
   hiding any differences in how code block content is formatted.

These masking patterns violated the principle stated in the Goals section: “Any
deviation in drop-in behavior is a bug and must be surfaced as a CI failure.”
The tests have been corrected to assert the full, Python-matching output.
They will fail in CI until the corresponding Rust bugs are fixed.

### P6: Extra Blank Line Before Code Fence (High)

**Behavior difference:**
- **Input:** Paragraph text tight against opening code fence (no blank line)
- **Python output:** Preserves tight transition (no blank line inserted)
- **Rust output:** Inserts blank line between paragraph and code fence

**Demonstrated diff (454 instances across 81 files in real-world corpus):**

````diff
 Add to root `package.json`:
+
 ```json
 {
   "scripts": {
````

**Root cause:** `render_block_children()` in `filling.rs:1039-1070` has a
`suppress_for_tight` check that only handles 3 tight transition types: HTML
comment→block, block→HTML comment, and paragraph→list.
When a paragraph is followed by a `CodeBlock` with `originally_tight=true`, none of the
rules match, so `need_separator` fires and inserts a blank line.
Python/Marko preserves the original spacing.

**Impact:** This is the **most common parity gap** — 454 instances (99.6% of all
differences) found in a real-world corpus.
Every Markdown document with tight paragraph→code fence transitions gets modified.
This violates the “identical output” requirement on a massive scale.

**Fix approach:** Add Rule 4 to `suppress_for_tight`: when `originally_tight` is true
and the child node is a `CodeBlock`, suppress the blank line separator.
May need to generalize to other tight transitions (e.g., any→CodeBlock) depending on
testing.

**Failing tests:**
- `test_d12_paragraph_before_code_fence_tight`
- `test_d12_inline_code_paragraph_before_code_fence`
- `test_d12_multiple_tight_code_fences`

### P7: Blockquote Blank Continuation Loses `>` Prefix (Medium)

**Behavior difference:**
- **Input:** `> ` (blockquote prefix + trailing spaces on blank continuation line)
- **Python output:** `> ` (blockquote-prefixed blank line preserved)
- **Rust output:** `` (bare empty line, no `>` prefix)

**Demonstrated diff (9 instances):**

```diff
 > 2. **Review the previous** for context:
->
+
 >    - Check the section
```

**Root cause:** The blockquote renderer strips the `>` prefix from blank continuation
lines inside blockquotes, outputting a bare empty line.
Python preserves the blockquote prefix on blank lines within blockquotes.

**Impact:** Medium — affects blockquotes with lists containing blank continuation lines.
Could cause re-parsing issues if the bare empty line breaks the blockquote context.

**Fix approach:** Ensure blockquote blank continuation lines retain the `>` prefix in
the rendered output.

**Failing tests:**
- `test_d13_blockquote_blank_continuation_preserves_prefix`
- `test_d13_blockquote_list_with_blank_continuation`

### P8: Escaped Backtick Stripped in Table Inline Code (Medium)

**Behavior difference:**
- **Input:** `` `throw new CLIError(\`${msg}: ${error.message}\`)` `` in table cell
- **Python output:** Both ``` escapes preserved
- **Rust output:** Trailing `\`` stripped → `` `throw new CLIError(\`${msg}:
  ${error.message}`)` ``

**Demonstrated diff (1 instance):**

```diff
-| Include original: `throw new CLIError(\`${msg}: ${error.message}\`)` |
+| Include original: `throw new CLIError(\`${msg}: ${error.message}`)` |
```

**Root cause:** Related to P3 (ESCAPE_CHARS) — backtick (```) within inline code in
table cells is not being properly preserved.
The first escaped backtick is kept but the trailing one is stripped.

**Fix approach:** Investigate how escaped backticks within inline code are handled,
particularly in table cell context.
May require extending the PUA placeholder system or fixing inline code rendering.

**Failing tests:**
- `test_d14_escaped_backtick_in_table_inline_code`

### P9: Smart Quote After Inline Code Backtick (Low)

**Behavior difference:**
- **Input:** `` `foo()`'s result ``
- **Python output:** Straight apostrophe preserved: `` `foo()`'s ``
- **Rust output:** Converted to smart quote: `` `foo()`\u{2019}s ``

**Root cause:** Rust’s smart quote engine converts apostrophes that immediately follow a
closing inline code backtick.
Python’s smart quote engine does not convert in this context, likely because the
backtick-ended inline code acts as a boundary that prevents the smart quote heuristic
from triggering.

**Fix approach:** Adjust the smart quote engine to not convert apostrophes that
immediately follow an inline code span boundary (closing backtick).

**Failing tests:**
- `test_d15_no_smart_quote_after_inline_code`

### Resolution Plan

**Priority order (by impact and fix difficulty):**

1. **P6 (code fence blank line)** — Easy fix, **highest real-world impact** (454
   instances). Add paragraph→CodeBlock to `suppress_for_tight`. Estimated: 1 hour.
2. **P3 (escape chars)** — Easy fix, high impact.
   Add missing chars to `ESCAPE_CHARS`. Estimated: 1 hour.
3. **P9 (smart quote after inline code)** — Easy fix, low impact.
   Adjust smart quote context.
   Estimated: 1 hour.
4. **P4 (nested list spacing)** — Medium fix, medium impact.
   Adjust list renderer spacing logic.
   Estimated: 2-4 hours.
5. **P7 (blockquote blank continuation)** — Medium fix, medium impact.
   Fix blockquote blank line rendering.
   Estimated: 2-4 hours.
6. **P8 (escaped backtick in table)** — Medium fix, medium impact.
   Fix inline code escape handling in tables.
   Estimated: 2-4 hours.
7. **P1 (reference links)** — Hard fix, critical impact.
   Requires pre-parse extraction system (similar to existing PUA escape system).
   Estimated: 1-2 days.
8. **P2 (footnote position)** — Hard fix, critical impact.
   Requires position tracking through comrak’s AST reordering.
   Estimated: 1-2 days.
9. **P5 (verbose)** — No fix needed.
   Accepted addition.

**Completion criteria:** All tryscript tests pass against both binaries with no masking
patterns. `diff` between Rust and Python output on all fixture files produces zero
differences.
