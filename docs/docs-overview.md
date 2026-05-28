# Docs Overview

> **Doc status:** Rust port-specific (no upstream equivalent).
> Indexes the Rust port docs and shows each one’s relationship to the upstream Python
> project so the mirroring boundary is explicit.

Every doc in this repo carries a `Doc status:` callout near the top stating its
relationship to upstream Python flowmark.
The three categories:

| Status | Meaning |
| --- | --- |
| **Mirrored** | Content comes from upstream and is regenerated locally. Currently: `README.md` only (via [`docs/shared/flowmark-readme-shared.md`](https://github.com/jlevy/flowmark/blob/main/docs/shared/flowmark-readme-shared.md) and the wrapper template, with a small perspective transform in `scripts/generate_rust_readme.py`). |
| **Parallel** | Same role as the upstream doc of the same name; content differs because the toolchain (Rust cargo + multi-channel release vs Python uv + PyPI) requires different commands. |
| **Rust port-specific** | No upstream equivalent. Documents the port lifecycle (parity, sync, history) or a Rust-only feature. |

## Doc index

### Top-level

| File | Status | Notes |
| --- | --- | --- |
| [`README.md`](../README.md) | Mirrored | Generated. Do not edit directly. Run `scripts/generate_rust_readme.py` to regenerate from the wrapper template + the shared upstream body. |
| [`CHANGELOG.md`](../CHANGELOG.md) | Rust port-specific | Rust port release history (independent semver). |
| [`CONTRIBUTING.md`](../CONTRIBUTING.md) | Rust port-specific | Onboarding for Rust port contributors. |

### `docs/` (development, release, port)

| File | Status | Notes |
| --- | --- | --- |
| [`development.md`](development.md) | Parallel | Build/test/lint workflow for Rust. Mirrors the role of [upstream `development.md`](https://github.com/jlevy/flowmark/blob/main/docs/development.md). |
| [`publishing.md`](publishing.md) | Parallel | Multi-channel release runbook (crates.io + PyPI + Homebrew + GitHub Releases). Upstream’s [`publishing.md`](https://github.com/jlevy/flowmark/blob/main/docs/publishing.md) covers PyPI only. |
| [`cache.md`](cache.md) | Rust port-specific | Incremental cache, a Rust-only feature. |
| [`rust-only-features.md`](rust-only-features.md) | Rust port-specific | The (currently small) set of features the Rust port has that upstream does not. |
| [`port-status.md`](port-status.md) | Rust port-specific | Current parity target, release status, tolerated variations, and porting principles compliance. |
| [`port-sync-playbook.md`](port-sync-playbook.md) | Rust port-specific | How to sync flowmark-rs with upstream Python releases (Mode B workflow). |
| [`porting-log-review.md`](porting-log-review.md) | Rust port-specific | Historical review log from the initial port. |
| [`templates/rust-readme-wrapper.md`](templates/rust-readme-wrapper.md) | Rust port-specific | Wrapper template the README generator splices around `{{ shared_docs_body }}`. |

### `docs/project/`

| Folder | Status | Notes |
| --- | --- | --- |
| `specs/active/` | Rust port-specific | In-progress feature/sync plans for the Rust port. |
| `specs/done/` | Rust port-specific | Archive of completed specs (parity discrepancies, sync releases). |
| `sync-artifacts/` | Rust port-specific | Per-sync artifacts capturing baseline → target diff triage, validation, and any intentional divergences. |

### `repos/` (submodules)

| Submodule | Role |
| --- | --- |
| [`repos/flowmark/`](https://github.com/jlevy/flowmark) | Upstream Python flowmark source, pinned to the parity-target release tag (currently `v0.7.0`). |
| [`repos/rust-porting-playbook/`](https://github.com/jlevy/rust-porting-playbook) | Reusable Python-to-Rust porting methodology, guidelines, and workflows. |

## What is mirrored from upstream

Exactly two pieces of upstream content flow into the Rust port:

1. **README body** comes from
   [`docs/shared/flowmark-readme-shared.md`](https://github.com/jlevy/flowmark/blob/main/docs/shared/flowmark-readme-shared.md)
   via the README generator, with a small perspective transform in
   `scripts/generate_rust_readme.py` so phrasing like “this Python reference
   implementation” reads correctly from the Rust repo’s perspective.
2. **Test fixtures** at `tests/testdocs/testdoc.orig.md` and
   `tests/testdocs/testdoc.expected.*.md` come from upstream’s
   [`tests/testdocs/`](https://github.com/jlevy/flowmark/tree/main/tests/testdocs) and
   are refreshed on every Mode B sync.

Everything else in this repo is Rust-port-specific.

## What is *not* mirrored (and why)

- **`development.md`, `publishing.md`:** covered by Parallel status above.
  The Rust toolchain (cargo, crates.io, maturin/PyPI, Homebrew) is different enough from
  Python (uv, PyPI) that mirroring would produce a doc full of “for Rust, ignore the
  previous five lines and do this instead” interjections.
  Cheaper for readers to have two short language-native docs that link to each other.
- **`AGENTS.md`, `CLAUDE.md`** (upstream has both), the Rust port uses Claude Code
  project-level settings under `.claude/` and tbd skill hooks instead of these files.
  Same purpose, different mechanism.
- **`SUPPLY-CHAIN-SECURITY.md`** (upstream has one), the Rust port’s supply-chain
  posture is enforced by `deny.toml` and the CI `Dependency audit` +
  `Semver compatibility` jobs; a parallel narrative doc could be useful and is tracked
  as possible future work.

## Keeping the mirror clean

When editing a doc, preserve its `Doc status:` callout.
If a doc’s relationship to upstream genuinely changes (new mirror, role split, etc.),
update the callout and this index together so the delineation stays accurate.
