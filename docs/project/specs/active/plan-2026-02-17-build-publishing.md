# Feature: Build, CI Hardening, and Publishing Improvements

**Date:** 2026-02-17 (last updated 2026-02-20)

**Author:** Joshua Levy

**Status:** Phases 1-4, 6 Complete — Phase 5 (binary releases) planned in detail, ready
for implementation

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

Set up automated cross-platform binary builds via
[cargo-dist](https://opensource.axo.dev/cargo-dist/) (v0.30.x, the current community
standard for Rust CLI distribution), with
[cargo-binstall](https://github.com/cargo-bins/cargo-binstall) compatibility, a shell
installer, and Homebrew tap support.

#### 5A: Background and Tool Choices

**Why cargo-dist?**
cargo-dist is the de facto standard for Rust binary distribution (used by uv, zoxide,
dump_syms, and hundreds of other Rust CLI tools). It generates cross-platform build CI,
shell/PowerShell installers, Homebrew formulae, and GitHub Release artifacts from a single
configuration file. It auto-detects `cargo-binstall` compatibility via the `repository`
field in `Cargo.toml` (already set), so `cargo binstall flowmark` will work automatically
once release artifacts exist.

**Alternatives considered:**
- **Manual GitHub Actions workflow** (ripgrep-style): Maximum flexibility but requires
  maintaining ~200 lines of custom workflow YAML per target, cross-compilation toolchains,
  and manual installer scripts. Overkill for a project this size.
- **cross-rs**: Good for cross-compilation but doesn't handle release orchestration,
  installers, or GitHub Releases. Would need to be combined with manual workflow YAML.
- **release-plz / cargo-release**: Handle version bumping and changelog automation but
  not binary builds. Complementary to cargo-dist but not a replacement.

**Configuration format:** cargo-dist now supports both `[workspace.metadata.dist]` in
`Cargo.toml` (legacy) and the standalone `dist-workspace.toml` file (preferred for new
setups, since v0.23+). We will use `dist-workspace.toml` to keep `Cargo.toml` clean.

#### 5B: Target Platforms

Build pre-compiled static binaries for four Unix targets (no Windows — this project uses
`libc` with `cfg(unix)` by design):

| Target Triple | OS | Arch | Libc | Notes |
| --- | --- | --- | --- | --- |
| `x86_64-unknown-linux-musl` | Linux | x86_64 | musl (static) | Most common Linux servers/CI |
| `aarch64-unknown-linux-musl` | Linux | ARM64 | musl (static) | AWS Graviton, ARM servers |
| `x86_64-apple-darwin` | macOS | x86_64 | system | Intel Macs (pre-2020) |
| `aarch64-apple-darwin` | macOS | ARM64 | system | Apple Silicon (M1+) |

**Why musl for Linux?** Static linking with musl produces fully self-contained binaries
that work on any Linux distribution regardless of glibc version. This avoids the common
"GLIBC_2.XX not found" errors. The binary size increase is negligible for a CLI tool.

**Why no `linux-gnu` targets?** With musl providing universal Linux compatibility, gnu
targets add CI cost without user benefit. Projects like uv include gnu targets for maximum
glibc performance, but for a text formatter the difference is immaterial.

#### 5C: Installer Strategy

**Shell installer (priority — ship immediately):**
cargo-dist generates a shell installer script that provides a `curl | sh` one-liner:
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/jlevy/flowmark-rs/releases/latest/download/flowmark-installer.sh | sh
```
The installer auto-detects the platform, downloads the correct binary, installs to
`~/.cargo/bin/` (or `$CARGO_HOME/bin/`), and updates PATH. This is the standard Rust
ecosystem install path (matching `cargo install` behavior).

**cargo-binstall (automatic — no configuration needed):**
Since `Cargo.toml` already has `repository = "https://github.com/jlevy/flowmark-rs"`,
cargo-binstall will automatically discover and install pre-built binaries from GitHub
Releases. Users can run:
```bash
cargo binstall flowmark
```
This downloads the pre-built binary instead of compiling from source, providing a much
faster install experience.

**Homebrew tap (Phase 5 scope — include in initial setup):**
cargo-dist can auto-generate and publish a Homebrew formula on each release. This requires:
1. A `jlevy/homebrew-tap` repository on GitHub
2. A `HOMEBREW_TAP_TOKEN` secret (personal access token with `repo` scope)
3. Setting `tap = "jlevy/homebrew-tap"` and adding `"homebrew"` to the installers list

Once configured, users can install via:
```bash
brew install jlevy/tap/flowmark
```
Homebrew handles auto-updates via `brew upgrade`.

**npm installer (not planned):**
cargo-dist supports npm-based installation but this adds complexity without clear user
benefit for a Markdown tool. Defer indefinitely.

#### 5D: Release Artifact Contents

Each platform release tarball (`.tar.xz` for Unix) will contain:
- `flowmark` binary (stripped, LTO-optimized — already configured in `[profile.release]`)
- `flowmark-rs` binary (alias, same binary)
- `LICENSE` (MIT)
- `README.md`

Additionally, cargo-dist generates:
- `flowmark-installer.sh` — Shell installer script
- SHA-256 checksums for all artifacts (cargo-dist default)
- `dist-manifest.json` — Machine-readable manifest of all release artifacts (used by
  cargo-binstall and other tools for auto-discovery)

#### 5E: Release Flow and Workflow Coordination

The release process involves two GitHub Actions workflows that chain together:

```
Developer pushes tag v0.X.Y
        │
        ▼
release.yml (cargo-dist) ──────────────────────────────┐
  ├─ plan: compute build matrix                         │
  ├─ build-local: compile for each target (4 jobs)      │
  ├─ build-global: shell installer, checksums           │
  ├─ host: create GitHub Release, upload all artifacts  │
  ├─ publish: update Homebrew tap                       │
  └─ announce: done                                     │
                                                        │
        GitHub Release "published" event ◄──────────────┘
        │
        ▼
publish.yml (existing OIDC workflow)
  ├─ run tests
  └─ cargo publish to crates.io
```

**Key coordination points:**
- `release.yml` triggers on tag push (`v*`) — this is the cargo-dist default
- `release.yml` creates the GitHub Release with binary artifacts
- `publish.yml` (already exists) triggers on `release: published` event
- `publish.yml` handles crates.io publishing via OIDC trusted publishing
- cargo-dist does NOT need to publish to crates.io — we keep the existing workflow

**Configuration to disable cargo-dist's crates.io publish:**
Set `publish-jobs = ["homebrew"]` in `dist-workspace.toml` (only the Homebrew publish
job, no built-in crates.io publish). Our existing `publish.yml` handles crates.io via
the GitHub Release event trigger.

#### 5F: Implementation Steps

**Step 5.1: Install cargo-dist and initialize configuration**

Install cargo-dist locally:
```bash
cargo install cargo-dist
```

Run interactive init (or with `--yes` for defaults):
```bash
cargo dist init
```

This generates:
- `dist-workspace.toml` — Top-level config file
- `.github/workflows/release.yml` — The release CI workflow

Then manually edit `dist-workspace.toml` to the desired configuration:

```toml
[workspace]
members = ["cargo:."]

[dist]
cargo-dist-version = "0.30.4"
ci = "github"
installers = ["shell", "homebrew"]
targets = [
  "aarch64-apple-darwin",
  "x86_64-apple-darwin",
  "x86_64-unknown-linux-musl",
  "aarch64-unknown-linux-musl",
]
publish-jobs = ["homebrew"]
tap = "jlevy/homebrew-tap"
install-path = "CARGO_HOME"
pr-run-mode = "plan"
checksum = "sha256"
create-release = true
source-tarball = false
```

Configuration notes:
- `installers = ["shell", "homebrew"]`: Generate shell installer + Homebrew formula.
  No `"powershell"` since this is Unix-only.
- `targets`: The four Unix targets described in 5B. No Windows targets.
- `publish-jobs = ["homebrew"]`: Only publish to Homebrew tap. crates.io publishing is
  handled by the existing `publish.yml` workflow.
- `tap = "jlevy/homebrew-tap"`: The Homebrew tap repository.
- `install-path = "CARGO_HOME"`: Install to `~/.cargo/bin/` for consistency with
  `cargo install`.
- `pr-run-mode = "plan"`: Run `dist plan` on PRs to catch release config errors early.
- `source-tarball = false`: GitHub auto-generates source tarballs; no need to duplicate.

**Step 5.2: Create the Homebrew tap repository**

This is a manual step (requires GitHub repo creation):

1. Create `jlevy/homebrew-tap` repository on GitHub (public, with a README)
2. Generate a GitHub personal access token (classic) with `repo` scope
3. Add the token as a secret named `HOMEBREW_TAP_TOKEN` in the `jlevy/flowmark-rs`
   repository settings (Settings → Secrets → Actions)

The tap repository is just a Git repo that Homebrew reads formulae from. cargo-dist
auto-pushes updated formulae on each release.

**Step 5.3: Review and customize the generated release workflow**

After `cargo dist init`, review `.github/workflows/release.yml`:

- Verify it triggers on `push: tags: ["v*"]` (not on release creation, to avoid
  circular triggers with `publish.yml`)
- Verify the build matrix includes all 4 targets
- Verify the `host` job creates a GitHub Release and uploads artifacts
- Verify musl builds use the correct cross-compilation setup (cargo-dist handles this
  automatically via `cross` or target-specific runners)
- Ensure the `HOMEBREW_TAP_TOKEN` secret is referenced correctly
- Confirm `id-token: write` permission is set if needed for attestations

Specific things to check/customize:
- The workflow should use `Swatinem/rust-cache@v2` for build caching (cargo-dist
  includes this by default)
- ARM Linux builds may use cross-compilation via `cross` or a dedicated ARM runner
- macOS builds use `macos-latest` (which is ARM64 on GitHub Actions) with an
  additional x86_64 build via `--target`

**Step 5.4: Coordinate release.yml with existing publish.yml**

Verify the trigger chain works correctly:

1. `release.yml` triggers on tag push (`v*`) — builds binaries, creates GitHub Release
2. `publish.yml` triggers on `release: published` — publishes to crates.io
3. Both workflows must not conflict or race

Check for potential issues:
- Ensure `release.yml` does NOT also trigger `publish.yml` via `workflow_dispatch`
- If `release.yml` creates the release as a draft first and then publishes, the
  `published` event fires once when the release is finalized (cargo-dist handles this
  correctly with `github-release = "announce"` which waits until all builds complete)
- Verify that `publish.yml` runs AFTER binaries are uploaded (it already does since it
  triggers on the `published` event, not on tag push)

**Step 5.5: Update documentation**

Update these files to reflect the new installation methods:

1. **README.md** — Add shell installer and Homebrew install instructions:
   ```markdown
   ## Installation

   Install from [crates.io](https://crates.io/crates/flowmark):
   ```bash
   cargo install flowmark
   ```

   Or install via the shell installer:
   ```bash
   curl --proto '=https' --tlsv1.2 -LsSf https://github.com/jlevy/flowmark-rs/releases/latest/download/flowmark-installer.sh | sh
   ```

   Or via Homebrew:
   ```bash
   brew install jlevy/tap/flowmark
   ```

   Or download a pre-built binary from
   [GitHub Releases](https://github.com/jlevy/flowmark-rs/releases).
   ```

2. **docs/publishing.md** — Add a section on the binary release flow and how the two
   workflows (release.yml and publish.yml) coordinate.

3. **CONTRIBUTING.md** — Add a note about `cargo dist plan` for verifying release config.

**Step 5.6: Verify locally with `cargo dist plan`**

Before pushing, run:
```bash
cargo dist plan
```

This executes the same logic as the CI `plan` step without building anything. Verify:
- All 4 targets are listed
- Shell installer is listed
- Homebrew formula is listed
- Artifact names look correct (e.g., `flowmark-v0.2.2-x86_64-unknown-linux-musl.tar.xz`)
- No errors or warnings about missing configuration

**Step 5.7: Test the full release cycle**

1. Merge the cargo-dist setup PR to main
2. Create a patch release (e.g., `v0.2.2`) to test the pipeline:
   - Bump version in `Cargo.toml`
   - Update CHANGELOG.md
   - Commit, push, merge to main
   - Tag and push: `git tag v0.2.2 && git push origin v0.2.2`
3. Watch `release.yml`: `gh run list --workflow=release.yml --limit 1`
4. Verify GitHub Release appears with all artifacts (4 tarballs + installer script +
   checksums)
5. Watch `publish.yml`: `gh run list --workflow=publish.yml --limit 1`
6. Verify crates.io publication: https://crates.io/crates/flowmark
7. Test each installation method from a clean environment:
   - `cargo install flowmark` (from crates.io)
   - `cargo binstall flowmark` (from GitHub Releases)
   - Shell installer (curl | sh)
   - `brew install jlevy/tap/flowmark` (from Homebrew tap)
   - Direct binary download from GitHub Releases
8. Verify `flowmark --version` shows correct version and parity info

**Step 5.8: Add cargo-dist version update to Dependabot**

The existing `.github/dependabot.yml` already covers Cargo and GitHub Actions. No changes
needed — cargo-dist's generated workflow uses pinned action versions that Dependabot will
update automatically.

However, the `cargo-dist-version` in `dist-workspace.toml` needs manual bumping when
upgrading cargo-dist. Consider adding a comment in the config file noting this.

#### 5G: Checksums and Security

cargo-dist provides several security features out of the box:

- **SHA-256 checksums**: Generated for every artifact by default (configurable via
  `checksum` setting). Each release includes a `dist-manifest.json` with checksums.
- **GitHub Attestations**: cargo-dist supports
  [GitHub artifact attestations](https://github.blog/2024-05-02-introducing-artifact-attestations-now-in-public-beta/)
  which provide cryptographic proof that artifacts were built in CI. Enable with
  `github-attestations = true` in `dist-workspace.toml`.
- **HTTPS-only installer**: The shell installer enforces HTTPS (`--proto '=https'`
  `--tlsv1.2`) for all downloads.
- **Reproducible builds**: cargo-dist pins its own version and uses `--locked` builds,
  so the same tag always produces the same artifacts.

**Signing (future consideration):**
cargo-dist does not yet have built-in binary signing (tracked in
[axodotdev/cargo-dist#1121](https://github.com/axodotdev/cargo-dist/issues/1121)).
For now, SHA-256 checksums + GitHub attestations provide sufficient integrity verification
for an open-source CLI tool. Signing can be added later when cargo-dist supports it or via
a custom post-build step using `cosign` or `minisign`.

#### 5H: Maintenance and Upgrades

After the initial setup, ongoing maintenance is minimal:

- **Updating cargo-dist**: Run `cargo dist init` again to regenerate the workflow with a
  newer version. Bump `cargo-dist-version` in `dist-workspace.toml`.
- **Adding targets**: Add new target triples to `dist-workspace.toml` and re-run
  `cargo dist init` to regenerate the workflow.
- **Adding installers**: Re-run `cargo dist init`, select new installers, and the
  workflow updates automatically.
- **PR validation**: The `plan` job runs on every PR, catching release config problems
  before they reach main.

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

### Future: Homebrew Tap — INCLUDED IN PHASE 5

Homebrew tap setup is now part of Phase 5 (steps 5.1, 5.2, and 5.7). cargo-dist handles
auto-generating and publishing the Homebrew formula on each release, so it is straightforward
to include in the initial binary distribution setup rather than deferring it.

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
3. ~~**cargo-dist vs manual release workflow**~~: **Resolved** — cargo-dist (v0.30.x)
   selected. It is the community standard, supports all needed targets (including musl
   static Linux builds), generates shell installers and Homebrew formulae, and integrates
   with the existing publish.yml workflow via the GitHub Release event chain. The
   `dist-workspace.toml` config file keeps Cargo.toml clean. See Phase 5 for full details.
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

- [cargo-dist documentation](https://opensource.axo.dev/cargo-dist/) — Primary tool for
  binary distribution (v0.30.x)
- [cargo-dist quickstart (Rust)](https://axodotdev.github.io/cargo-dist/book/quickstart/rust.html) —
  Setup guide
- [cargo-dist configuration reference](https://axodotdev.github.io/cargo-dist/book/reference/config.html) —
  All config keys
- [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) — Binary install from
  crates.io metadata (auto-compatible with cargo-dist releases)
- [GitHub artifact attestations](https://github.blog/2024-05-02-introducing-artifact-attestations-now-in-public-beta/) —
  Cryptographic build provenance
- [crates.io trusted publishing](https://doc.rust-lang.org/cargo/reference/registry-authentication.html)
- [ripgrep release workflow](https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/release.yml) —
  Example of a manual (non-cargo-dist) approach
- [Orhun’s automated Rust releases guide](https://blog.orhun.dev/automated-rust-releases/)
- [uv dist-workspace.toml](https://github.com/astral-sh/uv/blob/main/dist-workspace.toml) —
  Real-world cargo-dist config for a major Rust CLI
- Python flowmark project: https://github.com/jlevy/flowmark (reference for README
  structure, publishing process, release notes format)
- Current CI config: `.github/workflows/ci.yml`
- Current Cargo.toml: `Cargo.toml`
- Existing publish workflow: `.github/workflows/publish.yml`
