# Feature: Shared parity corpus for flowmark + flowmark-rs

**Date:** 2026-05-28

**Author:** Senior review (Claude) with @jlevy direction

**Status:** Draft — proposal pending upstream agreement

**Beads:** epic `fmr-bh2b`; tasks `fmr-i66m` (upstream), `fmr-9yd0` (Rust runner), `fmr-krz6` (baseline triage). Supersedes `fmr-i17c`.

## Overview

Stand up a language-neutral parity test corpus that drives both `jlevy/flowmark` (Python upstream) and `jlevy/flowmark-rs` (Rust port) from a single source of truth. Every fixture is added once; both implementations are exercised against the *same* (input, expected) pair; divergences are tracked explicitly against a shrinking baseline file on the Rust side rather than hidden in two parallel test trees.

The corpus lives in upstream Python, vendored at `parity_corpus/`. Rust consumes it via the existing `repos/flowmark` submodule with zero copy-paste.

## Goals

- **Single source of truth.** A new parity case is authored once and gates CI in both implementations on the next test run, with no manual translation step.
- **Shared assertions.** Both runners assert `format(input) == expected` against identical inputs and identical expecteds. Whatever a runner finds wrong is genuinely a behavior gap, not a test-file drift artifact.
- **Visible divergences.** Every Python↔Rust output difference is either a passing case in the corpus or a line in the Rust-side `known_divergences.txt` with a tracker bead or rationale. There is no third bucket of "silently different."
- **One-way baseline shrinkage.** The known-divergences file is monotone decreasing: closing a divergence (case now matches) MUST also delete its baseline line, or the test fails. Stale baseline entries cannot persist.
- **Initial seed from CommonMark spec.** All 655 CommonMark 0.31.2 spec examples enter the corpus on day one, plus hand-curated flowmark-specific families (reference images, badge pattern, historical D1-D20 reproducers).
- **No copy-paste of tests.** This explicitly replaces the prior "mirror upstream Python tests into Rust" approach that produced the D19/D20 blind spots.

## Non-Goals

- **CommonMark conformance.** We are not asserting flowmark renders the spec's expected HTML — flowmark is a formatter, not a renderer. The spec's *inputs* drive the corpus; the *expecteds* are Python flowmark's outputs.
- **Replacing internal unit tests.** `tests/test_*.py` and `tests/test_*.rs` cover formatter internals (helpers, line wrapping, table handling) that the corpus can't see. They stay.
- **Generating expecteds at test time.** The Rust runner does not invoke Python at test time. Expecteds are pre-generated and committed; the Rust suite has no Python runtime dependency.
- **Mirroring tests across languages.** The Rust runner is not a translation of the Python runner. They are two thin adapters reading the same data.
- **Multi-version Python coverage.** The corpus is generated against one Python flowmark version at a time. Re-seeding on each upstream release is part of the workflow, not an at-test-time concern.

## Background

Three observations push toward this design:

1. **Three latent parity bugs in PR #59 all traced to one missing test surface.** Image-ref inlining (D19), the badge pattern, and ref-def label lowercasing (D20) were each invisible to both repos' test suites until manual inspection of the PR diff caught them. `grep "!\[" tests/*.py` in upstream Python returned only an alert-syntax false positive and one inline-image example — there was nothing to mirror. The Rust port faithfully inherited the gap.
2. **A scratch differential run on the CommonMark 0.31.2 spec produced 69/655 (~10.5%) divergences** between Python flowmark v0.7.0 and the Rust port, across ~12 sections (backslash escapes, block quotes, fenced code, entity refs, HTML blocks, indented code, link refs, lists, list items, tabs). The signal is real and currently invisible in CI.
3. **Two thin runners on a shared corpus is strictly less work than two parallel test files.** The existing approach (port Python tests by hand into Rust) has a per-fixture cost paid twice and a discovery cost paid late. A shared corpus pays the discovery cost once.

The full proposal — including alternatives considered (tryscript extension, JSON manifest, blind test mirroring), open questions on file layout and license, and a worked example of the runner code — is in the artifact reviewed alongside this spec.

## Design

### Approach

Corpus lives in upstream Python at `parity_corpus/`. Two thin runners (one Python, one Rust) iterate the same data. Rust gates additionally on a shrinking known-divergences baseline.

### Components

**Upstream Python (`jlevy/flowmark`):**

- `parity_corpus/spec/spec.txt` — vendored CommonMark 0.31.2 spec, CC-BY-SA 4.0, attribution in `parity_corpus/LICENSE-COMMONMARK`.
- `parity_corpus/cases/spec/cm-NNN/{input.md,expected.md}` — 655 cases auto-generated from `spec.txt`.
- `parity_corpus/cases/flowmark/<group>/<case>/{input.md,expected.md}` — hand-curated cases (reference-image forms, badge pattern, D1-D20 reproducers, families mirroring `flowmark-rs/tests/test_syntactic_surface.rs`).
- `parity_corpus/manifest.toml` — index: id, path, section, source per case.
- `parity_corpus/README.md` — format spec, runner contract, how to add a case, how to regenerate after a Python flowmark behavior change.
- `parity_corpus/scripts/seed_spec_cases.py` — re-runnable seed/regeneration script.
- `tests/test_parity_corpus.py` — parametrized pytest, ~20 LOC, picked up by `make test`.

**Rust (`jlevy/flowmark-rs`):**

- `tests/test_parity_corpus.rs` — ~120 LOC integration test that reads the corpus from `repos/flowmark/parity_corpus/`, iterates `manifest.toml`, and asserts per case.
- `tests/parity_corpus_known_divergences.txt` — initial baseline (~69 entries), each `<case_id>  # <rationale or tracker>`.
- Submodule bump on `repos/flowmark` to a commit containing the corpus.
- Cross-references in `docs/parity-coverage-matrix.md` (replace the "Planned" pointer to `fmr-i17c` with a live pointer to the runner) and `docs/port-status.md` (note the new gate).

### API / file format

Each case is a directory with exactly two files:

```
cases/<group>/<id>/
├── input.md       # markdown to format
└── expected.md    # Python flowmark's canonical output at corpus creation time
```

No per-case metadata files in the common path — the manifest carries structured fields. The two-file pair is chosen over JSON/TOML embedding so cases are individually viewable, diffable, and editable in git without escaping multi-line markdown content.

Manifest schema (`parity_corpus/manifest.toml`):

```toml
schema_version = "1"
description = "Language-neutral parity corpus for flowmark."

[[case]]
id = "cm-001"               # unique stable ID; spec cases are cm-NNN
path = "spec/cm-001"        # relative to cases/
section = "Tabs"            # spec section or hand-curated family
source = "CommonMark spec 0.31.2 example 1"
# description is optional; the section + source already locates the case
```

Rust-side baseline (`tests/parity_corpus_known_divergences.txt`):

```
cm-017  # inline-code backslash escapes; tracked as fmr-XXXX
cm-078  # heading spacing around interruption; tracked as fmr-XXXX
cm-119  # blockquoted fenced code drops > prefix; tracked as fmr-XXXX
…
```

Comments (`# …`) on each line carry the rationale and either a bead ID (for a tracked fix) or a permanent-divergence label (e.g. `# library: marko/comrak differ on …`). No entry without context.

### Runner contracts

**Python:** for each manifest entry, `flowmark_markdown()(input.md.read_text()) == expected.md.read_text()`. By construction every case passes — Python is the source of truth that generated the expecteds.

**Rust:** for each manifest entry, let `actual = flowmark_format(input)` and `expected = expected.md`. Two assertions per run, both directions:
1. **No new divergences.** For any case NOT in the baseline, `actual == expected` (else fail with the case ID).
2. **No stale baseline entries.** For any case IN the baseline, `actual != expected` (else fail prompting the contributor to delete that baseline line — the case has been fixed elsewhere).

Together these make the baseline monotone-decreasing: it can only shrink.

## Implementation Plan

### Phase 1: Upstream Python — corpus + runner + seed (`fmr-i66m`)

One PR on `jlevy/flowmark`.

- [ ] Vendor `parity_corpus/spec/spec.txt` (CC-BY-SA 4.0) + `parity_corpus/LICENSE-COMMONMARK` attribution; reference from main LICENSE listing.
- [ ] Write `parity_corpus/scripts/seed_spec_cases.py` — parses `spec.txt`, extracts 655 examples, writes `cases/spec/cm-NNN/input.md`, runs `flowmark_markdown()` to produce `expected.md`, appends to `manifest.toml`.
- [ ] Hand-curate `cases/flowmark/` covering: ref-image full/collapsed/shortcut/with-title/spaces-in-label/case-insensitive/empty-alt; badge full-ref/collapsed-ref/shortcut-ref/inline; D1-D20 historical reproducers.
- [ ] Write `parity_corpus/manifest.toml` (auto-generated section for spec cases, hand-edited section for flowmark cases).
- [ ] Write `parity_corpus/README.md` documenting the format, runner contract, contributor workflow.
- [ ] Add `tests/test_parity_corpus.py` (~20 LOC parametrized pytest). Auto-picked up by `pytest` and therefore `make test`.
- [ ] Verify `make test` passes; runtime increase <10 seconds.
- [ ] Mention the corpus in the project README as a contributor-visible artifact.

### Phase 2: Rust runner + baseline (`fmr-9yd0`)

One PR on `jlevy/flowmark-rs` after Phase 1 lands and a flowmark version including the corpus is published (or pin the submodule to a pre-release commit).

- [ ] Bump `repos/flowmark` submodule to a commit containing `parity_corpus/`.
- [ ] Add `toml` to dev-dependencies (verify it's not transitively present already).
- [ ] Write `tests/test_parity_corpus.rs` (~120 LOC) implementing the bidirectional baseline gate above.
- [ ] Seed `tests/parity_corpus_known_divergences.txt` — re-run the corpus locally, dump diverging case IDs, annotate each with rationale or tracker bead (some entries will need lookup against existing D-series beads).
- [ ] Update `docs/parity-coverage-matrix.md` — replace the planned-`fmr-i17c` pointer with a live pointer to the corpus runner. Keep the targeted matrix as the curated complement.
- [ ] Update `docs/port-status.md` to note the new gate.
- [ ] Verify all three CI platforms (ubuntu, macos, windows) pass.

### Phase 3: Baseline triage (`fmr-krz6`)

Separate PRs on `jlevy/flowmark-rs`, clustered by spec section. Out of scope for the initial two-PR landing; tracked but not blocking.

- [ ] For each cluster (backslash escapes, block quotes, fenced code, etc.): minimal repro → root cause → fix in Rust OR document as permanent library-level divergence OR file upstream as Python bug. Remove the fixed entries from the baseline in the same PR.
- [ ] Target residual baseline ≤20 entries within two quarterly syncs, all documented as permanent.

## Testing Strategy

**Phase 1 (Python):** `make test` runs the new parametrized test; all 655 + hand-curated cases pass by construction. Lint clean (ruff check, ruff format, codespell, basedpyright).

**Phase 2 (Rust):** `cargo test` runs the new integration test; all non-baselined cases pass, all baselined cases fail-by-design (asserting the divergence is still present). Test output explicitly lists new-divergence failures and stale-baseline failures separately so CI logs are actionable.

**Cross-validation during Phase 2 seed:** run the same corpus through both binaries locally, confirm the divergence count matches the scratch run (~69 ±5) — any large delta signals corpus drift between scratch run and Phase 1 PR.

**Regression-detection live test:** intentionally introduce a divergence locally (e.g. swap a small render branch), confirm the Rust runner fails with the specific case ID; revert. Similarly delete a baseline line locally, confirm the runner fails prompting baseline removal.

## Rollout Plan

1. **File the proposal upstream.** Open a feature-request issue on `jlevy/flowmark` referencing this spec and the proposal artifact. Iterate on open questions (corpus location, manifest format, spec-seed scope, license placement) in the issue.
2. **Land Phase 1 upstream.** One PR; goes through normal review. Tag a flowmark release (or have flowmark-rs pin to the merge commit) so the submodule has something to bump to.
3. **Land Phase 2 in flowmark-rs.** Submodule bump + runner + baseline. CI gate live.
4. **Cross-reference docs.** Update `docs/parity-coverage-matrix.md` and `docs/port-status.md` to point at the new gate.
5. **Phase 3 in parallel.** Cluster PRs shrinking the baseline; ongoing.

## Open Questions

- **Corpus location:** `parity_corpus/` at upstream repo root (what this spec assumes; more discoverable to Rust as a sibling artifact), or `tests/parity_corpus/` (groups with existing test infra)? Resolve in the upstream issue.
- **Manifest format:** TOML (stdlib `tomllib` on Python 3.11+) vs JSON (universal, less ergonomic for multi-line). TOML proposed.
- **Spec-seed scope:** all 655 examples or filter spec cases that are obviously HTML-output-focused? Suggest all 655 initially — extra noise is cheaper than missed coverage.
- **License surfacing:** the project is MIT; adding a CC-BY-SA file requires the addition be confined to `parity_corpus/` and clearly attributed. Confirm comfort with that arrangement.
- **Rust seeding cadence:** how often does the Rust side re-seed the baseline against a new Python flowmark release? Suggest: per sync (every `repos/flowmark` submodule bump runs the corpus and updates the baseline).
- **Format-mode coverage:** the corpus tests `flowmark_markdown()` defaults. Should we also corpus-test alternate modes (semantic, plaintext, width-60, tight/loose lists)? Suggest no for v1 — these are well-covered by `tests/test_parity_discrepancies.rs` and the tryscript golden tests. Revisit if a divergence is found in a non-default mode that the corpus would have caught.

## References

- Bead epic: `fmr-bh2b` — Shared parity corpus (Python + Rust, single source of truth)
- Bead tasks: `fmr-i66m` (upstream PR), `fmr-9yd0` (Rust runner), `fmr-krz6` (baseline triage)
- Superseded bead: `fmr-i17c` (original Rust-only spec gate)
- PR #59 — the parity-bugs PR whose review surfaced the structural gap
- Proposal artifact: `/tmp/parity-corpus-proposal.md` (review draft, to be filed as a `jlevy/flowmark` issue)
- `docs/parity-coverage-matrix.md` — current curated targeted matrix; will cross-reference the corpus
- `docs/port-sync-playbook.md` — auto-sync workflow that already bumps `repos/flowmark`
- `tests/test_syntactic_surface.rs` — the targeted-matrix backstop; complements (does not replace) the corpus
- CommonMark spec: https://spec.commonmark.org/0.31.2/
