# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.2.6] (parity: flowmark-py 0.6.4)

Release infrastructure and documentation hardening release.
No formatter behavior changes; parity remains pinned to Python flowmark `0.6.4`.

### Improvements

- Stabilized embedded version metadata so release builds produce clean version strings
  (no spurious `-dev.unknown+gunknown` suffix)
- Codified canonical publishing runbook for hybrid Rust/Python multi-channel releases
- Refreshed top-level docs, README, and installation guidance (leading with `uv`/`uvx`)

### CI and release process

- Fixed clippy format-arg lint in version metadata build script
- Added `release_tag` passthrough to `publish.yml` and `pypi.yml` reusable workflows
  for stable version embedding in CI builds
- Moved completed specs from `active` to `done` and reorganized project docs

### Testing

- Retired legacy `cli-golden.tryscript.md` test without coverage loss (replaced by
  focused `file-discovery` and `stdin` tryscript tests)

## [0.2.5] (parity: flowmark-py 0.6.4)

First multi-channel release: crates.io, PyPI, GitHub Releases, and Homebrew.
No formatter behavior changes; parity remains pinned to Python flowmark `0.6.4`.

### Features

- Added PyPI distribution support for `flowmark-rs` via maturin (`pyproject.toml`) and
  a dedicated GitHub Actions workflow (`pypi.yml`)
- Added incremental cache lifecycle/inspection support in the CLI, including
  `--cache-dir`, `--show-cache`, and `--clear-cache`

### Fixes

- `--clear-cache` no longer requires resolving the current working directory
- Hardened PyPI smoke tests to install only locally built wheels (`--no-index`), and
  restricted PyPI publish job execution to release-triggered runs

## [0.2.4] (parity: flowmark-py 0.6.4)

Release process and documentation hardening release.
No formatter behavior changes; parity remains pinned to Python flowmark `0.6.4`.

### Improvements

- Clarified and standardized Homebrew installation guidance across README and publishing
  docs
- Tightened CLI help text and usage footer for clearer command-line guidance
- Added and documented README generation sync checks so generated docs stay in lockstep

### CI and release process

- Expanded publishing playbook with explicit release workflow sequencing and
  verification steps
- Added Homebrew tap update workflow details and validation commands
- Applied repository-wide Markdown formatting to keep release/process docs consistent

### Dependencies

- Bumped `clap` to `4.5.60`
- Bumped `tempfile` to `3.26.0`
- Bumped `anyhow` to `1.0.102`

## [0.2.3] (parity: flowmark-py 0.6.4)

Fixes release workflow for cross-compiled Linux ARM64 binaries.

### Fixes

- Fixed cross-compilation for `aarch64-unknown-linux-musl`: set `CC` env var for `cc-rs`
  to find the cross-compiler
- Release workflow now uses `fail-fast: false` so one target failure doesn’t cancel all
  other builds

## [0.2.2] (parity: flowmark-py 0.6.4)

Infrastructure release adding pre-built binaries for all major platforms.

### Features

- **Pre-built binaries** for 6 platforms via GitHub Releases: Linux (x86_64, ARM64),
  macOS (x86_64, ARM64), Windows (x86_64, ARM64). Archives include SHA256 checksums.
  `cargo binstall flowmark` now works automatically.
- **Windows CI** added to the test matrix (ubuntu + macOS + Windows)

### Fixes

- Fixed CRLF line-ending handling in golden tests for Windows compatibility
- Tryscript integration tests now correctly skip on Windows (bash-only)

## [0.2.1] (parity: flowmark-py 0.6.4)

Patch release fixing four formatting parity bugs discovered by corpus-wide comparison
against Python flowmark on 623 real-world files, plus new systematic parity testing
infrastructure.

### Fixes

- **Mixed loose/tight list code fences** (D12b): Code blocks inside loose list items no
  longer get spurious blank lines when the source had none
- **Blockquote blank line indentation** (D13): Blank separator lines inside blockquote
  lists now preserve the full list-content indent (e.g., `"> "`) instead of trimming to
  a bare `">"`
- **Smart quote after inline code** (D15): Apostrophes after code spans are now
  context-sensitive — `config`’s converts to a smart quote while `foo()`'s stays ASCII,
  matching Python’s behavior
- **Empty code blocks** (D16): Empty fenced code blocks no longer produce a spurious
  blank line between the opening and closing fences

### Testing

- New Python-generated golden file parity tests (`tests/parity/corner-cases.md`)
  covering all four fixed bugs across 5 formatting modes
- New cross-binary parity test suite comparing Rust and Python CLI output directly
- Parity verification scripts (`scripts/corpus-parity-check.sh`,
  `scripts/generate-parity-golden.sh`) for corpus-wide regression testing

## [0.2.0] (parity: flowmark-py 0.6.4)

First formal release.
Complete Rust port with full behavioral parity to Python flowmark v0.6.4.

### Highlights

- **Drop-in replacement** for Python flowmark — identical CLI interface, identical
  formatting output across all modes
- **Single binary, no runtime** — `cargo install flowmark`, no Python needed
- **Library crate** — embed formatting in Rust toolchains via `flowmark::FormatOptions`

### Features

- All formatting modes: default (width 88), semantic, auto, plaintext, custom width
- List spacing: preserve, tight, loose
- Typography: smart quotes, ellipsis conversion
- Cleanups: unbold headings
- File discovery: glob patterns, `.gitignore` support, `.flowmarkignore`, config file
  loading (`.flowmark.toml`, `flowmark.toml`, `pyproject.toml`)
- Batch multi-file processing with `--inplace` and `--auto`
- Claude Code skill integration (`--install-skill`, `--skill`)

### Parity Verification

- 430 tests (0 ignored, 0 failures)
- 292 Python tests mapped to Rust equivalents (CI-enforced)
- 15 parity discrepancies identified and resolved
- Golden reference document tested across 4 modes
- 11 tryscript golden tests for end-to-end CLI validation

### Infrastructure

- 12-check CI pipeline (fmt, clippy, test, MSRV, deny, docs, coverage, semver-checks)
- Automated crates.io publishing via trusted publishing (OIDC)
- Cross-platform testing (Ubuntu + macOS)
- Supply chain security via `deny.toml`

## [0.1.3] - 2025-11-01

Early development release.

[0.1.3]: https://github.com/jlevy/flowmark-rs/releases/tag/v0.1.3
[0.2.0]: https://github.com/jlevy/flowmark-rs/compare/v0.1.3...v0.2.0
[0.2.1]: https://github.com/jlevy/flowmark-rs/compare/v0.2.0...v0.2.1
[0.2.2]: https://github.com/jlevy/flowmark-rs/compare/v0.2.1...v0.2.2
[0.2.3]: https://github.com/jlevy/flowmark-rs/compare/v0.2.2...v0.2.3
[0.2.4]: https://github.com/jlevy/flowmark-rs/compare/v0.2.3...v0.2.4
[0.2.5]: https://github.com/jlevy/flowmark-rs/compare/v0.2.4...v0.2.5
[0.2.6]: https://github.com/jlevy/flowmark-rs/compare/v0.2.5...v0.2.6
[unreleased]: https://github.com/jlevy/flowmark-rs/compare/v0.2.6...HEAD
