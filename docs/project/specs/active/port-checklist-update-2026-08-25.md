---
title: Python-to-Rust Update Checklist for Markdown Preservation
description: Filled Flowmark-rs Mode B checklist for the v0.7.2 baseline through the pinned preservation-contract commit
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
| Current released Rust/Python baseline | Rust v0.3.2 / Python v0.7.2, Rust `015f23989af3e5cfb3f8b58dfc72822c534df25a` |
| Intermediate released target | v0.7.3, `7912c322417ae49c5c45ab099997c142cf392db8` |
| Preservation-contract target | `093c9249610965b37a458b32e37b5cc4738afe48` |
| Pre-cycle Rust reference | `cb744eb` |
| Shared-test foundation | `ccc8897ac0cb6c21c017f13ac68311198da71a48` |
| Previous playbook commit | `df36b99744405622defa56ad2e7b6f38129e121c` |
| Current playbook commit | `d24760a3fbd2951c730a199269aeb082abb46a42` |
| Shared schema | 1 |
| Sync artifact | `docs/sync-artifacts/2026-08-25-sync-v0.7.2-to-093c924.md` |

## Flowmark Adaptations to the Canonical Checklist

| Canonical instruction | Flowmark-rs adaptation | Reason |
| --- | --- | --- |
| Copy source fixtures into the Rust repository | Read all portable assets directly from pinned `repos/flowmark` | One versioned golden source prevents copy drift |
| Install the target Python executable for normal parity tests | Run the native Rust adapter and upstream tryscript against the built Rust binary | The portable contract must not depend on Python |
| Port every Python test one-to-one | Prefer one shared integration/golden case; add focused Rust tests only for language-specific code | Maps behavior rather than framework mechanics |
| Update fixed test-count constants | Validate stable IDs, references, schema, and mapping completeness | Counts change under parameterization and do not prove coverage |
| Treat coverage percentages as completion gates | Use coverage as gap discovery; use shared surface and exact-output gates for acceptance | Assertion quality and syntax breadth matter more than a scalar count |
| Run live cross-binary comparison in ordinary CI | Run it as a pinned baseline-transition audit; run versioned shared evidence in ordinary Rust CI | Keeps Rust CI portable while retaining source-oracle checks at acceptance time |

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
- [ ] Prove a clean remote clone can initialize `repos/flowmark` at `093c924`
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
- [ ] Add the shared desired-output inline-math cases (`fm-9m7k`).
- [ ] Add the shared desired-output block-math and container cases (`fm-8rmy`).
- [ ] Run every new math case against the pre-port Rust binary and record its exact
  disposition before implementation.

## Phase 1: Sync and Classify Upstream

### Source and Process Pins

- [x] Advance `repos/flowmark` from `f122829` to `093c924` locally.
- [x] Advance `repos/rust-porting-playbook` from `df36b99` to `d24760a`.
- [x] Record the shared source commit and change IDs in
  `admin/port-coverage-mapping/shared-conformance.toml`.
- [ ] Publish the exact Python target commit so the parent gitlink is remotely fetchable
  (`fm-zah1`).

### Delta Classification

- [x] Split the recovery into current Rust main (through Python v0.7.2), the released
  v0.7.2-to-v0.7.3 track, and the v0.7.3-to-`093c924` preservation-contract track.
- [ ] Merge Rust `origin/main` at `015f239` and preserve its release, security,
  dependency, skill, and v0.7.2 parity work (`fm-mfvi`).
- [x] Record the preservation branch as specification and shared-test infrastructure,
  not yet as implemented math behavior.
- [ ] Inventory every behavior, CLI, API, test, dependency, and generated-content change
  in v0.7.2-to-v0.7.3 (`fm-t81l`).
- [ ] Give every relevant released change a Rust disposition and evidence link.
- [ ] Refresh the supplemental function map for the new released baseline after the
  inventory is stable.

Completion gate: Phase 1 remains open until `fm-mfvi`, `fm-t81l`, and `fm-zah1` close.

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

## Phase 3: Port Preservation and Math

For each change ID, the order is shared red case, empirical Rust result, idiomatic Rust
implementation, focused Rust helper tests if needed, shared green case, ledger update,
and traceability update.

| Change ID | Behavior | Rust owner | State |
| --- | --- | --- | --- |
| `FM-MATH-INLINE-001` | Inline dollar-math recognition and byte preservation | `fm-fpbj` | Deferred pending shared desired outputs |
| `FM-CODE-SPAN-001` | Source-exact inline code preservation | `fm-82vu` | Deferred |
| `FM-EXT-RAW-HTML-001` | Opaque raw/extension syntax preservation | `fm-w1tn` | Deferred |
| `FM-REFERENCE-IDEMPOTENCE-001` | Reference-document fixed points | `fm-w467` | Deferred |

- [ ] Port the normalized byte model and preservation registry.
- [ ] Port pre-parse scanners with code-span precedence over math.
- [ ] Port inline math and prove delimiter/currency/escape corner cases.
- [ ] Port block math and container-boundary handling.
- [ ] Port restoration failure boundaries with stable errors and no silent corruption.
- [ ] Remove superseded PUA/NUL workarounds rather than stacking another mechanism.
- [ ] Confirm every new or changed shared case is represented in the change-ID map.
- [ ] Reduce the exact divergence ledger in the same commits that make cases pass.

## Phase 4: CLI and Filesystem Contract

- [x] Match the shared in-place backup suffix and exact file-tree result.
- [x] Match nonexistent-path stderr and usage exit status.
- [x] Support stdin with `--output` as required by the upstream transcript.
- [x] Match current `--surfaces`, `--skill`, and `--docs` workflows.
- [x] Run shared tests in isolated roots with explicit deterministic environment.
- [ ] Add math/preservation CLI cases for stdin, output files, in-place formatting,
  check mode, multiple files, and partial failure.
- [ ] Validate CRLF, missing final newline, UTF-8, BOM policy, symlinks, permissions,
  and atomic replacement wherever those observations belong to the contract.

## Phase 5: Mapping and Divergences

- [x] Add a machine-checked shared change-ID map with the exact upstream commit.
- [x] Replace wildcard/summary tolerance with exact case-ID entries.
- [x] Reject stale divergence entries.
- [x] Treat the YAML Python/Rust function mapping as supplementary evidence.
- [ ] Complete the v0.7.3 supplemental mapping refresh (`fm-t81l`).
- [ ] Resolve or receive explicit approval for every remaining CommonMark and historical
  parity divergence (`fmr-rz9f`).
- [ ] Confirm no active shared change ID has `deferred` status at release acceptance.

## Phase 6: Documentation and Workflow

- [x] Rewrite `docs/port-sync-playbook.md` around direct shared-corpus consumption.
- [x] Replace stale full-parity and fixed-count claims in `docs/port-status.md`.
- [x] Rewrite `tests/qa/rust-python-parity-e2e.qa.md` as a clean, language-neutral
  built-binary workflow.
- [x] Move the completed v0.6.5 sync plan out of `active/`.
- [x] Create this filled current checklist and a dated sync artifact.
- [x] Record why Python is a baseline-transition audit rather than a normal Rust CI
  dependency.
- [ ] Re-run documentation formatting and link checks after all edits.
- [ ] Re-run README generation if the released parity version or shared user docs
  change.

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
- [ ] Re-run all affected gates after the workflow/playbook commit.
- [ ] Merge current Rust main and rerun every gate (`fm-mfvi`).
- [ ] Run the pinned full differential and syntactic-class sweeps before advancing the
  released whole-program baseline.
- [ ] Run a clean-clone test after `093c924` becomes remotely fetchable.
- [ ] Record final commands, exit statuses, selected case IDs, and divergence delta in
  the sync artifact.

## Phase 8: Release Acceptance

- [ ] The exact Python and playbook submodule commits are fetchable from clean clones.
- [ ] The released baseline gap is closed.
- [ ] Every in-scope shared change ID is implemented.
- [ ] Every shared case passes or has an explicitly approved, tested disposition.
- [ ] No unexplained CLI, output-byte, file-tree, or idempotence differences remain.
- [ ] The public API and crates.io baseline have been checked if changed.
- [ ] Full test, lint, documentation, security, packaging, and release gates pass.
- [ ] Version correspondence and release notes state the exact achieved surface.
- [ ] Beads, mapping, divergence ledger, checklist, and sync artifact agree.

This checklist cannot be closed on the strength of test totals or a green subset.
Completion requires the exact evidence above and zero unexplained differences.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
