---
title: Flowmark Rust Port End-to-End QA
description: Clean, language-neutral validation of the built Rust CLI against the pinned Flowmark contract
author: Codex
---
# QA Playbook: Flowmark Rust Port End-to-End Validation

Use this runbook when accepting a new upstream commit, changing the conformance adapter,
closing a parity bead, or preparing a release.
It validates the Cargo-built executable against the exact shared assets in the pinned
Python submodule.

Most checks below are automated.
The manual work is limited to reviewing the complete golden diffs, confirming the
selected evidence matches the intended change, and recording the results.
If a manual observation can be made deterministic, add it to the shared manifest or
tryscript suite and remove it from this runbook.

## Run Record

Fill this section during execution.

| Field | Value |
| --- | --- |
| Date | `<YYYY-MM-DD>` |
| Rust commit | `<full SHA>` |
| Python submodule commit | `<full SHA>` |
| Shared schema | `<integer>` |
| Playbook submodule commit | `<full SHA>` |
| Change IDs under review | `<FM-...>` |
| Reviewer | `<name>` |

Final result: **Pending**

## 1. Establish a Reproducible Checkout

```bash
git status --short --branch
git submodule update --init --recursive
git submodule status
git -C repos/flowmark rev-parse HEAD
git -C repos/rust-porting-playbook rev-parse HEAD
```

Verify:

- [ ] The worktree contains no unexplained changes.
- [ ] Every submodule is initialized at the parent-recorded gitlink.
- [ ] The Flowmark commit equals `upstream_commit` in
  `admin/port-coverage-mapping/shared-conformance.toml`.
- [ ] The exact Flowmark commit is fetchable from its configured remote.
  If not, record `fm-zah1` as a release/remote-CI blocker.
- [ ] The playbook commit is the reviewed commit named in the current sync artifact.

Do not use an exact-SHA fetch from an existing source checkout as the publication proof:
if that object already exists locally, Git can report success without receiving it from
the remote. Use a fresh recursive clone or an empty object store.

Do not continue with a substituted tag or nearby commit.
The expected bytes and change IDs are valid only for the recorded source commit.

## 2. Build the Executable Once

```bash
cargo build --locked --all-features
```

Verify:

- [ ] `target/debug/flowmark` exists.
- [ ] The build used `Cargo.lock` without changing it.
- [ ] No Python formatter was installed or invoked to create expected output.

The conformance and tryscript harnesses must exercise the Cargo-built binary, not a
global `flowmark` command.

## 3. Run the Shared Contract

```bash
cargo test --locked --test test_conformance -- --nocapture
cargo test --locked --test test_tryscript_golden -- --nocapture
```

Verify:

- [ ] The native adapter loads schema version 1 and validates every referenced path.
- [ ] Every selected case either passes or has an exact current entry in
  `tests/parity_corpus_known_divergences.toml`.
- [ ] An unlisted mismatch fails.
- [ ] A stale known-divergence entry fails.
- [ ] The upstream tryscript documents run from isolated temporary copies.
- [ ] No test silently skips because Python, Node.js, or another executable is absent.

Record selected change IDs and case IDs in the dated sync artifact.
Do not use only a total test count; generated and parameterized cases make totals an
unstable proxy for coverage.

## 4. Run the Rust and Administrative Gates

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo test --locked --no-default-features
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features

cd python
uv run ruff check .
uv run basedpyright
uv run pytest -q
uv run flowmark-dev check-mapping
cd ..
```

Verify:

- [ ] All commands exit zero.
- [ ] No required test is ignored or quarantined without a bead.
- [ ] Rust-only tests remain focused on Rust-specific helpers and boundaries.
- [ ] The supplemental function map has no broken Rust references or unexplained missing
  entries for its declared baseline.

## 5. Review the Affected Golden Surface

Before accepting an upstream expectation change, run its exact case IDs in the Python
repository without write mode and inspect the complete diff.
Use the upstream runner’s documented selector and write commands; never edit expected
output to make Rust pass.

For every changed case, verify:

- [ ] Input, arguments, environment, exit status, stdout, stderr, and file-tree state
  are intentional.
- [ ] Meaningful Markdown whitespace is not normalized away.
- [ ] The case is small enough to diagnose or is deliberately a broad reference/corpus
  layer.
- [ ] The case has a stable `FM-*` change ID and bead owner.
- [ ] The new expectation reaches a fixed point when idempotence is part of the
  contract.
- [ ] The broader affected tag, reference document, CommonMark layer, and tryscript
  workflow still pass.

For math changes, inspect at least inline and block delimiters, escaped dollars,
currency, code precedence, links/images, tables, blockquotes, list indentation,
frontmatter, HTML containers, line endings, adjacent delimiters, unmatched markers, and
sibling containers with the same apparent indentation.
Use an observable transform in container-boundary cases so accidental over-protection
cannot look like a pass.

## 6. Audit a New Whole-Program Baseline

This phase is required when the declared Python baseline advances.
It is not required for an ordinary Rust-only change against an unchanged pinned
contract.

Run both pinned implementations over the same isolated real-world and syntactic-surface
corpora. `scripts/corpus-parity-check.sh` is the transition-audit helper:

```bash
cargo build --locked --release

FLOWMARK_PARITY_PYTHON_BIN=/absolute/path/to/flowmark \
FLOWMARK_PARITY_PYTHON_LABEL='flowmark <full-commit>' \
FLOWMARK_PARITY_EXPECTED_CORPUS_SHA256='<corpus-digest>' \
FLOWMARK_PARITY_REPORT_DIR='target/corpus-parity/<run-id>' \
scripts/corpus-parity-check.sh /absolute/path/to/corpus target/release/flowmark
```

Verify:

- [ ] The Python command resolves the exact proposed baseline, never `latest`.
- [ ] The corpus source, immutable source commit or reconstruction limit, file count,
  and digest are recorded in `docs/test-corpora.md` and the dated sync artifact.
- [ ] Both binaries select the same list, and that list contains every Markdown file in
  the corpus.
- [ ] The full diff is retained and reviewed without truncation.
- [ ] Every difference is classified as port defect, source defect, dependency/platform
  behavior, nondeterminism outside the contract, or approved intentional divergence.
- [ ] Each real class receives a minimal shared case before its fix.
- [ ] Current Rust main is merged (`fm-mfvi`) and the v0.7.2-to-v0.7.3 inventory bead
  `fm-t81l` is complete before a new broad parity claim.

The transition audit may require Python.
The normal Rust conformance suite must not.

## 7. Verify Packaged Behavior

```bash
cargo package --locked --allow-dirty
```

Extract the generated crate into a temporary directory and run its built binary for the
surfaces changed by this sync.
At minimum verify `--help`, `--version`, `--docs`, `--skill`, stdin formatting, and one
isolated file operation when those features are in scope.

Verify:

- [ ] Bundled documentation and skill resources are present.
- [ ] Packaged CLI output does not depend on files under `repos/`.
- [ ] The crate excludes development-only shared fixtures and submodules.
- [ ] Package creation does not modify tracked files.

## 8. Confirm Documentation and Traceability

```bash
rg -n "upstream_commit|schema_version|FM-" \
  admin/port-coverage-mapping/shared-conformance.toml \
  docs/port-status.md \
  docs/sync-artifacts/2026-08-26-sync-0d2bebb-to-e9d5805.md
git diff --check
```

Verify:

- [ ] The commit, schema, change IDs, Python beads, Rust beads, and statuses agree.
- [ ] Every ledger entry names an active case and tracker.
- [ ] The dated checklist contains each command and result.
- [ ] README generation is stable if shared user documentation changed.
- [ ] No document claims whole-program parity beyond the evidence.

## 9. Clean-Clone Gate

Before remote CI or release, test from a clean clone or equivalent fresh worktree:

```bash
git submodule update --init --recursive
cargo test --locked --all-features
```

Verify:

- [ ] No local sibling repository or Git alternate is required.
- [ ] No untracked fixture copy is required.
- [ ] No Python runtime is required for the Rust behavioral suite.
- [ ] The same exact upstream commit and expected bytes are used.

## Result Template

```text
Result: PASS | FAIL | BLOCKED
Rust commit: <full SHA>
Python contract commit: <full SHA>
Playbook commit: <full SHA>
Shared schema: <integer>
Change IDs: <IDs>
Passing evidence: <commands and selected cases>
Known divergences: <exact case IDs or ledger reference>
New divergences: <none or trackers>
Blockers: <none or bead IDs>
Manual golden review: <reviewer and scope>
```

A run is **PASS** only when every required phase succeeds and the report has no
unexplained differences.
An unavailable pinned gitlink is **BLOCKED**, even if a local checkout can be repaired
from a sibling repository.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
