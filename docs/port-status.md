# Project Status: flowmark-rs

**Last updated:** 2026-08-26

**Current Rust release:** v0.3.2

**Last declared whole-program Python baseline:** v0.7.3

**In-progress upstream contract:** commit `0d2bebb0fabb9ad8705ac797687f96335ca7cfe7`
(`v0.7.3-38-g0d2bebb`)

## Summary

The current branch ports the complete shared preservation contract through `0d2bebb`.
Rust now recognizes and preserves the same inline and block math forms as Python,
normalizes source bytes at the same boundary, accounts for protected source width while
wrapping, and implements the shared single-file output and invalid-UTF-8 behavior.

The design remains parser-independent at the contract boundary.
A byte-offset scanner finds opaque source regions before comrak sees the document; a
collision-safe, fixed-width bridge carries those regions through parsing and wrapping;
restoration validates the token stream and re-inserts the original source slices.
No math dialect flag is required.

The ordinary Rust suite consumes upstream expectations directly and does not invoke
Python. Small Rust tests cover only scanner, bridge, and adapter invariants.
The shared corpus supplies the portable end-to-end evidence used by both
implementations.

Do not describe this branch as whole-program parity with the in-progress commit.
The new preservation change IDs are exact, but code-span, raw/extension syntax, and the
remaining reference-idempotence surfaces still have explicit deferred owners.
One delivery gate also remains: publish the exact Python commit so a clean remote Rust
clone can initialize its submodule (`fm-zah1`).

## Pinned Sources

| Source | Recorded commit | Purpose |
| --- | --- | --- |
| Python Flowmark | `0d2bebb0fabb9ad8705ac797687f96335ca7cfe7` | Source, shared manifests, expected bytes, and tryscript documents |
| Rust porting playbook | `d24760a3fbd2951c730a199269aeb082abb46a42` | Latest `origin/main`; current update workflow and Rust guidance |
| Homebrew tap | `6567a9ffbf7d90c0c08ec55dc43583e060c5349b` | Historical release integration |

`admin/port-coverage-mapping/shared-conformance.toml` is the machine-checked source for
the Python commit, manifest path, schema, and shared change-ID mapping.
The parent submodule gitlink must match it.

The Python target exists on the local source branch but is not yet available from the
configured GitHub remote.
Remote Rust CI is therefore gated on `fm-zah1`: publish that exact commit, then prove
clean-clone submodule initialization before merging or releasing the Rust branch.

The playbook submodule was fetched and compared with `origin/main` during this cycle.
Both resolve to `d24760a`, so no playbook gitlink change is required.

## Evidence Model

| Layer | Role | Current state |
| --- | --- | --- |
| Shared conformance manifest | Exact stdout, stderr, exit, filesystem, timeout, and idempotence contract | 404 exact passes, 35 exact known divergences |
| Shared tryscript | End-to-end CLI workflows and fixture interactions | Rust executes upstream documents directly |
| Reference documents | Broad whole-document interactions in multiple modes | Exact for the current preservation target |
| CommonMark 0.31.2 | Large syntax-surface sweep | Active cases pass or have exact ledger entries |
| Historical parity corpus | Previously discovered cross-language corner cases | All active cases pass exactly |
| Rust-focused tests | Scanner, bridge, adapter, timeout, and path-safety invariants | Intentionally small and language-specific |
| Legacy YAML mapping | Function-level provenance at the released v0.7.3 baseline | 442 Python records: 395 mapped, 47 excluded; 665 native Rust tests inventoried |

Pinned cross-binary runs remain a transition audit when accepting a new baseline or
investigating a discrepancy.
They are not a normal CI dependency and never generate the golden outputs used to judge
Rust.

## Shared Change IDs

The status values below come from `admin/port-coverage-mapping/shared-conformance.toml`.

| Change ID | Python bead | Rust bead | Status |
| --- | --- | --- | --- |
| `FM-CLI-OUTPUT-001` | `fm-9r1n` | `fm-1mq0` | Implemented |
| `FM-CODE-SPAN-001` | `fm-ocpw` | `fm-82vu` | Deferred |
| `FM-COMMONMARK-001` | `fm-shou` | `fm-gc8d` | Known divergences |
| `FM-CONFORMANCE-001` | `fm-ltof` | `fm-gc8d` | Implemented |
| `FM-EXT-RAW-HTML-001` | `fm-w1tn` | `fm-w1tn` | Deferred |
| `FM-MATH-BLOCK-001` | `fm-6erm` | `fm-fpbj` | Implemented |
| `FM-MATH-INLINE-001` | `fm-ucy8` | `fm-fpbj` | Implemented |
| `FM-PARITY-BASELINE-001` | `fm-gc8d` | `fm-gc8d` | Known divergences |
| `FM-PRESERVE-CORE-001` | `fm-2tto` | `fm-1mq0` | Implemented |
| `FM-REFERENCE-IDEMPOTENCE-001` | `fm-w467` | `fm-w467` | Deferred |

Future shared behavior must receive a stable `FM-*` ID before language-specific
implementation starts.
The same ID identifies the Python case, Rust bead, mapping record, and validation
evidence.

## Preservation Architecture

The port follows five explicit stages:

1. Strictly decode UTF-8, normalize CRLF/CR, record a leading BOM, and canonicalize the
   terminal newline.
2. Scan byte offsets before Markdown parsing, with fenced and indented code taking
   precedence over math.
3. Replace recognized regions with fixed-width supplementary-private-use tokens while
   reversibly escaping authored token controls.
4. Parse, transform, and wrap prose while measuring each token by the original region’s
   logical source width.
5. Validate token order and block line boundaries, restore exact source slices, then
   restore the document-level BOM policy.

Inline recognition covers single and double dollar forms, `\(...\)`, GitLab and MyST
forms, and inline LaTeX environments.
Block recognition covers dollar displays, `\[...\]`, nested/starred/custom environments,
and container continuations in lists and blockquotes.
The scanners deliberately degrade safely on unmatched or mismatched openers.

## Divergence Policy

`tests/parity_corpus_known_divergences.toml` is a closed, exact ledger.
It currently contains 35 inherited CommonMark case IDs.
The preservation implementation removed three stale entries that now pass.

The ledger is not a wildcard allowlist:

- an unlisted mismatch fails;
- an entry for a missing or deferred case fails;
- a listed case that starts passing fails as stale;
- every entry names a live tracker and reason.

The current inherited differences are owned by `fmr-rz9f`. Remove passing case IDs in
the same commit as the fix; never broaden an old reason to cover a new failure class.

## Current Validation Snapshot

With the Python submodule at `0d2bebb` and the playbook at `d24760a`, the focused shared
conformance run passes all 404 active exact cases and retains 35 exact known
divergences. The implemented preservation IDs account for:

- 9 `FM-PRESERVE-CORE-001` cases;
- 25 `FM-MATH-INLINE-001` cases, including selected CommonMark examples;
- 9 `FM-MATH-BLOCK-001` cases;
- 1 `FM-CLI-OUTPUT-001` case.

The selected surface includes BOM and line endings, invalid UTF-8 and no-mutation
failures, sentinel collisions, inline and block dialects, code precedence, malformed
fallbacks, Markdown containers, tables, links and images, frontmatter and HTML
adjacency, semantic and width-boundary wrapping, file/config/check/output modes,
idempotence, and an adversarial linear-time case.

The dated sync artifact records the full pre-port red matrix and final command evidence.
The final local lint, all-features and no-default-features tests, documentation,
administration, build, crate verification, and packaged-resource smoke gates pass.
Only the remote clean-clone and publication gate remains for this cycle.

## Remaining Risks and Work

### P0: Publish the Exact Upstream Commit

The Rust gitlink cannot be initialized by a clean remote clone until Python commit
`0d2bebb` is published.
`fm-zah1` owns the gate.
No Rust push or release should precede it.

### P1: Extend Opaque Syntax Coverage

Code spans, raw/extension blocks, and remaining reference-document fixed points still
have deferred change IDs.
They should use the same scanner/registry/bridge rather than introducing a second
preservation mechanism.

### P1: Exercise Container Identity as the Corpus Grows

The shared suite covers nested quotes, list content columns, lazy continuation, tabs,
tables, and malformed boundaries.
New Markdown dialect cases should especially probe sibling lists with identical
indentation, mixed quote/list transitions, interruption by HTML or fences, and adjacent
protected blocks. These are the places where a coarse container signature is most likely
to over-consume or terminate early.

### P1: Keep Restoration Fail-Closed

The bridge detects missing, duplicate, reordered, or structurally damaged tokens.
The current public string API treats those states as internal invariants.
If third-party parser adapters become configurable, introduce a fallible formatting path
before allowing an invariant failure to cross the library boundary.

### P2: Reduce Inherited CommonMark Divergences

The 35 exact ledger entries are visible debt, not evidence of parity.
`fmr-rz9f` owns their class-level investigation and reduction.

## Completion Criteria for This Porting Cycle

The preservation sync is complete only when:

- the exact Python gitlink is fetchable by a clean clone;
- every in-scope shared case passes or has an explicitly approved, tested disposition;
- shared tryscript and conformance suites run against built Rust artifacts;
- no normal Rust behavior test depends on Python;
- the legacy function mapping has no unexplained drift;
- full local Rust, documentation, administration, build, and packaging gates pass;
- the exact upstream gitlink is published and passes the remote clean-clone gate;
- the checklist, sync artifact, mapping, ledger, and beads agree.

See the [Port Sync Playbook](port-sync-playbook.md), the
[current update checklist](project/specs/active/port-checklist-update-2026-08-25.md),
and the
[current preservation sync artifact](sync-artifacts/2026-08-26-sync-093c924-to-0d2bebb.md).

The released-baseline decision record remains
[Baseline Audit: Python Flowmark v0.7.2 to v0.7.3](sync-artifacts/2026-08-25-baseline-audit-v0.7.2-to-v0.7.3.md).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
