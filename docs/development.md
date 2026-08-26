# Development

> **Doc status:** Parallel to upstream
> [`docs/development.md`](https://github.com/jlevy/flowmark/blob/main/docs/development.md).
> Both cover the same role; content differs because the Rust toolchain (cargo,
> multi-channel release) and the Python toolchain (uv, PyPI-only) require different
> commands and steps.

How to build, test, and work on flowmark-rs.

## Prerequisites

- **Rust 1.85+** (see `rust-version` in `Cargo.toml` for MSRV)
- **Git submodules**, initialized with `git submodule update --init --recursive`
- **Node.js 22+** and **tryscript 0.1.7** for golden CLI tests
  (`npm install -g tryscript@0.1.7`)

All test dependencies are required.
Tests fail loudly when a dependency is missing, there is no skip logic.

## Building

```bash
cargo build --all-features
```

The `--all-features` flag enables the `cli` feature gate, which includes the binary
entry point and clap/anyhow dependencies.
Without it, only the library crate is built.

## Testing

```bash
cargo test --all-features
```

The full suite includes:

| Category | Description |
| --- | --- |
| Unit tests (`src/`) | Module-level tests for parsers, formatters, wrappers |
| Integration tests (`tests/`) | Full-pipeline formatting, CLI, file discovery |
| Doc tests | Library API usage example |
| Shared conformance tests | Exact process and filesystem behavior from the pinned upstream manifest |
| Shared tryscript tests | End-to-end CLI behavior from the pinned upstream suite |
| Rust-only golden tests | Rust-only features and regressions that have no portable source equivalent |

The native Rust adapter runs every portable case against the Cargo-built binary.
It reads inputs and reviewed outputs directly from `repos/flowmark`; it does not invoke
Python, regenerate expected output, normalize bytes, or copy fixtures into this
repository.
Exact known differences live in `tests/parity_corpus_known_divergences.toml`,
and the runner fails for both unlisted differences and entries that have become stale.

Zero `#[ignore]` annotations.
Zero skipped tests.

## Linting

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

The project uses pedantic clippy lints at deny level.
`unsafe_code` and `unwrap_used` are denied in `Cargo.toml`.

## Project Structure

```
Input → [YAML Frontmatter] → [comrak Parse] → [Typography] → [Cleanups] → [fill_markdown] → Output
```

| Module | Python Source | Purpose |
| --- | --- | --- |
| `formatter/filling.rs` | `flowmark/filling.py` | Core Markdown rendering pipeline |
| `formatter/markdown.rs` | `flowmark/formats/flowmark_markdown.py` | comrak configuration and workarounds |
| `parser/frontmatter.rs` | `flowmark/frontmatter.py` | YAML frontmatter preservation |
| `wrapping/text_filling.rs` | `flowmark/text_filling.py` | Plaintext wrapping |
| `wrapping/text_wrapping.rs` | `flowmark/text_wrapping.py` | Markdown-aware word splitting |
| `wrapping/sentence.rs` | `flowmark/sentence.py` | Sentence boundary detection |
| `wrapping/line_wrappers.rs` | `flowmark/line_wrappers.py` | Line wrapper composition |
| `transform/cleanups.rs` | `flowmark/cleanups.py` | Safe document cleanups |
| `typography/quotes.rs` | `flowmark/smartquotes.py` | Smart quote conversion |
| `typography/ellipses.rs` | `flowmark/ellipses.py` | Ellipsis conversion |
| `file_resolver/` | `flowmark/file_resolver.py` | File discovery and filtering |
| `config.rs` | `flowmark/config.py` | TOML config loading, three-way merge |
| `lib.rs` | `flowmark/__init__.py` | Public API |
| `main.rs` | `flowmark/__main__.py` | CLI entry point |

Key design decisions:

- **Feature-gated CLI:** library usable without clap/anyhow via `--no-default-features`
- **Zero `unsafe` code:** `unsafe_code = "deny"` (1 exception: SIGPIPE handler in
  `main.rs`)
- **No `unwrap()` in library:** `unwrap_used = "deny"`; all errors use `?` or `expect()`
  with messages
- **Pedantic clippy at deny level:** catches issues locally, not just in CI
- **Atomic file writes:** tempfile + persist pattern prevents corruption
- **13 comrak workarounds:** each tagged `COMRAK-WORKAROUNDn` in
  `src/formatter/filling.rs`

## CI Pipeline

The CI runs 12 checks on every push and PR:

| Job | What It Checks |
| --- | --- |
| `fmt` | `cargo fmt --check` |
| `clippy` | Pedantic clippy with `-D warnings` |
| `test` (Ubuntu, macOS, and Windows) | Full Rust suite against the pinned upstream assets |
| `test-lib-only` | Library builds and tests without CLI feature |
| `msrv` | Compiles on minimum supported Rust version (1.85) |
| `deny` | License allowlist and supply chain audit |
| `docs` | `cargo doc` with `-D warnings` |
| `coverage` | `cargo-llvm-cov` with Codecov upload |
| `semver-checks` | API breakage detection (PRs only) |
| `markdown-fmt` | Markdown formatting consistency |
| `check-mapping` | Supplementary function-level test provenance and shared change-ID traceability |
| `readme-sync` | README generation stays in sync with template |

## Configuration

Flowmark supports TOML-based configuration.
It searches for config files in this order (first match wins, walking up directories):

1. `.flowmark.toml`
2. `flowmark.toml`
3. `pyproject.toml` (only if it has a `[tool.flowmark]` section)

A `.flowmarkignore` file (gitignore syntax) can exclude paths from formatting.

## Rust-Only Features

The Rust port adds performance features not present in Python:

- Incremental cache (`--no-cache`, `--cache-dir`, `--show-cache`, `--clear-cache`)
- Parallel file processing (`--threads`)
- Stage-level performance stats (`--perf-stats`)

See [`docs/rust-only-features.md`](rust-only-features.md) and
[`docs/cache.md`](cache.md) for details.

## Releasing

See [`docs/publishing.md`](publishing.md) for the full release runbook (crates.io, PyPI,
GitHub Releases, Homebrew tap).

## Port Sync

For syncing with new Python flowmark upstream releases, see
[`docs/port-sync-playbook.md`](port-sync-playbook.md).

For the full port overview and parity verification history, see
[`docs/port-status.md`](port-status.md).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
