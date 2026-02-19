# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.2.0] (parity: flowmark-py 0.6.4)

First formal release.
Complete Rust port with full behavioral parity to Python flowmark v0.6.4.

### Highlights

- **Drop-in replacement** for Python flowmark — identical CLI interface, identical
  formatting output across all modes
- **Single binary, no runtime** — `cargo install flowmark`, no Python needed
- **Library crate** — embed formatting in Rust toolchains via
  `flowmark::FormatOptions`

### Features

- All formatting modes: default (width 88), semantic, auto, plaintext, custom width
- List spacing: preserve, tight, loose
- Typography: smart quotes, ellipsis conversion
- Cleanups: unbold headings
- File discovery: glob patterns, `.gitignore` support, `.flowmarkignore`,
  config file loading (`.flowmark.toml`, `flowmark.toml`, `pyproject.toml`)
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

[Unreleased]: https://github.com/jlevy/flowmark-rs/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/jlevy/flowmark-rs/compare/v0.1.3...v0.2.0
[0.1.3]: https://github.com/jlevy/flowmark-rs/releases/tag/v0.1.3
