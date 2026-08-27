---
title: Python-to-Rust Update Checklist for Markdown Preservation
description: Filled Flowmark-rs Mode B checklist through the complete shared Markdown-preservation contract
author: Codex
---
# Python-to-Rust Update Checklist: Markdown Preservation

This is the filled Flowmark-rs copy of the latest canonical
[subsequent-update checklist](../../../../repos/rust-porting-playbook/playbooks/port-checklist-update-template.md).
It records the complete porting cycle from the last declared Python baseline through the
current preservation-contract commit.
It remains active until the release gates at the end are satisfied.

## Run Identity

| Field | Value |
| --- | --- |
| Release mode | Mode B: upstream baseline/commit changes |
| Branch starting Python baseline | v0.6.5, `f1228297c8e2380685c6a10383f59164b96f6c22` |
| Merged Rust release baseline | Rust v0.3.2 / Python v0.7.2, Rust `015f23989af3e5cfb3f8b58dfc72822c534df25a` |
| Current branch parity baseline | v0.7.3, `7912c322417ae49c5c45ab099997c142cf392db8` |
| Preservation-contract target | `19c840eef664ac0c7fa8a7d3ce1b6252141ca648` |
| Pre-cycle Rust reference | `cb744eb` |
| Shared-test foundation | `ccc8897ac0cb6c21c017f13ac68311198da71a48` |
| Current-main merge | `c6449a5cba5b069c5bd29be0e08e83d036279b59` (second parent `015f23989af3e5cfb3f8b58dfc72822c534df25a`) |
| Previous playbook commit | `df36b99744405622defa56ad2e7b6f38129e121c` |
| Current playbook commit | `d24760a3fbd2951c730a199269aeb082abb46a42` |
| Shared schema | 1 |
| Math implementation | `b76f635` |
| Code/extensions implementation | `b040416` through `bb8eb08` |
| Native/test alignment | `bd7248e` through `c07bc61` |
| Hardened audit harness | `ab42ba3` |
| Sync artifacts | `docs/sync-artifacts/2026-08-26-sync-093c924-to-0d2bebb.md`; `docs/sync-artifacts/2026-08-26-sync-0d2bebb-to-e9d5805.md` |

## Flowmark Adaptations to the Canonical Checklist

| Canonical instruction | Flowmark-rs adaptation | Reason |
| --- | --- | --- |
| Copy source fixtures into the Rust repository | Read all portable assets directly from pinned `repos/flowmark` | One versioned golden source prevents copy drift |
| Install the target Python executable for normal parity tests | Run the native Rust adapter and upstream tryscript against the built Rust binary | The portable contract must not depend on Python |
| Port every Python test one-to-one | Prefer one shared integration/golden case; add focused Rust tests only for language-specific code | Maps behavior rather than framework mechanics |
| Update fixed test-count constants | Validate stable IDs, references, schema, and mapping completeness | Counts change under parameterization and do not prove coverage |
| Treat coverage percentages as completion gates | Use coverage as gap discovery; use shared surface and exact-output gates for acceptance | Assertion quality and syntax breadth matter more than a scalar count |
| Run live cross-binary comparison in ordinary CI | Run it as a pinned baseline-transition audit; run versioned shared evidence in ordinary Rust CI | Keeps Rust CI portable while retaining source-oracle checks at acceptance time |
| Recursively materialize every upstream path on Windows | Resolve the exact Python gitlink and extract `README.md` plus its shared `tests/` tree from that commit’s Git archive; retain complete recursive checkout on Linux and macOS | NTFS rejects existing colon-named upstream shortcut docs, while the Rust build and portable behavior assets need only those exported paths |

These adaptations follow the latest playbook’s rules to share fixture inputs, map one
source test to a shared golden test where appropriate, test the built Rust executable,
and keep golden evidence versioned with provenance.

## Preflight

- [x] Initialize all three submodules and verify their exact gitlinks.
- [x] Confirm the Python baseline changes; select Mode B.
- [x] Confirm the playbook submodule had no local edits before advancing it.
- [x] Read the current update checklist, sync/release workflow, test-coverage playbook,
  Python-to-Rust rules, and Rust testing rules.
- [x] Run the clean pre-sync Rust quality and test gates at `ccc8897`.
- [x] Create `fm-98jy` and `fm-4loc` for the workflow refresh.
- [x] Detect that the branch is 88 commits behind current Rust main.
- [x] Create `fm-mfvi` to merge current Rust main without restoring obsolete test
  architecture.
- [x] Create `fm-t81l` for the remaining released v0.7.2-to-v0.7.3 baseline gap.
- [x] Create `fm-zah1` for publication of the exact Python gitlink.
- [x] Prove a clean remote clone can initialize `repos/flowmark` at `783b445`
  (`fm-zah1`).

## Phase 0: Empirical Pre-Port Verification

- [x] Build the Rust binary before implementing preservation behavior.
- [x] Run the new shared current-behavior, reference, historical parity, and CommonMark
  cases against the existing Rust implementation.
- [x] Record exact current differences in `tests/parity_corpus_known_divergences.toml`
  (`fmr-rz9f`).
- [x] Verify the ledger is bidirectional: unlisted failures and stale passing entries
  both fail the suite.
- [x] Verify current upstream tryscript documents against the Cargo-built binary.
- [x] Add and review the shared desired-output inline-math cases.
- [x] Add and review the shared desired-output block-math and container cases.
- [x] Run every new math case against the pre-port Rust binary and record its exact
  disposition before implementation.
- [x] Record the pre-port matrix by change ID: core 4 pass/5 fail, inline 15 pass/10
  fail, block 1 pass/7 fail, and CLI output 0 pass/1 fail.
- [x] Promote the sibling-list identity finding from a Python-only scanner invariant to
  the shared case `preservation.math.block.sibling-boundaries`.

## Phase 1: Sync and Classify Upstream

### Source and Process Pins

- [x] Advance `repos/flowmark` from `f122829` through `093c924` and `0d2bebb` to the
  `e9d5805` behavior tip, its integration-golden successor `b027fde`, and the Python
  Python 3.10 compatibility successors through `783b445`, followed by the shared GLFM
  and issue-traceability target `19c840e`.
- [x] Advance `repos/rust-porting-playbook` from `df36b99` to `d24760a`.
- [x] Record the shared source commit and change IDs in
  `admin/port-coverage-mapping/shared-conformance.toml`.
- [x] Publish the exact Python target commit so the parent gitlink is remotely fetchable
  (`fm-zah1`).

### Delta Classification

- [x] Split the recovery into current Rust main (through Python v0.7.2), the released
  v0.7.2-to-v0.7.3 track, the math/foundation track through `0d2bebb`, and the
  code/extensions/fixed-point track through `e9d5805`, followed by formatting and
  integration-golden alignment through `b027fde`, followed by the compatibility-only
  `783b445` source pin.
- [x] Merge Rust `origin/main` at `015f239` and preserve its release, security,
  dependency, skill, and v0.7.2 parity work (`fm-mfvi`).
- [x] Record the preservation branch as specification and shared-test infrastructure,
  not yet as implemented math behavior.
- [x] Inventory every behavior, CLI, API, test, dependency, and generated-content change
  in v0.7.2-to-v0.7.3 (`fm-t81l`).
- [x] Give every relevant released change a Rust disposition and evidence link.
- [x] Regenerate the v0.7.2 Rust test inventory from Cargo, eliminate phantom deleted
  records, and repair mappings to the current tests and direct tryscript wrappers.
- [x] Refresh the supplemental function map for v0.7.3 after the inventory is stable.

Completion gate: the remote-gitlink gate `fm-zah1` is complete.

## Phase 2: Shared Contract Foundation

- [x] Implement the schema-versioned native Rust conformance adapter.
- [x] Validate paths, payload ownership, duplicate IDs, unsupported fields, symlinks,
  process timeouts, full stdout/stderr, exit codes, file trees, and idempotence.
- [x] Execute the upstream reference documents and historical parity cases directly.
- [x] Execute the upstream CommonMark registry without copying its fixtures.
- [x] Execute the upstream tryscript files from isolated temporary copies.
- [x] Pin tryscript 0.1.7 in development, CI, and publish workflows.
- [x] Remove Python execution from normal Rust behavior tests.
- [x] Remove copied portable fixtures and the deleted Python-coupled parity harness.
- [x] Bundle runtime docs and skill resources so packaged behavior does not depend on
  submodules.
- [x] Verify the crate package contains and serves its bundled `--docs` and `--skill`
  resources.

## Phase 3: Port Preservation, Math, Code, and Extensions

For each change ID, the order is shared red case, empirical Rust result, idiomatic Rust
implementation, focused Rust helper tests if needed, shared green case, ledger update,
and traceability update.

| Change ID | Behavior | Rust owner | State |
| --- | --- | --- | --- |
| `FM-PRESERVE-CORE-001` | Normalization, token collision, strict UTF-8, and failure atomicity | `fm-1mq0` | Implemented |
| `FM-MATH-INLINE-001` | Inline math dialect recognition, exact bytes, and source-width wrapping | `fm-fpbj` | Implemented |
| `FM-MATH-BLOCK-001` | Display math, environments, containers, and malformed fallback | `fm-fpbj` | Implemented |
| `FM-CLI-OUTPUT-001` | Direct single-file output path | `fm-1mq0` | Implemented |
| `FM-CODE-SPAN-001` | Source-exact inline code preservation | `fm-82vu` | Implemented |
| `FM-EXT-MULTILINE-TABLE-001` | Pandoc multiline tables | `fm-kr0a` | Implemented |
| `FM-EXT-OBSIDIAN-CALLOUT-001` | Obsidian callouts | `fm-aq78` | Implemented |
| `FM-EXT-COLON-CONTAINER-001` | Colon-fenced containers | `fm-dvl6` | Implemented |
| `FM-EXT-TOML-FRONTMATTER-001` | TOML frontmatter | `fm-bl2j` | Implemented |
| `FM-EXT-DEFINITION-LIST-001` | Pandoc definition lists | `fm-663e` | Implemented |
| `FM-EXT-GLFM-001` | GitLab bracketed references and multiline blockquotes | `fmr-bnkr` | Implemented |
| `FM-EXT-GRID-TABLE-001` | Pandoc grid tables | `fm-z8xh` | Implemented |
| `FM-EXT-RAW-HTML-001` | Raw HTML and angle-span fallback | `fm-w1tn` | Implemented |
| `FM-EXT-ATTRIBUTE-GROUP-001` | Markdown attribute groups | `fm-c57j` | Implemented |
| `FM-EXT-LINE-BLOCK-001` | Pandoc line blocks | `fm-mw49` | Implemented |
| `FM-EXT-MYST-WIKILINK-001` | MyST roles and wikilinks | `fm-5vlb` | Implemented |
| `FM-REFERENCE-IDEMPOTENCE-001` | Reference-document fixed points | `fm-w467` | Implemented |

- [x] Port the normalized byte model and preservation registry.
- [x] Port pre-parse scanners with code-span precedence over math.
- [x] Port inline math and prove delimiter/currency/escape corner cases.
- [x] Port block math, active container-frame identity, and malformed boundaries.
- [x] Port collision-safe fixed-width tokens and source-width-aware wrapping.
- [x] Port restoration failure boundaries with stable errors and no silent corruption.
- [x] Reuse the preservation registry and bridge for every implemented syntax family; do
  not stack family-specific parser workarounds over it.
- [x] Port automatic recognition for code spans and every registered opaque extension
  without adding dialect configuration.
- [x] Port the issue #67 follow-up for allowlisted GitLab references, reference pipes in
  tables, and compatible paired `>>>` fences.
- [x] Activate the three reference-document fixed-point cases.
- [x] Promote recovered-corpus failures for angle comparisons and prose pipes into
  shared cases before fixing Rust.
- [x] Confirm every new or changed shared case is represented in the change-ID map.
- [x] Reduce the exact divergence ledger in the same commit that makes cases pass.

## Phase 4: CLI and Filesystem Contract

- [x] Match the shared in-place backup suffix and exact file-tree result.
- [x] Match nonexistent-path stderr and usage exit status.
- [x] Support stdin with `--output` as required by the upstream transcript.
- [x] Match current `--surfaces`, `--skill`, and `--docs` workflows.
- [x] Run shared tests in isolated roots with explicit deterministic environment.
- [x] Add math/preservation CLI cases for stdin, output files, in-place formatting,
  check mode, project config, and invalid-file failure.
- [x] Validate CRLF, missing final newline, strict UTF-8, BOM policy, no mutation on
  failure, and atomic in-place replacement wherever those observations belong to the
  contract.
- [x] Keep multiple-file partial-failure, symlink, and permission behavior out of this
  contract until exact portable observations are specified; do not infer them from
  native implementation details.

## Phase 5: Mapping and Divergences

- [x] Add a machine-checked shared change-ID map with the exact upstream commit.
- [x] Replace wildcard/summary tolerance with exact case-ID entries.
- [x] Reject stale divergence entries.
- [x] Treat the YAML Python/Rust function mapping as supplementary evidence.
- [x] Complete the v0.7.3 supplemental mapping refresh (`fm-t81l`).
- [x] Mark every core, math, code, extension, CLI, and fixed-point change ID implemented
  at target `19c840e`.
- [x] Remove four ledger entries that became exact; retain 34 exact inherited CommonMark
  divergences.
- [x] Regenerate the Rust inventory after adding native preservation diagnostics and
  require zero missing or broken references.
- [ ] Resolve or receive explicit approval for every remaining CommonMark and historical
  parity divergence (`fmr-rz9f`).
- [x] Confirm no active shared change ID has `deferred` status.

## Phase 6: Documentation and Workflow

- [x] Rewrite `docs/port-sync-playbook.md` around direct shared-corpus consumption.
- [x] Replace stale full-parity and fixed-count claims in `docs/port-status.md`.
- [x] Rewrite `tests/qa/rust-python-parity-e2e.qa.md` as a clean, language-neutral
  built-binary workflow.
- [x] Move the completed v0.6.5 sync plan out of `active/`.
- [x] Create this filled current checklist and a dated sync artifact.
- [x] Record why Python is a baseline-transition audit rather than a normal Rust CI
  dependency.
- [x] Re-run documentation formatting and local-link checks after the merge.
- [x] Re-run README generation for the merged v0.7.2 baseline and shared user docs.
- [x] Re-run README generation for the declared v0.7.3 parity baseline.
- [x] Revise the Flowmark workflow for shared regression promotion, observable
  container-boundary tests, and parser-independent preservation adapters.
- [x] Verify the pinned playbook is still the latest `origin/main` at `d24760a`.
- [x] Create the dated `093c924`-to-`0d2bebb` implementation artifact.
- [x] Create the dated `0d2bebb`-to-`e9d5805` implementation artifact.
- [x] Recover and document the external corpus provenance and reconstruction limits.
- [x] Harden the external audit around explicit paths, digests, complete selection, and
  retained diffs.

## Phase 7: Validation

- [x] `cargo fmt --all -- --check` at shared-test foundation commit.
- [x] `cargo clippy --locked --all-targets --all-features -- -D warnings` at shared-test
  foundation commit.
- [x] `cargo test --locked --all-features` at shared-test foundation commit.
- [x] `cargo test --locked --no-default-features` at shared-test foundation commit.
- [x] Rust administration Ruff, BasedPyright, pytest, and mapping checks at shared-test
  foundation commit.
- [x] `cargo build --locked --all-features` at shared-test foundation commit.
- [x] `cargo package --locked --allow-dirty` plus packaged docs/skill smoke tests at
  shared-test foundation commit.
- [x] Re-run all affected gates after the workflow/playbook commit.
- [x] Merge current Rust main and rerun every gate (`fm-mfvi`).
- [x] Run the pinned full differential and syntactic-class sweeps before advancing the
  released whole-program baseline.
- [x] Run a clean-clone test after `783b445` becomes remotely fetchable.
- [x] Keep full recursive submodule checkout on Linux and macOS; on Windows, record the
  exact Python gitlink, verify the fetched object in an isolated bare repository, and
  export only `README.md` and the shared `tests/` tree from that commit, with the
  verified source recorded as the exported tree’s detached `HEAD`.
- [x] Record post-merge commands, exit statuses, selected case IDs, and divergence delta
  in the sync artifact.
- [x] Record the Python shared-conformance, Rust focused conformance, formatter, clippy,
  and preservation-unit evidence for implementation commit `b76f635`.
- [x] Append the final full-gate evidence after documentation and mapping stabilize.
- [x] Re-run formatting, clippy, all-feature tests, no-default-feature tests, and native
  shared conformance at `19c840e`: 482 exact passes and the unchanged 34-entry
  CommonMark ledger.
- [x] Re-run the final local matrix: all-features and no-default-features tests, clippy,
  rustdoc warnings, build warnings, crate verification, packaged docs/skill smoke,
  administration lint/types/tests, and mapping validation.
- [x] Retain the final 670-file transition audit at
  `target/corpus-parity/2026-08-26-b027fde-final/`; both binaries selected every file
  and produced zero byte differences.

## Phase 8: Release Acceptance

- [x] The exact Python and playbook submodule commits are fetchable from clean clones.
- [x] The released baseline gap is closed.
- [x] Every preservation-cycle change ID is implemented.
- [x] Every active shared case passes or has an exact inherited ledger disposition.
- [x] No unexplained preservation CLI, output-byte, file-tree, or idempotence difference
  remains.
- [x] The public API change is additive (`reformat_bytes`); the crates.io release
  baseline remains v0.3.2 until release planning.
- [x] Full local test, lint, documentation, administration, build, and packaging gates
  pass.
- [x] Remote clean-clone and publication gates pass.
- [x] Remote PR CI and security gates pass on Python 3.10–3.14 and Rust Linux, macOS,
  Windows, and MSRV 1.85.
- [ ] Release-planning gates pass.
- [ ] Version correspondence and release notes state the exact achieved surface.
- [x] Beads, mapping, divergence ledger, checklist, and sync artifact agree.

This checklist cannot be closed on the strength of test totals or a green subset.
Completion requires the exact evidence above and zero unexplained differences.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
