# Feature: Build, CI Hardening, and Publishing Improvements

**Date:** 2026-02-17 (last updated 2026-03-01)

**Author:** Joshua Levy

**Status:** Phases 1-7 Complete — release orchestration and multi-channel publishing
framework implemented (GitHub Releases, crates.io, PyPI, Homebrew)

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
- Pre-built binaries for all major platforms — Linux (x86_64, arm64), macOS (x86_64,
  arm64), Windows (x86_64, arm64) — via GitHub Releases.
- SHA256 checksums for all release artifacts.
- Automated release workflow triggered by version tags.
- Dependency updates automated via Dependabot.
- Code coverage tracked and visible.
- README and CONTRIBUTING docs ready for public consumption.

**Future (not in scope for initial phases):**
- Shell installer (`curl | sh`) and PowerShell installer.
- Shell completions and man pages.

**Shipped (Phase 7):**
- Homebrew tap is live (`brew tap jlevy/flowmark && brew install flowmark`).

## Non-Goals

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
- ~~No Homebrew tap or formula.~~ **Done** (`jlevy/homebrew-flowmark`)
- ~~No PyPI distribution.~~ **Done** — see
  [PyPI distribution spec](plan-2026-03-01-pypi-distribution.md)
- No root README.md.
- ~~Missing `readme` and `documentation` fields in Cargo.toml.~~ **Done**
- No CONTRIBUTING.md or CHANGELOG.md.
- No shell completions or man page generation.

## Design

### Approach

Use a **custom GitHub Actions release workflow** modeled on
[casey/just](https://github.com/casey/just)’s release workflow for binary distribution.
This is the dominant approach among popular Rust CLI tools (12 of 14 surveyed in the
[binary distribution research](https://github.com/jlevy/rust-porting-playbook/blob/main/docs/project/research/research-rust-cli-binary-distribution.md)
use custom workflows).
just is the closest comparable project: pure Rust, single maintainer, focused CLI
utility with all-musl Linux targets and Windows support.

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
- [x] **Add `flowmark-rs` binary alias** — Added second `[[bin]]` entry in `Cargo.toml`
  pointing to the same `src/main.rs`. `cargo install flowmark` now installs both
  `flowmark` and `flowmark-rs` binaries, so users can explicitly pick the Rust
  implementation when both Python and Rust versions are installed.
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

Set up automated cross-platform binary builds via a custom GitHub Actions release
workflow, modeled on
[casey/just](https://github.com/casey/just/blob/master/.github/workflows/release.yaml).
This is the dominant approach among popular Rust CLI tools — 12 of 14 tools surveyed in
the
[binary distribution research](https://github.com/jlevy/rust-porting-playbook/blob/main/docs/project/research/research-rust-cli-binary-distribution.md)
use custom workflows.
just is the closest comparable project (pure Rust, focused CLI, single maintainer,
all-musl Linux, Windows support).

#### 5A: Background and Tool Choices

**Why a custom workflow (not cargo-dist)?**

A survey of 14 popular Rust CLI tools found that 12 use fully custom GitHub Actions
release workflows.
Only Astral’s uv and ruff use cargo-dist, both in a heavily customized
way for Python wheel builds.
See the
[binary distribution research](https://github.com/jlevy/rust-porting-playbook/blob/main/docs/project/research/research-rust-cli-binary-distribution.md)
for the full survey and analysis.

The custom workflow approach is preferred for flowmark-rs because:
- Battle-tested by ripgrep, bat, fd, just, typst, jj, and others
- Full control and visibility — every line is understood
- No version coupling or breaking changes from an external tool
- Straightforward for a pure-Rust CLI with no C dependencies
- Supports all targets including `aarch64-pc-windows-msvc` (which cargo-dist does not)

**Primary template:** just’s
[release.yaml](https://github.com/casey/just/blob/master/.github/workflows/release.yaml)
and [bin/package](https://github.com/casey/just/blob/master/bin/package) script.

**Alternatives considered:**
- **cargo-dist** (v0.30.x): Generates workflow + installers + Homebrew from TOML config.
  Appealing for zero-YAML setup, but: version coupling (must keep `cargo-dist-version`
  in sync with generated YAML), opaque generated workflow (~300 lines), pre-1.0 with
  breaking changes, cross-compilation issue #74 still open, and no
  `aarch64-pc-windows-msvc` support.
  Only 2 of 14 surveyed tools use it.
- **cross-rs**: Good for cross-compilation but doesn’t handle release orchestration.
  Not needed for flowmark-rs since all targets can be built with direct linker flags or
  native runners (just’s approach).
- **release-plz / cargo-release**: Handle version bumping and changelog automation but
  not binary builds. Complementary, can be added later.

#### 5B: Target Platforms

Build pre-compiled binaries for 6 targets covering all major platforms:

| Target Triple | OS | Arch | Runner | Cross-Compilation |
| --- | --- | --- | --- | --- |
| `x86_64-unknown-linux-musl` | Linux | x86_64 | `ubuntu-latest` | Native (musl-tools) |
| `aarch64-unknown-linux-musl` | Linux | ARM64 | `ubuntu-latest` | `--codegen linker=aarch64-linux-gnu-gcc` |
| `x86_64-apple-darwin` | macOS | x86_64 | `macos-latest` | `--target` on ARM runner |
| `aarch64-apple-darwin` | macOS | ARM64 | `macos-latest` | Native |
| `x86_64-pc-windows-msvc` | Windows | x86_64 | `windows-latest` | Native |
| `aarch64-pc-windows-msvc` | Windows | ARM64 | `windows-latest` | `rustup target add` |

**Why musl for Linux?** Static linking with musl produces fully self-contained binaries
that work on any Linux distribution regardless of glibc version.
just, typst, and jj all use musl-only for Linux.
The performance difference is negligible for a text formatter.

**Why include Windows?** The core Markdown processing is platform-agnostic.
Unix-specific code (SIGPIPE handling, file permissions) is behind `#[cfg(unix)]` guards
and simply absent on Windows.
13 of 14 surveyed tools include Windows.

**Cross-compilation approach (just’s model):** No Docker, no `cross-rs`. For Linux
ARM64, install `gcc-aarch64-linux-gnu` via apt and pass
`--codegen linker=aarch64-linux-gnu-gcc` via RUSTFLAGS. For macOS x86_64, build via
`--target` on the ARM64 runner (macOS supports this natively).
For Windows ARM64, `rustup target add aarch64-pc-windows-msvc` on the x86_64 runner.
All of this works for pure-Rust projects with no C dependencies.

#### 5C: Installer Strategy

**Phase 5 scope (ship immediately):**
- **GitHub Releases** with `.tar.gz` (Unix) and `.zip` (Windows) archives
- **SHA256 checksums** via a unified `SHA256SUMS` file (just’s checksum job pattern)
- **`cargo binstall` compatibility** — automatic with standard archive naming
  (`flowmark-vX.Y.Z-TARGET.tar.gz`). Since `Cargo.toml` has the `repository` field set,
  `cargo binstall flowmark` will discover and install pre-built binaries.

**Future (defer to later):**
- Shell installer (`curl | sh`) — can copy from starship or just’s install scripts
- PowerShell installer — for Windows users who don’t use `cargo install`

**Shipped (Phase 7):**
- Homebrew tap via `jlevy/homebrew-flowmark` is live — see Phase 7 below

#### 5D: Release Artifact Contents

Each platform archive will contain:
- `flowmark` binary (or `flowmark.exe` on Windows), stripped and LTO-optimized (already
  configured in `[profile.release]`)
- `LICENSE` (MIT)
- `README.md`

The release will also include:
- `SHA256SUMS` — unified checksum file for all artifacts (generated by a separate
  checksum job after all builds complete)

Archive formats follow the standard convention:
- `.tar.gz` for Linux and macOS (consistent with just, ripgrep, jj)
- `.zip` for Windows (consistent with just, ripgrep, typst)

Archive naming: `flowmark-vX.Y.Z-TARGET.tar.gz` (e.g.,
`flowmark-v0.2.2-x86_64-unknown-linux-musl.tar.gz`). This is the standard convention
that `cargo-binstall` auto-detects.

#### 5E: Release Flow and Workflow Coordination

The release process now uses one orchestrated workflow (`release.yml`) with reusable
channel workflows:

```
Developer pushes tag v0.X.Y (or runs workflow_dispatch dry-run)
        |
        v
release.yml (orchestrator)
  |-- plan: resolve tag, dry-run vs publish, prerelease mode
  |-- package: matrix build for each target (6 jobs)
  |     |-- install target deps (apt/rustup)
  |     |-- cargo build --release --target $TARGET
  |     '-- create release archives (.tar.gz / .zip)
  |-- checksum: generate SHA256SUMS from built artifacts
  |-- call publish.yml (reusable crates channel workflow)
  |     |-- run tests
  |     |-- cargo publish --dry-run
  |     '-- cargo publish (publish mode only, idempotent skip if already published)
  |-- call pypi.yml (reusable PyPI channel workflow)
  |     |-- build wheels/sdist + smoke tests
  |     |-- wheel-content validation
  |     '-- uv publish (publish mode only, rerun-safe with --check-url)
  |-- announce: create/update GitHub Release after channels complete
  '-- homebrew: update tap formula after successful channel publish
```

**Key coordination points:**
- `release.yml` supports both `push: tags` and `workflow_dispatch` dry-runs.
- `publish.yml` and `pypi.yml` are reusable (`workflow_call`) and can also be run
  manually via `workflow_dispatch`.
- GitHub Release creation is gated to run after channel workflows complete.
- Homebrew tap updates are gated after successful crates.io + PyPI publish.

#### 5F: Implementation Steps

**Step 5.1: Create `.github/workflows/release.yml`** (fmr-eldq)

Write the release workflow with three jobs, modeled on just’s release.yaml:

1. **`prerelease`** — determines if the tag is a stable release or prerelease
2. **`package`** — matrix of 6 targets, each: install deps, build, create archive,
   publish to GitHub Release
3. **`checksum`** — downloads all release artifacts, generates `SHA256SUMS`, uploads it

Key workflow details:
- Trigger: `push: tags: ['*']`
- Global RUSTFLAGS: `--deny warnings --codegen target-feature=+crt-static`
- Target-specific RUSTFLAGS for cross-compilation (linker overrides)
- Dependencies installed via apt for Linux ARM targets
- `softprops/action-gh-release@v2` for uploading archives and checksums
- `actions/checkout@v6`, `Swatinem/rust-cache@v2` for caching

**Step 5.2: Add Windows CI testing** (fmr-dqqo)

Add `windows-latest` to the CI test matrix in `ci.yml` to catch platform-specific issues
before release:
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
```

**Step 5.3: Update documentation** (fmr-rg6a)

Update these files to reflect the new installation methods:

1. **README.md** — Add pre-built binary download instructions:

   ````markdown
   ## Installation

   Install from [crates.io](https://crates.io/crates/flowmark):
   ```bash
   cargo install flowmark
   ````

   Or download a pre-built binary from
   [GitHub Releases](https://github.com/jlevy/flowmark-rs/releases).

   ```
   ```

2. **docs/publishing.md** — Add a section on the binary release flow and how the two
   workflows (release.yml and publish.yml) coordinate.

**Step 5.4: Test the full release cycle** (fmr-9dh1)

1. Merge the release workflow PR to main
2. Create a patch release (e.g., `v0.2.2`) to test the pipeline:
   - Bump version in `Cargo.toml`
   - Update CHANGELOG.md
   - Commit, push, merge to main
   - Tag and push: `git tag v0.2.2 && git push origin v0.2.2`
3. Watch `release.yml`: `gh run list --workflow=release.yml --limit 1`
4. Verify GitHub Release appears with all artifacts (6 archives + SHA256SUMS)
5. Watch `publish.yml`: `gh run list --workflow=publish.yml --limit 1`
6. Verify crates.io publication: https://crates.io/crates/flowmark
7. Test installation methods:
   - `cargo install flowmark` (from crates.io)
   - `cargo binstall flowmark` (from GitHub Releases)
   - Direct binary download from GitHub Releases
8. Verify `flowmark --version` shows correct version and parity info

#### 5G: Checksums and Security

- **SHA256 checksums**: Generated via a post-build `checksum` job that downloads ALL
  release artifacts with `gh release download`, runs `shasum -a 256 * > SHA256SUMS`, and
  uploads the result. This is just’s approach and ensures the checksum file covers every
  artifact.
- **Static CRT linking**: Windows binaries use `--codegen target-feature=+crt-static` to
  statically link the C runtime, avoiding “VCRUNTIME140.dll not found” errors.
  This is standard practice (just, cargo-dist, and others all do this).
- **`--locked` builds**: Ensures Cargo.lock is respected for reproducible builds.
- **`--deny warnings`**: Global RUSTFLAGS prevent building with warnings.

**Future security enhancements:**
- GitHub artifact attestations (`actions/attest-build-provenance`) — as fd and jj do
- Homebrew formula with SHA256 verification
- Binary signing via `cosign` or `minisign`

#### 5H: Maintenance and Upgrades

The custom workflow has minimal ongoing maintenance:

- **Adding targets**: Add a new entry to the matrix in `release.yml`.
- **Updating action versions**: Dependabot already covers GitHub Actions updates.
- **Updating Rust toolchain**: The workflow uses whatever stable toolchain is on the
  runner (just’s approach).
  No pinning needed unless reproducibility is a concern.
- **Adding installers**: Shell/PowerShell installers can be added as separate workflows
  when needed. Homebrew tap automation is integrated into `release.yml`.

### Phase 6: Documentation and Community — DONE

Standard open source project documentation.

- [x] **Write CONTRIBUTING.md** (fmr-nc8i) — Created `/CONTRIBUTING.md` with
  prerequisites, build/test/lint commands, PR guidelines, and link to publishing docs.
- [x] **Add CHANGELOG.md** (fmr-4v5g) — Created `/CHANGELOG.md` with v0.2.0 entry
  including parity version reference.
  Follows Keep a Changelog format.
- [x] **Update badges in README.md** (fmr-7ayu) — 5 badges: @ojoshe X follow, CI,
  crates.io, docs.rs, MSRV. Removed codecov (token not configured, badge showed
  “unknown”).
- [x] **Add `--version` parity info** (fmr-19zr) — Created `build.rs` that reads
  `[package.metadata.parity]` version and emits `PARITY_VERSION` env var.
  `src/main.rs` uses `long_version` to display:
  `flowmark 0.2.0 (parity: flowmark-py 0.6.4)`.
- [x] **Verify `cargo doc` output** (fmr-ghvq) — Docs build cleanly with `-D warnings`.
  No broken links or missing documentation.

### Phase 7: Homebrew Tap — COMPLETE (PUBLISHED + AUTOMATED)

The Homebrew tap is now published, so macOS and Linux users can install via:

```bash
brew tap jlevy/flowmark
brew install flowmark
```

#### Approach

Use a **separate repository** (`jlevy/homebrew-flowmark`) following Homebrew’s naming
convention. When a user runs `brew tap jlevy/flowmark`, Homebrew automatically looks for
a repo named `jlevy/homebrew-flowmark`.

The formula uses **pre-built binaries** from GitHub Releases (not building from source),
so installs are fast and don’t require the Rust toolchain.

The tap repository is tracked as a submodule at `repos/homebrew-flowmark` for
convenience.

#### Implementation Steps

1. **Create `jlevy/homebrew-flowmark` repository** — **DONE** — with a single formula
   file `Formula/flowmark.rb` that:
   - Detects platform (macOS ARM64 vs x86_64, Linux ARM64 vs x86_64)
   - Downloads the corresponding archive from GitHub Releases
   - Verifies SHA256 checksums
   - Installs the `flowmark` binary

2. **Add as submodule** — **DONE** — tracked at `repos/homebrew-flowmark`.

3. **Automate formula updates** — **DONE** — `release.yml` now includes a gated
   `homebrew` job that runs only after successful channel publish (crates.io + PyPI) and
   only for stable tags. It updates `jlevy/homebrew-flowmark/Formula/flowmark.rb` with
   the new version and per-target SHA256 values from generated `SHA256SUMS`.

   **Why not `mislav/bump-homebrew-formula-action`?** That action explicitly cannot
   handle formulas with Ruby `if...else` conditionals for platform-specific downloads.
   The custom job handles this formula structure directly.

   **Authentication:** Uses a fine-grained token stored as `HOMEBREW_TAP_TOKEN` with
   push access to `jlevy/homebrew-flowmark`.

4. **Update README.md** — **Pending** — Add `brew install` instructions to the
   Installation section.

5. **Update `docs/publishing.md`** — **Pending** — Once automation is in place, simplify
   Step 6 to just verify (no manual formula edits needed).

6. **Publish and validate tap installation** — **DONE** — Homebrew formula is available
   in `jlevy/homebrew-flowmark` and install flow works via:
   `brew tap jlevy/flowmark && brew install flowmark`.

#### Formula Structure

```ruby
class Flowmark < Formula
  desc "Markdown auto-formatter for clean diffs and semantic line breaks"
  homepage "https://github.com/jlevy/flowmark-rs"
  version "X.Y.Z"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/jlevy/flowmark-rs/releases/download/vX.Y.Z/flowmark-vX.Y.Z-aarch64-apple-darwin.tar.gz"
      sha256 "..."
    else
      url "https://github.com/jlevy/flowmark-rs/releases/download/vX.Y.Z/flowmark-vX.Y.Z-x86_64-apple-darwin.tar.gz"
      sha256 "..."
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/jlevy/flowmark-rs/releases/download/vX.Y.Z/flowmark-vX.Y.Z-aarch64-unknown-linux-musl.tar.gz"
      sha256 "..."
    else
      url "https://github.com/jlevy/flowmark-rs/releases/download/vX.Y.Z/flowmark-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz"
      sha256 "..."
    end
  end

  def install
    bin.install "flowmark"
  end

  test do
    system "#{bin}/flowmark", "--version"
  end
end
```

#### Future: homebrew-core Submission

Once the project has sufficient traction (stars, downloads), the formula can be
submitted to [homebrew-core](https://github.com/Homebrew/homebrew-core) for direct
`brew install flowmark` without a tap.
The formula would need to be adapted to build from source
(`depends_on "rust" => :build`) per homebrew-core policy.

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
3. ~~**cargo-dist vs manual release workflow**~~: **Resolved** — Custom GitHub Actions
   workflow selected, modeled on casey/just.
   A survey of 14 popular Rust CLI tools found 12 use custom workflows; only 2 use
   cargo-dist (both heavily customized).
   The custom approach provides full control, no version coupling, and supports all
   targets including `aarch64-pc-windows-msvc` (which cargo-dist does not).
   See Phase 5 and the
   [binary distribution research](https://github.com/jlevy/rust-porting-playbook/blob/main/docs/project/research/research-rust-cli-binary-distribution.md)
   for full analysis.
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
   - Homebrew via tap (`brew tap jlevy/flowmark && brew install flowmark`)
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

- [Binary distribution research](https://github.com/jlevy/rust-porting-playbook/blob/main/docs/project/research/research-rust-cli-binary-distribution.md)
  — Survey of 14 Rust CLI tools’ release practices (the basis for the Phase 5 approach)
- [just release.yaml](https://github.com/casey/just/blob/master/.github/workflows/release.yaml)
  — Primary template for the release workflow
- [just bin/package](https://github.com/casey/just/blob/master/bin/package) — just’s
  packaging script (archive creation, static CRT linking)
- [typst release.yml](https://github.com/typst/typst/blob/main/.github/workflows/release.yml)
  — Alternative template (SHA-pinned actions, ripgrep-derived)
- [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) — Binary install from
  crates.io metadata (auto-compatible with standard archive naming)
- [GitHub artifact attestations](https://github.blog/2024-05-02-introducing-artifact-attestations-now-in-public-beta/)
  — Cryptographic build provenance (future enhancement)
- [crates.io trusted publishing](https://doc.rust-lang.org/cargo/reference/registry-authentication.html)
- [Orhun’s automated Rust releases guide](https://blog.orhun.dev/automated-rust-releases/)
- Python flowmark project: https://github.com/jlevy/flowmark (reference for README
  structure, publishing process, release notes format)
- Current CI config: `.github/workflows/ci.yml`
- Current Cargo.toml: `Cargo.toml`
- Existing publish workflow: `.github/workflows/publish.yml`
