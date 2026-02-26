# Project Status: flowmark-rs v0.2.0

**Last updated:** 2026-02-19

**Target release:** v0.2.0 (first formal release of the Rust port)

**Python parity target:** flowmark v0.6.4

## Overview

flowmark-rs is a complete Rust port of the Python
[flowmark](https://github.com/jlevy/flowmark) Markdown auto-formatter.
The port achieves full behavioral parity with Python flowmark v0.6.4 — identical CLI
interface, identical formatting output across all modes, and identical file discovery
behavior.

This is one of the first fully automated ports of a complex Python program to Rust,
managed through a systematic
[rust-porting-playbook](https://github.com/jlevy/rust-porting-playbook) methodology.
The port demonstrates that complex cross-language ports can be managed with rigorous
test-driven parity verification.

## Porting Principles Compliance

This port adheres to the 8 non-negotiable principles from the
[Porting Principles and Anti-Patterns](../repos/rust-porting-playbook/guidelines/porting-principles-and-antipatterns.md)
guide. Each principle was learned from an actual mistake during agent-driven porting.
None are hypothetical.

### Parity Definition (Principle 1)

Parity is defined as: **the Rust binary is a drop-in replacement for the Python binary
at the CLI level — identical flags, identical output, identical file discovery,
identical error behavior.**

**Tolerated variations** (closed list):
- Nonexistent file error format: Python uses `[Errno 2]` (a Python runtime artifact),
  Rust uses `Path not found:`. Both include `Error:` prefix and filename.
- `--help` layout: minor formatting differences between clap (Rust) and argparse
  (Python)
- `--version` output: Rust appends parity info
  (`flowmark 0.2.0 (parity: flowmark-py 0.6.4)`)

Everything not on this list is required to be identical.

### Active Parity Pursuit (Principle 2)

- 15 parity discrepancies discovered (D1-D15), all resolved
- Every discrepancy had a failing test before the fix
- Zero parity gaps hidden behind passing tests
- No passive documentation of gaps — every gap was treated as a severe blocker

### Tests Always Run in CI (Principle 3)

All 430 tests run in CI on every commit.
No test file is orphaned.
The CI test job installs all required external tools:
- Python flowmark v0.6.4 (via `uv tool install`)
- tryscript (via `npm install -g`)

### Tests Never Hide Failures (Principle 4)

- No “graceful skip” logic — tests fail loudly when dependencies are missing
- Golden test expected outputs come from the Python reference implementation
- No output truncation, path stripping, or assertion weakening
- D11 cross-binary parity tests invoke both Python and Rust binaries with identical
  arguments

### Fix the Process, Not the Test (Principle 5)

- Zero `#[ignore]` annotations in the entire test suite
- No tests disabled, commented out, or weakened
- When tests needed external tools, CI was updated to provide them

### Environment Dependencies Explicit and Enforced (Principle 6)

**Required test dependencies** (all installed in CI):
- Python flowmark v0.6.4 (`uv tool install flowmark==0.6.4`)
- tryscript (`npm install -g tryscript@latest`)
- Node.js 22+ (for tryscript)

CI workflows install all dependencies before running tests.
The publish workflow mirrors the test job’s dependency setup.

### Ignored Tests Tracked (Principle 7)

Zero `#[ignore]` annotations.
Zero ignored tests. No technical debt from silenced failures.

### Disparities Tested Before Fixed (Principle 8)

Every discrepancy followed the test-before-fix protocol:
1. Write test against Python’s behavior (expected output from Python)
2. Confirm test fails against Rust
3. Investigate the class of behavior (e.g., D4 tight list spacing led to investigating
   all list spacing modes)
4. Fix, verify test goes green

All 13 comrak library workarounds are documented with `COMRAK-WORKAROUNDn` labels in
`src/formatter/filling.rs` with rationale for each.

## Release Readiness

### What’s Done

| Area | Status | Details |
| --- | --- | --- |
| **Core formatting** | Complete | Identical output across all modes (default, semantic, auto, plaintext) |
| **List spacing** | Complete | Tight, loose, and preserve modes match Python exactly |
| **Typography** | Complete | Smart quotes and ellipsis conversion match Python |
| **File discovery** | Complete | Glob, gitignore, `.flowmarkignore`, config file loading |
| **Config loading** | Complete | `.flowmark.toml`, `flowmark.toml`, `pyproject.toml [tool.flowmark]` |
| **CLI interface** | Complete | All flags match Python, including `--auto`, `--inplace`, `--skill` |
| **Error handling** | Complete | Error messages match Python (see tolerated variations above) |
| **Library crate** | Complete | Public API via `FormatOptions::reformat_text()`, feature-gated CLI |
| **CI pipeline** | Complete | 12 checks: fmt, clippy, test (Ubuntu+macOS), lib-only, MSRV, deny, docs, coverage, semver, markdown-fmt, check-mapping |
| **Test coverage** | Complete | 430 tests, 0 ignored, 0 failures |
| **Test mapping** | Complete | 292 Python tests mapped, 0 excluded, 0 missing |
| **Parity verification** | Complete | All 15 discrepancies (D1-D15) resolved, 33 parity-specific tests |
| **crates.io metadata** | Complete | README, description, keywords, categories, documentation link |
| **Trusted publishing** | Complete | OIDC configured for crates.io |
| **Publish workflow** | Complete | `.github/workflows/publish.yml` with test-before-publish |
| **Documentation** | Complete | README, CONTRIBUTING, CHANGELOG, publishing guide, sync playbook |
| **Claude Code skill** | Complete | `--install-skill` and `--skill` flags working |

### Remaining Before Release

| Item | Priority | Bead | Notes |
| --- | --- | --- | --- |
| **First publish** | P0 | fmr-bfam | Tag `v0.2.0`, create GitHub Release → triggers publish workflow |
| **Binary release workflow** | P2 | fmr-q3pu | Pre-built binaries via cargo-dist — deferred |
| **Homebrew tap** | P3 | — | `brew install jlevy/tap/flowmark` — future work |
| **Shell completions** | P3 | — | bash/zsh/fish via `clap_complete` — future work |

### Release Checklist

- [x] All formatting modes match Python v0.6.4 exactly
- [x] All 430 tests pass (0 ignored, 0 failures)
- [x] CI pipeline green (12/12 checks)
- [x] `cargo publish --dry-run` succeeds
- [x] Trusted publishing (OIDC) configured on crates.io
- [x] Publish workflow includes full test suite with all external deps
- [x] README, CONTRIBUTING, CHANGELOG ready
- [ ] Create GitHub Release tagged `v0.2.0` (fmr-bfam)
- [ ] Verify crates.io publish succeeds
- [ ] Verify on https://crates.io/crates/flowmark

## Build & Quality Verification

Compliance verified 2026-02-19:

| Check | Result |
| --- | --- |
| `cargo fmt --check` | PASS |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | PASS (0 warnings) |
| `cargo doc --locked --no-deps --all-features` (with `-D warnings`) | PASS |
| `cargo test --all-features` | PASS (430 tests, 0 failures) |
| `cargo test --locked --no-default-features` | PASS (library-only) |
| `cargo publish --dry-run` | PASS |
| `#[ignore]` tests | 0 |
| `unsafe` code blocks | 1 (SIGPIPE handler in `main.rs`, annotated) |
| `FIXME`/`TODO`/`HACK` in source | 0 |
| `COMRAK-WORKAROUND` comments | 13 (all documented with rationale) |

## Test Summary

| Category | Count | Description |
| --- | --- | --- |
| Unit tests (in `src/`) | 46 | Module-level tests for parsers, formatters, wrappers |
| Integration tests (in `tests/`) | 372 | Full-pipeline formatting, CLI, file discovery |
| Doc tests | 1 | Library API usage example |
| Tryscript golden tests | 11 | End-to-end CLI behavior specs |
| **Total** | **430** | **0 ignored, 0 failures** |

### Parity Testing

- **33 parity-specific tests** verify exact output match with Python across all
  discrepancy areas (D1-D15)
- **5 D11 tests** invoke both Python and Rust binaries, comparing stderr and exit codes
- **Golden reference document** tested across 4 modes (default, semantic, auto,
  plaintext)
- **292 Python tests** have verified Rust counterparts — see
  [admin/port-coverage-mapping/](../admin/port-coverage-mapping/) for the full mapping

### Test Dependencies

Tests require the following external tools (all installed in CI):
- **Python flowmark v0.6.4** — for D11 cross-binary parity tests
  (`uv tool install flowmark==0.6.4`)
- **tryscript** — for golden CLI tests (`npm install -g tryscript@latest`)
- **Rust toolchain** — stable, MSRV 1.85

## Architecture

```
Input → [YAML Frontmatter] → [comrak Parse] → [Typography] → [Cleanups] → [fill_markdown] → Output
```

### Module Structure

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

### Key Design Decisions

- **Zero `unsafe` code** — `unsafe_code = "deny"` in Cargo.toml (1 exception: SIGPIPE
  handler)
- **No `unwrap()` in library** — `unwrap_used = "deny"`, all errors use `?` or
  `expect()` with messages
- **Feature-gated CLI** — library usable without clap/anyhow via `--no-default-features`
- **Pedantic clippy at deny level** — catches issues locally, not just in CI
- **Supply chain security** — `deny.toml` with license allowlist
- **Atomic file writes** — tempfile + persist pattern prevents corruption
- **13 comrak workarounds** — documented in `src/formatter/filling.rs` module docs, each
  tagged with `COMRAK-WORKAROUNDn` and rationale

## Documentation Index

### Root Documents

| Document | Description |
| --- | --- |
| [README.md](../README.md) | Project overview, installation, usage |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | Build, test, lint instructions |
| [CHANGELOG.md](../CHANGELOG.md) | Release notes |

### Operational Guides

| Document | Description |
| --- | --- |
| [docs/publishing.md](publishing.md) | Release process, crates.io, trusted publishing |
| [docs/port-sync-playbook.md](port-sync-playbook.md) | Sync with Python upstream, porting methodology, test mapping procedures |
| [tests/qa/rust-python-parity-e2e.qa.md](../tests/qa/rust-python-parity-e2e.qa.md) | Manual end-to-end QA playbook for parity, tryscript sanity, and docs/version alignment |
| [admin/](../admin/) | Port administration: test mapping data, dev tools overview |

### Specifications (Active)

| Document | Status | Description |
| --- | --- | --- |
| [Exact Parity](project/specs/active/plan-2026-02-17-exact-parity.md) | **Complete** | Full parity requirements and verification |
| [Parity Discrepancies](project/specs/active/plan-2026-02-18-parity-discrepancies.md) | **Complete** | All 15 discrepancies (D1-D15) resolved |
| [Build & Publishing](project/specs/active/plan-2026-02-17-build-publishing.md) | **Phases 1-4,6 Done** | CI, crates.io, publish workflow (Phase 5: binary releases deferred) |
| [Tryscript Golden Tests](project/specs/active/plan-2026-02-17-comprehensive-tryscript-golden-tests.md) | **Implemented** | Comprehensive CLI golden test suite |
| [Test Mapping](project/specs/active/plan-2026-02-17-test-mapping-meta-test.md) | **Implemented** | Cross-language test provenance tracking |
| [Code Review](project/specs/active/code-review-2026-02-17.md) | **Complete** | Senior code review with P0-P3 issues (P0-P1 fixed) |
| [Playbook Sync](project/specs/active/plan-2026-02-17-playbook-review-sync.md) | Draft | Bidirectional doc sync with porting playbook |

### Specifications (Done)

| Document | Description |
| --- | --- |
| [Porting Plan](project/specs/done/porting-plan.md) | Original porting plan (all phases complete) |

### Test Provenance

| Document | Description |
| --- | --- |
| [admin/port-coverage-mapping/](../admin/port-coverage-mapping/) | Python-to-Rust test mapping (292 tests) |
| [admin/port-coverage-mapping/test-mapping.yaml](../admin/port-coverage-mapping/test-mapping.yaml) | Hand-maintained 1:1 and 1:N test mappings |

### Porting Playbook (Submodule)

The [rust-porting-playbook](https://github.com/jlevy/rust-porting-playbook) is available
at `repos/rust-porting-playbook/` with these key documents:

| Document | Description |
| --- | --- |
| [Porting Principles](../repos/rust-porting-playbook/guidelines/porting-principles-and-antipatterns.md) | 8 non-negotiable principles for agent-driven porting |
| [Porting Rules](../repos/rust-porting-playbook/guidelines/python-to-rust-porting-rules.md) | Type mappings, patterns, and acceptance criteria |
| [Test Coverage](../repos/rust-porting-playbook/guidelines/test-coverage-for-porting.md) | Test strategy, coverage targets, cross-validation |
| [Python-to-Rust Playbook](../repos/rust-porting-playbook/playbooks/python-to-rust-playbook.md) | Step-by-step porting process |
| [Sync Release Workflow](../repos/rust-porting-playbook/playbooks/python-to-rust-sync-release-workflow.md) | Two-stage release refresh process for existing ports |
| [Code Review Checklist](../repos/rust-porting-playbook/playbooks/rust-code-review-checklist.md) | Rust code review checklist for ports |

## Porting Methodology

This port was built using the
[rust-porting-playbook](https://github.com/jlevy/rust-porting-playbook), following an
8-phase methodology:

1. **Analysis** — Understand Python source structure and dependencies
2. **Scaffolding** — Set up Rust project with matching module structure
3. **Core porting** — Translate Python logic to idiomatic Rust
4. **Test porting** — Map all Python tests to Rust equivalents
5. **Parity verification** — Golden test comparison across all modes
6. **CI hardening** — 12-check pipeline with coverage and semver checks
7. **Documentation** — Specs, playbooks, and operational guides
8. **Publishing** — crates.io metadata, trusted publishing, release workflow

### Key Metrics

| Metric | Value |
| --- | --- |
| Python source lines | 5,279 |
| Rust source lines | ~3,500 |
| Rust/Python LOC ratio | 0.66x (Rust is more concise) |
| Python test functions | 292 |
| Rust test functions | 430 |
| Parity discrepancies found | 15 |
| Parity discrepancies resolved | 15 (100%) |
| comrak library workarounds | 13 (all documented) |
| Ignored tests | 0 |
| `unsafe` code | 1 block (SIGPIPE handler, annotated) |
| `FIXME`/`TODO`/`HACK` | 0 |

## CI Pipeline

| Job | What It Checks |
| --- | --- |
| `fmt` | `cargo fmt --check` |
| `clippy` | Pedantic clippy with `-D warnings` |
| `test` (Ubuntu + macOS) | Full test suite with Python parity + tryscript golden tests |
| `test-lib-only` | Library builds and tests without CLI feature |
| `msrv` | Compiles on minimum supported Rust version (1.85) |
| `deny` | License allowlist and supply chain audit |
| `docs` | `cargo doc` with `-D warnings` |
| `coverage` | `cargo-llvm-cov` with Codecov upload |
| `semver-checks` | API breakage detection (PRs only) |
| `markdown-fmt` | Markdown formatting consistency |
| `check-mapping` | Test mapping completeness (292/292) |

## Version Convention

Each release documents which Python version it targets:

> flowmark v0.2.0 (parity: flowmark-py v0.6.4)

The Rust version follows its own semver independently.
The parity note indicates which Python version’s behavior is fully covered.
