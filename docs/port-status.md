# Project Status: flowmark-rs

**Last updated:** 2026-08-25

**Current Rust release:** v0.3.2

**Last declared whole-program Python baseline:** v0.7.2

**In-progress upstream contract:** commit `093c9249610965b37a458b32e37b5cc4738afe48`
(`v0.7.3-23-g093c924`)

## Summary

flowmark-rs is a production Rust port of Python Flowmark.
The current development branch has adopted the upstream language-neutral conformance
manifest and tryscript suite directly.
Current Rust `main` through v0.3.2 / Python v0.7.2 was merged at `c6449a5` without
restoring copied portable fixtures or Python-coupled Rust tests.
This is the foundation for porting Markdown preservation and math by stable behavior IDs
rather than by copying Python tests or fixtures.

Do not describe the current branch as whole-program parity with the in-progress commit.
The portable test foundation is green, but two implementation tracks and one delivery
gate remain open:

1. Audit and close the remaining released upstream delta from Python v0.7.2 through
   v0.7.3 (`fm-t81l`).
2. Implement the deferred preservation, math, code-span, and extension change IDs.
3. Publish the exact upstream commit so a clean remote Rust clone can initialize its
   Python submodule (`fm-zah1`).

The distinction is intentional.
A passing shared subset proves that subset; it does not retroactively classify every
upstream commit since the last released parity baseline.

## Pinned Sources

| Source | Recorded commit | Purpose |
| --- | --- | --- |
| Python Flowmark | `093c9249610965b37a458b32e37b5cc4738afe48` | Source, shared manifests, expected bytes, and tryscript documents |
| Rust porting playbook | `d24760a3fbd2951c730a199269aeb082abb46a42` | Current update workflow and Rust guidance |
| Homebrew tap | `6567a9ffbf7d90c0c08ec55dc43583e060c5349b` | Historical release integration |

`admin/port-coverage-mapping/shared-conformance.toml` is the machine-checked source for
the Python commit, manifest path, schema, and shared change-ID mapping.
The parent submodule gitlink must match it.

The Python target commit is currently available from the local source branch but not
from the configured GitHub remote.
Remote Rust CI is therefore gated on `fm-zah1`: push or merge that exact upstream
commit, then prove that a clean clone can initialize the submodule.

## Current Evidence Model

| Layer | Role | Current state |
| --- | --- | --- |
| Shared conformance manifest | Exact stdout, stderr, exit, filesystem, and idempotence contract | Native Rust runner implemented |
| Shared tryscript | End-to-end CLI workflows and fixture interactions | Rust executes upstream documents directly |
| Reference documents | Broad whole-document interactions in multiple modes | Consumed from pinned upstream |
| CommonMark 0.31.2 | Large syntax-surface sweep | Active cases pass or have exact ledger entries |
| Historical parity corpus | Previously discovered cross-language corner cases | All active cases pass exactly |
| Rust-focused tests | Adapter parsing, timeout, path safety, Rust-only behavior | Kept small and language-specific |
| Legacy YAML mapping | Function-level provenance at the v0.7.2 baseline | Supplementary; refreshed from Cargo’s executable inventory with no missing or broken mappings |

Ordinary Rust tests do not invoke Python.
Python runs the same portable contract in its own repository.
Pinned cross-binary runs remain a transition audit when accepting a new baseline or
investigating a discrepancy.

## Shared Change IDs

The status values below come from `admin/port-coverage-mapping/shared-conformance.toml`.

| Change ID | Python bead | Rust bead | Status |
| --- | --- | --- | --- |
| `FM-CONFORMANCE-001` | `fm-ltof` | `fm-gc8d` | Implemented |
| `FM-COMMONMARK-001` | `fm-shou` | `fm-gc8d` | Known divergences |
| `FM-PARITY-BASELINE-001` | `fm-gc8d` | `fm-gc8d` | Known divergences |
| `FM-MATH-INLINE-001` | `fm-ucy8` | `fm-fpbj` | Deferred |
| `FM-CODE-SPAN-001` | `fm-ocpw` | `fm-82vu` | Deferred |
| `FM-EXT-RAW-HTML-001` | `fm-w1tn` | `fm-w1tn` | Deferred |
| `FM-REFERENCE-IDEMPOTENCE-001` | `fm-w467` | `fm-w467` | Deferred |

Future shared behavior must receive a stable `FM-*` ID before language-specific
implementation starts.
The same ID must identify the Python case, Rust bead, mapping record, and validation
evidence.

## Divergence Policy

`tests/parity_corpus_known_divergences.toml` is a closed, exact ledger.
At this status snapshot it contains 38 active CommonMark case IDs.
The ledger is not a wildcard allowlist:

- an unlisted mismatch fails;
- a ledger entry for a missing or deferred case fails;
- a listed case that starts passing fails as stale;
- each entry names a live tracker and reason.

The tracker for the current inherited corpus differences is `fmr-rz9f`. As work is
ported, remove passing case IDs in the same commit as the fix.
Do not rewrite a reason to cover a different failure class.

## Validation Snapshot

The post-merge commit `c6449a5` passed these local gates with the Python submodule at
`093c924` and the playbook at `d24760a`:

- `cargo fmt --all -- --check`
- `cargo clippy --locked --all-targets --all-features -- -D warnings`
- `cargo test --locked --all-features`
- `cargo test --locked --no-default-features`
- Rust-repository administration checks: Ruff, BasedPyright, 15 pytest tests, and
  mapping validation
- `cargo build --locked --all-features`
- `cargo package --locked --allow-dirty`, including packaged `--docs` and `--skill`
  content checks
- README generation, skill-mirror validation, workflow-script tests, Markdown
  formatting, and local-link validation

The shared corpus reported 304 exact passes and 38 exact known divergences; all
historical parity cases pass after the merge.
The divergence gate also caught and removed more than 50 stale entries that current Rust
main had already fixed.

This snapshot proves the merged foundation at its pinned upstream commit.
Re-run the relevant gates after each gitlink, manifest, implementation, or workflow
change.

## Current Risks and Required Work

### P0: Math and opaque Markdown preservation

Math is the first behavior implementation priority.
Inline and block math must be protected before Markdown parsing, preserved byte-for-byte
through formatting, restored after rendering, and exercised in containers and feature
interactions. The shared desired-output cases must land before the Rust algorithm.

### P1: Released baseline gap

The preservation branch now includes Rust v0.3.2 with its declared Python v0.7.2
baseline. `fm-t81l` owns the remaining v0.7.2-to-v0.7.3 behavior, test, CLI, API, and
dependency inventory.
Until that closes, the branch is an in-progress port, not a new parity release.

### P1: Upstream commit availability

The Rust gitlink cannot be initialized by a clean remote clone until upstream commit
`093c924` is published.
`fm-zah1` owns that gate.

### P1: Exact known divergences

The current CommonMark and historical parity entries are visible debt, not accepted as
proof of parity. `fmr-rz9f` owns their class-level investigation and reduction.

### P2: Supplemental mapping maintenance

The YAML mapping now covers the v0.7.2 Python baseline and is checked against Cargo’s
deduplicated executable test inventory.
It remains a useful language-specific map, but its count is not a completion claim.
Refresh it during `fm-t81l` after deciding which v0.7.3 tests are best represented by a
shared integration case and which need focused Rust tests.

## Completion Criteria for the Current Porting Cycle

The preservation sync is complete only when:

- the exact Python gitlink is fetchable by a clean clone;
- current Rust main is merged and the v0.7.2-to-v0.7.3 baseline delta has no
  unclassified behavior or test changes;
- every active shared case passes or has an explicitly approved, tested, tracked
  disposition;
- math and other in-scope preservation change IDs are implemented in both languages;
- shared tryscript and conformance suites run against built artifacts;
- no Rust CI behavior test depends on Python;
- the legacy function mapping has no unexplained drift;
- full Rust, documentation, packaging, and release gates pass;
- the dated checklist and sync artifact contain exact command evidence.

See [Port Sync Playbook](port-sync-playbook.md) for the procedure and
[current update checklist](project/specs/active/port-checklist-update-2026-08-25.md) for
the live execution record.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
