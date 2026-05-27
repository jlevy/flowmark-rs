---
title: Auto-port flowmark Python v0.6.4 → v0.6.5
description: Sync flowmark-rs to upstream Python flowmark v0.6.5
author: Claude Code (with Joshua Levy)
---
# Feature: Auto-port flowmark Python v0.6.4 → v0.6.5

**Date:** 2026-05-07 (last updated 2026-05-07)

**Author:** Claude Code (auto-port agent)

**Status:** Draft

## Overview

Mode B upstream sync: bump the Python parity baseline from v0.6.4 to v0.6.5,
incorporate any behavior changes, port all new tests, and refresh fixtures, mapping
files, and metadata so flowmark-rs ships at full parity with the latest released Python
flowmark.

## Goals

- Pin `repos/flowmark` submodule to `v0.6.5`.
- Update parity metadata (`Cargo.toml`, `.github/workflows/ci.yml`,
  `python/src/flowmark_dev_tools/cli.py` `DEFAULT_REF`, `README.md`,
  `docs/port-status.md`).
- Port every behavior change from `v0.6.4..v0.6.5` if comrak doesn't already cover it.
- Port every new Python test (1:1 mapping or explicit `excluded` rationale).
- Refresh `admin/port-coverage-mapping/*.yaml` and `python/tests/test_smoke.py` counts.
- Refresh fixtures and the generated Rust README.
- Validate cross-binary parity against the user's reported diff churn examples.
- Produce a sync artifact at `docs/sync-artifacts/2026-05-07-sync-v0.6.4-to-v0.6.5.md`.
- All gates green: `cargo fmt`, `cargo clippy`, `cargo test --all-features`,
  `pytest python/tests/test_smoke.py`, `flowmark-dev check-mapping`,
  `scripts/corpus-parity-check.sh`, `FLOWMARK_PARITY_PYTHON=1` cross-binary tests.
- Improve the port-sync docs and (if applicable) the rust-porting-playbook submodule
  based on rough edges found during this run.

## Non-Goals

- Cutting a Rust release. This spec lands the sync; the publishing playbook
  (`docs/publishing.md`) handles release.
- Re-architecting the sync tooling. Doc fixes and small ergonomic improvements only.
- Changes to comrak or its workarounds beyond what's required for v0.6.5.

## Background

Python flowmark released v0.6.5 with the following commit set vs v0.6.4 (10 commits):

| Commit | Type | Title |
| --- | --- | --- |
| `badab00` | docs | Add badges to README |
| `443861f` | **fix** | GFM punctuation flanking rules to prevent tilde doubling near parens |
| `9fda859` | **feat (cli)** | align help footer with Rust and add agent guidance |
| `a49c77c` | docs | split shared body and generate python readme from wrapper |
| `d866c08` | docs | sync generated README and add `flowmark-py` alias |
| `8b78649` | docs | callout styling on python readme header |
| `0749a05` | test | make packaging entrypoint import py310-safe |
| `c24bb50` | build | run flowmark formatting in default `make` target (Python) |
| `f72e854` | docs (skill) | remove uvx-specific run-on-save note |
| `f122829` | merge | PR #40 (merge commit) |

### Triage summary

| Change | Rust porting impact |
| --- | --- |
| Tilde GFM punctuation flanking fix (Python regex/find override) | Behavior already correct in Rust because comrak implements full GFM flanking. Verified against all 10 new fixtures via `./target/release/flowmark`. **Port only the tests.** |
| CLI help-footer cleanup (Python aligns to Rust) | Rust already emits the new footer style. **Port only the tests** (adapt for clap's help layout). |
| `flowmark-py` console-script alias | Python packaging only. Rust already ships both `flowmark` and `flowmark-rs` binaries. Test maps cleanly via Cargo metadata read; port via Rust integration test or mark `excluded` with rationale. |
| `SKILL.md` VS Code/Cursor run-on-save section | Rust ships its own `SKILL.md` under `src/skills/` (or equivalent). Need to mirror the section. |
| Skill content tests (`test_skill.py`) | Port the two new sub-tests against Rust's `--skill`/`--docs` output. |
| Tryscript golden additions (help footer, `wc -l | tr -d ' '` fix) | Rust's tryscript suite is structured differently; verify equivalent coverage exists, add if not. |
| Python README split into shared+wrapper, `generate-python-readme.py` | Rust's `generate_rust_readme.py` already reads `repos/flowmark/docs/shared/flowmark-readme-shared.md`. Re-run README generation. |
| Misc Python-only docs/build changes | No Rust impact. |

### Cross-binary diff churn (from user)

When a user formats a doc with the Python binary, then with Rust, they observe diff
churn including:

- Smart-quote/curly-apostrophe differences in some contexts.
- Semantic-line-break placement at sentence boundaries.

This work item must reproduce these examples, decide whether each is a known
discrepancy (e.g., D-series), an artifact of running different binary versions, or a new
parity gap that requires a fix-or-document decision.

## Design

### Approach

Mode B per
[`repos/rust-porting-playbook/playbooks/python-to-rust-sync-release-workflow.md`](../../../../repos/rust-porting-playbook/playbooks/python-to-rust-sync-release-workflow.md):

1. Stage upstream submodule pointer + fixtures.
2. Triage commit-by-commit; produce sync artifact early.
3. Port behavior + tests, with comrak coverage verified empirically before skipping
   anything.
4. Re-run discovery + mapping; resolve every new `missing` entry.
5. Validate; commit; open draft PR.

### Components

- `repos/flowmark` submodule pointer: `v0.6.4` → `v0.6.5`.
- `Cargo.toml [package.metadata.parity].version`: `0.6.4` → `0.6.5`.
- `.github/workflows/ci.yml` `FLOWMARK_PY_VERSION`: `0.6.4` → `0.6.5`.
- `python/src/flowmark_dev_tools/cli.py` `DEFAULT_REF`: `v0.6.4` → `v0.6.5`.
- `README.md` last-sync line + `docs/port-status.md` parity target.
- `admin/port-coverage-mapping/{python-tests,rust-tests,test-mapping}.yaml` regenerated.
- `python/tests/test_smoke.py` `EXPECTED_PYTHON_TEST_COUNT` and `EXPECTED_RUST_TEST_COUNT`.
- `tests/testdocs/` golden files refreshed from `repos/flowmark/tests/testdocs/`.
- New Rust tests: `tests/test_strikethrough.rs` (10 GFM-flanking cases), `tests/test_cli_help.rs` (4 cases adapted for clap), `tests/test_packaging_entrypoints.rs` (1 case validating Cargo bin aliases) or excluded with rationale, `tests/test_skill.rs` (2 cases for VS Code/Cursor section).
- Rust `SKILL.md` content updated with VS Code/Cursor section.
- Rust README regenerated via `scripts/generate_rust_readme.py`.

### API Changes

None.
The public Rust API is unchanged.
Internal: only test additions and string-content tweaks to `SKILL.md`.

## Implementation Plan

Single phase (sync release).

- [x] **Triage and artifact.** Create `docs/sync-artifacts/2026-05-07-sync-v0.6.4-to-v0.6.5.md` with commit table, categorization, and Rust-impact decisions.
- [x] **Bump submodule.** `cd repos/flowmark && git checkout v0.6.5`.
- [x] **Refresh fixtures.** `cp repos/flowmark/tests/testdocs/{testdoc.orig.md,testdoc.expected.*.md} tests/testdocs/`.
- [x] **Bump metadata.** `Cargo.toml`, `.github/workflows/ci.yml`, `python/src/flowmark_dev_tools/cli.py`, `README.md` last-sync, `docs/port-status.md`.
- [x] **Empirical comrak verification.** Run all 10 new strikethrough cases through `./target/release/flowmark`; record exact outputs in the sync artifact.
- [x] **Port new tests.** Rust counterparts for: 10 strikethrough flanking, 4 CLI help, 1 packaging entrypoint (or excluded), 2 SKILL/docs, plus tryscript additions where applicable.
- [x] **Update Rust SKILL.md content** with the VS Code/Cursor run-on-save section.
- [x] **Re-run discovery + mapping.** `flowmark-dev discover-python --local-path ../repos/flowmark`, `flowmark-dev discover-rust`, `flowmark-dev init-mapping`, `flowmark-dev check-mapping` until exit 0.
- [x] **Update smoke counts.** `EXPECTED_PYTHON_TEST_COUNT`, `EXPECTED_RUST_TEST_COUNT`, `EXPECTED_MAPPING_COUNT` if changed.
- [x] **Regenerate Rust README** via `scripts/generate_rust_readme.py`.
- [x] **Reproduce user churn examples.** Capture exact Python vs Rust output for the user's TODO.md / runbook / performance-notes diffs; classify each.
- [x] **Validate locally.** `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --locked --all-features`, `FLOWMARK_PARITY_PYTHON=1 cargo test --locked --test test_parity_cross_binary`, `pytest python/tests/test_smoke.py -q`, `flowmark-dev check-mapping`, `scripts/generate-parity-golden.sh`, `cargo build --release`, `scripts/corpus-parity-check.sh`.
- [x] **Improve port-sync docs.** Fix script-name typo in playbook (`generate-rust-readme.py` → `generate_rust_readme.py`), surface submodule init reminder, add automated upstream-update check command if reasonable.
- [x] **Improve rust-porting-playbook submodule** if anything generalizable surfaces; prepare upstream PR.
- [x] **Commit and push** on `claude/review-python-porting-0i29L`; open draft PR.

## Testing Strategy

- **Behavior parity:** all 10 new strikethrough cases produce byte-exact same output in
  both binaries.
- **CLI parity:** `flowmark --help` output diff vs Python is within tolerated variations
  (clap layout differences only, per `docs/port-status.md`).
- **Test parity:** `flowmark-dev check-mapping` reports zero `missing`. Smoke counts
  updated and stable.
- **Corpus parity:** `scripts/corpus-parity-check.sh` zero diffs.
- **Cross-binary parity:** `FLOWMARK_PARITY_PYTHON=1 cargo test ... test_parity_cross_binary`
  passes against installed Python flowmark v0.6.5.
- **Doc parity:** Rust README regeneration is byte-stable on subsequent runs.
- **Churn examples:** user's three diff snippets produce identical Python and Rust
  output once both binaries are at v0.6.5 / parity.

## Stabilization addendum (post-review)

Deep differential testing during review (full-corpus Python-vs-Rust diff, plus a
truth-table sweep of every reference-link form against Python v0.6.5 and `main`)
surfaced two genuine, previously-untested formatter parity gaps. Both are fixed in this
PR with discriminating tests on the Rust side and verification against Python:

- **D17 — thematic-break spacing (Rust-only bug).** comrak forced blank lines around
  `* * *`/`---`; Python (both v0.6.5 and `main`) preserves the source's tight spacing.
  Fixed by extending `render_block_children`'s tight-suppression rules. 5 tests.
- **D18 — reference-link normalization (upstream issue #45).** A reference link whose
  text equals its normalized label now renders as the unambiguous collapsed form
  `[text][]` instead of the fragile shortcut `[text]`; distinct labels render as the
  full form `[text][label]`. This adopts the upstream fix already released after v0.6.5
  (commit `0af9e24`) and is an **intentional, documented divergence from released
  v0.6.5** (see tolerated variations in `docs/port-status.md`). 10 tests + 2 unit tests,
  verified byte-for-byte against Python `main` (v0.6.6.dev).

Deferred to a future sync (feature-level, not stabilization): upstream `main`'s
atomic-aware semantic wrapping and table-row-adjacent-to-paragraph handling. Tracked
separately.

## Rollout Plan

- Land sync as a single PR on `claude/review-python-porting-0i29L`.
- Cutting the actual Rust release (`v0.2.7` or similar) is out of scope; it follows the
  publishing playbook in a separate PR.

## Open Questions

- For `test_packaging_entrypoints.py`: do we port as a Rust integration test that
  inspects `Cargo.toml` for the `[[bin]]` entries, or mark it `excluded` with rationale
  citing Rust's `[[bin]]` aliasing being structurally different from Python's
  `console_scripts`? **Tentative:** port it — the assertion (both names → same
  entrypoint) is meaningful in Rust too.
- The `wc -l | tr -d ' '` tweak in the Python tryscript is a macOS portability fix; do
  we want to mirror this preventively in Rust tryscript fixtures? **Tentative:** yes,
  cheap and consistent.

## References

- Sync runbook: [`docs/port-sync-playbook.md`](../../../port-sync-playbook.md)
- Mode B definition:
  [`repos/rust-porting-playbook/playbooks/python-to-rust-sync-release-workflow.md`](../../../../repos/rust-porting-playbook/playbooks/python-to-rust-sync-release-workflow.md)
- Update checklist:
  [`repos/rust-porting-playbook/playbooks/port-checklist-update-template.md`](../../../../repos/rust-porting-playbook/playbooks/port-checklist-update-template.md)
- Upstream commits range: `repos/flowmark` `v0.6.4..v0.6.5`
- Sync artifact (to be created):
  `docs/sync-artifacts/2026-05-07-sync-v0.6.4-to-v0.6.5.md`
