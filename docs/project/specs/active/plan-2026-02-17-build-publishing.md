# Feature: Build, CI Hardening, and Publishing Improvements

**Date:** 2026-02-17 (last updated 2026-02-17)

**Author:** Joshua Levy

**Status:** Planning

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
- One-line install via Homebrew (`brew install jlevy/tap/flowmark`).
- Shell installer for quick install on any Unix system (`curl | sh`).
- Automated release workflow triggered by version tags.
- Dependency updates automated via Dependabot.
- Code coverage tracked and visible.
- README and CONTRIBUTING docs ready for public consumption.

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
| `cargo clippy` with pedantic deny | Present (source-level in Cargo.toml) |
| `unsafe_code = "deny"` | Present |
| `unwrap_used = "deny"` | Present |
| `RUSTFLAGS="-D warnings"` on test jobs | Present |
| `RUSTDOCFLAGS="-D warnings"` on docs job | Present |
| `cargo test --locked --all-features` | Present |
| Cross-platform testing (ubuntu + macOS) | Present |
| `--no-default-features` test job | Present |
| MSRV check (1.85) | Present |
| `cargo-deny` with deny.toml | Present |
| `Swatinem/rust-cache@v2` | Present |
| Release profile (LTO, strip, panic=abort) | Present |

### Gaps Identified

| # | Gap | Priority | Bead |
| --- | --- | --- | --- |
| 1 | Missing `--locked` on clippy job | P1 | fmr-mk46 |
| 2 | Missing `--locked` on docs job | P1 | fmr-9eda |
| 3 | Missing `CARGO_PROFILE_TEST_DEBUG: 0` | P2 | fmr-b035 |
| 4 | No code coverage (cargo-llvm-cov) | P2 | fmr-hj6z |
| 5 | No cargo-semver-checks | P2 | fmr-8un1 |
| 6 | No Dependabot config | P3 | fmr-zvbe |
| 7 | No cargo-nextest | P3 | fmr-rj25 |

### Publishing Gaps

- No release workflow (no binary builds, no GitHub Releases).
- No crates.io publishing automation.
- No Homebrew tap or formula.
- No root README.md.
- Missing `readme` and `documentation` fields in Cargo.toml.
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

### Phase 1: CI Quick Fixes — PENDING

Minimal-effort improvements to the existing CI pipeline.

- [ ] Add `--locked` to clippy job
  (`cargo clippy --locked --all-targets --all-features`) (fmr-mk46)
- [ ] Add `--locked` to docs job (`cargo doc --locked --no-deps --all-features`)
  (fmr-9eda)
- [ ] Add `CARGO_PROFILE_TEST_DEBUG: 0` to global `env:` block (fmr-b035)

### Phase 2: CI Enhancements — PENDING

Higher-impact CI additions.

- [ ] Add code coverage job with `cargo-llvm-cov` and Codecov upload (fmr-hj6z)
- [ ] Add `cargo-semver-checks` job for API breakage detection (fmr-8un1)
- [ ] Add `.github/dependabot.yml` for weekly Cargo dependency updates (fmr-zvbe)
- [ ] Consider `cargo-nextest` for faster test execution (fmr-rj25)

### Phase 3: Crates.io Readiness — PENDING

Prepare Cargo.toml metadata and verify publishability.

- [ ] Add `readme = "README.md"` to Cargo.toml
- [ ] Add `documentation = "https://docs.rs/flowmark"` to Cargo.toml
- [ ] Verify `cargo publish --dry-run` succeeds
- [ ] Write root README.md (project description, install instructions, usage examples,
  badges)
- [ ] First manual `cargo publish` to claim the crate name
- [ ] Set up trusted publishing (OIDC) on crates.io for future CI-driven publishes

### Phase 4: Binary Release Workflow — PENDING

Set up automated cross-platform binary builds via cargo-dist.

- [ ] Install and run `cargo dist init` to bootstrap configuration
- [ ] Configure targets: `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`,
  `x86_64-apple-darwin`, `aarch64-apple-darwin`
- [ ] Configure installers: shell, homebrew
- [ ] Review and customize generated `.github/workflows/release.yml`
- [ ] Test release workflow with a `v0.1.0` tag (or pre-release tag)
- [ ] Verify artifacts: tarball contents include binary + LICENSE + README

### Phase 5: Homebrew Tap — PENDING

Make `brew install jlevy/tap/flowmark` work.

- [ ] Create `jlevy/homebrew-tap` repository on GitHub
- [ ] Configure cargo-dist to auto-update the tap formula on release
- [ ] Test `brew install jlevy/tap/flowmark` from a clean environment
- [ ] Add Homebrew install instructions to README

### Phase 6: CLI Polish for Release — PENDING

Shell completions, man pages, and other niceties expected of a polished CLI.

- [ ] Add `clap_complete` for shell completion generation (bash, zsh, fish)
- [ ] Add `clap_mananual` for man page generation (or a build script approach)
- [ ] Include completions and man page in release artifacts
- [ ] Add `--version` output that includes git hash (via build script or `vergen`)

### Phase 7: Documentation and Community — PENDING

Standard open source project documentation.

- [ ] Write CONTRIBUTING.md (build instructions, test commands, PR guidelines)
- [ ] Add CHANGELOG.md (can be minimal initially; automate later with git-cliff)
- [ ] Add badges to README (CI status, crates.io version, docs.rs, codecov, MSRV)
- [ ] Review and update LICENSE file if needed
- [ ] Ensure `cargo doc` output is clean and useful for library consumers

## Open Questions

1. **Crate name availability**: Is `flowmark` available on crates.io?
   Need to check before first publish.
2. **Version strategy**: Start at `0.1.0` (current) or bump to `0.2.0` for the public
   release? Semver conventions suggest `0.x` is fine for pre-1.0 software.
3. **cargo-dist vs manual release workflow**: cargo-dist is simpler but less flexible.
   For a project this size, cargo-dist is likely the right choice initially.
4. **Shell completions scope**: Should completions be generated at build time (build
   script) or at runtime (`flowmark completions bash`)? Runtime is simpler for
   distribution; build-time is standard for cargo-dist artifacts.

## References

- [cargo-dist documentation](https://opensource.axo.dev/cargo-dist/)
- [crates.io trusted publishing](https://doc.rust-lang.org/cargo/reference/registry-authentication.html)
- [ripgrep release workflow](https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/release.yml)
- [Orhun’s automated Rust releases guide](https://blog.orhun.dev/automated-rust-releases/)
- Current CI config: `.github/workflows/ci.yml`
- Current Cargo.toml: `Cargo.toml`
