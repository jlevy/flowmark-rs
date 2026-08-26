# Docs Overview

> **Doc status:** Rust port-specific (no upstream equivalent).
> Indexes the Rust port docs and shows each one’s relationship to the upstream Python
> project so the mirroring boundary is explicit.

Every doc in this repo carries a `Doc status:` callout near the top stating its
relationship to upstream Python flowmark.
The three categories:

| Status | Meaning |
| --- | --- |
| **Mirrored** | Content comes from upstream and is regenerated or verified locally. Currently: the shared `README.md` body and the packaged runtime skill used by `--skill` / `--install-skill`. |
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
| [`test-corpora.md`](test-corpora.md) | Rust port-specific | Ownership, provenance, and execution rules for shared and external test corpora. |
| [`parity-coverage-matrix.md`](parity-coverage-matrix.md) | Rust port-specific | Per-AST-node × syntactic-form matrix mapped to the tests that prove parity for each row. Structural backstop against inheriting upstream test gaps. |
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
| [`repos/flowmark/`](https://github.com/jlevy/flowmark) | Upstream Python source and authoritative shared test assets, pinned to the exact in-progress contract commit. |
| [`repos/rust-porting-playbook/`](https://github.com/jlevy/rust-porting-playbook) | Reusable Python-to-Rust porting methodology, guidelines, and workflows. |

## What is mirrored from upstream

Three pieces of upstream content flow into the Rust port:

1. **README body** comes from
   [`docs/shared/flowmark-readme-shared.md`](https://github.com/jlevy/flowmark/blob/main/docs/shared/flowmark-readme-shared.md)
   via the README generator, with a small perspective transform in
   `scripts/generate_rust_readme.py` so phrasing like “this Python reference
   implementation” reads correctly from the Rust repo’s perspective.
2. **Runtime skill sources** at `src/skills/` mirror
   [`src/flowmark/skills/`](https://github.com/jlevy/flowmark/tree/main/src/flowmark/skills)
   byte-for-byte so both CLIs print and install the same bundle.
   The README generator checks this invariant against the pinned `repos/flowmark`
   submodule. Public discovery and `npx skills add` remain exclusively in
   `jlevy/flowmark`; this repository intentionally has no root `skills/flowmark/`
   bundle.
3. **Portable test assets** are consumed directly from the pinned `repos/flowmark`
   submodule. They include the language-neutral manifest, reference and topic documents,
   CommonMark registry, historical parity cases, and tryscript sessions.
   Rust does not keep a copied fixture tree or invoke Python in its normal behavior
   suite.

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
