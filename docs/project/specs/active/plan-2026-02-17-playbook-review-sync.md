# Feature: Review, Sync, and Improve Porting Playbook

**Date:** 2026-02-17 (last updated 2026-02-17)

**Author:** Joshua Levy

**Status:** Draft

## Overview

Systematically reconcile all documentation in the flowmark-rs project against the
rust-porting-playbook, ensuring:

1. **flowmark-rs docs** accurately reflect the current state of the port (not the old
   flowmark-rs-1 implementation)
2. **Playbook case study** is updated with correct metrics from the completed second
   port
3. **Playbook reference docs and guidelines** incorporate lessons learned from this port
4. **Errors, omissions, and stale data** in both repos are identified and fixed

This is a bidirectional sync: flowmark-rs docs inform playbook improvements, and
playbook best practices inform remaining flowmark-rs cleanup.

## Goals

- Complete inventory of every document in both repos with current status assessment
- Identify all factual errors, stale metrics, and contradictions across both repos
- Update playbook case study to reflect the current flowmark-rs (250 tests, not old 141)
- Integrate the 13 pending Phase 7C observations into the playbook
- Apply playbook review findings (from the 3 playbook spec plans) relevant to this port
- Ensure flowmark-rs docs follow playbook-recommended organizational patterns
- Track all remaining code-level fixes from code-review-2026-02-17.md against playbook
  best practices

## Non-Goals

- Implementing the code fixes themselves (tracked separately in beads)
- Conducting a new case study port (this reconciles the existing one)
- Restructuring the playbook’s 8-phase methodology
- Adding entirely new playbook documents (only updating existing ones)

## Background

### Two Ports, One Case Study

The rust-porting-playbook was built primarily from the **first** flowmark-rs port
(flowmark-rs-1, in `attic/flowmark-rs-1/`). That port achieved:
- 141 tests (93 unit + 42 integration + 6 doctests), 2 ignored
- ~95% cross-validation match
- 14 library workarounds, 3 accepted differences
- Rust/Python LOC ratio: ~1.7x app code

The **current** flowmark-rs (this repo) is a fresh reimplementation that achieved:
- 250 tests (27 unit + 223 integration), 0 ignored
- 202 mapped Python tests + 79 excluded (infrastructure)
- 100% of ported tests passing, 0 partial mappings
- Rust/Python code lines ratio: 1.00x (5,284 vs 5,279)

The playbook case study docs still describe the **old** port.
This spec addresses that gap.

### Document Inventory

#### flowmark-rs docs (this repo: `docs/`)

| Document | Lines | Current Status | Notes |
| --- | --- | --- | --- |
| `porting-plan.md` | 137 | Stale | Architecture/plan — work is complete |
| `porting-checklist.md` | 643 | Duplicate | Copy of playbook’s `python-to-rust-playbook.md` |
| `code-review-2026-02-17.md` | 471 | Current | Fresh review with P0-P3 issues |
| `specs/active/plan-2026-02-17-exact-parity.md` | 751 | Complete | Comprehensive parity spec |
| `specs/active/plan-2026-02-17-test-mapping-meta-test.md` | 482 | Implemented | Test mapping infrastructure |

#### rust-porting-playbook (attic: `attic/rust-porting-playbook/`)

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

**Case studies (7 files, ~5,000 lines):**

| Document | Lines | Status |
| --- | --- | --- |
| `case-studies/flowmark/flowmark-port-analysis.md` | ~326 | STALE — describes old port |
| `case-studies/flowmark/flowmark-port-library-choices.md` | ~257 | Partially current |
| `case-studies/flowmark/flowmark-port-decision-log.md` | ~523 | STALE — old decisions |
| `case-studies/flowmark/flowmark-port-migration-plan.md` | ~3,339 | STALE — old migration |
| `case-studies/flowmark/flowmark-port-cross-validation.md` | ~189 | STALE — old validation |
| `case-studies/flowmark/flowmark-port-comrak-bug.md` | ~211 | Partially current |
| `case-studies/flowmark/flowmark-port-wrapping-solution.md` | ~155 | STALE — old approach |

**Playbook specs (3 files):**

| Document | Status |
| --- | --- |
| `docs/project/specs/active/plan-2026-02-08-playbook-review-fixes.md` | 53+ fixes identified, beads closed |
| `docs/project/specs/active/plan-2026-02-09-meta-playbook-improvement.md` | Meta-process designed |
| `docs/project/specs/active/plan-2026-02-12-comprehensive-playbook-review.md` | 4-phase review planned |

### Key Discrepancies Found

#### D1: Stale Case Study Metrics (CRITICAL)

The playbook README and case study docs describe the **old** flowmark-rs-1 port:

| Metric | Playbook Says | Actual (Current Port) |
| --- | --- | --- |
| Test count | 141 (93+42+6), 2 ignored | 250 (27+223), 0 ignored |
| Python test mapping | Not tracked | 202 mapped, 79 excluded, 0 missing |
| Rust/Python LOC ratio (app) | ~1.7x | 1.03x (2,610 vs 2,531) |
| Rust/Python LOC ratio (total) | ~1.8x | 0.69x (6,909 vs 10,052) |
| Code lines ratio | Not stated | 1.00x (5,284 vs 5,279) |
| Cross-validation | ~95% match | 100% of ported tests |
| Workarounds | “14 fixable, 3 unfixable” | Different set (new implementation) |

The LOC ratio difference is dramatic: the playbook says Rust is 1.7x larger, but the
actual code-lines ratio is 1:1. The total-lines ratio actually favors Rust (0.69x)
because Python has 3x more docstrings/comments.

#### D2: Workaround Count Inconsistencies

The playbook review spec (plan-2026-02-08) identified that workaround counts differ
across documents: 13, 14, 15, 17 appear in different places.
The current port has a different set of workarounds than the old port.

#### D3: `porting-checklist.md` is a Stale Duplicate

`docs/porting-checklist.md` in this repo is a copy of the playbook’s
`reference/python-to-rust-playbook.md`. It may be out of sync and creates maintenance
burden.

#### D4: `porting-plan.md` Has Incomplete Acceptance Criteria

The porting plan has unchecked acceptance criteria (`- [ ]`) even though the work is
complete. The plan itself is informative but doesn’t reflect current status.

#### D5: Phase 7C Observations Not Integrated

The exact-parity spec records: “Phase C (integrating changes into playbook) pending
human review.” 13 observations were recorded in
`attic/rust-porting-playbook/case-studies/flowmark/flowmark-port-observations-2.md` but
not yet integrated.

#### D6: Code Review Findings vs Playbook Best Practices

The code-review-2026-02-17.md identified issues that should have been caught by playbook
best practices. These represent either playbook gaps or implementation oversights:

| Code Review Issue | Playbook Coverage |
| --- | --- |
| P0: Clippy failures (9 errors) | Playbook says clippy pedantic — but as warn not deny |
| P0: Formatting violations | Playbook says `cargo fmt` — not enforced |
| P0.5: Lint config gaps (warn vs deny) | Playbook says pedantic warn, code review says deny |
| P1: Dead dependencies | Playbook has no guidance on dependency pruning |
| P1: Dead error variants | Playbook has no dead-code-detection guidance |
| P2: Code duplication (fence tracking) | Code review checklist has nothing on duplication |
| P2: Unnecessary allocations | Code review checklist covers “hot path allocation” |
| P2: Boolean parameter overload | Playbook recommends options structs |
| P3: Stale comments ("Same as Black") | Playbook has no Python-reference-cleanup guidance |

#### D7: Comment Convention Mismatch

The playbook review spec identified that `XXX:` is non-standard and recommended `HACK:`
for library workarounds and `FIXME:` for items needing future resolution.
The playbook itself was updated but the case study docs and potentially the flowmark-rs
code may still use `XXX:`.

#### D8: Non-Compiling Code Examples in Playbook

The playbook review spec (plan-2026-02-08) identified multiple non-compiling code
examples in the porting guide, including incorrect `build.rs` patterns and `env!` macro
usage. Status of these fixes needs verification.

#### D9: Critical Playbook Fixes Not Yet Applied

The plan-2026-02-08 spec identified 53+ fixes across 5 severity phases.
While the beads were closed, it’s unclear if all changes were actually applied to the
playbook documents. This needs verification.

#### D10: Playbook References Archived/Stale Crates

Multiple references to archived or stale crates:
- `serde_yaml` → should be `serde_yaml_ng` (archived March 2024)
- `once_cell` → should be `std::sync::LazyLock` (stable since Rust 1.80)
- `actions/checkout@v5` → should be `@v6`
- `actions/create-release@v1` → archived, use `softprops/action-gh-release@v2`
- `color-eyre` → maintenance-only status should be noted
- comrak version references may be stale (0.29 vs current 0.50+)

* * *

## Design

### Approach

The reconciliation is organized into phases, each producing a concrete deliverable.
Phases are ordered to build on each other: inventory first, then analysis, then changes.

### Reconciliation Strategy

For each document pair (flowmark-rs doc ↔ playbook doc), we will:

1. **Verify factual accuracy** against the current codebase state
2. **Identify stale content** that describes the old port
3. **Flag contradictions** between documents
4. **Propose specific changes** with file, section, and text
5. **Categorize** as FIX / ADD / CLARIFY / GENERALIZE / VALIDATE (per playbook’s own
   meta-process)

* * *

## Implementation Plan

### Phase 1: Verify Playbook Fix Status

Before making new changes, verify what has already been done.

- [ ] Check if the 53+ fixes from `plan-2026-02-08-playbook-review-fixes.md` were
  actually applied to the playbook documents (beads closed ≠ changes committed)
- [ ] Check the status of the comprehensive review from
  `plan-2026-02-12-comprehensive-playbook-review.md`
- [ ] Identify which playbook spec changes are already implemented vs still pending
- [ ] Check `XXX:` → `HACK:`/`FIXME:` convention change status in playbook docs
- [ ] Check for non-compiling code examples identified in the review
- [ ] Grep playbook for archived crate references (`serde_yaml`, `once_cell`,
  `actions/checkout@v5`, `actions/create-release@v1`)
- [ ] Create a status matrix: {fix-id, target-file, applied-or-not}

### Phase 2: Review flowmark-rs Docs Against Current State

Ensure this project’s own docs are accurate.

- [ ] **`porting-plan.md`**: Check all acceptance criteria boxes that are now met.
  Add “Status: Complete” header.
  Verify module mapping matches actual `src/` layout.
- [ ] **`porting-checklist.md`**: Determine if this should be removed (it’s a duplicate
  of the playbook). If kept, verify it matches the current playbook version.
  Decision: remove or convert to a project-specific checklist with checked items.
- [ ] **`code-review-2026-02-17.md`**: Cross-reference all findings against playbook
  best practices. For each finding, note whether the playbook covers it, and if not, flag
  as a playbook gap (ADD).
- [ ] **Exact parity spec**: Verify “Complete” status is accurate.
  Check all appendices for correctness.
- [ ] **Test mapping spec**: Verify “Implemented” status.
  Check that workflow descriptions match actual `flowmark-dev` CLI behavior.
- [ ] Check for any `XXX:` comments in flowmark-rs source code that should be
  `HACK:`/`FIXME:` per updated playbook convention
- [ ] Verify `HACK:` and `FIXME:` comments exist where playbook says they should (all
  library workarounds documented)

### Phase 3: Review Playbook Case Study Against Current Port

Update all 7 case study documents to reflect the current port.

- [ ] **`flowmark-port-analysis.md`**: Update metrics (LOC, test counts, ratios).
  Update “what’s automatable” assessment based on this port’s experience.
  Note that the new port used a cross-language test mapping system not in the original.

- [ ] **`flowmark-port-library-choices.md`**: Verify library choices match current
  `Cargo.toml`. Update comrak version references.
  Note any new library decisions.

- [ ] **`flowmark-port-decision-log.md`**: Update or add entries for decisions made in
  the new port (e.g., test mapping infrastructure, CI hardening, lint configuration).
  Fix the D7 wrapping solution contradiction identified in plan-2026-02-08.

- [ ] **`flowmark-port-migration-plan.md`**: This is the longest doc (3,339 lines).
  Decide: update in place, add a “v2 port” appendix, or create a separate doc for the
  new port’s migration narrative.

- [ ] **`flowmark-port-cross-validation.md`**: Update with current cross-validation
  results (100% mapped tests passing, 0 ignored).
  Update escape handling table.

- [ ] **`flowmark-port-comrak-bug.md`**: Verify still relevant.
  Check if any comrak bugs were fixed upstream since the original doc.

- [ ] **`flowmark-port-wrapping-solution.md`**: Update with current wrapping approach.
  The doc describes two approaches — verify which one the current port uses and update.

- [ ] **Reconcile workaround counts** across all 7 case study docs.
  Establish a single authoritative count for the current port by grepping `HACK:`
  comments in source.

### Phase 4: Review Playbook Reference Docs

Review each reference doc against this port’s experience.

- [ ] **`python-to-rust-playbook.md`** (core playbook):
  - [ ] Verify effort allocation table sums to 100% (identified as 105% in review)
  - [ ] Update “Key insight” with data from both ports
  - [ ] Check Phase 4.3 (submodule setup) — the current port uses `attic/` clones
    instead of submodules; document both approaches
  - [ ] Check Phase 4.6 (version tracking) — verify recommendation matches practice
  - [ ] Validate Phase 7 (finalize) CLI parity section against actual CLI state
  - [ ] Check Phase 8 (sync) — not yet exercised; note this

- [ ] **`python-to-rust-mapping-reference.md`**:
  - [ ] Verify type mappings against actual code translations
  - [ ] Check `dict` → `HashMap` insertion-order warning is present
  - [ ] Check `assert` → `debug_assert!` dangerous mapping is fixed
  - [ ] Verify version constraint mappings (identified as swapped)
  - [ ] Check for `Cow<'_, str>` in type mappings
  - [ ] Check for `re.search()` and `re.fullmatch()` regex mappings
  - [ ] Check `str.find()` byte-offset warning
  - [ ] Verify dunder methods → traits table exists

- [ ] **`python-to-rust-porting-guide.md`**:
  - [ ] Verify `build.rs` code examples compile
  - [ ] Check version tracking recommendations against actual practice
  - [ ] Validate cross-validation script template
  - [ ] Check 9 critical pitfalls against this port’s experience

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

### Phase 5: Review Playbook Guidelines

Review each guideline against this port’s experience.

- [ ] **`python-to-rust-porting-rules.md`**:
  - [ ] Check Pitfall #6 (identical wrong/correct examples — identified in review)
  - [ ] Verify `assert` → `debug_assert!` fix applied
  - [ ] Check `frozenset` note accuracy
  - [ ] Check acceptance criteria include clippy
  - [ ] Verify comment convention uses `HACK:`/`FIXME:` (not `XXX:`)

- [ ] **`python-to-rust-cli-porting.md`**:
  - [ ] Validate argparse → clap mappings against actual CLI
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

### Phase 6: Integrate Phase 7C Observations

Complete the pending meta-playbook Phase C work.

- [ ] Read `flowmark-port-observations-2.md` (13 observations from exact-parity spec)
- [ ] Triage each observation using the improvement-triage-template categories
  (FIX/ADD/CLARIFY/GENERALIZE/VALIDATE)
- [ ] Draft specific text changes for each non-VALIDATE observation
- [ ] Prioritize by impact and severity
- [ ] Create implementation list organized by target file

### Phase 7: Update Playbook README and Cross-References

- [ ] Update README.md case study metrics table with current port data
- [ ] Update the “Case studies completed” table
- [ ] Verify all cross-references between docs are correct
- [ ] Check all internal links resolve
- [ ] Update “validated by N case studies” if applicable

### Phase 8: Consolidate Findings into Action Items

- [ ] Compile all FIX/ADD/CLARIFY/GENERALIZE items from Phases 1-7
- [ ] Organize by target file for efficient editing
- [ ] Prioritize: factual errors first, then missing content, then clarity improvements
- [ ] Create beads for each actionable change
- [ ] Determine which changes go to the playbook repo vs this repo

* * *

## Detailed Reconciliation Map

### flowmark-rs Code Review → Playbook Gap Analysis

For each code review finding, identify whether the playbook addresses it:

| # | Code Review Finding | Priority | Playbook Addresses? | Playbook Gap? |
| --- | --- | --- | --- | --- |
| 1 | Clippy `inefficient_to_string` (9 errors) | P0 | Partial — says clippy pedantic but as warn | CLARIFY: warn→deny |
| 2 | `cargo fmt` violations | P0 | Yes — says `cargo fmt --check` | VALIDATE |
| 3 | Dead dependencies (serde, toml, unicode-segmentation) | P1 | No guidance on pruning unused deps | ADD |
| 4 | Dead error variants (Config, Other) | P1 | Code review checklist: “Dead code removed” | VALIDATE |
| 5 | Code duplication (fence tracking 3x) | P2 | Not in code review checklist | ADD |
| 6 | Unnecessary allocation in ellipsis check | P2 | “Hot paths allocation-aware” in checklist | VALIDATE |
| 7 | Vec\<char\> allocation | P2 | Same as above | VALIDATE |
| 8 | Boolean parameter overload (8-11 params) | P2 | Checklist: “Enums over booleans” | VALIDATE |
| 9 | Unused `_name` field in AtomicPattern | P2 | “Dead code removed” in checklist | VALIDATE |
| 10 | Unnecessary `info.clone()` | P2 | “No gratuitous clones” in checklist | VALIDATE |
| 11 | Repeated `.expect()` calls | P2 | Not specifically covered | Minor |
| 12 | Stale “Same as Black” comment | P3 | No Python-reference cleanup guidance | ADD |
| 13 | No doc-tests | P3 | Checklist: “Doc-tests compile and pass” | VALIDATE |
| 14 | Lint config: warn vs deny disconnect | P0.5 | Playbook says warn; code review says deny | CLARIFY |

### Playbook Metric Corrections Needed

| Metric | Playbook Currently Says | Should Say (Current Port) | Source |
| --- | --- | --- | --- |
| Test count | 141 (93+42+6) | 250 (27+223) | exact-parity spec |
| Tests ignored | 2 | 0 | exact-parity spec |
| Python tests mapped | Not tracked | 202 mapped + 79 excluded | test-mapping spec |
| Rust/Python LOC ratio (app) | ~1.7x | 1.03x | Appendix C of parity spec |
| Rust/Python LOC ratio (total) | ~1.8x | 0.69x | Appendix C of parity spec |
| Code lines ratio | Not stated | 1.00x (5,284 vs 5,279) | Appendix C of parity spec |
| Python LOC (app) | ~2,000 | 2,531 (code) / 4,433 (total) | Appendix C |
| Python LOC (tests) | ~1,500 | 2,748 (code) / 5,619 (total) | Appendix C |
| Rust LOC (app) | ~3,400 | 2,610 (code) / 3,485 (total) | Appendix C |
| Rust LOC (tests) | ~2,900 | 2,674 (code) / 3,688 (total) | Appendix C |
| Cross-validation | ~95% match | 100% ported tests passing | exact-parity spec |
| Library workarounds | “14 fixable, 3 unfixable” | TBD — grep current HACK: comments | Code audit needed |
| Performance | 20-40x improvement | TBD — benchmark current port | Benchmark needed |
| Binary size | 2.5MB | TBD — measure current binary | Measurement needed |

### New Lessons for Playbook (from Current Port)

These patterns/tools were developed during this port but are not in the playbook:

| Lesson | Playbook Impact | Category |
| --- | --- | --- |
| Cross-language test mapping system (YAML-based) | Major — reusable for any port | ADD |
| `flowmark-dev` CLI for test discovery + mapping | Major — tool pattern for ports | ADD |
| Test mapping as CI hard gate | Important — enforcement pattern | ADD |
| Golden test with 4 formatting modes | Important — testing pattern | ADD |
| Exact parity spec as tracking document | Important — project management | ADD |
| `pub(crate)` visibility audit as post-port step | Medium — code quality | ADD |
| Unicode PUA placeholder pattern for escape preservation | Niche — text processing | ADD (case study) |
| Lint configuration: pedantic as deny, not warn | Important — stricter than playbook | CLARIFY |
| `warnings = "deny"` in Cargo.toml (not just CI) | Important — consistency | ADD |
| Edition 2024 `resolver = "3"` | Important — already identified | FIX |

* * *

## Testing Strategy

Each phase produces a deliverable document or set of changes.
Validation:

- **Phase 1**: Status matrix verified by grep/search of playbook files
- **Phase 2**: Each doc checked against current `cargo test`, `cargo clippy`, CI status
- **Phase 3**: Metrics verified against Appendix C of exact-parity spec
- **Phase 4-5**: Each reference/guideline item verified against actual code/config
- **Phase 6**: Observation triage completeness (all 13 triaged)
- **Phase 7**: All links in playbook README resolve correctly
- **Phase 8**: All action items have associated beads

## Open Questions

1. **Should `docs/porting-checklist.md` be removed?** It’s a duplicate of the playbook.
   Options: (a) remove and reference playbook directly, (b) convert to a
   project-specific filled-in checklist, (c) keep as-is.

2. **Should the playbook case study describe both ports?** Options: (a) update all docs
   to reflect only the current port, (b) keep old port data and add “v2 port” sections,
   (c) maintain both as separate case study entries.

3. **Where should the test mapping system be documented in the playbook?** It’s a
   significant contribution.
   Options: (a) new reference doc, (b) section in `test-coverage-for-porting.md`, (c)
   section in the playbook itself under Phase 5.

4. **Should `porting-plan.md` be updated or archived?** The work is complete.
   Options: (a) update with “Complete” status and checked boxes, (b) move to
   `docs/project/specs/completed/`, (c) leave as historical record.

5. **How to handle the migration plan doc (3,339 lines)?** It describes the old port in
   great detail. Options: (a) rewrite for current port, (b) add appendix for current
   port, (c) archive old and create new, (d) leave as historical.

## References

- Exact parity spec: `docs/project/specs/active/plan-2026-02-17-exact-parity.md`
- Test mapping spec:
  `docs/project/specs/active/plan-2026-02-17-test-mapping-meta-test.md`
- Code review: `docs/code-review-2026-02-17.md`
- Porting plan: `docs/porting-plan.md`
- Porting checklist: `docs/porting-checklist.md`
- Playbook repo: `attic/rust-porting-playbook/`
- Playbook README: `attic/rust-porting-playbook/README.md`
- Meta-playbook: `attic/rust-porting-playbook/reference/meta-improving-this-playbook.md`
- Playbook review fixes spec:
  `attic/rust-porting-playbook/docs/project/specs/active/plan-2026-02-08-playbook-review-fixes.md`
- Comprehensive review spec:
  `attic/rust-porting-playbook/docs/project/specs/active/plan-2026-02-12-comprehensive-playbook-review.md`
- Phase 7C observations:
  `attic/rust-porting-playbook/case-studies/flowmark/flowmark-port-observations-2.md`
