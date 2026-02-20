# Feature: Parity Review and Playbook Sync

**Date:** 2026-02-19 (last updated 2026-02-20)

**Author:** Joshua Levy

**Status:** Draft

## Overview

This spec consolidates all remaining work to achieve full CLI/feature parity with Python
flowmark v0.6.4 and to reconcile the project's documentation with the
rust-porting-playbook.

It covers two work streams:

1. **Part A: CLI & Feature Parity** — All core features (file resolver, config loading,
   CLI flags, skill system) have been ported. Tryscript golden tests have been established.
   Remaining work: housekeeping (stale YAMLs, mapping notes), upstream contributions, and
   formal acceptance validation.
2. **Part B: Playbook Review & Sync** — Update the porting playbook case study with
   current metrics, integrate lessons learned, fix stale documentation, and backfill 13
   pending observations.

**Epic bead:** fmr-7mmt (CLI parity)

## Current Parity Status (audited 2026-02-20)

### Test Mapping

| Metric | Value |
| --- | --- |
| Python tests (v0.6.4) | 292 |
| Mapped to Rust | 292 (100%) |
| Excluded | 0 |
| Missing | 0 |
| Partial | 0 |
| Rust tests total | 442 (`cargo test --list`) |
| Rust tests passing | 437 |
| Rust tests failing | 5 (D11 binary comparison — require Python binary in PATH) |
| Rust tests ignored | 0 |
| Extra Rust tests (no Python equivalent) | 102 (Rust-only unit tests, edge cases, tryscript wrappers) |
| Tryscript golden test files | 11 |
| check-mapping | **PASS** |

### Phase Completion

| Phase | Status | Evidence |
| --- | --- | --- |
| Phase 1: File Resolver | **COMPLETE** | `src/file_resolver/` (453 LOC), 35 tests passing |
| Phase 2: Config Loading | **COMPLETE** | `src/config.rs` (381 LOC), 20 tests passing |
| Phase 3: CLI Flag Parity | **COMPLETE** | All flags in `src/main.rs`, 21 tests passing |
| Phase 4: Skill System | **COMPLETE** | `src/skills/mod.rs` (82 LOC), 11 tests passing |
| Phase 5: Tryscript Golden Tests | **COMPLETE** | 11 tryscript files, 12 Rust test wrappers, CI job |
| Phase 6: Test Mapping & CI | **MOSTLY COMPLETE** | 292/292 mapped, CI with check-mapping; YAML stale |
| Phase 7: Upstream Contributions | **NOT STARTED** | Tryscript tests not yet PR'd to Python repo |
| Phase 8: Playbook Review & Sync | **NOT STARTED** | All playbook docs still stale |
| Phase 9: Final Acceptance | **MOSTLY MET** | See checklist below |

### Feature-by-Feature Parity Status

Every Python v0.6.4 feature has been audited against the Rust implementation.

#### Core Formatting — FULL PARITY

| Feature | Python | Rust | Status |
| --- | --- | --- | --- |
| Markdown line wrapping (width-based) | `fill_markdown()` | `fill_markdown()` | Identical |
| Semantic line breaks (sentence-based) | `--semantic` | `--semantic` | Identical |
| Smart quotes | `--smartquotes` | `--smartquotes` | Identical |
| Ellipses | `--ellipses` | `--ellipses` | Identical |
| Cleanups (unbold headings) | `--cleanups` | `--cleanups` | Identical |
| Plaintext mode | `--plaintext` | `--plaintext` | Identical |
| YAML frontmatter preservation | `split_frontmatter()` | `split_frontmatter()` | Identical |
| List spacing (preserve/loose/tight) | `--list-spacing` | `--list-spacing` | Identical |
| Width control (0=disable) | `--width` | `--width` | Identical |
| GFM: strikethrough, tables, alerts | Marko AST | Comrak AST | Identical output |
| GFM: task lists, footnotes, math | Marko AST | Comrak AST | Identical output |
| HTML tag/template tag handling | Marko AST | Comrak + workarounds | Identical output |
| Escape handling | Marko AST | Comrak + postprocessing | Identical output |
| Hard break preservation | Marko AST | Comrak AST | Identical output |

#### File Operations — FULL PARITY

| Feature | Python | Rust | Status |
| --- | --- | --- | --- |
| Stdin/stdout processing | `-` arg | `-` arg | Identical |
| In-place editing | `--inplace` | `--inplace` | Identical |
| Backup files | `--nobackup` | `--nobackup` | Identical |
| Auto mode preset | `--auto` | `--auto` | Identical expansion |
| Multi-file processing | Sequential | Sequential | Identical |
| Output file | `--output` | `--output` | Identical |
| Verbose mode | `--verbose` | `--verbose` | Identical |

#### File Discovery — FULL PARITY

| Feature | Python | Rust | Status |
| --- | --- | --- | --- |
| Directory recursion | `os.walk` | `ignore::WalkBuilder` | Identical behavior |
| Glob pattern expansion | `pathlib.glob` | `glob` crate | Identical |
| Default includes (`*.md`) | `DEFAULT_INCLUDES` | `DEFAULT_INCLUDES` | Identical |
| Default excludes (35 patterns) | `DEFAULT_EXCLUDES` | `DEFAULT_EXCLUDES` | Identical |
| `--extend-include` | Append patterns | Append patterns | Identical |
| `--exclude` (replace defaults) | Replace all | Replace all | Identical |
| `--extend-exclude` (add to defaults) | Append patterns | Append patterns | Identical |
| `.gitignore` integration | `pathspec` | `ignore` crate | Identical |
| `--no-respect-gitignore` | Disable `.gitignore` | Disable `.gitignore` | Identical |
| `--force-exclude` on explicit files | Filter explicit | Filter explicit | Identical |
| `--files-max-size` (default 1MB) | Size check | Size check | Identical |
| `--list-files` mode | Print and exit | Print and exit | Identical |
| `.flowmarkignore` support | Walk-up search | Walk-up search | Identical |
| Deduplication and sorting | `set` + `sorted` | `BTreeSet` | Identical |

#### Config Loading — FULL PARITY

| Feature | Python | Rust | Status |
| --- | --- | --- | --- |
| `.flowmark.toml` discovery | Walk up dirs | Walk up dirs | Identical |
| `flowmark.toml` discovery | Walk up dirs | Walk up dirs | Identical |
| `pyproject.toml [tool.flowmark]` | Section extract | Section extract | Identical |
| Section flattening (`[formatting]`) | Flatten sub-tables | Flatten sub-tables | Identical |
| Kebab-to-snake mapping (6 keys) | Lookup table | Lookup table | Identical |
| 3-way merge (CLI > config > defaults) | Sentinel parser | `ValueSource` API | Identical |
| Auto-locked fields in `--auto` | 6 locked fields | 6 locked fields | Identical |
| Malformed TOML -> empty config | Catch + warn | Catch + warn | Identical |
| Unknown key warnings to stderr | `eprintln` | `eprintln` | Identical |

#### Skill System — FULL PARITY

| Feature | Python | Rust | Status |
| --- | --- | --- | --- |
| `--skill` (print SKILL.md) | Read from package | `include_str!()` | Identical output |
| `--install-skill` | Write to `~/.claude/` | Write to `~/.claude/` | Identical |
| `--agent-base` custom dir | Custom path | Custom path | Identical |
| `--docs` (print documentation) | Read README.md | Embed/fallback | Functionally equivalent |
| Directory creation | `mkdir -p` | `create_dir_all` | Identical |
| Error handling | `PermissionError` | `Err(String)` | Identical UX |

#### Error Messages — NEAR-FULL PARITY

| Error Scenario | Python Message | Rust Message | Match |
| --- | --- | --- | --- |
| No input | `Error: No input specified. Provide files, directories (use '.' for current directory), or '-' for stdin. Use --help for more options.` | Identical | Exact |
| `--auto` no files | `Error: --auto requires at least one file or directory argument (use '.' for current directory, --help for more options)` | Identical | Exact |
| `--list-files` no files | `Error: --list-files requires at least one file or directory argument (use '.' for current directory, --help for more options)` | Identical | Exact |
| `--inplace` with stdin | `Error: Cannot use \`inplace\` with stdin` | Identical | Exact |
| `--output` with multiple files | `Error: Cannot specify output file when processing multiple files (use --inplace instead)` | Identical | Exact |
| Nonexistent file | `Error: [Errno 2] No such file or directory: 'X'` (exit 2) | `Error: Path not found: X` (exit 1) | **Accepted difference** |

### All Remaining Gaps (exhaustive)

These are ALL known remaining gaps between Python v0.6.4 and Rust, with nothing omitted:

#### Gap 1: Nonexistent File Error Format (Accepted)

- **Python**: `Error: [Errno 2] No such file or directory: 'nonexistent.md'` with exit
  code 2
- **Rust**: `Error: Path not found: nonexistent.md` with exit code 1
- **Reason**: `[Errno 2]` is a Python-ism that cannot be meaningfully replicated in Rust.
  Both messages start with `Error:` and include the filename.
- **Impact**: Low. Tools parsing stderr would need adjustment, but no user would notice.
- **Decision**: Accepted difference. Documented in `test_d11_nonexistent_file_error_format`.

#### Gap 2: Version Output Format (Accepted)

- **Python**: `flowmark 0.6.4`
- **Rust**: `flowmark 0.7.0 (parity: flowmark-py 0.6.4)`
- **Reason**: Rust version is a different release line. Parity annotation aids debugging.
- **Impact**: None for typical usage. Only affects `--version` output parsing.
- **Decision**: Accepted difference. Intentional.

#### Gap 3: D11 Parity Tests Require Both Binaries (Infrastructure)

Five tests in `tests/test_parity_discrepancies.rs` (D11 group) compare Rust binary output
against Python binary output at runtime. They **pass in CI** (where Python flowmark is
installed via `uv tool install flowmark==0.6.4`) but **fail locally** when `flowmark`
(Python) is not in PATH.

- `test_d11_no_args_error_matches_python`
- `test_d11_auto_no_args_error_matches_python`
- `test_d11_inplace_stdin_error_matches_python`
- `test_d11_output_multiple_files_error_matches_python`
- `test_d11_nonexistent_file_error_format`

This is a test infrastructure requirement, not a feature gap.

#### Gap 4: Stale Rust Test YAML (Housekeeping)

`admin/port-coverage-mapping/rust-tests.yaml` records 408 tests, but `cargo test --list`
finds 442 tests. 34 tests have been added since the YAML was last regenerated. The YAML
needs regeneration via `flowmark-dev discover-rust`, and the smoke test assertion in
`python/tests/test_smoke.py` needs updating from 408 to the new count.

#### Gap 5: Stale Mapping Notes (Housekeeping)

Three mapping entries reference bugs as "Ignored: known bug" but the bugs are FIXED:

| Bug ID | Test | Actual Status |
| --- | --- | --- |
| fmr-2tll | `test_escape_in_list_item` / `test_mixed_escapes` | FIXED |
| fmr-4l1x | `test_heading_with_hard_break_in_list` | FIXED |
| fmr-5ojk | `test_list_item_with_tag_on_continuation_line` | FIXED |

The mapping notes should be updated to remove the "Ignored" language and note the fix.

#### Gap 6: Python Source Not Available Locally (Setup)

The original spec references `attic/flowmark/` for Python source, but this directory does
not exist. Python source lives at `repos/flowmark/` which is populated on demand by
`flowmark-dev discover-python` (clones from GitHub). Local manual comparison of Python vs
Rust source requires running `flowmark-dev discover-python --local-path <path>` or
cloning the Python repo separately.

#### Gap 7: 102 Unmapped Rust Tests (By Design)

`check-mapping` reports 102 Rust tests not referenced in the mapping. These are Rust-only
additions that have no Python equivalent:

- 46 unit tests in `src/` (Comrak workaround tests, PUA encoding, regex tests)
- 25 edge case tests (`tests/test_edge_cases.rs`)
- 11 tryscript golden test wrappers (`tests/test_tryscript_golden.rs`)
- 3 PUA collision/safety tests
- 2 CRLF frontmatter tests
- 2 permission preservation tests
- 2 path traversal safety tests
- 2 smart quote redistribution tests
- Various other Rust-specific tests

This is not a gap. These are extra coverage in Rust beyond what Python has.

## Goals

- [x] Port all remaining Python CLI features: file resolver, config loading, CLI flags,
  skill system
- [x] Port all previously excluded tests
- [x] Achieve `check-mapping` pass: 292 mapped, 0 excluded, 0 missing, 0 partial
- [x] Establish tryscript-based golden tests for end-to-end CLI validation
- [ ] Regenerate stale YAMLs and clean up mapping notes (Gap 4, Gap 5)
- [ ] Update playbook case study to reflect the current port (Phase 8)
- [ ] Integrate 13 Phase 7C observations into the playbook (Phase 8.6)
- [ ] Backfill all lessons from the porting log into playbook documents (Phase 8)
- [ ] Fix all stale metrics, contradictions, and documentation gaps (Phase 8)
- [ ] PR tryscript tests to the Python flowmark repo (Phase 7)

## Non-Goals

- Performance benchmarking (tracked separately as fmr-aq8o)
- Adding features beyond Python v0.6.4
- Restructuring the playbook's 8-phase methodology
- Adding entirely new playbook documents (only updating existing ones)

## Background

### Part A: CLI & Feature Parity

The exact parity spec (Phases 1-9) achieved byte-for-byte formatting parity across all
formatting modes. All Python v0.6.4 features have been ported to Rust:

| Feature Area | Rust Module | Tests | Status |
| --- | --- | --- | --- |
| File resolver | `src/file_resolver/` (453 LOC) | 35 | Complete |
| Config loading | `src/config.rs` (381 LOC) | 20 | Complete |
| CLI flags (all 22) | `src/main.rs` | 21 | Complete |
| Skill system | `src/skills/mod.rs` (82 LOC) | 11 | Complete |
| Core formatting | `src/formatter/`, `src/wrapping/`, etc. | 250+ | Complete |
| Tryscript golden tests | `tests/tryscript/` (11 files) | 12 wrappers | Complete |

### Part B: Playbook Review & Sync

The rust-porting-playbook was built primarily from the **first** flowmark-rs port
(flowmark-rs-1, in `repos/rust-porting-playbook/`). That port achieved:

- 141 tests (93 unit + 42 integration + 6 doctests), 2 ignored
- ~95% cross-validation match
- 14 library workarounds, 3 accepted differences
- Rust/Python LOC ratio: ~1.7x app code

The **current** flowmark-rs (this repo) is a fresh reimplementation that achieved:

- 442 tests, 0 ignored
- 292 mapped Python tests, 0 excluded
- 100% of Python tests passing, 0 partial mappings
- Rust/Python code lines ratio: ~1.00x

The playbook case study docs still describe the **old** port. This spec addresses that
gap.

### Key Discrepancies Found

#### D1: Stale Case Study Metrics (CRITICAL)

The playbook README and case study docs describe the **old** flowmark-rs-1 port:

| Metric | Playbook Says | Actual (Current Port) |
| --- | --- | --- |
| Test count | 141 (93+42+6), 2 ignored | 442, 0 ignored |
| Python test mapping | Not tracked | 292 mapped, 0 excluded, 0 missing |
| Rust/Python LOC ratio (app) | ~1.7x | ~1.0x |
| Cross-validation | ~95% match | 100% of mapped tests |
| Workarounds | "14 fixable, 3 unfixable" | Different set (new implementation) |

#### D2: Workaround Count Inconsistencies

Workaround counts differ across documents: 13, 14, 15, 17 appear in different places.
The current port has a different set of workarounds than the old port.

#### D3: `porting-checklist.md` is a Stale Duplicate

`docs/project/specs/active/porting-checklist.md` in this repo is a copy of the
playbook's `reference/python-to-rust-playbook.md`. It may be out of sync.

#### D5: Phase 7C Observations Not Integrated

13 observations were recorded in
`repos/rust-porting-playbook/case-studies/flowmark/flowmark-port-observations-2.md` but
not yet integrated into the playbook.

#### D6: Code Review Findings vs Playbook Best Practices

| Code Review Issue | Playbook Coverage |
| --- | --- |
| P0: Clippy failures (9 errors) | Playbook says clippy pedantic — but as warn not deny |
| P0: Formatting violations | Playbook says `cargo fmt` — not enforced |
| P0.5: Lint config gaps (warn vs deny) | Playbook says pedantic warn, code review says deny |
| P1: Dead dependencies | Playbook has no guidance on dependency pruning |
| P1: Dead error variants | Playbook has no dead-code-detection guidance |
| P2: Code duplication (fence tracking) | Code review checklist has nothing on duplication |
| P2: Unnecessary allocations | Code review checklist covers "hot path allocation" |
| P2: Boolean parameter overload | Playbook recommends options structs |
| P3: Stale comments ("Same as Black") | Playbook has no Python-reference-cleanup guidance |

#### D7-D10: Other Issues

- **D7**: `XXX:` comment convention -> should be `HACK:`/`FIXME:` per updated playbook
- **D8**: Non-compiling code examples in playbook porting guide
- **D9**: 53+ playbook fixes (plan-2026-02-08) — unclear if all applied
- **D10**: Playbook references archived/stale crates (`serde_yaml`, `once_cell`,
  `actions/checkout@v5`, `color-eyre`)

### Comprehensive Retrospective → Playbook Action Map

A full retrospective of the porting log (L1-L11), code review (P0-P3), spec analysis,
and new methodology developed during this port. Every finding is mapped to a specific
playbook action, target document, and work item (WI-N).

#### Lessons Learned (L1-L11) → Playbook Principles & Guidelines

| # | Finding | Action | Target Document(s) | WI |
| --- | --- | --- | --- | --- |
| F1 | L1: Always verify source language's actual byte output | ADD technique | `porting-principles-and-antipatterns.md` | WI-5 |
| F2 | L2: Use `assert_eq!` with exact output, not weak assertions | ADD technique | `porting-principles-and-antipatterns.md`, `test-coverage-for-porting.md` | WI-5 |
| F3 | L3: Test edge cases, not just happy path | ADD technique | `test-coverage-for-porting.md` | WI-5 |
| F4 | L4: Comrak loose/tight classification is recurring bug source | ADD to case study | `flowmark-port-analysis.md` | WI-7 |
| F5 | L5: Post-merge corpus validation essential | ADD as Principle 9 candidate | `porting-principles-and-antipatterns.md` | WI-5 |
| F6 | L6: Smart quote context depends on surrounding chars | ADD to case study | `flowmark-port-analysis.md` | WI-7 |
| F7 | L7: Don't trust CI alone — read the diff | ADD technique | `porting-principles-and-antipatterns.md` | WI-5 |
| F8 | L8: Error parity is first-class surface | ADD technique | `python-to-rust-cli-porting.md` | WI-5, WI-9 |
| F9 | L9: Extract corner-cases from corpus into regression corpus | ADD as Principle 9 candidate | `porting-principles-and-antipatterns.md` | WI-5 |
| F10 | L10: Red/green discipline for parity fixes | ADD technique | `porting-principles-and-antipatterns.md` | WI-5 |
| F11 | L11: Dynamic parity assertions > static assertions | ADD as Principle 9 | `porting-principles-and-antipatterns.md` | WI-5 |
| F12 | PR #17 false parity: tests passed but were wrong | ADD anti-pattern | `porting-principles-and-antipatterns.md` | WI-5 |

#### Code Review Findings → Playbook Gaps

| # | Finding | Action | Target Document(s) | WI |
| --- | --- | --- | --- | --- |
| F13 | P0: Clippy pedantic as `deny`, not `warn` | FIX | `rust-project-setup.md`, `python-to-rust-porting-rules.md` | WI-4 |
| F14 | P0.5: Lint config Cargo.toml `warn` vs CI `deny` mismatch | FIX | `rust-project-setup.md` | WI-4 |
| F15 | P1: Dead dependency detection (`cargo machete`) not covered | ADD | `rust-code-review-checklist.md` | WI-8 |
| F16 | P1: Dead error variants / speculative additions not covered | ADD | `rust-code-review-checklist.md` | WI-8 |
| F17 | P2: Code duplication detection not in checklist | ADD | `rust-code-review-checklist.md` | WI-8 |
| F18 | P3: Stale Python-reference comments — no cleanup guidance | ADD | `python-to-rust-porting-rules.md` | WI-9 |

#### Spec Analysis → Playbook Factual Fixes

| # | Finding | Action | Target Document(s) | WI |
| --- | --- | --- | --- | --- |
| F19 | Effort allocation sums to 105% | FIX | `python-to-rust-playbook.md` | WI-4 |
| F20 | Pitfall #6 has identical wrong/correct examples | FIX | `python-to-rust-porting-rules.md` | WI-4 |
| F21 | `assert` → `debug_assert!` dangerous mapping | FIX | `python-to-rust-mapping-reference.md` | WI-4 |
| F22 | Stale crate references (`serde_yaml`, `once_cell`) | FIX | `rust-cli-best-practices.md`, `rust-general-rules.md` | WI-4 |
| F23 | Stale GitHub Actions versions (`actions/checkout@v5`) | FIX | `rust-project-setup.md` | WI-4 |
| F24 | Non-compiling `build.rs` examples | FIX | `python-to-rust-porting-guide.md` | WI-4 |
| F25 | `color-eyre` maintenance status not noted | FIX | `rust-cli-best-practices.md` | WI-4 |
| F26 | `XXX:` → `HACK:`/`FIXME:` convention | FIX | `python-to-rust-porting-rules.md`, source code | WI-2, WI-4 |
| F27 | Version constraint mappings swapped | FIX | `python-to-rust-mapping-reference.md` | WI-4 |
| F35 | Edition 2024 `resolver = "3"` not documented | FIX | `rust-project-setup.md` | WI-4 |

#### New Methodology → Playbook New Content

| # | Finding | Action | Target Document(s) | WI |
| --- | --- | --- | --- | --- |
| F28 | Cross-language test mapping system (YAML-based) | ADD reference doc | NEW: `cross-language-test-mapping.md` | WI-6 |
| F29 | `flowmark-dev` CLI for test discovery + mapping | ADD tool pattern | NEW: `cross-language-test-mapping.md` | WI-6 |
| F30 | Test mapping as CI hard gate | ADD enforcement pattern | NEW: `cross-language-test-mapping.md`, `python-to-rust-playbook.md` | WI-6 |
| F31 | Golden test with 4 formatting modes | ADD testing pattern | `test-coverage-for-porting.md` | WI-9 |
| F32 | Exact parity spec as tracking document | ADD project management | `python-to-rust-playbook.md` | WI-8 |
| F33 | `pub(crate)` visibility audit as post-port step | ADD step | `rust-code-review-checklist.md` | WI-8 |
| F34 | Unicode PUA placeholder pattern for escape preservation | ADD technique | `flowmark-port-analysis.md` | WI-7 |

#### Case Study & Infrastructure Updates

| # | Finding | Action | Target Document(s) | WI |
| --- | --- | --- | --- | --- |
| F36 | All 7 case study docs use old port metrics | UPDATE | All `case-studies/flowmark/*.md` | WI-7 |
| F37 | Workaround counts inconsistent (13/14/15/17) | RECONCILE | All case study docs | WI-7 |
| F38 | 13 Phase 7C observations not integrated | TRIAGE+APPLY | Various playbook docs | WI-10 |
| F39 | Playbook README uses old port data | UPDATE | `README.md` | WI-11 |
| F40 | Internal cross-reference links may be stale | VERIFY | All playbook docs | WI-11 |
| F41 | `rust-tests.yaml` stale (408 vs 442) | REGENERATE | `admin/port-coverage-mapping/rust-tests.yaml` | WI-3 |
| F42 | Smoke test assertions stale | UPDATE | `python/tests/test_smoke.py` | WI-3 |
| F43 | Mapping notes reference fixed bugs as "Ignored" | CLEAN UP | `admin/port-coverage-mapping/test-mapping.yaml` | WI-3 |
| F44 | `porting-checklist.md` is stale duplicate | REMOVE | `docs/porting-checklist.md` | WI-2 |

### Document Inventory

#### flowmark-rs docs (this repo: `docs/`)

| Document | Lines | Current Status | Notes |
| --- | --- | --- | --- |
| `project/specs/done/porting-plan.md` | 137 | Complete | Updated and moved to specs/done |
| `porting-checklist.md` | 643 | Duplicate | Copy of playbook's `python-to-rust-playbook.md` |
| `code-review-2026-02-17.md` | 471 | Current | Fresh review with P0-P3 issues |
| `specs/active/plan-2026-02-17-exact-parity.md` | 751 | Complete | Comprehensive parity spec |
| `specs/active/plan-2026-02-17-test-mapping-meta-test.md` | 482 | Implemented | Test mapping infrastructure |

#### rust-porting-playbook (`repos/rust-porting-playbook/`)

**Reference docs (11 files, ~5,289 lines):**

| Document | Lines | Status |
| --- | --- | --- |
| `reference/python-to-rust-playbook.md` | ~619 | Core doc — needs metric updates |
| `reference/python-to-rust-mapping-reference.md` | ~788 | Comprehensive — review for accuracy |
| `reference/python-to-rust-porting-guide.md` | ~807 | Detailed methodology — check code examples |
| `reference/rust-cli-best-practices.md` | ~832 | Extensive — check versions/deps |
| `reference/rust-code-review-checklist.md` | ~285 | Good shape — validate against code review |
| `reference/python-to-rust-test-coverage-playbook.md` | ~312 | Review against actual test strategy |
| `reference/port-checklist-initial-template.md` | ~546 | Template — validate completeness |
| `reference/port-checklist-update-template.md` | ~379 | Template — validate completeness |
| `reference/meta-improving-this-playbook.md` | ~236 | Process doc — current |
| `reference/case-study-observations-template.md` | ~252 | Template — current |
| `reference/case-study-improvement-triage-template.md` | ~148 | Template — current |

**Guidelines (6 files, ~2,246 lines):**

| Document | Lines | Status |
| --- | --- | --- |
| `guidelines/python-to-rust-porting-rules.md` | ~360 | Core rules — multiple fixes identified |
| `guidelines/python-to-rust-cli-porting.md` | ~285 | CLI porting — validate |
| `guidelines/rust-general-rules.md` | ~286 | General Rust — review |
| `guidelines/rust-cli-app-patterns.md` | ~403 | CLI patterns — review |
| `guidelines/rust-project-setup.md` | ~626 | Project setup — critical for accuracy |
| `guidelines/test-coverage-for-porting.md` | ~286 | Test coverage — review |

**Case studies (8 files, ~5,400 lines):**

| Document | Lines | Status |
| --- | --- | --- |
| `case-studies/flowmark/flowmark-port-analysis.md` | ~326 | STALE — describes old port |
| `case-studies/flowmark/flowmark-port-library-choices.md` | ~257 | Partially current |
| `case-studies/flowmark/flowmark-port-decision-log.md` | ~523 | STALE — old decisions |
| `case-studies/flowmark/flowmark-port-migration-plan-v1.md` | ~3,339 | Renamed — v1 port |
| `case-studies/flowmark/flowmark-port-migration-plan-v2.md` | ~400 | NEW — v2 port |
| `case-studies/flowmark/flowmark-port-cross-validation.md` | ~189 | STALE — old validation |
| `case-studies/flowmark/flowmark-port-comrak-bug.md` | ~211 | Partially current |
| `case-studies/flowmark/flowmark-port-wrapping-solution.md` | ~155 | STALE — old approach |

## Design

### Approach

Part A (CLI parity) is complete. The methodology matched the exact parity spec: port
each Python module, write tests that match the Python test suite, validate via
cross-language test mapping, enforce in CI. Remaining Part A work is housekeeping and
upstream contributions.

Part B (playbook sync) is a documentation reconciliation: inventory every document,
verify against current state, fix discrepancies, backfill lessons.

## Implementation Plan

### Phase 1-4: Feature Porting (COMPLETE)

All four feature-porting phases have been implemented:

- **Phase 1: File Resolver** — `src/file_resolver/` with 4 submodules (`mod.rs`,
  `config.rs`, `defaults.rs`, `gitignore.rs`, `resolver.rs`), 453 LOC total, 35 Rust
  tests passing (`tests/test_file_resolver.rs`)
- **Phase 2: Config Loading** — `src/config.rs`, 381 LOC, `FlowmarkConfig` struct with
  TOML loading, section flattening, kebab-to-snake mapping, three-way merge, 20 Rust
  tests passing (`tests/test_config.rs`)
- **Phase 3: CLI Flag Parity** — All 22 flags implemented in `src/main.rs` Args struct
  including `--extend-include`, `--exclude`, `--extend-exclude`,
  `--no-respect-gitignore`, `--force-exclude`, `--list-files`, `--files-max-size`,
  `--skill`, `--install-skill`, `--agent-base`, `--docs`. Explicit-flag tracking via
  clap `ValueSource`. 21 Rust tests passing (`tests/test_cli_file_discovery.rs`)
- **Phase 4: Skill System** — `src/skills/mod.rs`, 82 LOC, with `get_skill_content()`,
  `get_docs_content()`, `install_skill()`. SKILL.md embedded via `include_str!()`.
  11 Rust tests passing (`tests/test_skill.rs`)

### Phase 5: Tryscript CLI Golden Tests (COMPLETE)

11 tryscript golden test files established in `tests/tryscript/`:

| File | Scenarios |
| --- | --- |
| `auto-mode.tryscript.md` | Auto mode on directory and files |
| `cli-golden.tryscript.md` | Core CLI formatting scenarios |
| `config-interaction.tryscript.md` | Config file loading and merging |
| `errors-version.tryscript.md` | Error messages and version output |
| `file-discovery.tryscript.md` | File discovery with patterns |
| `file-ops.tryscript.md` | In-place editing, backups |
| `formatting.tryscript.md` | Formatting modes |
| `list-spacing.tryscript.md` | List spacing modes |
| `stdin.tryscript.md` | Stdin processing |
| `typography-tests.tryscript.md` | Smart quotes and ellipses |
| `verbose-docs.tryscript.md` | Verbose mode and docs output |

12 Rust test wrapper functions in `tests/test_tryscript_golden.rs` invoke these via
`tryscript run`. CI pre-installs tryscript globally (`npm install -g tryscript@latest`)
and Python flowmark (`uv tool install flowmark==0.6.4`).

### Phase 6: Test Mapping and CI (MOSTLY COMPLETE)

Completed:
- [x] All 292 Python tests mapped (0 excluded, 0 missing)
- [x] `check-mapping` passes in CI
- [x] Tryscript CI job in `.github/workflows/ci.yml`
- [x] `flowmark-dev check-mapping` as CI gate

Remaining housekeeping:
- [ ] Regenerate `admin/port-coverage-mapping/rust-tests.yaml` (records 408, actual 442)
- [ ] Update `python/tests/test_smoke.py` Rust test count assertion (408 -> new count)
- [ ] Clean up stale mapping notes referencing fixed bugs (fmr-2tll, fmr-4l1x, fmr-5ojk)

### Phase 7: Upstream Contributions (NOT STARTED)

**Bead:** fmr-03xy | **Priority:** P2

PR tryscript tests and any needed end-to-end tests to the Python flowmark repo
(`github.com/jlevy/flowmark`) to ensure parity.

- [ ] PR tryscript tests to the Python flowmark repo (if not already present)
- [ ] PR any missing CLI test coverage discovered during the audit
- [ ] Bump the Python source pin from `v0.6.4` to the version that includes the new
  tests (once merged)
- [ ] Update `flowmark-dev discover-python` to pick up new test functions

### Phase 8: Playbook Review & Sync (NOT STARTED)

Systematically reconcile all documentation in the flowmark-rs project against the
rust-porting-playbook. This is bidirectional: flowmark-rs docs inform playbook
improvements, and playbook best practices inform remaining flowmark-rs cleanup.

#### 8.1: Verify Playbook Fix Status

Before making new changes, verify what has already been done.

- [ ] Check if the 53+ fixes from `plan-2026-02-08-playbook-review-fixes.md` were
  actually applied to the playbook documents (beads closed != changes committed)
- [ ] Check the status of the comprehensive review from
  `plan-2026-02-12-comprehensive-playbook-review.md`
- [ ] Identify which playbook spec changes are already implemented vs still pending
- [ ] Check `XXX:` -> `HACK:`/`FIXME:` convention change status in playbook docs
- [ ] Check for non-compiling code examples identified in the review
- [ ] Grep playbook for archived crate references (`serde_yaml`, `once_cell`,
  `actions/checkout@v5`, `actions/create-release@v1`)
- [ ] Create a status matrix: {fix-id, target-file, applied-or-not}

#### 8.2: Review flowmark-rs Docs Against Current State

Ensure this project's own docs are accurate.

- [x] **`porting-plan.md`**: Updated with "Status: Complete" header, checked acceptance
  criteria, verified module mapping against actual `src/` layout, moved to
  `docs/project/specs/done/porting-plan.md`.
- [ ] **`porting-checklist.md`**: Determine if this should be removed (it's a duplicate
  of the playbook). If kept, verify it matches the current playbook version.
  Decision: remove or convert to a project-specific checklist with checked items.
- [ ] **`code-review-2026-02-17.md`**: Cross-reference all findings against playbook
  best practices. For each finding, note whether the playbook covers it, and if not, flag
  as a playbook gap (ADD).
- [ ] **Exact parity spec**: Verify "Complete" status is accurate. Check all appendices
  for correctness.
- [ ] **Test mapping spec**: Verify "Implemented" status. Check that workflow descriptions
  match actual `flowmark-dev` CLI behavior.
- [ ] Check for any `XXX:` comments in flowmark-rs source code that should be
  `HACK:`/`FIXME:` per updated playbook convention
- [ ] Verify `HACK:` and `FIXME:` comments exist where playbook says they should (all
  library workarounds documented)

#### 8.3: Review Playbook Case Study Against Current Port

Update all 7 case study documents to reflect the current port.

- [ ] **`flowmark-port-analysis.md`**: Update metrics (LOC, test counts, ratios).
  Update "what's automatable" assessment based on this port's experience.
  Note that the new port used a cross-language test mapping system not in the original.

- [ ] **`flowmark-port-library-choices.md`**: Verify library choices match current
  `Cargo.toml`. Update comrak version references. Note any new library decisions.

- [ ] **`flowmark-port-decision-log.md`**: Update or add entries for decisions made in
  the new port (e.g., test mapping infrastructure, CI hardening, lint configuration).
  Fix the D7 wrapping solution contradiction identified in plan-2026-02-08.

- [ ] **`flowmark-port-migration-plan.md`**: This is the longest doc (3,339 lines).
  Decide: update in place, add a "v2 port" appendix, or create a separate doc for the
  new port's migration narrative.

- [ ] **`flowmark-port-cross-validation.md`**: Update with current cross-validation
  results (100% mapped tests passing, 0 ignored). Update escape handling table.

- [ ] **`flowmark-port-comrak-bug.md`**: Verify still relevant. Check if any comrak bugs
  were fixed upstream since the original doc.

- [ ] **`flowmark-port-wrapping-solution.md`**: Update with current wrapping approach.
  The doc describes two approaches — verify which one the current port uses and update.

- [ ] **Reconcile workaround counts** across all 7 case study docs. Establish a single
  authoritative count for the current port by grepping `HACK:` comments in source.

#### 8.4: Review Playbook Reference Docs

Review each reference doc against this port's experience.

- [ ] **`python-to-rust-playbook.md`** (core playbook):
  - [ ] Verify effort allocation table sums to 100% (identified as 105% in review)
  - [ ] Update "Key insight" with data from both ports
  - [ ] Check Phase 4.3 (submodule setup) — the current port uses `repos/` clones
    instead of submodules; document both approaches
  - [ ] Check Phase 4.6 (version tracking) — verify recommendation matches practice
  - [ ] Validate Phase 7 (finalize) CLI parity section against actual CLI state
  - [ ] Check Phase 8 (sync) — not yet exercised; note this

- [ ] **`python-to-rust-mapping-reference.md`**:
  - [ ] Verify type mappings against actual code translations
  - [ ] Check `dict` -> `HashMap` insertion-order warning is present
  - [ ] Check `assert` -> `debug_assert!` dangerous mapping is fixed
  - [ ] Verify version constraint mappings (identified as swapped)
  - [ ] Check for `Cow<'_, str>` in type mappings
  - [ ] Check for `re.search()` and `re.fullmatch()` regex mappings
  - [ ] Check `str.find()` byte-offset warning
  - [ ] Verify dunder methods -> traits table exists

- [ ] **`python-to-rust-porting-guide.md`**:
  - [ ] Verify `build.rs` code examples compile
  - [ ] Check version tracking recommendations against actual practice
  - [ ] Validate cross-validation script template
  - [ ] Check 9 critical pitfalls against this port's experience

- [ ] **`rust-cli-best-practices.md`**:
  - [ ] Verify recommended crate versions are current
  - [ ] Check `color-eyre` maintenance status note
  - [ ] Verify CI workflow uses current GitHub Actions versions
  - [ ] Check `cargo-dist` mentioned alongside `cargo-release`
  - [ ] Validate lint configuration against code review recommendations

- [ ] **`rust-code-review-checklist.md`**:
  - [ ] Run the checklist against the code-review-2026-02-17.md findings
  - [ ] Identify any findings the checklist would NOT have caught
  - [ ] Flag checklist gaps as ADD items

- [ ] **`python-to-rust-test-coverage-playbook.md`**:
  - [ ] Compare recommended test strategy against actual test mapping approach
  - [ ] Note that flowmark-rs developed a cross-language test mapping system beyond what
    the playbook describes — flag as ADD

- [ ] **Checklist templates** (`port-checklist-initial-template.md`,
  `port-checklist-update-template.md`):
  - [ ] Walk through each checklist item against the flowmark-rs port
  - [ ] Mark items that were done, skipped, or done differently
  - [ ] Flag missing checklist items discovered during this port

#### 8.5: Review Playbook Guidelines

Review each guideline against this port's experience.

- [ ] **`python-to-rust-porting-rules.md`**:
  - [ ] Check Pitfall #6 (identical wrong/correct examples — identified in review)
  - [ ] Verify `assert` -> `debug_assert!` fix applied
  - [ ] Check `frozenset` note accuracy
  - [ ] Check acceptance criteria include clippy
  - [ ] Verify comment convention uses `HACK:`/`FIXME:` (not `XXX:`)

- [ ] **`python-to-rust-cli-porting.md`**:
  - [ ] Validate argparse -> clap mappings against actual CLI
  - [ ] Check SIGPIPE handling recommendation matches implementation
  - [ ] Verify exit code guidance

- [ ] **`rust-general-rules.md`**:
  - [ ] Check Edition 2024 guidance completeness
  - [ ] Verify `LazyLock` recommendation (not `once_cell`)
  - [ ] Check ownership patterns against actual code

- [ ] **`rust-cli-app-patterns.md`**:
  - [ ] Validate project structure recommendation against actual structure
  - [ ] Check error handling pattern (main() contradiction identified in review)
  - [ ] Verify `ExitCode` recommendation matches implementation

- [ ] **`rust-project-setup.md`**:
  - [ ] Validate Cargo.toml recommendations against actual Cargo.toml
  - [ ] Check lint configuration (warn vs deny)
  - [ ] Verify CI workflow recommendations against actual `.github/workflows/`
  - [ ] Check `deny.toml` recommendations against actual `deny.toml`
  - [ ] Verify release profile against actual profile
  - [ ] Check `resolver = "3"` for Edition 2024

- [ ] **`test-coverage-for-porting.md`**:
  - [ ] Compare coverage targets against actual coverage
  - [ ] Check `insta` snapshot testing mention
  - [ ] Validate cross-validation CI example

#### 8.6: Integrate Phase 7C Observations

Complete the pending meta-playbook Phase C work.

- [ ] Read `flowmark-port-observations-2.md` (13 observations from exact-parity spec)
- [ ] Triage each observation using the improvement-triage-template categories
  (FIX/ADD/CLARIFY/GENERALIZE/VALIDATE)
- [ ] Draft specific text changes for each non-VALIDATE observation
- [ ] Prioritize by impact and severity
- [ ] Create implementation list organized by target file

#### 8.7: Update Playbook README and Cross-References

- [ ] Update README.md case study metrics table with current port data
- [ ] Update the "Case studies completed" table
- [ ] Verify all cross-references between docs are correct
- [ ] Check all internal links resolve
- [ ] Update "validated by N case studies" if applicable

#### 8.8: Consolidated Work Items

All findings from Phases 8.1-8.7 and the retrospective mapping (F1-F44) have been
consolidated into 12 discrete work items. Each is bead-sized and has a clear scope.

**Priority P1 — Prerequisites and Critical Fixes:**

**WI-1: Initialize playbook submodule and audit prior fixes**
- Bead: fmr-xxmm | Scope: Phase 8.1 | Repo: both
- `git submodule update --init repos/rust-porting-playbook`
- Check if 53+ fixes from `plan-2026-02-08` were actually applied
- Check status of comprehensive review from `plan-2026-02-12`
- Create status matrix: {fix-id, target-file, applied-or-not}
- **Prerequisite for WI-4 through WI-11**

**WI-2: Clean up flowmark-rs docs**
- Bead: fmr-cwct | Scope: Phase 8.2 | Repo: flowmark-rs
- Remove stale `docs/porting-checklist.md` (F44)
- Grep source for `XXX:` → convert to `HACK:`/`FIXME:` per convention (F26)
- Verify `HACK:` comments exist for all COMRAK-WORKAROUND labels
- Cross-reference `code-review-2026-02-17.md` findings against playbook coverage

**WI-3: Housekeeping — stale YAMLs and mapping notes**
- Bead: fmr-hasj | Scope: Phase 6 remaining | Repo: flowmark-rs
- Regenerate `rust-tests.yaml` via `flowmark-dev discover-rust` (F41)
- Update `python/tests/test_smoke.py` Rust test count assertion (F42)
- Clean mapping notes for fmr-2tll, fmr-4l1x, fmr-5ojk — remove "Ignored" (F43)

**WI-4: Playbook critical factual fixes**
- Bead: fmr-mzel | Scope: Phase 8.4/8.5 critical items | Repo: playbook
- Depends on: WI-1
- Findings: F13, F14, F19-F27, F35 (11 factual fixes)
- Fix effort allocation 105% → 100% in `python-to-rust-playbook.md`
- Fix Pitfall #6 identical examples in `porting-rules.md`
- Fix `assert` → `debug_assert!` mapping in `mapping-reference.md`
- Fix stale crate refs (`serde_yaml`, `once_cell`) in `cli-best-practices.md`
- Fix stale GitHub Actions versions in `project-setup.md`
- Fix non-compiling `build.rs` examples in `porting-guide.md`
- Fix `color-eyre` status in `cli-best-practices.md`
- Fix `XXX:` → `HACK:`/`FIXME:` in `porting-rules.md`
- Fix version constraint mappings in `mapping-reference.md`
- Fix lint config warn vs deny in `project-setup.md`
- Fix Edition 2024 `resolver = "3"` in `project-setup.md`

**Priority P2 — Important New Content:**

**WI-5: Graduate lessons L1-L11 into playbook**
- Bead: fmr-hr43 | Scope: Phase 8.5/8.6 | Repo: playbook
- Depends on: WI-1, WI-4
- Findings: F1-F12 (12 lessons + anti-patterns)
- Add L1-L11 techniques to `porting-principles-and-antipatterns.md`
- Propose Principle 9: dynamic parity assertions (from L5/L9/L11)
- Add PR #17 false parity as anti-pattern case study
- Add red/green discipline (L10) to porting principles
- Add error parity guidance (L8) to `python-to-rust-cli-porting.md`
- Add corpus validation (L5/L9) to porting principles

**WI-6: Create cross-language test mapping reference doc**
- Bead: fmr-xei7 | Scope: Phase 8.4 new content | Repo: playbook
- Depends on: WI-1
- Findings: F28, F29, F30
- Create new `reference/cross-language-test-mapping.md`
- Document YAML-based mapping system (python-tests.yaml, rust-tests.yaml,
  test-mapping.yaml)
- Document `flowmark-dev` CLI tool pattern for automated discovery
- Document CI enforcement pattern (`check-mapping` as hard gate)
- Link from playbook Phase 5, `test-coverage-playbook.md`, and `README.md`

**WI-12: PR tryscript tests to Python repo**
- Bead: fmr-03xy (existing) | Scope: Phase 7 | Repo: upstream (flowmark)
- PR tryscript tests and end-to-end CLI tests to Python flowmark repo
- Bump Python source pin once merged
- Update `flowmark-dev discover-python` to pick up new tests

**Priority P3 — Broader Updates:**

**WI-7: Update case study docs with v2 port data**
- Bead: fmr-5hjg | Scope: Phase 8.3 | Repo: playbook
- Depends on: WI-1
- Findings: F4, F6, F34, F36, F37
- Update all 7 `case-studies/flowmark/*.md` docs with current metrics
- Add "v2 port" sections with 442 tests, 292 mapped, ~1.0x LOC ratio
- Reconcile workaround counts across all docs
- Add PUA placeholder pattern, comrak loose/tight analysis, smart quote analysis

**WI-8: Update playbook reference docs**
- Bead: fmr-kmfo | Scope: Phase 8.4 non-critical items | Repo: playbook
- Depends on: WI-1, WI-4
- Findings: F15, F16, F17, F32, F33
- Update `python-to-rust-playbook.md` with data from both ports
- Update `rust-code-review-checklist.md` with dead-dep detection (F15), dead error
  variants (F16), code duplication (F17), `pub(crate)` audit (F33)
- Update `python-to-rust-test-coverage-playbook.md` with test mapping approach
- Add exact parity spec as project management pattern (F32)
- Walk through checklist templates against this port

**WI-9: Update playbook guidelines**
- Bead: fmr-ohhi | Scope: Phase 8.5 non-critical items | Repo: playbook
- Depends on: WI-1, WI-4
- Findings: F8, F18, F31
- Update `python-to-rust-cli-porting.md` with error parity guidance
- Update `test-coverage-for-porting.md` with golden test patterns
- Add stale Python-reference comment cleanup guidance to `porting-rules.md`
- Validate remaining guideline docs per Phase 8.5 checklist

**WI-10: Integrate Phase 7C observations**
- Bead: fmr-hugi | Scope: Phase 8.6 | Repo: playbook
- Depends on: WI-1
- Finding: F38
- Read and triage 13 observations from `flowmark-port-observations-2.md`
- Categorize each as FIX/ADD/CLARIFY/GENERALIZE/VALIDATE
- Draft specific text changes for non-VALIDATE items
- Apply to target docs

**WI-11: Update playbook README and cross-references**
- Bead: fmr-af9y | Scope: Phase 8.7 | Repo: playbook
- Depends on: WI-7
- Findings: F39, F40
- Update README.md case study metrics table with current port data
- Update "Case studies completed" table
- Verify all cross-references and internal links
- Update "validated by N case studies" text

#### Work Item Dependencies

```
WI-1 ──┬──→ WI-4 ──┬──→ WI-5
       │           ├──→ WI-8
       │           └──→ WI-9
       ├──→ WI-6
       ├──→ WI-7 ──→ WI-11
       └──→ WI-10

WI-2 (independent — this repo)
WI-3 (independent — this repo)
WI-12 (independent — upstream)
```

#### Summary by Repo

| Repo | Work Items | Total Findings |
| --- | --- | --- |
| flowmark-rs (this repo) | WI-2, WI-3 | F26, F41-F44 |
| Both repos | WI-1 | Audit |
| Playbook repo | WI-4 through WI-11 | F1-F40 |
| Upstream (Python flowmark) | WI-12 | Phase 7 |

### Phase 9: Final Acceptance

**Bead:** fmr-h01s | **Depends on:** Phase 6

- [x] **Every** Python CLI flag has a Rust equivalent with identical behavior — verified
  by audit (all 22 flags)
- [x] `flowmark --auto .` works identically in both Python and Rust — validated by
  `auto-mode.tryscript.md`
- [x] `flowmark --list-files .` produces identical sorted file lists — validated by
  `file-discovery.tryscript.md`
- [x] Config loading from `.flowmark.toml` and `pyproject.toml [tool.flowmark]` works —
  20 tests + `config-interaction.tryscript.md`
- [x] `.flowmarkignore` patterns are respected — 35 file resolver tests
- [x] Gitignore integration works (and `--no-respect-gitignore` disables it) — tests +
  tryscript
- [x] Skill system works: `--skill`, `--install-skill`, `--agent-base`, `--docs` — 11
  tests + `verbose-docs.tryscript.md`
- [x] All previously-excluded tests are ported and passing — 292/292 mapped
- [x] Tryscript golden tests pass in CI for Rust — 11 tryscript files
- [ ] Tryscript golden tests pass for Python — tests not in Python repo yet (Phase 7)
- [x] `check-mapping` passes: 292 mapped, 0 excluded, 0 missing, 0 partial
- [ ] Every mapping entry manually reviewed for accuracy — not done
- [ ] Tryscript tests contributed upstream to the Python repo — not done (Phase 7)
- [x] All existing tests continue to pass (no regressions) — 437 pass; 5 fail only when
  Python binary unavailable (pass in CI)

## Testing Strategy

- **Part A**: Each feature area has tests ported from the corresponding Python test file.
  Total: 292 mapped Python tests, 442 Rust tests. CI enforces via `check-mapping`
  (292 mapped, 0 excluded). Tryscript golden tests provide end-to-end CLI validation.
- **Part B**: Each phase produces a deliverable document or set of changes. Validation by
  grep/search of playbook files, verification against current `cargo test`,
  `cargo clippy`, CI status.

## Decisions Made

1. **`porting-checklist.md`: Remove.** It's a stale duplicate of the playbook's
   `python-to-rust-playbook.md`. No backward compatibility needed for docs.

2. **Case study versioning: Add "v2 port" sections.** Keep old port data and add v2
   sections to each case study doc.

3. **Test mapping system: New reference doc.** Create a new reference doc in the
   playbook (e.g., `reference/cross-language-test-mapping.md`) and link from the
   playbook's Phase 5, the test coverage playbook, and the README.

4. **`porting-plan.md`: Updated and moved to `specs/done/`.** Updated with accurate
   module layout, checked acceptance criteria, current metrics, and "Status: Complete"
   header.

5. **Migration plan: Renamed v1, created v2.** Renamed the existing 3,339-line migration
   plan to `flowmark-port-migration-plan-v1.md` with a note pointing to v2. Created new
   `flowmark-port-migration-plan-v2.md` documenting the current port's architecture.

## Open Questions

None remaining. All decisions resolved.

## Future Work (tracked separately)

| Item | Priority | Bead | Notes |
| --- | --- | --- | --- |
| **Performance optimization + benchmarks** | P1 | fmr-aq8o | File resolver `--list-files` 4x slower than Python due to excessive syscalls; fix with `ignore::WalkBuilder`, then benchmark |
| **Property-based testing** (proptest) | P3 | -- | Idempotency, width invariants, round-trip properties |
| **justfile** for common dev workflows | P3 | -- | `just test`, `just lint`, `just check-mapping` |
| **Release workflow** (GitHub Actions) | P3 | -- | Automated binary builds + crates.io publish (see build-publishing spec) |
| **README and CHANGELOG** | P3 | -- | Public-facing documentation (see build-publishing spec) |
| **`clap_complete` shell completions** | P4 | -- | Generate bash/zsh/fish completions |
| **Color flag** (`--color auto/always/never`) | P4 | -- | Standard CLI convention |

## References

- Exact parity spec: `docs/project/specs/active/plan-2026-02-17-exact-parity.md`
- Test mapping spec:
  `docs/project/specs/active/plan-2026-02-17-test-mapping-meta-test.md`
- Code review: `docs/project/specs/active/code-review-2026-02-17.md`
- Porting plan: `docs/project/specs/done/porting-plan.md`
- Porting log: `docs/porting-log-review.md`
- Port sync playbook: `docs/port-sync-playbook.md`
- Playbook repo: `repos/rust-porting-playbook/`
- Playbook README: `repos/rust-porting-playbook/README.md`
- Meta-playbook:
  `repos/rust-porting-playbook/reference/meta-improving-this-playbook.md`
- Playbook review fixes spec:
  `repos/rust-porting-playbook/docs/project/specs/active/plan-2026-02-08-playbook-review-fixes.md`
- Comprehensive review spec:
  `repos/rust-porting-playbook/docs/project/specs/active/plan-2026-02-12-comprehensive-playbook-review.md`
- Phase 7C observations:
  `repos/rust-porting-playbook/case-studies/flowmark/flowmark-port-observations-2.md`
- Comprehensive tryscript spec:
  `docs/project/specs/active/plan-2026-02-17-comprehensive-tryscript-golden-tests.md`
- **Python source**: Cloned on demand by `flowmark-dev discover-python` from
  `https://github.com/jlevy/flowmark` at tag `v0.6.4`
- **Golden testing methodology**: `tbd guidelines golden-testing-guidelines`
- **Tryscript documentation**: `npx tryscript@latest readme` (overview),
  `npx tryscript@latest docs` (syntax reference)
- **Tryscript repo**: https://github.com/jlevy/tryscript
