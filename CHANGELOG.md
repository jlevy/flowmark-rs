# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased][unreleased]

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

[unreleased]: https://github.com/jlevy/flowmark-rs/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/jlevy/flowmark-rs/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/jlevy/flowmark-rs/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/jlevy/flowmark-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/jlevy/flowmark-rs/compare/v0.1.3...v0.2.0
[0.1.3]: https://github.com/jlevy/flowmark-rs/releases/tag/v0.1.3
