# Feature: Distribute flowmark-rs on PyPI via Maturin

**Date:** 2026-03-01 (last updated 2026-03-01)

**Author:** Claude (agent)

**Status:** Draft

**Related issue:**
[#36 — Distribute flowmark-rs on PyPI via maturin](https://github.com/jlevy/flowmark-rs/issues/36)

**Research:**
[Research: Distributing Rust CLI Binaries as Python Packages via PyPI](../research/research-2026-03-01-rust-cli-pypi-distribution.md)

## Overview

Publish flowmark-rs to PyPI as a pre-built binary so users can install and run it with:

```bash
uvx flowmark-rs          # run on demand
uv tool install flowmark-rs  # persistent install
pip install flowmark-rs      # classic pip
```

No Rust toolchain required for end users.
This is the same pattern used by ruff, uv, and maturin itself: a pure Rust CLI binary,
packaged as a Python wheel, published to PyPI.

## Goals

- `uvx flowmark-rs` works on Linux (x86_64, aarch64), macOS (x86_64, aarch64), and
  Windows (x86_64)
- PyPI package name: `flowmark-rs`
- Both `flowmark` and `flowmark-rs` binaries included in the wheel
- Automated publishing via GitHub Actions on each release
- PyPI trusted publishing (OIDC) — no long-lived API tokens
- Separate workflow from existing `release.yml` (binary archives) and `publish.yml`
  (crates.io)
- Source distribution (sdist) available as fallback for unsupported platforms

## Non-Goals

- PyO3 bindings or Python extension module (this is a standalone CLI)
- Replacing existing distribution channels (crates.io, Homebrew, GitHub Releases)
- npm or other non-Python distribution
- Supporting exotic platforms (s390x, ppc64, riscv64, armv7) — can be added later
- Python wrapper for `python -m flowmark_rs` — can be added later as polish

## Background

flowmark-rs already has a mature release pipeline:

| Channel | Status | Workflow |
| --- | --- | --- |
| GitHub Releases (binaries) | Active | `release.yml` — 6 targets |
| crates.io (source + binary) | Active | `publish.yml` — OIDC trusted publishing |
| Homebrew tap | Active | `jlevy/homebrew-flowmark` |
| **PyPI** | **Not yet** | **This spec** |

The original flowmark is a Python package on PyPI.
Adding PyPI distribution to the Rust rewrite keeps the install experience familiar and
leverages the Python ecosystem's ubiquity.

The
[research brief](../research/research-2026-03-01-rust-cli-pypi-distribution.md)
found that every major Rust CLI distributed via PyPI uses maturin with
`bindings = "bin"` — this is the universal standard approach.

### Key Design Decisions from Research

1. **Build tool:** maturin with `bindings = "bin"` (unanimous across all projects)
2. **Workflow:** Separate `pypi.yml`, not embedded in existing `release.yml`
   (clean separation of concerns, failure isolation)
3. **Versioning:** Dynamic from `Cargo.toml` (`dynamic = ["version"]`) —
   simpler than manual sync for a single-crate project
4. **Targets:** Standard coverage (5-7 targets), covering ~99% of users
5. **Python wrapper:** Deferred to polish phase — the binary is directly on PATH
   without one
6. **Publishing:** `uv publish --trusted-publishing always` via OIDC

## Design

### Approach

Add a `pyproject.toml` at the repo root with maturin configuration, and create a
`.github/workflows/pypi.yml` workflow that builds platform-specific wheels and publishes
to PyPI when a GitHub Release is published.

The workflow fires on the same `release: published` event as the existing `publish.yml`
(crates.io), so a single `gh release create` triggers both crates.io and PyPI publishing.

### Architecture

```
Developer pushes tag v0.X.Y
        |
        v
release.yml (existing)
  |-- builds binary archives for GitHub Releases
  '-- creates GitHub Release
        |
        v
GitHub Release "published" event
        |
        ├─── publish.yml (existing) → crates.io
        |
        └─── pypi.yml (NEW) → PyPI
               |-- build-linux-x86_64    (manylinux_2_17)
               |-- build-linux-aarch64   (manylinux_2_17)
               |-- build-macos-x86_64
               |-- build-macos-aarch64
               |-- build-windows-x86_64
               |-- build-sdist
               '-- publish (uv publish --trusted-publishing always)
```

### Platform Targets

| Target | Platform Tag | Runner | maturin manylinux |
| --- | --- | --- | --- |
| `x86_64-unknown-linux-gnu` | `manylinux_2_17_x86_64` | `ubuntu-latest` | `2_17` |
| `aarch64-unknown-linux-gnu` | `manylinux_2_17_aarch64` | `ubuntu-latest` | `2_17` |
| `x86_64-apple-darwin` | `macosx_10_12_x86_64` | `macos-13` | N/A |
| `aarch64-apple-darwin` | `macosx_11_0_arm64` | `macos-14` | N/A |
| `x86_64-pc-windows-msvc` | `win_amd64` | `windows-latest` | N/A |

**Why manylinux_2_17:** The Rust compiler requires glibc >= 2.17.
This covers CentOS 7+, Ubuntu 14.04+, Debian 8+ — virtually all Linux in use.
This is what ruff and uv use for most targets.

**Why not musl for PyPI:** The existing `release.yml` builds musl targets for GitHub
Releases (statically linked).
For PyPI, glibc (manylinux) wheels are the standard and cover ~95% of users.
Musl (musllinux) wheels for Alpine can be added later.

**macOS runners:** `macos-13` is x86_64, `macos-14` is ARM64 (Apple Silicon).
This matches ruff/uv's approach.

### File Layout

**New files at repo root:**

```
pyproject.toml               ← maturin build configuration
.github/workflows/pypi.yml   ← wheel build + publish workflow
```

**Interaction with existing `python/` directory:** The existing `python/pyproject.toml`
is for `flowmark-dev-tools` (a development-only package, marked
`Private :: Do Not Upload`).
It uses hatchling as its build backend.
The new root-level `pyproject.toml` uses maturin as its build backend.
These are completely separate — maturin reads the root `pyproject.toml` and the existing
`python/pyproject.toml` is unaffected.

### Binaries in the Wheel

`Cargo.toml` defines two binary targets:

```toml
[[bin]]
name = "flowmark"
path = "src/main.rs"

[[bin]]
name = "flowmark-rs"
path = "src/main.rs"
```

Maturin with `bindings = "bin"` packages all binary targets from the crate into the
wheel's `scripts` directory.
After `pip install flowmark-rs`, both `flowmark` and `flowmark-rs` commands will be
available on PATH.

### Versioning

Use `dynamic = ["version"]` in `pyproject.toml`.
Maturin reads the version from `Cargo.toml` automatically:

```toml
[project]
dynamic = ["version"]
```

This means the version in `Cargo.toml` is the single source of truth.
No manual sync needed.

## Implementation Plan

### Phase 1: Configuration and Local Testing

Set up the maturin configuration and verify it works locally.

- [ ] **1.1: Create root `pyproject.toml`** — Add at the repo root with maturin build
  configuration:

  ```toml
  [build-system]
  requires = ["maturin>=1.9,<2.0"]
  build-backend = "maturin"

  [project]
  name = "flowmark-rs"
  description = "A fast, configurable Markdown auto-formatter for semantic line breaks and clean diffs — written in Rust"
  requires-python = ">=3.8"
  license = { text = "MIT" }
  readme = "README.md"
  dynamic = ["version"]
  classifiers = [
      "Development Status :: 4 - Beta",
      "Environment :: Console",
      "Programming Language :: Rust",
      "Topic :: Text Processing :: Markup :: Markdown",
  ]

  [project.urls]
  Repository = "https://github.com/jlevy/flowmark-rs"
  Documentation = "https://docs.rs/flowmark"

  [tool.maturin]
  bindings = "bin"
  strip = true
  ```

- [ ] **1.2: Update `.gitignore`** — Add `target/wheels/` and any maturin build
  artifacts if not already ignored.

- [ ] **1.3: Update `Cargo.toml` exclude** — Add `pyproject.toml` to the crate's
  `exclude` list if it shouldn't be included in the crates.io package.
  (Maturin files are not needed for the Rust crate.)

- [ ] **1.4: Local build test** — Verify `maturin build --release` produces a wheel:

  ```bash
  uv tool install maturin
  maturin build --release
  ls target/wheels/
  ```

  Expected: a `.whl` file named something like
  `flowmark_rs-0.2.4-py3-none-{platform}.whl`

- [ ] **1.5: Local install test** — Verify the wheel installs and works:

  ```bash
  maturin develop --release
  flowmark-rs --help
  flowmark --help
  ```

  Both commands should work and show the same help text.

### Phase 2: CI Workflow

Create the GitHub Actions workflow for building and publishing wheels.

- [ ] **2.1: Create `.github/workflows/pypi.yml`** — The full workflow with build matrix
  and publish job.
  Structure:

  **Trigger:** `release: types: [published]` + `workflow_dispatch`

  **Build jobs (5 parallel):**

  Each build job:
  1. `actions/checkout@v6`
  2. `PyO3/maturin-action@v1` with `command: build`,
     `args: --release --locked --out dist`, and the appropriate `target` and `manylinux`
  3. Smoke test: install the wheel and run `flowmark-rs --version` (where possible)
  4. `actions/upload-artifact@v4` to save the wheel

  **sdist job:**
  1. `actions/checkout@v6`
  2. `PyO3/maturin-action@v1` with `command: sdist`, `args: --out dist`
  3. `actions/upload-artifact@v4`

  **Publish job:**
  1. `needs: [all build jobs + sdist]`
  2. `environment: release`
  3. `permissions: id-token: write`
  4. `astral-sh/setup-uv@v7`
  5. `actions/download-artifact@v4` with `pattern: wheels-*`, `merge-multiple: true`
  6. `uv publish --trusted-publishing always wheels/*`

  Key details:
  - Pin `maturin-version` to a specific version (e.g., `v1.12.1`)
  - Use `manylinux: "2_17"` for Linux glibc targets
  - Use `macos-13` for x86_64 macOS, `macos-14` for ARM64 macOS
  - Add `--locked` flag for reproducible builds

- [ ] **2.2: Test workflow with `workflow_dispatch`** — Before a real release, trigger
  the workflow manually to verify the build matrix works.
  The publish step will fail (no PyPI project yet), but the builds should succeed and
  produce wheels.

### Phase 3: PyPI Setup and First Publish

Set up the PyPI project and do the first publish.

- [ ] **3.1: Register `flowmark-rs` on PyPI** — Either:
  - Create a "pending" trusted publisher at
    `https://pypi.org/manage/account/publishing/` (for new projects), or
  - Manually upload a first release to create the project, then configure trusted
    publishing

  Trusted publisher configuration:
  - Owner: `jlevy`
  - Repository: `flowmark-rs`
  - Workflow: `pypi.yml`
  - Environment: `release`

- [ ] **3.2: Create GitHub environment** — Create a `release` environment in the
  GitHub repo settings for the publish job.
  Optionally add protection rules (require approval, limit to `main` branch).

- [ ] **3.3: (Optional) Test with TestPyPI first** — Temporarily modify the workflow to
  publish to TestPyPI (`https://test.pypi.org/legacy/`) to verify the end-to-end flow.

- [ ] **3.4: First real publish** — Create a GitHub Release (or use an existing one) to
  trigger the `pypi.yml` workflow.
  Monitor the workflow run and verify success.

- [ ] **3.5: Verify installation** — After publish, verify on all platforms:

  ```bash
  uvx flowmark-rs --version
  uv tool install flowmark-rs && flowmark-rs --help
  pip install flowmark-rs && flowmark-rs --help
  ```

### Phase 4: Documentation and Polish

Update docs to include the new install method.

- [ ] **4.1: Update README.md** — Add `uvx flowmark-rs` and `pip install flowmark-rs`
  to the Installation section.
  This should be added to the README template
  (`docs/templates/rust-readme-wrapper.md`) and regenerated.

- [ ] **4.2: Update `docs/publishing.md`** — Add a section on PyPI publishing:
  - How the `pypi.yml` workflow works
  - How it chains with the existing release flow
  - Verification steps after publish
  - PyPI trusted publishing configuration

- [ ] **4.3: Update the build-publishing spec** — Mark this PyPI work as done in the
  existing spec and add cross-references.

- [ ] **4.4: (Optional) Add musl targets** — Add `x86_64-unknown-linux-musl` and
  `aarch64-unknown-linux-musl` as `musllinux_1_2` wheels for Alpine Linux support.
  This is low priority since the glibc wheels cover most users.

- [ ] **4.5: Update flowmark (Python) repo** — Create a branch on `jlevy/flowmark` with
  a README update explaining that users can upgrade to the high-performance Rust version:

  > **High-Performance Rust Version**
  >
  > A Rust rewrite of flowmark is available as
  > [flowmark-rs](https://github.com/jlevy/flowmark-rs).
  > It produces identical output but runs significantly faster.
  > If you have [uv](https://docs.astral.sh/uv/) installed, you can switch with no
  > other dependencies:
  >
  > ```bash
  > uvx flowmark-rs@latest
  > ```

  This goes in the flowmark Python repo's README, near the installation section.

- [ ] **4.6: (Optional) Add Python wrapper** — Add a thin Python wrapper package
  (following ruff's pattern) to enable `python -m flowmark_rs`.
  Files:
  - `py/flowmark_rs/__init__.py`
  - `py/flowmark_rs/__main__.py`
  - `py/flowmark_rs/_find_bin.py`

  And add `python-source = "py"` to `[tool.maturin]`.

## Testing Strategy

### Build Verification

- Each build job in the CI workflow runs a smoke test: install the wheel in a clean
  virtualenv and run `flowmark-rs --version`
- Cross-compiled builds (Linux aarch64) cannot be smoke-tested on the runner but the
  wheel structure is verified

### Installation Verification

After each release:
- `uvx flowmark-rs --version` — verify on-demand execution
- `uv tool install flowmark-rs` — verify persistent installation
- `pip install flowmark-rs` — verify classic pip install
- `flowmark --help` and `flowmark-rs --help` — verify both binaries are available

### Compatibility Testing

- The sdist allows source builds on unsupported platforms (requires Rust toolchain)
- Manylinux_2_17 ensures Linux compatibility with glibc 2.17+ (CentOS 7+ era)
- macOS 10.12+ / 11.0+ (Intel / Apple Silicon)
- Windows x86_64

## Rollout Plan

1. Merge the `pyproject.toml` and `pypi.yml` workflow to `main`
2. Set up PyPI trusted publishing for `flowmark-rs`
3. Create the next release tag (e.g., the next version bump)
4. The `pypi.yml` workflow fires automatically and publishes to PyPI
5. Verify `uvx flowmark-rs` works
6. Update README and docs

## User Migration Path

The key value proposition: any user of the original Python flowmark can upgrade to the
high-performance Rust version with zero dependencies beyond `uv`:

```diff
- uvx flowmark@latest
+ uvx flowmark-rs@latest
```

The `@latest` suffix ensures they always get the newest version.
No Python runtime, no Rust toolchain, no Homebrew — just `uv`.

This should be documented in:

1. **flowmark-rs README** — in the Installation section
2. **flowmark (Python) README** — as a "High-Performance Alternative" or "Upgrade" note,
   explaining that users can switch to the Rust version for significantly faster
   formatting by changing `uvx flowmark` to `uvx flowmark-rs`
3. **flowmark (Python) repo** — a PR on a branch updating the README with migration
   instructions (Phase 4 below)

## Open Questions

1. **Package name:** `flowmark-rs` avoids conflict with the Python `flowmark` package.
   Is this the desired name, or should it be just `flowmark` (potentially conflicting
   with/replacing the Python package)?
   **Recommendation:** Use `flowmark-rs` to keep them distinct.

2. **Both binaries in wheel:** Maturin packages all `[[bin]]` targets from Cargo.toml.
   Verify that both `flowmark` and `flowmark-rs` are included in the wheel.
   If maturin only includes one, we may need to configure which binary to include or
   adjust the Cargo.toml.

3. **Existing `python/pyproject.toml` interaction:** Verify that having a root-level
   `pyproject.toml` (maturin) and a `python/pyproject.toml` (hatchling) causes no
   conflicts with tools like `uv`, `pip`, or IDEs.

## References

- [Research: Distributing Rust CLI Binaries via PyPI](../research/research-2026-03-01-rust-cli-pypi-distribution.md) —
  comprehensive background and analysis
- [Issue #36](https://github.com/jlevy/flowmark-rs/issues/36) — original proposal with
  detailed configuration
- [Build-Publishing Spec](plan-2026-02-17-build-publishing.md) — existing release
  infrastructure
- [Maturin User Guide](https://www.maturin.rs/) — maturin documentation
- [Maturin `bin` Bindings](https://www.maturin.rs/bindings) — binary distribution docs
- [PyO3/maturin-action](https://github.com/PyO3/maturin-action) — GitHub Actions for
  cross-platform builds
- [Ruff's pyproject.toml](https://github.com/astral-sh/ruff/blob/main/pyproject.toml) —
  primary real-world reference
- [PyPI Trusted Publishers](https://docs.pypi.org/trusted-publishers/) — OIDC setup
- [simple-modern-uv](https://github.com/jlevy/simple-modern-uv) — workflow patterns
