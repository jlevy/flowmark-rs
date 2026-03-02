# Project Status: flowmark-rs

**Last updated:** 2026-03-02

**Current release:** v0.2.5
([crates.io](https://crates.io/crates/flowmark),
[PyPI](https://pypi.org/project/flowmark-rs/),
[Homebrew](https://github.com/jlevy/homebrew-flowmark))

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
- `--version` output: Rust includes explicit port/version provenance metadata
  (`flowmark 0.2.0-dev.<N>+g<hash> (Rust port of flowmark-py 0.6.4; base v0.2.0)`)

Everything not on this list is required to be identical.

### Active Parity Pursuit (Principle 2)

- 15 parity discrepancies discovered (D1-D15), all resolved
- Every discrepancy had a failing test before the fix
- Zero parity gaps hidden behind passing tests
- No passive documentation of gaps — every gap was treated as a severe blocker

### Tests Always Run in CI (Principle 3)

All 470+ tests run in CI on every commit.
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

## Release Status

All release channels are live as of v0.2.5:

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
| **CI pipeline** | Complete | 12 checks: fmt, clippy, test (Ubuntu+macOS), lib-only, MSRV, deny, docs, coverage, semver, markdown-fmt, check-mapping, readme-sync |
| **Test coverage** | Complete | 470+ tests, 0 ignored, 0 failures |
| **Test mapping** | Complete | 292 Python tests mapped, 0 excluded, 0 missing |
| **Parity verification** | Complete | All 15 discrepancies (D1-D15) resolved, 33 parity-specific tests |
| **crates.io** | Live | [crates.io/crates/flowmark](https://crates.io/crates/flowmark) |
| **PyPI** | Live | [pypi.org/project/flowmark-rs](https://pypi.org/project/flowmark-rs/) (`uvx flowmark-rs`) |
| **GitHub Releases** | Live | Pre-built binaries for macOS, Linux, Windows |
| **Homebrew tap** | Live | `brew install jlevy/flowmark/flowmark` |
| **Trusted publishing** | Complete | OIDC configured for crates.io and PyPI |
| **Documentation** | Complete | README, CONTRIBUTING, CHANGELOG, publishing guide, sync playbook |
| **Claude Code skill** | Complete | `--install-skill` and `--skill` flags working |

### Future Work

| Item | Priority | Notes |
| --- | --- | --- |
| Shell completions | P3 | bash/zsh/fish via `clap_complete` |

## Test Summary

| Category | Description |
| --- | --- |
| Unit tests (`src/`) | Module-level tests for parsers, formatters, wrappers |
| Integration tests (`tests/`) | Full-pipeline formatting, CLI, file discovery |
| Doc tests | Library API usage example |
| Tryscript golden tests | End-to-end CLI behavior specs |
| D11 parity tests | Cross-binary comparison (invokes both Python and Rust) |
| **Total** | **470+ tests, 0 ignored, 0 failures** |

### Parity Testing

- **33 parity-specific tests** verify exact output match with Python across all
  discrepancy areas (D1-D15)
- **5 D11 tests** invoke both Python and Rust binaries, comparing stderr and exit codes
- **Golden reference document** tested across 4 modes (default, semantic, auto,
  plaintext)
- **292 Python tests** have verified Rust counterparts (CI-enforced via test mapping)

For build/test/lint instructions, see [`docs/development.md`](development.md).

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

## Related Documentation

| Document | Purpose |
| --- | --- |
| [`docs/development.md`](development.md) | Building, testing, linting, project structure, CI pipeline |
| [`docs/port-sync-playbook.md`](port-sync-playbook.md) | Syncing with Python upstream, test mapping, porting methodology |
| [`docs/publishing.md`](publishing.md) | Release process (crates.io, PyPI, GitHub Releases, Homebrew) |
| [`docs/porting-log-review.md`](porting-log-review.md) | Bug log and lessons learned from the porting process |
| [`tests/qa/rust-python-parity-e2e.qa.md`](../tests/qa/rust-python-parity-e2e.qa.md) | Manual end-to-end QA playbook |

### Specifications (all complete)

| Document | Description |
| --- | --- |
| [Porting Plan](project/specs/done/porting-plan.md) | Original porting plan (all phases complete) |
| [Exact Parity](project/specs/done/plan-2026-02-17-exact-parity.md) | Full parity requirements and verification |
| [Parity Discrepancies](project/specs/done/plan-2026-02-18-parity-discrepancies.md) | All 15 discrepancies (D1-D15) resolved |
| [Build & Publishing](project/specs/done/plan-2026-02-17-build-publishing.md) | CI, crates.io, PyPI, release workflows |
| [Tryscript Golden Tests](project/specs/done/plan-2026-02-17-comprehensive-tryscript-golden-tests.md) | Comprehensive CLI golden test suite |
| [Test Mapping](project/specs/done/plan-2026-02-17-test-mapping-meta-test.md) | Cross-language test provenance tracking |
| [Code Review](project/specs/active/code-review-2026-02-17.md) | Senior code review (P0-P1 fixed) |

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
| Rust test functions | 470+ |
| Parity discrepancies found | 15 |
| Parity discrepancies resolved | 15 (100%) |
| comrak library workarounds | 13 (all documented) |
| Ignored tests | 0 |
| `unsafe` code | 1 block (SIGPIPE handler, annotated) |
| `FIXME`/`TODO`/`HACK` | 0 |

## Version Convention

Each release documents which Python version it targets, and dev builds include
commits-ahead and git hash metadata:

> flowmark 0.2.5-dev.<N>+g<hash> (Rust port of flowmark-py 0.6.4; base v0.2.5)

The Rust version follows its own semver independently.
The port note indicates which Python version’s behavior is fully covered.
