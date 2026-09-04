# Project Status: flowmark-rs

> **Doc status:** Rust port-specific (no upstream equivalent).

**Last updated:** 2026-09-04

**Current Rust release:** v0.4.0

**Last declared whole-program Python baseline:** v0.8.0

**In-progress upstream contract:** `7dfd0421d483a42dee29edef999f866b04294720`

## Summary

The v0.4.0 source candidate implements the complete Python v0.8.0 shared contract at
`7dfd042`. Math remains the highest-priority syntax family, but the same automatic,
parser-independent mechanism now covers inline code, fenced and indented code, and the
supported opaque Markdown extensions without requiring dialect flags.

The contract includes source-exact math, code spans, Pandoc multiline and grid tables,
definition lists, line blocks, Obsidian callouts, colon containers, TOML frontmatter,
raw HTML, attribute groups, MyST roles, wikilinks, GitLab bracketed references, and
GitLab multiline blockquotes.
Malformed or ambiguous openers degrade to ordinary Markdown behavior.
Whole-document semantic, cleanup, and typography outputs reach fixed points.

The ordinary Rust suite reads versioned inputs and expected bytes directly from the
pinned Python submodule.
It does not invoke Python.
Focused Rust tests cover only language-specific scanner, bridge, parser-adapter, path,
timeout, and error invariants.

This release declares whole-program parity with Python v0.8.0 except for 32 inherited,
explicitly ledgered CommonMark differences.
The target Python commit is available from the configured remote, and a fresh recursive
clone initializes every gitlink without a local alternate.

## Pinned Sources

| Source | Recorded commit | Purpose |
| --- | --- | --- |
| Python Flowmark | `7dfd0421d483a42dee29edef999f866b04294720` | Source, shared manifest, expected bytes, reference documents, CommonMark, and tryscript |
| Rust porting playbook | `d24760a3fbd2951c730a199269aeb082abb46a42` | Latest reviewed `origin/main`; canonical update workflow and Rust guidance |
| Released parity baseline | Python v0.8.0 at `7dfd0421d483a42dee29edef999f866b04294720` | Supplemental whole-program mapping and release correspondence |

`admin/port-coverage-mapping/shared-conformance.toml` is the machine-checked source for
the in-progress commit, schema, manifest path, divergence ledger, and change-ID map.
The parent `repos/flowmark` gitlink must match it exactly.

The playbook submodule was fetched and compared with `origin/main` during this cycle.
Both resolve to `d24760a`, so no new playbook gitlink change is required.

## Evidence Model

| Layer | Role | Current state |
| --- | --- | --- |
| Shared conformance manifest | Exact stdout, stderr, exit, filesystem, timeout, and idempotence contract | 495 exact passes, 32 exact known divergences |
| Shared tryscript | End-to-end CLI workflows and fixture interactions | Rust executes upstream documents against Cargo-built artifacts |
| Reference and topic documents | Broad whole-document and cross-family interactions | Exact for all implemented change IDs; fixed-point cases active |
| CommonMark 0.31.2 | Large standard-Markdown syntax sweep | Active cases pass or have exact ledger entries |
| Historical parity cases | Previously discovered cross-parser corner cases | All active cases exact |
| Rust-focused tests | Scanner, bridge, adapter, timeout, and path-safety invariants | Intentionally small and language-specific |
| External 670-file corpus | Baseline-transition differential audit | Both binaries selected all files; zero byte differences |
| Legacy YAML mapping | Function-level provenance at released v0.8.0 | Supplemental evidence, not the portable truth source |

See [Test Corpora and Provenance](test-corpora.md) for the ownership, execution, and
reconstruction rules for each layer.

## Shared Change IDs

All current statuses come from `admin/port-coverage-mapping/shared-conformance.toml`.

| Change ID | State |
| --- | --- |
| `FM-CLI-OUTPUT-001` | Implemented |
| `FM-CODE-SPAN-001` | Implemented |
| `FM-CONFORMANCE-001` | Implemented |
| `FM-EXT-ATTRIBUTE-GROUP-001` | Implemented |
| `FM-EXT-COLON-CONTAINER-001` | Implemented |
| `FM-EXT-DEFINITION-LIST-001` | Implemented |
| `FM-EXT-GLFM-001` | Implemented |
| `FM-EXT-GRID-TABLE-001` | Implemented |
| `FM-EXT-LINE-BLOCK-001` | Implemented |
| `FM-EXT-MULTILINE-TABLE-001` | Implemented |
| `FM-EXT-MYST-WIKILINK-001` | Implemented |
| `FM-EXT-OBSIDIAN-CALLOUT-001` | Implemented |
| `FM-EXT-RAW-HTML-001` | Implemented |
| `FM-EXT-TOML-FRONTMATTER-001` | Implemented |
| `FM-MATH-BLOCK-001` | Implemented |
| `FM-MATH-INLINE-001` | Implemented |
| `FM-PRESERVE-CORE-001` | Implemented |
| `FM-REFERENCE-IDEMPOTENCE-001` | Implemented |
| `FM-COMMONMARK-001` | 32 exact known divergences |
| `FM-PARITY-BASELINE-001` | Exact except for the same inherited ledger |

Future portable behavior must receive a stable `FM-*` ID before language-specific
implementation starts.
The same ID identifies the Python cases, beads, Rust mapping record, and dated
validation evidence.

## Preservation Architecture

The port follows five explicit stages:

1. Strictly decode UTF-8, normalize CRLF/CR, record a leading BOM, and canonicalize the
   terminal newline.
2. Scan byte offsets before Markdown parsing.
   Existing fenced and indented code take precedence, followed by registered opaque
   extensions and math.
3. Replace recognized regions with collision-safe fixed-width tokens while reversibly
   escaping authored token-control scalars.
4. Parse, transform, and wrap prose while measuring each token by the original source
   region’s logical width and container context.
5. Validate token count, order, and block boundaries; restore exact source slices; then
   restore the document-level BOM and newline policy.

Recognition is automatic.
Users do not need to select a Markdown dialect or enable math.
The scanner’s role is preservation, not semantic validation: a closed recognized region
is opaque, while unmatched or structurally ambiguous syntax remains available to the
ordinary parser.

The parser bridge is intentionally separate from recognition.
A future parser adapter may need different scaffolding, but it must consume the same
portable regions and exact restoration side table.

## Divergence Policy

`tests/parity_corpus_known_divergences.toml` is a closed, bidirectional ledger with 32
inherited CommonMark case IDs owned by `fmr-rz9f`.

- An unlisted mismatch fails.
- An entry for a missing or deferred case fails.
- A listed case that begins passing fails as stale.
- Every entry names a live tracker and reason.

No new divergence was added for math, code, extensions, or whole-document idempotence.

## Current Risks

### Remote CI and Release Scope

The Python v0.8.0 source PR passed its Python 3.10–3.14 matrix and merged at `7dfd042`.
The Rust v0.4.0 candidate passes the complete local formatter, lint, behavior,
library-only, documentation, supply-chain, shared-contract, and package gates.
Its release-prep PR is merged only after the hosted Linux, macOS, Windows, MSRV,
security, mapping, semver, and packaging gates pass against that exact Python gitlink.
A fresh recursive checkout resolves Python `7dfd042` and the porting playbook `d24760a`
from their configured remotes without a local alternate.
Publication remains separately gated by the required non-publishing release-workflow dry
run in [`docs/publishing.md`](publishing.md).

### Windows Shared-Source Checkout

The Python repository contains long-standing managed shortcut-document filenames with
colons, which NTFS cannot materialize.
Linux and macOS CI initialize the complete recursive submodule tree.
Windows CI resolves the same exact Python gitlink, verifies the fetched commit in an
isolated bare repository, extracts `README.md` plus the shared `tests/` tree directly
from that commit’s Git archive, and records the verified commit as the exported tree’s
detached `HEAD` without checking out the incompatible paths.
The extracted tree contains every portable conformance, tryscript, topic, reference, and
CommonMark asset used by the Rust suite.
The Windows result therefore proves formatter behavior against the shared contract, but
it does not prove that the unrelated upstream documentation tree is Windows-checkout
compatible.

### External Corpus Reconstruction

The historical 623-file `attic/test-docs` corpus remains unrecoverable.
A later 670-file AI Trade Arena snapshot was recovered, identified by digest, and used
successfully, but 199 files differ from the closest immutable source commit.
It is strong local transition evidence, not a portable CI fixture.
The checked-in shared corpus remains the release contract.

### Parser Boundary Interactions

The recovered corpus found two interaction classes after the focused suite was green:
angle comparisons mistaken for HTML, and prose union pipes mistaken for table cells.
New syntax families should continue to probe ambiguous punctuation, container identity,
adjacent protected blocks, table detection, lazy continuation, and malformed boundaries.

### Inherited CommonMark Differences

The 32 exact ledger entries are visible debt rather than parity.
Reducing them is useful, but they do not obscure the result of any new shared change ID.

## Completion Criteria

This porting cycle is locally complete when:

- every in-scope shared change ID is implemented;
- all shared cases pass or have an exact inherited ledger disposition;
- shared tryscript and conformance run against Cargo-built artifacts;
- no normal Rust behavior test invokes Python or copies portable goldens;
- the external audit selects the same complete corpus and has no unexplained diff;
- lint, test, documentation, administration, build, and package gates pass;
- the checklist, sync artifacts, mapping, ledger, and beads agree.

The v0.4.0 source candidate meets these local completion criteria.
Its immutable publication is governed by the release runbook and release beads, which
record the hosted PR, fresh-clone, dry-run, registry, artifact, and Homebrew results.

See the [Port Sync Playbook](port-sync-playbook.md), the
[current update checklist](project/specs/active/port-checklist-update-2026-08-25.md),
the [math/foundation artifact](sync-artifacts/2026-08-26-sync-093c924-to-0d2bebb.md),
and the
[code/extensions artifact](sync-artifacts/2026-08-26-sync-0d2bebb-to-e9d5805.md).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
