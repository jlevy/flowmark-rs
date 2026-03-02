# Flowmark Rust Port: Summary Retrospective

Date: 2026-03-02

## Executive Summary

The Python-to-Rust port of Flowmark is complete and operationally mature.
Across the done-spec history, the project moved from initial architecture and parity work
into CI hardening, release engineering, multi-channel distribution, and performance
optimization.

Core outcomes:

- Behavioral parity with Python flowmark v0.6.4 was achieved and defended with expanded
  test infrastructure.
- Cross-language test mapping was implemented and enforced in CI.
- CI quality gates were hardened (strict lints, warnings-as-errors, coverage,
  semver checks, multi-OS testing).
- Distribution now covers crates.io, GitHub Releases, Homebrew, and PyPI.
- Performance moved from already-fast Rust baseline to materially faster parallel and
  incremental-cache-aware workflows.
- Lessons were fed back into the rust-porting playbook and process templates.

## Sources Reviewed

This retrospective is based on all specs currently in
`docs/project/specs/done` and the bead data currently visible in
`.tbd/data-sync/issues`.

### Done Specs (full index)

1. [docs/project/specs/done/porting-plan.md](docs/project/specs/done/porting-plan.md)
2. [docs/project/specs/done/code-review-2026-02-17.md](docs/project/specs/done/code-review-2026-02-17.md)
3. [docs/project/specs/done/plan-2026-02-17-build-publishing.md](docs/project/specs/done/plan-2026-02-17-build-publishing.md)
4. [docs/project/specs/done/plan-2026-02-17-comprehensive-tryscript-golden-tests.md](docs/project/specs/done/plan-2026-02-17-comprehensive-tryscript-golden-tests.md)
5. [docs/project/specs/done/plan-2026-02-17-exact-parity.md](docs/project/specs/done/plan-2026-02-17-exact-parity.md)
6. [docs/project/specs/done/plan-2026-02-17-test-mapping-meta-test.md](docs/project/specs/done/plan-2026-02-17-test-mapping-meta-test.md)
7. [docs/project/specs/done/plan-2026-02-18-parity-discrepancies.md](docs/project/specs/done/plan-2026-02-18-parity-discrepancies.md)
8. [docs/project/specs/done/plan-2026-02-19-parity-review-and-playbook-sync.md](docs/project/specs/done/plan-2026-02-19-parity-review-and-playbook-sync.md)
9. [docs/project/specs/done/plan-2026-02-25-cli-help-cleanup.md](docs/project/specs/done/plan-2026-02-25-cli-help-cleanup.md)
10. [docs/project/specs/done/plan-2026-02-26-perf-comparison-profiling.md](docs/project/specs/done/plan-2026-02-26-perf-comparison-profiling.md)
11. [docs/project/specs/done/plan-2026-02-27-parallel-file-processing.md](docs/project/specs/done/plan-2026-02-27-parallel-file-processing.md)
12. [docs/project/specs/done/plan-2026-02-27-incremental-cache-and-performance-roadmap.md](docs/project/specs/done/plan-2026-02-27-incremental-cache-and-performance-roadmap.md)
13. [docs/project/specs/done/plan-2026-03-01-pypi-distribution.md](docs/project/specs/done/plan-2026-03-01-pypi-distribution.md)

## Bird's-Eye Process Timeline

1. Foundation and architecture: Master porting plan established module boundaries,
   dependencies, and acceptance goals.
2. Coverage and parity system: Cross-language test mapping tooling created for
   Python->Rust provenance and CI enforcement.
3. Exact parity delivery: High-priority output mismatches fixed (P and D discrepancy
   tracks), ending in byte-for-byte parity claims with broad test backing.
4. Code review hardening: External review findings converted into concrete engineering
   tasks and lint policy tightening.
5. CI and release engineering: Build and publish workflows matured from crate-only to
   full multi-channel release orchestration.
6. CLI and docs polish: Help/usage UX improved and documentation synchronization
   tightened.
7. Performance phase: Baseline profiling quantified bottlenecks; parallel processing
   delivered major throughput gains; incremental cache roadmap implemented as follow-on
   optimization track.
8. Ecosystem and playbook feedback: Porting lessons were fed back into reusable
   playbook guidance and process templates.

## What Was Done (Organized by Workstream)

### 1) Port Architecture and Core Formatter

- Established Rust module architecture (`formatter`, `wrapping`, `parser`, `transform`,
  `typography`, `file_resolver`, `skills`).
- Reimplemented behavior on top of `comrak` with custom rendering and parity-oriented
  post-processing.
- Preserved CLI/library split with feature-gated CLI and strict lint posture.

### 2) Parity and Correctness

- Implemented cross-language test mapping lifecycle and CI checks.
- Closed discrepancy tracks across plaintext, list spacing, blockquote/footnote,
  wrapping edge cases, and CLI error compatibility.
- Eliminated test masking patterns so gaps fail visibly instead of hiding behind weak
  assertions.
- Expanded tryscript/golden strategy as executable parity contract.

### 3) Engineering Quality and CI Discipline

- Adopted strict lint policy (`warnings=deny`, pedantic clippy posture).
- Enforced formatter consistency and warning-free builds in CI.
- Added/strengthened checks: docs build, coverage, semver checks, dependency audit,
  workflow script tests, multi-platform test matrix.

### 4) Packaging and Release Operations

- Crates.io trusted publishing path established.
- GitHub Release artifact packaging/checksum flow implemented.
- Homebrew tap flow established and codified.
- PyPI/maturin wheel+sdist distribution path integrated.
- Release orchestration moved to reusable workflows plus script-driven planning logic.

### 5) Performance and Scalability

- Baseline profiling measured major speedups vs Python and identified string/alloc-heavy
  hotspots.
- Parallel file processing added (rayon + threading controls + skip-unchanged behavior).
- Incremental cache architecture implemented with invalidation/fingerprint behavior and
  supporting tests/benchmark plan.

### 6) Documentation and Playbook Sync

- Port status and planning docs evolved from active execution to completed record.
- Playbook sync spec captured lessons learned and transformed them into reusable guidance.
- Publishing process consolidated into a canonical operational runbook.

## What Worked Well

- Spec-first execution with explicit goals/non-goals kept scope controlled.
- Bead decomposition turned large parity/release work into tractable units.
- TDD and parity-first gates prevented silent regressions.
- Tooling quality (mapping checker, tryscript, CI workflows, script tests) made
  correctness auditable.
- Performance work was evidence-driven (benchmarks + profiling before optimization).

## Main Challenges and How They Were Addressed

1. Parser/rendering semantics mismatch (Marko vs Comrak): Addressed by targeted
   rendering logic, explicit discrepancy tracking, and parity tests for each discovered
   gap.
2. Hidden parity drift risk: Addressed by forbidding masking patterns and requiring
   failing tests per known gap.
3. Multi-channel release complexity and partial-failure risk: Addressed by script-driven
   workflow planning, idempotent channel behavior, orchestrated release flows, and
   explicit manual Homebrew checkpoint.
4. Documentation/status drift over time: Addressed by moving specs to done and
   consolidating process docs; remaining stale internal status lines are called out
   below.

## Status Reconciliation Notes

The done-folder move is correct for project state, but several historical inline status
fields in individual specs are stale relative to current completion:

- `plan-2026-02-17-comprehensive-tryscript-golden-tests.md` still says `Status: Draft`.
- `plan-2026-02-27-incremental-cache-and-performance-roadmap.md` still says
  `Status: In Review` while its own bead checklist marks implementation complete.
- `plan-2026-03-01-pypi-distribution.md` still reflects phased partial status from
  2026-03-01 snapshot.
- `plan-2026-02-25-cli-help-cleanup.md` notes a Phase 4 TODO.

These are historical artifacts, not blockers, but worth normalizing in a follow-up docs
sweep.

## Bead Review

### A) Beads Referenced in Done Specs (`fmr-*`)

- Unique `fmr-*` bead IDs referenced across done specs: **106**
- Total mentions across done specs: **153**

### By major track

- Build/publishing track (`plan-2026-02-17-build-publishing.md`): 26 unique beads
- Exact parity track (`plan-2026-02-17-exact-parity.md`): 58 unique beads
- Parity+playbook sync track (`plan-2026-02-19-parity-review-and-playbook-sync.md`): 18
  unique beads
- Incremental cache/perf roadmap (`plan-2026-02-27-incremental-cache-and-performance-roadmap.md`): 8 unique beads

### Spec bead inventories (complete)

Build/publishing:

`fmr-19zr fmr-4v5g fmr-67o0 fmr-6evz fmr-7ayu fmr-7cf1 fmr-8un1 fmr-8yos fmr-9dh1 fmr-9eda fmr-aarf fmr-aq8o fmr-b035 fmr-db47 fmr-dqqo fmr-eldq fmr-ghvq fmr-hj6z fmr-mk46 fmr-nc8i fmr-rg6a fmr-rj25 fmr-swma fmr-tm0t fmr-xnxy fmr-zvbe`

Exact parity:

`fmr-0a97 fmr-0u55 fmr-2tll fmr-3i50 fmr-4l1x fmr-5255 fmr-5gkb fmr-5ojk fmr-7o1d fmr-81j7 fmr-86se fmr-8ixa fmr-8pya fmr-9kth fmr-9s7o fmr-a4on fmr-afg0 fmr-afof fmr-albo fmr-avjf fmr-bzra fmr-desq fmr-dpjh fmr-draa fmr-e38z fmr-el2i fmr-fzth fmr-gocw fmr-gpi6 fmr-gydk fmr-he1d fmr-jcsj fmr-kd36 fmr-kqxb fmr-m7o8 fmr-myhs fmr-n1w9 fmr-n69j fmr-n6ve fmr-nbp4 fmr-nswa fmr-ol08 fmr-oodm fmr-ow2t fmr-p2pr fmr-q9og fmr-r9k6 fmr-vpg4 fmr-wbsk fmr-wcjh fmr-wjjm fmr-wonk fmr-xado fmr-xcr9 fmr-xkh3 fmr-y3zv fmr-yhmk fmr-zygd`

Parity+playbook sync:

`fmr-03xy fmr-2tll fmr-4l1x fmr-5hjg fmr-5ojk fmr-7mmt fmr-af9y fmr-aq8o fmr-cwct fmr-h01s fmr-hasj fmr-hr43 fmr-hugi fmr-kmfo fmr-mzel fmr-ohhi fmr-xei7 fmr-xxmm`

Incremental cache roadmap:

`fmr-2z00 fmr-8tpy fmr-leqz fmr-m4z9 fmr-qb08 fmr-unp8 fmr-ynyg fmr-ysne`

### Notable deferred or follow-up beads in historical records

- `fmr-rj25` (`cargo-nextest`) marked deferred.
- `fmr-03xy` (upstream Python tryscript PR) recorded as deferred upstream work.
- Performance follow-up family (`fmr-aq8o` and cache roadmap beads) was explicitly
  separated into dedicated performance specs.

### B) Local `.tbd` Data-Sync Issue Records (`is-*`)

Snapshot in `.tbd/data-sync/issues` currently shows **17 open issues** (all from the
2026-02-17 code-review breakdown):

| ID | Kind | Priority | Status | Title |
| --- | --- | --- | --- | --- |
| `is-01khq2wmecvc71nghm99ss2rym` | bug | P0 | open | Fix 9 clippy inefficient_to_string errors in filling.rs and tag_handling.rs |
| `is-01khq2wnb20mpmeb80crq5swcq` | task | P0 | open | Run cargo fmt to fix all formatting violations |
| `is-01khq2wp93fmjfkqq1pr413dmm` | task | P0 | open | Tighten lints: warnings=deny, clippy pedantic=deny in Cargo.toml |
| `is-01khq2wg8qfp9evjh1fcdf5zbp` | epic | P1 | open | Code review: address all findings from 2026-02-17 review |
| `is-01khq2wq2gvvm5rb5vrvztzn6f` | task | P1 | open | Add RUSTFLAGS=-D warnings to CI test jobs |
| `is-01khq2wthm8vyqb629d9tk0aj3` | task | P1 | open | Remove dead dependencies: toml, serde, unicode-segmentation from Cargo.toml |
| `is-01khq2wwajnfm7cj7eejha25mt` | task | P1 | open | Remove dead error variants Error::Config and Error::Other from error.rs |
| `is-01khq2wx59v6haffre22mtvkbg` | task | P2 | open | Extract fence-tracking helper to eliminate 3x code duplication in filling.rs |
| `is-01khq2wy31vj9p1krvg3x476cx` | task | P2 | open | Replace nc.to_string() regex check with char method in ellipses.rs |
| `is-01khq2x1ht4nt06cz04c3h327h` | task | P2 | open | Replace Vec<char> collection with iterator in remove_period_escapes_preserving_code |
| `is-01khq2x36k0apfgvm7p7r9r51t` | feature | P2 | open | Introduce FormatOptions struct to replace boolean parameter lists in public API |
| `is-01khq2x4022hsc66eyd2axjzde` | task | P2 | open | Remove unused _name field from AtomicPattern struct |
| `is-01khq2x5hr80fc5hm1p8nfz05h` | task | P2 | open | Remove unnecessary info.clone() in code block rendering in filling.rs |
| `is-01khq2x6havw6sabnmfxsbmczn` | task | P2 | open | Extract repeated lines.last().expect() calls to local variable in line_wrappers.rs |
| `is-01khq2x9xkmpg8w86na13nzhp6` | task | P3 | open | Update DEFAULT_WRAP_WIDTH comment to remove Python Black reference |
| `is-01khq2xbphx47s1wf2dyz5tgtj` | task | P3 | open | Add doc-tests (examples) for reformat_text and fill_markdown |
| `is-01khq2xcjnx2p7dgczhrj2hry7` | task | P3 | open | Remove || true from mapping check in CI to enforce completeness |

Interpretation:

- These `is-*` records appear to be a stale local sync view of the code-review
  decomposition.
- Many corresponding outcomes are reflected as completed in later specs and code history.
- If desired, a cleanup pass should reconcile/close synced `is-*` records to match the
  completed port state.

## Final Retrospective Assessment

This port succeeded because it treated correctness, operability, and release engineering
as first-class engineering outcomes, not post-port cleanup.
The strongest patterns worth reusing in future Rust/Python hybrid ports are:

1. parity-first acceptance with explicit discrepancy IDs,
2. spec-driven work decomposition into bead-sized units,
3. mapping and tryscript-based cross-language test contracts,
4. strict CI gates and script-tested release orchestration,
5. benchmark/profiling-led performance work after correctness stabilization.

That combination produced a robust drop-in replacement and a reusable delivery framework
that can be upstreamed into the Rust porting playbook templates.
