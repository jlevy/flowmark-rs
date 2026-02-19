# Feature: Build, CI Hardening, and Publishing Improvements

**Date:** 2026-02-17 (last updated 2026-02-18)

**Author:** Joshua Levy

**Status:** In Progress

**Epic bead:** fmr-8yos

## Overview

flowmark-rs is approaching readiness for public release as a high-quality open source
Rust tool. This spec covers two areas: (1) hardening the CI pipeline to match
best-in-class Rust projects, and (2) setting up publishing infrastructure so the CLI is
conveniently installable via crates.io, Homebrew, and pre-built binaries.

The goal is an exemplary open source project where every operational detail — CI,
release automation, packaging, documentation — meets or exceeds the standard set by
popular Rust CLI tools like ripgrep, bat, and fd.

## Goals

- CI pipeline covers all best-practice checks with zero gaps.
- Library crate published on crates.io with proper metadata and trusted publishing.
- Pre-built binaries for Linux (x86_64, arm64), macOS (x86_64, arm64) via GitHub
  Releases.
- Shell installer for quick install on any Unix system (`curl | sh`).
- Automated release workflow triggered by version tags.
- Dependency updates automated via Dependabot.
- Code coverage tracked and visible.
- README and CONTRIBUTING docs ready for public consumption.

**Future (not in scope for this plan):**
- One-line install via Homebrew (`brew install jlevy/tap/flowmark`).
- Shell completions and man pages.

## Non-Goals

- Windows support (project uses `libc` with `cfg(unix)` — Unix-targeted by design).
- Feature work on the formatter itself (covered by the exact-parity spec).
- Performance benchmarking or optimization.
- Full changelog automation (can be added later with git-cliff/release-plz).

## Background

### Current State

The CI pipeline is already well above average:

| Practice | Status |
| --- | --- |
| `cargo fmt --check` | Present |
| `cargo clippy --locked` with pedantic deny | Present (source-level in Cargo.toml) |
| `unsafe_code = "deny"` | Present |
| `unwrap_used = "deny"` | Present |
| `RUSTFLAGS="-D warnings"` on test jobs | Present |
| `RUSTDOCFLAGS="-D warnings"` on docs job | Present |
| `cargo test --locked --all-features` | Present |
| `cargo doc --locked` | Present |
| Cross-platform testing (ubuntu + macOS) | Present |
| `--no-default-features` test job | Present |
| MSRV check (1.85) | Present |
| `cargo-deny` with deny.toml | Present |
| `Swatinem/rust-cache@v2` | Present |
| Release profile (LTO, strip, panic=abort) | Present |
| `CARGO_PROFILE_TEST_DEBUG: 0` | Present |
| Code coverage (`cargo-llvm-cov` + Codecov) | Present |
| `cargo-semver-checks` (PR-only) | Present |
| Dependabot (Cargo + GitHub Actions, weekly) | Present |

### Gaps Identified

| # | Gap | Priority | Bead | Status |
| --- | --- | --- | --- | --- |
| 1 | Missing `--locked` on clippy job | P1 | fmr-mk46 | **Done** |
| 2 | Missing `--locked` on docs job | P1 | fmr-9eda | **Done** |
| 3 | Missing `CARGO_PROFILE_TEST_DEBUG: 0` | P2 | fmr-b035 | **Done** |
| 4 | No code coverage (cargo-llvm-cov) | P2 | fmr-hj6z | **Done** |
| 5 | No cargo-semver-checks | P2 | fmr-8un1 | **Done** |
| 6 | No Dependabot config | P3 | fmr-zvbe | **Done** |
| 7 | No cargo-nextest | P3 | fmr-rj25 | Deferred |

### Publishing Gaps

- No release workflow (no binary builds, no GitHub Releases).
- No crates.io publishing automation.
- No Homebrew tap or formula.
- No root README.md.
- ~~Missing `readme` and `documentation` fields in Cargo.toml.~~ **Done**
- No CONTRIBUTING.md or CHANGELOG.md.
- No shell completions or man page generation.

## Design

### Approach

Use **cargo-dist** for binary release automation (the current community standard for
Rust CLI distribution).
This generates the release workflow, shell/PowerShell installers, and Homebrew formula
updates automatically.

For crates.io publishing, start with manual `cargo publish` for the first release, then
set up trusted publishing (OIDC) for subsequent releases.

Phases are ordered so quick CI fixes come first, then publishing infrastructure, then
polish.

## Implementation Plan

### Phase 1: CI Quick Fixes — DONE

Minimal-effort improvements to the existing CI pipeline.

- [x] Add `--locked` to clippy job
  (`cargo clippy --locked --all-targets --all-features`) (fmr-mk46)
- [x] Add `--locked` to docs job (`cargo doc --locked --no-deps --all-features`)
  (fmr-9eda)
- [x] Add `CARGO_PROFILE_TEST_DEBUG: 0` to global `env:` block (fmr-b035)

### Phase 2: CI Enhancements — DONE

Higher-impact CI additions.

- [x] Add code coverage job with `cargo-llvm-cov` and Codecov upload (fmr-hj6z)
- [x] Add `cargo-semver-checks` job for API breakage detection (fmr-8un1)
- [x] Add `.github/dependabot.yml` for weekly Cargo + GitHub Actions dependency updates
  (fmr-zvbe)
- [ ] Consider `cargo-nextest` for faster test execution (fmr-rj25) — deferred (P3)

### Phase 3: Crates.io Readiness — DONE

Prepare Cargo.toml metadata, write README, and verify publishability.

- [x] Add `readme = "README.md"` to `Cargo.toml` (line 8)
- [x] Add `documentation = "https://docs.rs/flowmark"` to `Cargo.toml` (line 10)
- [x] **Bump version to `0.2.0`** (fmr-xnxy) — `Cargo.toml` line 3: change
  `version = "0.1.0"` → `version = "0.2.0"`. Run `cargo check` to verify lockfile
  updates cleanly.
- [x] **Rename `package.metadata.python_source` to `package.metadata.parity`**
  (fmr-7cf1) — `Cargo.toml` lines 58-59: renamed section header and updated
  docs/port-sync-playbook.md references.
- [x] **Write root README.md** (fmr-swma) — Created `/README.md` with project
  description, installation, CLI usage, library usage example, and license.
- [x] **Verify `cargo publish --dry-run` succeeds** (fmr-6evz) — Updated
  `package.exclude` to trim crate to 70 files (234 KiB compressed).
  Dry-run passes.
- [x] **(Manual) Set up trusted publishing (OIDC) on crates.io** (fmr-db47) — Registered
  `jlevy/flowmark-rs` (Owner ID: 2058167), workflow `publish.yml` as trusted publisher.
- [ ] **(Manual) First publish** (fmr-tm0t) — After publish workflow is in place, create
  a GitHub Release tagged `v0.2.0` to trigger the automated publish.
  Or run `cargo publish` manually for the first time.
  Depends on: fmr-aarf (publish workflow), fmr-db47 (OIDC setup).

### Phase 4: Publish Workflow — DONE

Create `.github/workflows/publish.yml` for automated crates.io publishing, and write the
publishing docs.

- [x] **Create `.github/workflows/publish.yml`** (fmr-aarf) — Created with OIDC trusted
  publishing, test-before-publish safety check, and manual dispatch trigger.
- [x] **Write `docs/publishing.md`** (fmr-67o0) — Created with pre-release checklist,
  release instructions, OIDC setup guide, and troubleshooting.

### Phase 5: Binary Release Workflow — PENDING

Set up automated cross-platform binary builds via cargo-dist.

- [ ] **Run `cargo dist init` and configure** (fmr-t8qq) — Install cargo-dist
  (`cargo install cargo-dist`) and run `cargo dist init` to bootstrap configuration.
  This generates:
  - `[workspace.metadata.dist]` section in `Cargo.toml`
  - `.github/workflows/release.yml`
  - `dist-workspace.toml` (or equivalent config) Configure targets:
    `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-apple-darwin`,
    `aarch64-apple-darwin`. Enable `shell` installer (generates `curl | sh` one-liner).
    Homebrew installer deferred to future.
- [ ] **Review generated release workflow** (fmr-c1l6) — Review
  `.github/workflows/release.yml` for correctness.
  Depends on: fmr-t8qq.
  - Triggers on tag push (`v*`)
  - Builds binaries for all 4 targets
  - Creates GitHub Release with artifacts
  - Uploads tarballs containing: binary, LICENSE, README.md
- [ ] **Coordinate with publish workflow** (fmr-ttf7) — Ensure release creation triggers
  the publish workflow (Phase 4). Depends on: fmr-aarf, fmr-c1l6. The flow: push tag →
  release.yml builds binaries and creates GitHub Release → publish.yml triggers on
  release published → publishes to crates.io.
- [ ] **(Manual) Test with `v0.2.0` tag** (fmr-ya3n) — Push `v0.2.0` tag, verify release
  workflow creates artifacts, verify publish workflow publishes to crates.io.
  Depends on: fmr-ttf7.

### Phase 6: Documentation and Community — DONE

Standard open source project documentation.

- [x] **Write CONTRIBUTING.md** (fmr-nc8i) — Created `/CONTRIBUTING.md` with
  prerequisites, build/test/lint commands, PR guidelines, and link to publishing docs.
- [x] **Add CHANGELOG.md** (fmr-4v5g) — Created `/CHANGELOG.md` with v0.2.0 entry
  including parity version reference.
  Follows Keep a Changelog format.
- [x] **Update badges in README.md** (fmr-7ayu) — 5 badges: @ojoshe X follow, CI,
  crates.io, docs.rs, MSRV. Removed codecov (token not configured, badge showed
  "unknown").
- [x] **Add `--version` parity info** (fmr-19zr) — Created `build.rs` that reads
  `[package.metadata.parity]` version and emits `PARITY_VERSION` env var.
  `src/main.rs` uses `long_version` to display:
  `flowmark 0.2.0 (parity: flowmark-py 0.6.4)`.
- [x] **Verify `cargo doc` output** (fmr-ghvq) — Docs build cleanly with `-D warnings`.
  No broken links or missing documentation.

### Future: Homebrew Tap — DEFERRED

Not part of this plan.
Tracked for future work.

- [ ] Create `jlevy/homebrew-tap` repository on GitHub
- [ ] Configure cargo-dist to auto-update the tap formula on release
- [ ] Test `brew install jlevy/tap/flowmark` from a clean environment
- [ ] Add Homebrew install instructions to README

### Future: CLI Polish — DEFERRED

Not part of this plan.
Tracked for future work.
The CLI is self-documenting via `--help`, so these are nice-to-haves rather than
blockers.

- [ ] Add `clap_complete` for shell completion generation (bash, zsh, fish)
- [ ] Add `clap_mangen` for man page generation (or a build script approach)
- [ ] Include completions and man page in release artifacts

## Open Questions

1. ~~**Crate name availability**~~: **Resolved** — `flowmark` is already published on
   crates.io (v0.1.3, Nov 2025) under this repo.
   Next publish will be an update.
2. ~~**Version strategy**~~: **Resolved** — Use `0.2.0` as the first formal release
   (since `0.1.3` is already on crates.io from earlier work).
   Each release links to the Python version it targets for parity (see Version
   Convention below).
3. **cargo-dist vs manual release workflow**: cargo-dist is simpler but less flexible.
   For a project this size, cargo-dist is likely the right choice initially.
4. ~~**Shell completions scope**~~: **Deferred** — moved to future work (not blocking
   initial release).

## Version Convention

Each flowmark (Rust) release explicitly documents which Python flowmark version it
targets for behavioral parity.

**Format in GitHub Release notes, CHANGELOG, and crate description:**

> flowmark v0.2.0 (parity: flowmark-py v0.6.4)

**Where this appears:**
- GitHub Release title/body
- CHANGELOG.md entries
- `--version` output: `flowmark 0.2.0 (parity: flowmark-py 0.6.4)`
- Cargo.toml `description` or `package.metadata` (for reference, not displayed on
  crates.io)

**Rules:**
- The Rust version follows its own semver independently (0.2.0, 0.2.1, 0.3.0, ...).
- The Python parity version is informational — it says “this release matches the
  behavior of Python flowmark vX.Y.Z.”
- When the Rust version adds features beyond Python parity, the parity note still
  indicates which Python version’s behavior is fully covered.

## README Structure

Keep the README minimal.
Explain what this is, how to install it, and where to go for more.
Do not duplicate feature documentation from the Python project.

**Key points to convey:**

- This is a Rust port of [flowmark](https://github.com/jlevy/flowmark) (Python), a
  Markdown auto-formatter.
  Identical CLI, identical output.
- The port was automated and fully tested using the
  [rust-porting-playbook](https://github.com/jlevy/rust-porting-playbook).
- Link to the Python project for full documentation (features, CLI reference,
  configuration, IDE setup, agent use, etc.).

**Sections:**

1. **Project description** — 1-2 paragraphs: what it is, that it’s an automated
   fully-tested port, links to Python project and rust-porting-playbook
2. **Installation** — install methods:
   - `cargo install flowmark` (from crates.io)
   - Pre-built binaries from GitHub Releases
   - Homebrew (future)
3. **Performance** — brief comparison table (Rust vs Python wall-clock times on
   reference doc, measured with `hyperfine`). See exact-parity spec fmr-aq8o for
   benchmark methodology.
4. **Library Usage** — brief `use flowmark::FormatOptions` example, link to docs.rs
5. **Badges** — @ojoshe X follow, CI, crates.io, docs.rs, MSRV

## Release Notes Format

Follow the Python project’s release notes convention, extended with the parity version.

```markdown
## flowmark v0.2.0 (parity: flowmark-py v0.6.4)

### What's Changed

#### New Features

**Short title of feature**

Description of the new capability.

#### Bug Fixes

**Short title of fix**

Description of what was fixed.

#### Breaking Changes

**Short title of breaking change**

Description of what changed and how to migrate.

### Full Changelog

https://github.com/jlevy/flowmark-rs/compare/v0.1.3...v0.2.0
```

**Guidelines** (from Python project’s `docs/publishing.md`):
- Use `## What's Changed` as the top-level heading
- Group under `### Bug Fixes`, `### New Features`, `### Breaking Changes` as appropriate
- Use `**bold**` for short titles of individual changes
- Always include the Full Changelog compare link
- For small releases, a simple bullet list is acceptable

## Pre-Release Checklist

Adapted from Python project’s `docs/publishing.md`:

1. Verify all changes committed and pushed: `git status && git log origin/main..HEAD`
2. Run linting and tests locally:
   `cargo fmt --check && cargo clippy --all-targets --all-features && cargo test --all-features`
3. Confirm CI passing: `gh run list --limit 3`
4. Determine version (semver): `gh release list --limit 1`
5. Update `Cargo.toml` version + parity metadata
6. Create GitHub Release with `gh release create` using structured release notes
7. Verify publish workflow succeeds: `gh run list --workflow=publish.yml --limit 1`
8. Verify on crates.io: https://crates.io/crates/flowmark

## References

- [cargo-dist documentation](https://opensource.axo.dev/cargo-dist/)
- [crates.io trusted publishing](https://doc.rust-lang.org/cargo/reference/registry-authentication.html)
- [ripgrep release workflow](https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/release.yml)
- [Orhun’s automated Rust releases guide](https://blog.orhun.dev/automated-rust-releases/)
- Python flowmark project: https://github.com/jlevy/flowmark (reference for README
  structure, publishing process, release notes format)
- Current CI config: `.github/workflows/ci.yml`
- Current Cargo.toml: `Cargo.toml`
