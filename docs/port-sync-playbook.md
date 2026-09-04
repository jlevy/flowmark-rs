# Port Sync Playbook

> **Doc status:** Rust port-specific (no upstream equivalent).

This is the Flowmark-specific operating procedure for keeping the Rust port aligned with
Python Flowmark. It adapts the canonical
[Python-to-Rust update checklist](../repos/rust-porting-playbook/playbooks/port-checklist-update-template.md)
and
[sync-release workflow](../repos/rust-porting-playbook/playbooks/python-to-rust-sync-release-workflow.md)
to Flowmark’s shared, language-neutral test system.

Use the canonical playbook for general porting policy.
Use this document for the exact repositories, evidence, commands, and acceptance rules
used by Flowmark.

## Contract and Authority

The port has two distinct upstream references.
Do not collapse them into one version claim:

- `Cargo.toml` `[package.metadata.parity].version` is the last released Python baseline
  whose whole-program port was declared complete.
- `admin/port-coverage-mapping/shared-conformance.toml` pins the exact in-progress
  upstream commit and shared schema used by the current porting cycle.

The behavioral evidence is ordered as follows:

1. The reviewed cases and expected bytes in the pinned `repos/flowmark` submodule.
2. The upstream tryscript documents executed against the Cargo-built Rust binary.
3. The language-neutral conformance manifest executed by the native Rust adapter.
4. The exact, bidirectional known-divergence ledger.
5. Focused Rust tests for Rust-specific code and failure boundaries.
6. The legacy Python-to-Rust function map as supplementary provenance.

Test counts, function names, and a passing Rust unit suite are not parity evidence by
themselves. A claim must name the upstream commit, schema, selected case IDs, and
divergence disposition.

## Why Flowmark Uses Shared Fixtures Directly

The canonical playbook permits copied fixtures when source and port cannot consume one
authoritative copy. Flowmark does not have that limitation.
The Python repository owns the versioned inputs, expected stdout and stderr, exit codes,
file trees, and tryscript documents.
Rust reads those assets directly from its pinned submodule.

This is a deliberate Flowmark adaptation:

- There is one reviewed golden byte stream, not a Python copy and a Rust copy.
- A submodule update exposes every fixture change in the gitlink and upstream diff.
- Both implementations can execute the same case IDs and selectors.
- The Rust adapter remains idiomatic Rust and tests the built executable contract.
- Ordinary Rust CI needs Git, Rust, Node.js, and pinned tryscript, but no Python
  runtime.

Python still runs the same shared cases in the upstream repository.
A pinned cross-binary differential sweep is useful when accepting a new baseline or
investigating a discrepancy, but it is an audit layer, not the portable Rust test
contract.

Do not restore `tests/testdocs`, `tests/parity`, or `tests/tryscript` copies in the Rust
repository. Do not generate Rust expectations from the Rust binary.
Do not make normal Rust tests invoke Python.

## Repository Map

| Concern | Authoritative location |
| --- | --- |
| Pinned Python source and shared assets | `repos/flowmark` |
| Pinned porting process | `repos/rust-porting-playbook` |
| Shared manifest | `repos/flowmark/tests/parity_corpus/manifest.toml` |
| Shared CLI sessions | `repos/flowmark/tests/tryscript/*.tryscript.md` |
| Native Rust conformance adapter | `tests/support/conformance.rs` |
| Rust conformance entry tests | `tests/test_conformance.rs` |
| Exact known differences | `tests/parity_corpus_known_divergences.toml` |
| Commit and change-ID traceability | `admin/port-coverage-mapping/shared-conformance.toml` |
| Supplemental function mapping | `admin/port-coverage-mapping/*.yaml` |
| Corpus ownership and provenance | `docs/test-corpora.md` |
| Current execution checklist | `docs/project/specs/active/port-checklist-update-2026-08-25.md` |
| Current sync evidence | `docs/sync-artifacts/2026-08-26-sync-0d2bebb-to-e9d5805.md` |

## Start Every Sync Cleanly

Initialize all submodules before inspecting or testing the port:

```bash
git submodule update --init --recursive
git submodule status
git status --short --branch
```

All recorded gitlinks must be obtainable from their configured remotes before a Rust PR
is ready for remote CI. A local-only upstream commit is acceptable while developing, but
it is an explicit publication dependency.
A successful exact-SHA fetch from a checkout that already contains the object is not
evidence of remote availability because Git can satisfy it locally.
Use a fresh recursive clone or an empty object store for the publication gate.
For local recovery only, fetch the exact commit from the sibling source checkout; never
silently substitute an older commit:

```bash
git -C repos/flowmark fetch /absolute/path/to/flowmark <exact-commit>
git -C repos/flowmark switch --detach <exact-commit>
```

Read these current playbook files before changing the port:

- [Porting principles](../repos/rust-porting-playbook/guidelines/porting-principles-and-antipatterns.md)
- [Python-to-Rust porting rules](../repos/rust-porting-playbook/guidelines/python-to-rust-porting-rules.md)
- [Test coverage for porting](../repos/rust-porting-playbook/guidelines/test-coverage-for-porting.md)
- [Rust testing rules](../repos/rust-porting-playbook/guidelines/rust-testing-rules.md)
- [Update checklist](../repos/rust-porting-playbook/playbooks/port-checklist-update-template.md)

## Choose the Release Mode

Use Mode A when the released Python parity version and pinned upstream commit do not
change. Use Mode B whenever either reference changes.

For a branch target newer than the last released Python baseline, Mode B remains open
until both of these tracks are complete:

1. Every intervening released behavior change is ported or has an approved disposition.
2. Every shared branch change ID has matching Rust evidence or a tracked deferral.

A green subset of the preservation corpus does not close an older baseline gap.

## Mode B Workflow

### 1. Record Baseline and Target

Record all of the following in a dated sync artifact before coding:

- released Python baseline version and commit;
- exact target version or branch commit;
- old and new Python submodule gitlinks;
- old and new rust-porting-playbook gitlinks;
- shared manifest schema and change IDs;
- known divergences at the start of the run.

Use an immutable commit, not a moving branch name, in committed evidence.

### 2. Inventory the Upstream Delta

Run the full baseline-to-target inspection without truncating its evidence:

```bash
git -C repos/flowmark log --reverse --oneline <baseline>..<target>
git -C repos/flowmark diff --name-status <baseline>..<target>
git -C repos/flowmark diff --stat <baseline>..<target>
```

Classify behavior, CLI, public API, tests, dependencies, generated documentation, and
refactors separately.
For the preservation program, map portable behavior to stable `FM-*` change IDs.
Keep unrelated release debt in a separate track so it cannot hide behind the new shared
suite.

### 3. Perform Empirical Pre-Port Triage

Build the existing Rust executable once, before implementing a behavior change, and run
the new shared cases against it.
Record one disposition per change ID:

- Rust already passes; tests and traceability only.
- Rust code change required.
- Known divergence with a bead and exact failing case IDs.
- Deferred upstream case with an owner and unblock condition.

The replacement Markdown parser can differ in either direction.
Passing a new case does not prove the surrounding syntax class; run the whole relevant
tag or change-ID selection.
Record the exact pass/fail matrix before changing code.
If a source unit invariant can affect CLI bytes, promote it to a shared discriminating
case during review rather than leaving it visible to only one language.

### 4. Port Shared Behavior Tests First

Portable behavior belongs in the upstream manifest or tryscript documents.
A shared case must specify exact observable behavior:

- argument vector and environment;
- stdin or isolated before-tree;
- stdout and stderr bytes;
- exit status;
- resulting filesystem tree and file bytes where applicable;
- idempotence when it is part of the contract;
- stable case ID, change ID, tags, and description.

Use small cases for syntax classes and broad documents for interaction coverage.
The layered Flowmark corpus intentionally overlaps:

- focused custom preservation and CLI cases;
- historical parity corner cases;
- the large reference documents;
- CommonMark 0.31.2 examples;
- end-to-end tryscript workflows.

That overlap is useful.
It catches local defects and whole-document interactions while keeping each portable
expectation language-neutral.

Container-sensitive cases must distinguish logical container identity from indentation.
At minimum, cover continuations and siblings separately: two list items can have the
same depth and content column but must not share an unmatched delimiter.
Prefer an observable transform inside the candidate span so accidental over-protection
cannot pass merely because the input was already a fixed point.

### 5. Implement Idiomatic Rust

Port behavior, not Python implementation structure.
Use focused Rust unit tests for language-specific adapters, parsing helpers, timeout
handling, path safety, and error classification.
Do not duplicate a shared golden assertion as a Rust string literal.

For opaque Markdown, keep recognition before the destination parser and keep the parser
bridge separate from source semantics.
A parser-specific AST shape may require scaffolding, but the shared scanner owns region
selection and the side table owns exact restoration.
Do not use parser recognition as an implicit dialect configuration layer.

For every shared change ID, update `admin/port-coverage-mapping/shared-conformance.toml`
with:

- Python bead;
- Rust bead;
- current status;
- exact pinned upstream commit at the file level.

If a shared case fails, add its exact ID to `tests/parity_corpus_known_divergences.toml`
with a live Rust tracker.
The native runner must fail when an unlisted difference appears, a ledger entry points
to a missing or deferred case, or a listed difference starts passing and becomes stale.

### 6. Review Golden Changes

Expected-output changes are contract changes.
Review them in the Python repository before advancing the Rust gitlink:

1. Run the selected cases without write mode and inspect the complete diff.
2. Confirm the new bytes represent intended Python behavior.
3. Accept only explicit case IDs with the upstream runner’s write mode.
4. Re-run the selected cases and the broader affected layer.
5. Commit the expectation and implementation together, or document why the expectation
   intentionally precedes the port.

Never bulk-accept snapshots, accept in CI, edit expected bytes to fit Rust, or normalize
meaningful Markdown whitespace.

### 7. Refresh Supplementary Mapping

The legacy YAML map remains useful for Python-specific unit and API coverage.
Refresh it when the target baseline changes, but do not use raw counts as an acceptance
criterion:

```bash
cd python
uv run flowmark-dev discover-python --ref v0.8.0
uv run flowmark-dev discover-rust
uv run flowmark-dev init-mapping
uv run flowmark-dev check-mapping
cd ..
```

Use the exact proposed released tag, not the in-progress contract gitlink.
A local path is valid only when that checkout is detached at the same released baseline.

Map a Python test to the shared integration/golden contract when that is the real Rust
counterpart. Use small Rust tests for code that cannot be shared.
Every exclusion or blocked mapping needs a current rationale and bead.

### 8. Validate the Built Port

Run the portable gates from a clean checkout with initialized submodules:

```bash
cargo fmt --all -- --check
cargo build --locked --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo test --locked --no-default-features
cargo doc --locked --no-deps

cd python
uv run ruff check .
uv run basedpyright
uv run pytest -q
uv run flowmark-dev check-mapping
cd ..
```

The `python/` commands validate Rust-repository administration tooling; they do not run
the Python formatter as the behavioral oracle.
CI must also install the repository’s pinned tryscript version and run the Cargo-built
binary against the upstream tryscript documents.

Before claiming a new whole-program baseline, also run a pinned differential corpus
sweep and a syntactic class sweep.
Capture the complete diff.
Convert every new difference into a shared discriminating case and disposition before
fixing it.

External corpora are transition evidence, not portable truth sources.
Require an explicit corpus path, immutable Python label, aggregate corpus digest, equal
selected file lists, and complete retained output diff.
The audit must fail if either binary skips a Markdown file.
Record source provenance or reconstruction limits in
[`test-corpora.md`](test-corpora.md) and the dated sync artifact.

Use the repository helper for that gate:

```bash
cargo build --locked --release

FLOWMARK_PARITY_PYTHON_BIN=/absolute/path/to/flowmark \
FLOWMARK_PARITY_PYTHON_LABEL='flowmark <full-commit>' \
FLOWMARK_PARITY_EXPECTED_CORPUS_SHA256='<corpus-digest>' \
FLOWMARK_PARITY_REPORT_DIR='target/corpus-parity/<run-id>' \
scripts/corpus-parity-check.sh /absolute/path/to/corpus target/release/flowmark
```

### 9. Finish the Record

Update the dated checklist, sync artifact, `docs/port-status.md`, change-ID map,
divergence ledger, and affected beads.
Record every command and result.
Re-run any generator that embeds upstream content, then verify the generated result is
stable.

When senior review discovers a portable corner case after the first green run, use this
closure loop:

1. Add a discriminating shared case at the source contract.
2. Prove the source implementation passes it.
3. Re-run it against the current port; do not hand-copy the expected bytes.
4. Fix the port and rerun the complete affected change ID.
5. Advance the immutable source gitlink and every traceability record together.

Do not mark a sync complete while:

- the target upstream gitlink is unavailable to a clean clone;
- a baseline-to-target behavior change is unclassified;
- a changed upstream test is unmapped;
- a shared case has an untracked result;
- a known-divergence entry is stale or vague;
- release metadata claims a broader parity surface than the evidence proves.

## Updating the Rust Porting Playbook

Advance the playbook submodule as its own reviewed commit:

```bash
git -C repos/rust-porting-playbook fetch --prune origin
git -C repos/rust-porting-playbook switch --detach <reviewed-commit>
git diff --submodule=log -- repos/rust-porting-playbook
```

Read the changed workflow and guideline files, record Flowmark-specific adaptations in
the current checklist, and run link and documentation checks.
Do not edit the submodule from the Flowmark-rs parent unless the work is intentionally
being contributed to the playbook repository on its own branch.

## Markdown Documentation

Run Flowmark only on documentation, never on golden Markdown fixtures:

```bash
uvx --from flowmark==0.8.0 flowmark --auto \
  --extend-exclude "tests/" \
  --extend-exclude "repos/" \
  --extend-exclude "README.md" \
  docs/ CONTRIBUTING.md
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
