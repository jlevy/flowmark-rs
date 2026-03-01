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

## Verified Dependency Versions (as of 2026-03-01)

| Package / Action | Verified Latest | Pin in Workflow |
| --- | --- | --- |
| maturin (tool) | **1.12.5** (Feb 28, 2026) | `maturin-version: v1.12.5` |
| `PyO3/maturin-action` | **v1.50.1** (Mar 1, 2025) | `@v1` (major tag) |
| `astral-sh/setup-uv` | **v7** | `@v7` |
| uv (tool) | **0.10.7** (Feb 27, 2026) | Latest (auto-resolved) |
| `actions/checkout` | **v6.0.2** (Jan 9, 2026) | `@v6` |
| `actions/upload-artifact` | **v4** (stable) | `@v4` |
| `actions/download-artifact` | **v4** (stable) | `@v4` |

Note: `actions/upload-artifact@v7` and `actions/download-artifact@v8` exist (Feb 26,
2026) but are bleeding-edge with the new non-zipped artifact feature.
Use v4 for stability; upgrade later via Dependabot.

## Implementation Plan

### Phase 1: Configuration and Local Testing

Set up the maturin configuration and verify it works locally.

- [ ] **1.1: Create `/pyproject.toml`** (new file at repo root, alongside `Cargo.toml`)

  This file tells maturin how to build the Python wheel.
  It must be at the repo root (where `Cargo.toml` lives).

  **Exact contents:**

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
  Changelog = "https://github.com/jlevy/flowmark-rs/blob/main/CHANGELOG.md"

  [tool.maturin]
  bindings = "bin"
  strip = true
  ```

  **Key settings explained:**
  - `name = "flowmark-rs"` — PyPI package name (distinct from `flowmark` Python package)
  - `dynamic = ["version"]` — maturin reads version from `Cargo.toml` line 3
    (`version = "0.2.4"`)
  - `bindings = "bin"` — standalone binary, not a Python extension module
  - `strip = true` — strip debug symbols (redundant with `Cargo.toml` line 75
    `strip = true` in `[profile.release]`, but explicit is good)
  - `requires-python = ">=3.8"` — matches ruff; this is the install-side constraint, not
    a runtime requirement (the binary is pure Rust)

  **Interaction with existing files:**
  - `python/pyproject.toml` (hatchling, `flowmark-dev-tools`) is unaffected — maturin
    only reads the root `pyproject.toml`
  - `Cargo.toml` is already correct — has two `[[bin]]` targets (lines 19-27), both
    with `required-features = ["cli"]`, and `[features] default = ["cli"]` (line 30)

- [ ] **1.2: Update `/.gitignore`** — Append after line 36:

  ```
  # Maturin build artifacts
  *.whl
  ```

  The `target/` directory (line 36) already covers `target/wheels/` from maturin.
  Adding `*.whl` catches any wheels left in the repo root.

- [ ] **1.3: Update `/Cargo.toml` exclude list** — Line 13 currently:

  ```toml
  exclude = [".claude/", ".tbd/", ".github/", "docs/", "python/", "tests/tryscript/", "repos/", "admin/", "attic/"]
  ```

  Append `"pyproject.toml"` to exclude the maturin config from the crates.io package:

  ```toml
  exclude = [".claude/", ".tbd/", ".github/", "docs/", "python/", "tests/tryscript/", "repos/", "admin/", "attic/", "pyproject.toml"]
  ```

- [ ] **1.4: Local build test** — Verify maturin builds a wheel with both binaries:

  ```bash
  uv tool install maturin
  maturin build --release
  ls target/wheels/
  # Expected: flowmark_rs-0.2.4-cp38-abi3-{platform}.whl or similar
  # Verify wheel contents:
  python -m zipfile -l target/wheels/flowmark_rs-*.whl
  # Should show both flowmark and flowmark-rs in the scripts directory
  ```

  **Important:** Check that both `flowmark` and `flowmark-rs` binaries appear in the
  wheel's `.data/scripts/` directory.
  If only one appears, maturin may need the `--bin` flag or we may need to investigate
  how it handles multiple `[[bin]]` targets with `required-features`.

- [ ] **1.5: Local install test** — Verify the installed wheel works:

  ```bash
  maturin develop --release
  flowmark-rs --version
  # Expected: flowmark 0.2.4 (parity: flowmark-py 0.6.4)
  flowmark --version
  # Expected: same output
  flowmark-rs --help
  echo "# Test" | flowmark-rs -
  # Expected: formatted output
  ```

### Phase 2: CI Workflow

Create `.github/workflows/pypi.yml` — the full workflow for building wheels and
publishing to PyPI.

- [ ] **2.1: Create `.github/workflows/pypi.yml`**

  **File:** `.github/workflows/pypi.yml` (new file)

  **Full workflow structure with exact action versions:**

  ```yaml
  name: Publish to PyPI

  on:
    release:
      types: [published]
    workflow_dispatch:

  permissions:
    contents: read

  env:
    CARGO_TERM_COLOR: always

  jobs:
    # ── Build: Linux x86_64 (glibc) ────────────────────────────────
    build-linux-x86_64:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v6
        - uses: PyO3/maturin-action@v1
          with:
            maturin-version: v1.12.5
            command: build
            args: --release --locked --out dist
            target: x86_64-unknown-linux-gnu
            manylinux: "2_17"
        - uses: actions/upload-artifact@v4
          with:
            name: wheels-linux-x86_64
            path: dist/*.whl

    # ── Build: Linux aarch64 (glibc, cross-compiled) ───────────────
    build-linux-aarch64:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v6
        - uses: PyO3/maturin-action@v1
          with:
            maturin-version: v1.12.5
            command: build
            args: --release --locked --out dist
            target: aarch64-unknown-linux-gnu
            manylinux: "2_17"
        - uses: actions/upload-artifact@v4
          with:
            name: wheels-linux-aarch64
            path: dist/*.whl

    # ── Build: macOS x86_64 ────────────────────────────────────────
    build-macos-x86_64:
      runs-on: macos-13  # x86_64 runner
      steps:
        - uses: actions/checkout@v6
        - uses: PyO3/maturin-action@v1
          with:
            maturin-version: v1.12.5
            command: build
            args: --release --locked --out dist
            target: x86_64-apple-darwin
        - name: Smoke test
          run: |
            pip install --find-links dist flowmark-rs --force-reinstall
            flowmark-rs --version
        - uses: actions/upload-artifact@v4
          with:
            name: wheels-macos-x86_64
            path: dist/*.whl

    # ── Build: macOS aarch64 (Apple Silicon) ───────────────────────
    build-macos-aarch64:
      runs-on: macos-14  # ARM64 runner
      steps:
        - uses: actions/checkout@v6
        - uses: PyO3/maturin-action@v1
          with:
            maturin-version: v1.12.5
            command: build
            args: --release --locked --out dist
            target: aarch64-apple-darwin
        - name: Smoke test
          run: |
            pip install --find-links dist flowmark-rs --force-reinstall
            flowmark-rs --version
        - uses: actions/upload-artifact@v4
          with:
            name: wheels-macos-aarch64
            path: dist/*.whl

    # ── Build: Windows x86_64 ──────────────────────────────────────
    build-windows-x86_64:
      runs-on: windows-latest
      steps:
        - uses: actions/checkout@v6
        - uses: PyO3/maturin-action@v1
          with:
            maturin-version: v1.12.5
            command: build
            args: --release --locked --out dist
            target: x86_64-pc-windows-msvc
        - name: Smoke test
          run: |
            pip install --find-links dist flowmark-rs --force-reinstall
            flowmark-rs --version
        - uses: actions/upload-artifact@v4
          with:
            name: wheels-windows-x86_64
            path: dist/*.whl

    # ── Build: source distribution ─────────────────────────────────
    build-sdist:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v6
        - uses: PyO3/maturin-action@v1
          with:
            maturin-version: v1.12.5
            command: sdist
            args: --out dist
        - uses: actions/upload-artifact@v4
          with:
            name: wheels-sdist
            path: dist/*.tar.gz

    # ── Publish to PyPI ────────────────────────────────────────────
    publish:
      needs:
        - build-linux-x86_64
        - build-linux-aarch64
        - build-macos-x86_64
        - build-macos-aarch64
        - build-windows-x86_64
        - build-sdist
      runs-on: ubuntu-latest
      environment: release
      permissions:
        id-token: write  # Required for PyPI trusted publishing (OIDC)
      steps:
        - uses: astral-sh/setup-uv@v7
        - uses: actions/download-artifact@v4
          with:
            pattern: wheels-*
            merge-multiple: true
            path: wheels/
        - name: List wheels
          run: ls -la wheels/
        - name: Publish to PyPI
          run: uv publish --trusted-publishing always wheels/*
  ```

  **Key decisions and references:**
  - Trigger matches existing `publish.yml` (line 4-5): `release: types: [published]`
  - `maturin-version: v1.12.5` — pinned to latest stable (Feb 28, 2026)
  - `manylinux: "2_17"` — minimum for Rust glibc builds; matches ruff/uv
  - `macos-13` for x86_64, `macos-14` for ARM64 — matches ruff/uv runner selection
  - Smoke tests on native platforms only (macOS, Windows) — Linux aarch64 is
    cross-compiled so cannot be tested on the runner
  - `environment: release` — matches the PyPI trusted publisher configuration
  - `uv publish --trusted-publishing always` — explicitly requires OIDC (fails rather
    than falling back to tokens)

- [ ] **2.2: Test workflow with `workflow_dispatch`** — Push to main, then trigger
  manually from the Actions tab or via:

  ```bash
  gh workflow run pypi.yml --repo jlevy/flowmark-rs
  gh run list --workflow=pypi.yml --repo jlevy/flowmark-rs --limit 1
  gh run watch --repo jlevy/flowmark-rs <run-id>
  ```

  The publish step will fail (no PyPI project yet), but all 5 build jobs + sdist should
  produce wheel artifacts.
  Download and inspect them:

  ```bash
  gh run download --repo jlevy/flowmark-rs <run-id>
  python -m zipfile -l wheels-linux-x86_64/flowmark_rs-*.whl
  ```

### Phase 3: PyPI Setup and First Publish

Manual steps that require PyPI account access (owner action).

- [ ] **3.1: Register `flowmark-rs` on PyPI via pending trusted publisher**

  Go to `https://pypi.org/manage/account/publishing/` and add a pending publisher:
  - PyPI project name: `flowmark-rs`
  - Owner: `jlevy`
  - Repository: `flowmark-rs`
  - Workflow name: `pypi.yml`
  - Environment name: `release`

  This creates the PyPI project automatically on first successful publish.

- [ ] **3.2: Create GitHub `release` environment**

  In GitHub repo settings → Environments → New environment → `release`.
  Optional protection rules:
  - Restrict to `main` branch
  - Require approval (for manual oversight)

- [ ] **3.3: Test with TestPyPI first** (optional but recommended)

  Create a separate trusted publisher on `https://test.pypi.org/manage/account/publishing/`
  with the same settings.
  Temporarily add a workflow dispatch job that publishes to TestPyPI:

  ```bash
  uv publish --index-url https://test.pypi.org/legacy/ --trusted-publishing always wheels/*
  ```

  Then test: `uvx --index-url https://test.pypi.org/simple/ flowmark-rs --version`

- [ ] **3.4: First real publish** — Create the next GitHub Release (e.g., tag a new
  patch version).
  This triggers both `publish.yml` (crates.io) and `pypi.yml` (PyPI) simultaneously.

  ```bash
  gh run list --workflow=pypi.yml --repo jlevy/flowmark-rs --limit 1
  gh run watch --repo jlevy/flowmark-rs <run-id>
  ```

- [ ] **3.5: Verify installation on all platforms**

  ```bash
  # On-demand execution
  uvx flowmark-rs --version
  uvx flowmark-rs --help

  # Persistent install
  uv tool install flowmark-rs
  flowmark-rs --version
  flowmark --version  # Both binaries should work

  # Classic pip
  pip install flowmark-rs
  flowmark-rs --version

  # Format a test file
  echo "# Hello World\nThis is a test of flowmark-rs installed via PyPI." | uvx flowmark-rs -
  ```

### Phase 4: Documentation and Polish

Update docs to include the new install method.

- [ ] **4.1: Update `/README.md`** — Add PyPI install methods to the Installation
  section.
  Also update the README template at `docs/templates/rust-readme-wrapper.md` so future
  regenerations include it.

  Add to the Installation section (after the Homebrew entry):

  ```markdown
  ### PyPI (via uv or pip)

  ```bash
  uvx flowmark-rs          # run on demand (no install needed)
  uv tool install flowmark-rs  # persistent install
  pip install flowmark-rs      # classic pip
  ```
  ```

- [ ] **4.2: Update `/docs/publishing.md`** — Add a new section "Step 7: Verify PyPI
  Publication" after the existing Step 6 (Homebrew).
  Include:
  - How `pypi.yml` triggers on the same release event as `publish.yml`
  - Verification commands (`uvx flowmark-rs --version`)
  - Link to the PyPI project page
  - Trusted publishing configuration reference

  Also update the "Release Workflows" section to mention the third workflow:
  - `release.yml` → binary archives for GitHub Releases
  - `publish.yml` → crates.io
  - `pypi.yml` → PyPI (NEW)

- [ ] **4.3: Update the build-publishing spec** — In
  `docs/project/specs/active/plan-2026-02-17-build-publishing.md`, add a cross-reference
  in the "Publishing Gaps" section noting that PyPI distribution is now covered by this
  separate spec.

- [ ] **4.4: (Optional) Add musl targets** — Add two more build jobs to `pypi.yml`:

  ```yaml
  build-linux-musl-x86_64:
    # target: x86_64-unknown-linux-musl
    # No manylinux setting needed — maturin auto-tags as musllinux_1_2
  build-linux-musl-aarch64:
    # target: aarch64-unknown-linux-musl
  ```

  This adds Alpine Linux support.

- [ ] **4.5: Update flowmark (Python) repo** — Checkout `jlevy/flowmark` and create a
  branch with a README update.
  Add a section near the top (after the description, before Installation):

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

  Create a PR on `jlevy/flowmark` for this change.

- [ ] **4.6: (Optional) Add Python wrapper** — Add a thin Python wrapper package
  (following ruff's pattern) to enable `python -m flowmark_rs`.

  **New files:**
  - `py/flowmark_rs/__init__.py` — exports `find_flowmark_rs_bin()` function that
    locates the binary in the virtualenv's scripts directory
  - `py/flowmark_rs/__main__.py` — enables `python -m flowmark_rs` by exec-ing the
    binary (Unix: `os.execvp()`, Windows: `subprocess.run()`)
  - `py/flowmark_rs/_find_bin.py` — binary locator that searches virtualenv bin dir,
    system paths, and `sysconfig.get_path("scripts")`

  **Config change in `/pyproject.toml`:**
  Add `python-source = "py"` to `[tool.maturin]` and add `[project.scripts]`:
  ```toml
  [tool.maturin]
  python-source = "py"
  ```

### Phase 5: Rust-Porting Playbook Update

Map all learnings from this research and implementation into the
[rust-porting-playbook](https://github.com/jlevy/rust-porting-playbook) (submodule at
`repos/rust-porting-playbook`).

- [ ] **5.1: Add PyPI distribution guide** — Create or update a research/guide document
  in the playbook's `docs/project/research/` directory covering:
  - The maturin `bindings = "bin"` approach for Rust CLI → PyPI distribution
  - `pyproject.toml` configuration template (generalized from flowmark-rs)
  - GitHub Actions workflow template with maturin-action
  - Platform target matrix with manylinux/musllinux/macOS/Windows tags
  - PyPI trusted publishing (OIDC) setup
  - Version management (dynamic from `Cargo.toml`)

- [ ] **5.2: Add process recommendations** — Document the recommended process for any
  Rust CLI project to add PyPI distribution:
  - When to use this approach (CLI tools that have Python-ecosystem users)
  - Which targets to start with (the 5-target minimum vs. 17-target comprehensive)
  - How to handle the package naming (avoiding conflicts with existing Python packages)
  - Testing checklist (`uvx`, `pip install`, smoke tests in CI)
  - Integration with existing release workflows (separate workflow, same trigger)

- [ ] **5.3: Reference projects** — Add a comparison table of how major Rust CLI
  projects distribute via PyPI (ruff, uv, maturin, tpchgen-cli, celq) with links to
  their configurations.
  This is a condensed version of the findings from the
  [research brief](../research/research-2026-03-01-rust-cli-pypi-distribution.md).

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
