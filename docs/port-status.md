# Project Status: flowmark-rs

**Last updated:** 2026-05-28

**Current release:** v0.3.0 ([crates.io](https://crates.io/crates/flowmark),
[PyPI](https://pypi.org/project/flowmark-rs/),
[Homebrew](https://github.com/jlevy/homebrew-flowmark))

**Python parity target:** flowmark v0.7.0

## Overview

flowmark-rs is a complete Rust port of the Python
[flowmark](https://github.com/jlevy/flowmark) Markdown auto-formatter.
The port achieves full behavioral parity with Python flowmark v0.7.0 — identical CLI
interface, identical formatting output across all modes, and identical file discovery
behavior.

This is one of the first fully automated ports of a complex Python program to Rust,
managed through a systematic
[rust-porting-playbook](https://github.com/jlevy/rust-porting-playbook) methodology.
The port demonstrates that complex cross-language ports can be managed with rigorous
test-driven parity verification.

## Porting Principles Compliance

This port adheres to the 8 non-negotiable principles from the
[Porting Principles and Anti-Patterns](https://github.com/jlevy/rust-porting-playbook/blob/main/guidelines/porting-principles-and-antipatterns.md)
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
  (`flowmark 0.2.0-dev.<N>+g<hash> (Rust port of flowmark-py 0.7.0; base v0.2.0)`)

Everything not on this list is required to be identical.

### Active Parity Pursuit (Principle 2)

- 18 parity discrepancies discovered (D1-D18), all resolved
- Every discrepancy had a failing test before the fix
- Zero parity gaps hidden behind passing tests
- No passive documentation of gaps — every gap was treated as a severe blocker

### Tests Always Run in CI (Principle 3)

All 501 tests run in CI on every commit.
No test file is orphaned.
The CI test job installs all required external tools:

- Python flowmark v0.7.0 (via `uv tool install`)
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

- Python flowmark v0.7.0 (`uv tool install flowmark==0.7.0`)
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

All 13 numbered comrak library workarounds (COMRAK-WORKAROUND1–13, 66 comment
references) are documented with `COMRAK-WORKAROUNDn` labels in
`src/formatter/filling.rs` with rationale for each.

## Release Status

All release channels are live as of v0.3.0:

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
| **Test coverage** | Complete | 501 tests, 0 ignored, 0 failures |
| **Test mapping** | Complete | 323 Python tests mapped, 24 excluded (Python library API), 0 missing |
| **Parity verification** | Complete | All 18 discrepancies (D1-D18) resolved, 48 parity-specific tests |
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
| **Total** | **501 tests, 0 ignored, 0 failures** |

### Parity Testing

- **48 parity-specific tests** verify exact output match with Python across all
  discrepancy areas (D1-D18)
- **5 D11 tests** invoke both Python and Rust binaries, comparing stderr and exit codes
- **Golden reference document** tested across 4 modes (default, semantic, auto,
  plaintext)
- **347 Python tests** tracked (323 mapped, 24 excluded as Python library API;
  CI-enforced)

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
| [Code Review](project/specs/done/code-review-2026-02-17.md) | Senior code review (P0-P1 fixed) |

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
| Python total LOC | ~12,400 (~4,250 app + ~5,800 test + ~1,770 golden) |
| Rust total LOC | ~16,150 (~7,250 app + ~7,300 test + ~1,520 golden) |
| Rust/Python app ratio | ~1.7x |
| Python test functions | 347 |
| Rust test functions | 516 (manifest; 501 cargo-discovered) |
| Parity discrepancies found | 18 |
| Parity discrepancies resolved | 18 (100%) |
| comrak library workarounds | 13 numbered (66 comment references) |
| Ignored tests | 0 |
| `unsafe` code | 1 block (SIGPIPE handler, annotated) |
| `FIXME`/`TODO`/`HACK` | 0 |

## Process Retrospective

### Task Management with tbd

All work in this port was managed using [tbd](https://github.com/jlevy/get-tbd), a
git-native issue tracking tool.
tbd tracks work as “beads” — lightweight, dependency-aware tasks that live alongside the
code in git. Every bug, feature, spec phase, and code review finding was tracked as a
bead, giving full traceability from requirement to implementation.
This systematic decomposition was essential for managing a port of this complexity.

**235 issues** were tracked across the full project lifecycle: 225 closed, 10 remaining
(optional/future work).

### Process Timeline

1. **Foundation and architecture**: Master porting plan established module boundaries,
   dependencies, and acceptance goals.
2. **Coverage and parity system**: Cross-language test mapping tooling created for
   Python→Rust provenance and CI enforcement.
3. **Exact parity delivery**: High-priority output mismatches fixed (P and D discrepancy
   tracks), ending in byte-for-byte parity claims with broad test backing.
4. **Code review hardening**: External review findings converted into concrete
   engineering tasks and lint policy tightening.
5. **CI and release engineering**: Build and publish workflows matured from crate-only
   to full multi-channel release orchestration.
6. **CLI and docs polish**: Help/usage UX improved and documentation synchronization
   tightened.
7. **Performance phase**: Baseline profiling quantified bottlenecks; parallel processing
   delivered major throughput gains; incremental cache implemented as follow-on
   optimization.
8. **Ecosystem and playbook feedback**: Porting lessons were fed back into reusable
   playbook guidance and process templates.

### Workstreams

**Port architecture and core formatter** — Established Rust module architecture
(`formatter`, `wrapping`, `parser`, `transform`, `typography`, `file_resolver`,
`skills`). Reimplemented behavior on top of `comrak` with custom rendering and
parity-oriented post-processing.
Preserved CLI/library split with feature-gated CLI and strict lint posture.

**Parity and correctness** — Implemented cross-language test mapping lifecycle and CI
checks. Closed discrepancy tracks across plaintext, list spacing, blockquote/footnote,
wrapping edge cases, and CLI error compatibility.
Eliminated test masking patterns so gaps fail visibly instead of hiding behind weak
assertions. Expanded tryscript/golden strategy as executable parity contract.

**Engineering quality and CI discipline** — Adopted strict lint policy (`warnings=deny`,
pedantic clippy posture).
Enforced formatter consistency and warning-free builds in CI. Added/strengthened checks:
docs build, coverage, semver checks, dependency audit, workflow script tests,
multi-platform test matrix.

**Packaging and release operations** — Crates.io trusted publishing path established.
GitHub Release artifact packaging/checksum flow implemented.
Homebrew tap flow established and codified.
PyPI/maturin wheel+sdist distribution path integrated.
Release orchestration moved to reusable workflows plus script-driven planning logic.

**Performance and scalability** — Baseline profiling measured major speedups vs Python
and identified string/alloc-heavy hotspots.
Parallel file processing added (rayon + threading controls + skip-unchanged behavior).
Incremental cache architecture implemented with invalidation/fingerprint behavior and
supporting tests.

**Documentation and playbook sync** — Port status and planning docs evolved from active
execution to completed record.
Playbook sync spec captured lessons learned and transformed them into reusable guidance.
Publishing process consolidated into a canonical operational runbook.

### What Worked Well

- Spec-first execution with explicit goals/non-goals kept scope controlled.
- Bead decomposition turned large parity/release work into tractable units.
- TDD and parity-first gates prevented silent regressions.
- Tooling quality (mapping checker, tryscript, CI workflows, script tests) made
  correctness auditable.
- Performance work was evidence-driven (benchmarks + profiling before optimization).

### Challenges and How They Were Addressed

1. **Parser/rendering semantics mismatch** (Marko vs comrak): Addressed by targeted
   rendering logic, explicit discrepancy tracking, and parity tests for each discovered
   gap.
2. **Hidden parity drift risk**: Addressed by forbidding masking patterns and requiring
   failing tests per known gap.
3. **Multi-channel release complexity**: Addressed by script-driven workflow planning,
   idempotent channel behavior, orchestrated release flows, and explicit manual Homebrew
   checkpoint.
4. **Documentation/status drift**: Addressed by moving specs to done and consolidating
   process docs.

### Key Takeaways

The strongest patterns worth reusing in future Rust/Python ports:

1. Parity-first acceptance with explicit discrepancy IDs.
2. Spec-driven work decomposition into bead-sized units.
3. Mapping and tryscript-based cross-language test contracts.
4. Strict CI gates and script-tested release orchestration.
5. Benchmark/profiling-led performance work after correctness stabilization.

### Full Issue Overview

Complex spec-driven tasks are tracked with beads via
[**tbd**](https://github.com/jlevy/tbd).

The complete issue history below (from `tbd list --all --pretty`) shows every task, bug,
feature, and epic tracked during the port, organized hierarchically with dependency
structure. Summary: 158 tasks, 61 bugs, 9 features, 7 epics.

```
fmr-unf9      P0  ✓ closed  [task] Set up Rust project structure (Cargo.toml, module hierarchy)
fmr-oozb      P0  ✓ closed  [task] Port all Python tests to Rust
fmr-nseu      P0  ✓ closed  [task] Implement fill_markdown pipeline with all normalization fixups
fmr-7imb      P0  ✓ closed  [task] Achieve 100% test conformance
fmr-kd36      P0  ✓ closed  [epic] Spec: Exact Cross-Language Parity
├── fmr-qr8x      P0  ✓ closed  [task] Complete test-mapping.yaml with all 281 entries
├── fmr-ie4p      P1  ✓ closed  [task] Run check-mapping and capture baseline gap report
├── fmr-flji      P1  ✓ closed  [task] Port 15 missing tag formatting tests (test_tag_formatting.rs)
├── fmr-a3zl      P1  ✓ closed  [task] Port 32 missing wrapping tests (test_wrapping.rs)
├── fmr-3ugc      P1  ✓ closed  [task] Port 5 missing escape handling tests (test_escape_handling.rs)
├── fmr-fph3      P1  ✓ closed  [task] Port 7 scattered missing tests (alerts, strikethrough, heading, code blocks, width)
├── fmr-361w      P2  ✓ closed  [task] Review smartquotes for behavioral parity
├── fmr-xqvt      P1  ✓ closed  [task] Verify: check-mapping exits 0, all cargo tests pass, golden test matches
└── fmr-y9ri      P1  ✓ closed  [task] Port 5 missing smartquotes integration tests (test_smartquotes.rs)
fmr-n1w9      P0  ✓ closed  [bug] Fix 9 clippy inefficient_to_string errors in filling.rs and tag_handling.rs
fmr-5gkb      P0  ✓ closed  [task] Run cargo fmt to fix all formatting violations
fmr-5255      P0  ✓ closed  [task] Tighten lints: warnings=deny, clippy pedantic=deny in Cargo.toml
fmr-lko7      P1  ✓ closed  [task] Implement core modules (error, config, frontmatter)
fmr-kpx9      P1  ✓ closed  [task] Implement text wrapping and sentence splitting
fmr-qyt2      P1  ✓ closed  [task] Implement Markdown formatter with comrak
fmr-olud      P1  ✓ closed  [task] Implement typography transforms (smart quotes, ellipses)
fmr-g3xo      P1  ✓ closed  [task] Implement document transforms and cleanups
fmr-1ydm      P1  ✓ closed  [task] Implement reformat API and CLI
fmr-vyuf      P1  ✓ closed  [feature] Build test mapping discovery scripts and meta-test infrastructure
fmr-vy2e      P1  ✓ closed  [task] Populate test_mapping.json with verified mappings for all Python tests
fmr-rrqw      P1  ✓ closed  [epic] Spec: Cross-Language Test Mapping (Port Coverage)
├── fmr-c6fy      P1  ✓ closed  [task] Phase 1: Python project scaffold and discovery CLI prototype
├── fmr-3em6      P1  ✓ closed  [task] Switch discover-rust to cargo-based discovery with regex fallback
├── fmr-wd0h      P1  ✓ closed  [task] Add idempotent merge to discover-python and discover-rust
├── fmr-gmil      P1  ✓ closed  [task] Re-generate YAML artifacts with full 178-test Rust manifest
├── fmr-0t2p      P2  ✓ closed  [task] Run ruff and basedpyright, fix lint/type issues
├── fmr-os5n      P1  ✓ closed  [task] Populate test-mapping.yaml with verified mappings for all Python tests
├── fmr-bt5r      P1  ✓ closed  [task] CI integration: run check-mapping in CI pipeline
└── fmr-2n3f      P1  ✓ closed  [task] Add Python smoke test for YAML round-trip serialization
fmr-5ojk      P1  ✓ closed  [bug] Bug: Extra blank line before HTML comment tag on list continuation line
fmr-2tll      P1  ✓ closed  [bug] Bug: Escape at start of list item content not preserved (1\. removed instead of kept)
fmr-4l1x      P1  ✓ closed  [bug] Bug: Extra blank line before heading in list item with hard break
fmr-p2pr      P1  ✓ closed  [task] Complete partial test: test_other_escaped_chars (add dollar, underscore, bracket, backtick assertions)
fmr-b5gl      P1  ✓ closed  [task] Fix 70 clippy warnings across lib and bin (doc_markdown, format_push_string, manual_repeat_n, etc.)
fmr-0uh0      P1  ✓ closed  [task] P1: Refactor main() error handling - anyhow, ExitCode, SIGPIPE
fmr-e8zb      P1  ✓ closed  [task] P1: Wire up atomic file writes with tempfile
fmr-ft54      P1  ✓ closed  [task] P1: Fix CI workflow and add deny.toml
fmr-qk59      P1  ✓ closed  [task] P1: Wire up tracing or remove unused deps
fmr-ow2t      P1  ✓ closed  [epic] Code review: address all findings from 2026-02-17 review
fmr-wcjh      P1  ✓ closed  [task] Add RUSTFLAGS=-D warnings to CI test jobs
fmr-y3zv      P1  ✓ closed  [task] Remove dead dependencies: toml, serde, unicode-segmentation from Cargo.toml
fmr-oodm      P1  ✓ closed  [task] Remove dead error variants Error::Config and Error::Other from error.rs
fmr-vuwg      P1  ✓ closed  [task] Review, sync, and improve porting playbook
fmr-mk46      P1  ✓ closed  [task] CI: Add --locked flag to clippy job to prevent Cargo.lock drift
fmr-9eda      P1  ✓ closed  [task] CI: Add --locked flag to docs job to prevent Cargo.lock drift
fmr-8yos      P1  ○ open   [epic] Build, CI hardening, and publishing improvements
├── fmr-eldq      P1  ✓ closed  [task] Phase 5.1: Create release.yml workflow for cross-platform binary builds
├── fmr-dqqo      P1  ✓ closed  [task] Phase 5.2: Add Windows CI testing to ci.yml
├── fmr-rg6a      P2  ✓ closed  [task] Phase 5.3: Update README.md and docs/publishing.md for binary releases
├── fmr-9dh1      P1  ✓ closed  [task] Phase 5.4: Test full release cycle with a patch release
├── fmr-s112      P1  ○ open   [epic] Distribute flowmark-rs on PyPI via maturin
├── fmr-2dho      P1  ✓ closed  [task] 1.1: Create root pyproject.toml with maturin config
├── fmr-8a2d      P1  ✓ closed  [task] 1.2: Update .gitignore for maturin artifacts
├── fmr-i1c6      P1  ✓ closed  [task] 1.3: Update Cargo.toml exclude list for pyproject.toml
├── fmr-mnbj      P1  ✓ closed  [task] 1.4: Local maturin build test (verify wheel with both binaries)
├── fmr-ujk0      P1  ✓ closed  [task] 1.5: Local install test (maturin develop, verify both commands)
├── fmr-1j2b      P1  ✓ closed  [task] 2.1: Create .github/workflows/pypi.yml with 5-target build matrix
├── fmr-jckq      P1  ✓ closed  [task] 2.2: Test pypi.yml workflow via workflow_dispatch (dry run)
├── fmr-drao      P1  ✓ closed  [task] 3.1: Register flowmark-rs on PyPI (pending trusted publisher)
├── fmr-ipyh      P1  ✓ closed  [task] 3.2: Create GitHub release environment for PyPI OIDC
├── fmr-en1o      P2  ○ open   [task] 3.3: (Optional) Test with TestPyPI first
├── fmr-2sp2      P1  ✓ closed  [task] 3.4: First real PyPI publish via GitHub Release trigger
├── fmr-foeq      P1  ○ open   [task] 3.5: Verify installation (uvx, uv tool install, pip install)
├── fmr-ca7s      P2  ✓ closed  [task] 4.1: Update README.md with PyPI install methods (uvx, pip)
├── fmr-3w2f      P2  ✓ closed  [task] 4.2: Update docs/publishing.md with PyPI workflow docs
├── fmr-2b14      P2  ✓ closed  [task] 4.3: Update build-publishing spec with PyPI cross-reference
├── fmr-sgqv      P3  ○ open   [task] 4.4: (Optional) Add musl targets to pypi.yml for Alpine
├── fmr-gl6v      P2  ✓ closed  [task] 4.5: Update flowmark Python repo README with Rust migration note
├── fmr-b11z      P3  ○ open   [task] 4.6: (Optional) Add Python wrapper for python -m flowmark_rs
├── fmr-dww0      P2  ✓ closed  [task] 5.1: Add PyPI distribution guide to rust-porting-playbook
├── fmr-43p2      P2  ✓ closed  [task] 5.2: Add process recommendations for Rust CLI PyPI distribution
└── fmr-4h35      P2  ✓ closed  [task] 5.3: Add reference projects comparison table to playbook
├── fmr-rj25      P3  ○ open   [task] CI: Consider cargo-nextest for faster parallel test execution
└── fmr-l15v      P3  ○ open   [task] Add Codecov integration: configure CODECOV_TOKEN secret and restore badge
fmr-7mmt      P1  ✓ closed  [epic] Phase 10: CLI & Feature Parity — exact drop-in replacement for Python flowmark
└── fmr-5u8i      P3  ○ open   [bug] Python plaintext mode uses html_md_word_splitter instead of simple_word_splitter
fmr-t834      P1  ✓ closed  [task] 10.1: Port file resolver module (31 tests)
fmr-z8j5      P1  ✓ closed  [task] 10.2: Port config loading — TOML, three-way merge (20 tests)
fmr-4sc5      P1  ✓ closed  [task] 10.3: CLI flag parity — add 11 missing flags (19 tests)
fmr-qa6p      P1  ✓ closed  [task] 10.3b: Port skill system — --skill, --install-skill, --docs (9 tests)
fmr-t3va      P1  ✓ closed  [task] 10.4: Tryscript CLI golden tests — baseline against Python, replicate for Rust
fmr-v2de      P1  ✓ closed  [task] 10.5: Update test mapping and CI — 281 mapped, 0 excluded, tryscript CI job
fmr-h01s      P1  ✓ closed  [task] 10.7: Final acceptance — review all 281+ mappings, sign off on completeness
fmr-fvw7      P1  ✓ closed  [task] Senior engineering review: code quality and correctness issues
fmr-he1d      P1  ✓ closed  [bug] Regex compiled in loop in min_fence_length (filling.rs:961)
fmr-86se      P1  ✓ closed  [bug] usize underflow in fill_text width calculation (text_filling.rs:103)
fmr-a4on      P1  ✓ closed  [bug] expand_glob skips exclusion/gitignore filters (resolver.rs:230)
fmr-albo      P1  ✓ closed  [bug] Gitignore matching uses bare filename instead of relative path (resolver.rs:163)
fmr-wbsk      P1  ✓ closed  [bug] Ellipsis conversion fails when followed by curly double quotes (smartquotes interaction)
fmr-7o1d      P1  ✓ closed  [bug] Inplace mode loses file permissions (atomic_write doesn't preserve mode)
fmr-s6q1      P1  ✓ closed  [task] Create fixture directory tree for comprehensive tryscript tests
fmr-5aot      P1  ✓ closed  [task] Write 10 tryscript test files (82 scenarios) for comprehensive CLI testing
fmr-nx0r      P1  ✓ closed  [task] Capture golden output and cross-validate Rust vs Python
fmr-fvl6      P1  ✓ closed  [task] Head-to-head parity comparison: run all content fixtures through both binaries, diff outputs
fmr-exfq      P1  ✓ closed  [bug] GAP1: Missing blank lines between consecutive footnote definitions
fmr-nshi      P1  ✓ closed  [bug] GAP2: Multi-paragraph footnotes collapsed to single line
fmr-vzze      P1  ✓ closed  [bug] GAP3: Autolinks <url> converted to [url](url) instead of preserved
fmr-ncv1      P1  ✓ closed  [bug] GAP4: Bare URLs converted to markdown links [url](url)
fmr-ozcy      P1  ✓ closed  [bug] GAP11: Blank line inserted between paragraph ending with colon and following list
fmr-vf3i      P1  ✓ closed  [bug] GAP12: Blank line inserted after HTML comment opening tags
fmr-pkqy      P1  ✓ closed  [bug] GAP13: Missing blank line before HTML comment closing tags after lists/tables
fmr-72dd      P1  ✓ closed  [bug] GAP14: Smart quotes not applied at paragraph-join boundaries in auto mode
fmr-81j7      P1  ✓ closed  [bug] D7: Footnote body continuation list items collapsed onto one line
fmr-xcr9      P1  ✓ closed  [bug] D8: Footnote body blockquote continuation collapsed onto first line
fmr-eiku      P1  ✓ closed  [bug] Fix autolink false positive for relative-path links where text == URL
fmr-8ixa      P1  ✓ closed  [bug] CLI error handling parity: fix duplicate error messages, add --inplace stdin validation
fmr-7q10      P1  ✓ closed  [task] Replace manual file resolver walker with ignore::WalkBuilder for ~10x syscall reduction
fmr-0u55      P1  ✓ closed  [bug] P6: Extra blank line before code fence (paragraph→CodeBlock tight transition)
fmr-afof      P1  ✓ closed  [bug] Tight mode list spacing: complex item detection rewrite
fmr-desq      P1  ✓ closed  [bug] Loose mode Rules 3/4: paragraph→list and paragraph→code block blank line suppression
fmr-8pya      P1  ✓ closed  [bug] Loose mode footnote FNDEF: preamble→list separator missing in loose mode
fmr-gydk      P1  ✓ closed  [bug] Golden test regression: item_needs_child_spacing complex sublist check in Preserve mode
fmr-8gjj      P1  ✓ closed  [feature] Add rayon parallel file processing to CLI
fmr-lbpo      P1  ✓ closed  [feature] Add skip-unchanged optimization to reformat_file
fmr-leqz      P1  ✓ closed  [epic] Spec: Incremental cache and performance roadmap
├── fmr-qb08      P1  ✓ closed  [task] Cache core: add incremental manifest, fingerprint, and atomic persistence
├── fmr-ynyg      P1  ✓ closed  [task] CLI/config wiring: incremental flags and merge precedence
├── fmr-m4z9      P1  ✓ closed  [task] Integrate cache-aware file processing path in formatter loop
├── fmr-8tpy      P1  ✓ closed  [task] Add stage-level perf instrumentation and refresh cross-formatter benchmarks
├── fmr-2z00      P1  ✓ closed  [task] Validation: cache correctness, invalidation, and CLI coverage
├── fmr-ysne      P2  ✓ closed  [task] Hotspot follow-up: optimize dominant fill_markdown stages
└── fmr-unp8      P2  ✓ closed  [task] Benchmark + docs: first-run and second-run performance reporting
fmr-turs      P1  ✓ closed  [epic] Spec: cache lifecycle commands and cache observability
├── fmr-b3rl      P1  ✓ closed  [task] CLI: add --show-cache and --clear-cache execution paths
├── fmr-16xa      P1  ✓ closed  [task] Tests: cover cache lifecycle commands in integration and golden suites
├── fmr-p01n      P1  ✓ closed  [task] Docs: document cache lifecycle controls in rust docs/readme/spec
└── fmr-yqy8      P1  ✓ closed  [task] Validation: run lint/tests, push, and confirm CI
fmr-1rkg      P2  ✓ closed  [task] Review previous implementation (attic/flowmark-rs-1) for architectural choices
fmr-m5lz      P2  ✓ closed  [epic] [epic] Apply porting playbook best practices (27 documents to review)
fmr-lvab      P2  ✓ closed  [task] Meta-playbook review: fold learnings back into porting playbook
fmr-zk3y      P2  ✓ closed  [task] Update exact parity spec: mark phases 3-4 DONE, update remaining work
fmr-yya8      P2  ✓ closed  [task] Re-run discover-rust to update rust-tests.yaml (178→243 tests)
fmr-lrqa      P2  ✓ closed  [task] P2: Add edge case tests from old impl review (5 tests)
fmr-evgi      P2  ✓ closed  [task] P2: Project metadata, tooling config (rustfmt.toml, justfile, deny.toml, Cargo metadata)
fmr-q42z      P2  ✓ closed  [task] P2: Restrict visibility with pub(crate)
fmr-q9og      P2  ✓ closed  [task] Extract fence-tracking helper to eliminate 3x code duplication in filling.rs
fmr-0a97      P2  ✓ closed  [task] Replace nc.to_string() regex check with char method in ellipses.rs
fmr-xqyk      P2  ✓ closed  [task] Replace Vec<char> collection with iterator in remove_period_escapes_preserving_code
fmr-yhmk      P2  ✓ closed  [feature] Introduce FormatOptions struct to replace boolean parameter lists in public API
fmr-avjf      P2  ✓ closed  [task] Remove unused _name field from AtomicPattern struct
fmr-jbes      P2  ✓ closed  [task] Remove unnecessary info.clone() in code block rendering in filling.rs
fmr-rqwl      P2  ✓ closed  [task] Extract repeated lines.last().expect() calls to local variable in line_wrappers.rs
fmr-b035      P2  ✓ closed  [task] CI: Add CARGO_PROFILE_TEST_DEBUG=0 to reduce cache size
fmr-hj6z      P2  ✓ closed  [task] CI: Add code coverage job with cargo-llvm-cov
fmr-8un1      P2  ✓ closed  [task] CI: Add cargo-semver-checks job to prevent accidental API breakage
fmr-03xy      P2  ✓ closed  [task] 10.6: Upstream contributions — PR tryscript tests to Python flowmark repo
fmr-aq8o      P2  ✓ closed  [task] Add performance benchmarks comparing Rust vs Python flowmark
fmr-afg0      P2  ✓ closed  [bug] Smart quotes char-boundary redistribution is fragile (filling.rs:1107)
fmr-myhs      P2  ✓ closed  [bug] Column off-by-one in sentence wrapper (line_wrappers.rs:103)
fmr-draa      P2  ✓ closed  [bug] PUA placeholder collision with input content (filling.rs:1028)
fmr-gpi6      P2  ✓ closed  [bug] read_ignore_file silently drops all patterns on one bad line (gitignore.rs:34)
fmr-9s7o      P2  ✓ closed  [bug] CRLF line endings not preserved in frontmatter parsing (frontmatter.rs:30)
fmr-ol08      P2  ✓ closed  [bug] install_skill has no path validation, allows traversal (skills/mod.rs:51)
fmr-n6ve      P2  ✓ closed  [bug] should_include_explicit skips same-named directory component (resolver.rs:96)
fmr-hwji      P2  ✓ closed  [epic] Spec: Comprehensive Tryscript Golden Test Suite
├── fmr-d9t9      P1  ✓ closed  [task] Phase 1: Create all fixture files and directories
├── fmr-r7rs      P1  ✓ closed  [task] Phase 2.1: Write formatting.tryscript.md (12 scenarios)
├── fmr-11xm      P1  ✓ closed  [task] Phase 2.2: Write typography-tests.tryscript.md (8 scenarios)
├── fmr-ofml      P1  ✓ closed  [task] Phase 2.3: Write list-spacing.tryscript.md (6 scenarios)
├── fmr-ezxk      P1  ✓ closed  [task] Phase 2.4: Write auto-mode.tryscript.md (6 scenarios)
├── fmr-3h6w      P1  ✓ closed  [task] Phase 2.5: Write file-ops.tryscript.md (8 scenarios)
├── fmr-57ca      P1  ✓ closed  [task] Phase 2.6: Write stdin.tryscript.md (4 scenarios)
├── fmr-60vu      P1  ✓ closed  [task] Phase 2.7: Write file-discovery.tryscript.md (14 scenarios)
├── fmr-imji      P1  ✓ closed  [task] Phase 2.8: Write config-interaction.tryscript.md (10 scenarios)
├── fmr-1qlp      P1  ✓ closed  [task] Phase 2.9: Write verbose-docs.tryscript.md (6 scenarios)
├── fmr-r3r9      P1  ✓ closed  [task] Phase 2.10: Write errors-version.tryscript.md (8 scenarios)
├── fmr-6m8j      P1  ✓ closed  [task] Phase 2.11: Validate all tests against Python binary
├── fmr-6kwl      P1  ✓ closed  [task] Phase 3: Run tests against Rust and fix parity bugs
├── fmr-4oyd      P2  ✓ closed  [task] Phase 4: Add tryscript to Rust CI workflow
├── fmr-3qbv      P3  ✓ closed  [task] Phase 5: Upstream fixtures and tests to Python repo
├── fmr-s4wj      P3  ✓ closed  [task] Phase 6: Retire old cli-golden.tryscript.md
└── fmr-iusl      P2  ✓ closed  [task] Add tryscript CI integration and retire old test file
fmr-8vfd      P2  ○ open   [feature] Implement signposts format (SP/0.1) for knowledge flow maps
fmr-4fn7      P2  ✓ closed  [epic] Code quality improvements from parity fixes review
fmr-59h1      P2  ✓ closed  [bug] Replace LinkRefDef struct with HashSet<String> — dead fields never read
fmr-77g8      P2  ✓ closed  [bug] Use collision-resistant marker prefixes for REFDEF/FNDEF HTML comments
fmr-37nt      P2  ✓ closed  [task] Add unit tests for extract_link_ref_defs, extract_footnote_defs, encode_ref_links
fmr-g5j4      P2  ✓ closed  [bug] GAP5: Email addresses inside template tags get linkified
fmr-9tg5      P2  ✓ closed  [bug] GAP6: Escaped char backslash counted in line width for wrapping
fmr-mf7n      P2  ✓ closed  [bug] GAP7: Plaintext mode collapses fenced code blocks to single line
fmr-59xf      P2  ✓ closed  [bug] GAP8: Semantic mode sentence-break differences after closing paren/quote
fmr-lxdb      P2  ✓ closed  [bug] GAP9: Extra blank line between HTML comment blocks and following text
fmr-xt8i      P2  ✓ closed  [bug] GAP10: Smart quotes and ellipsis not applied inside footnote definition bodies
fmr-n69j      P2  ✓ closed  [bug] D1: Plaintext mode collapses code blocks instead of preserving fence structure
fmr-r9k6      P2  ✓ closed  [bug] D4: Tight list spacing inserts extra blank lines between nested sublists
fmr-3i50      P2  ✓ closed  [bug] D6: Nested blockquotes get extra blank separator lines
fmr-yjc0      P2  ✓ closed  [bug] Fix clippy inefficient_to_string in protect_autolinks (4 instances)
fmr-dihn      P2  ✓ closed  [bug] D9: Empty/whitespace input produces no output — Python outputs trailing newline
fmr-gocw      P2  ✓ closed  [bug] D10: HTML entities decoded by comrak instead of preserved (&amp; → &)
fmr-oyj6      P2  ✓ closed  [task] Run hyperfine benchmarks: Rust vs Python on --list-files and --auto formatting
fmr-e38z      P2  ✓ closed  [bug] P7: Blockquote blank continuation line loses > prefix
fmr-9kth      P2  ✓ closed  [bug] P8: Escaped backtick stripped in table inline code
fmr-dpjh      P2  ✓ closed  [bug] Plaintext mode: paired Jinja tag regex incorrectly matches two closing tags
fmr-xkh3      P2  ✓ closed  [bug] Nested blockquote blank separator: source position tracking needed
fmr-tw36      P2  ✓ closed  [task] Re-run benchmarks and update REPORT.md with parallel results
fmr-611r      P3  ✓ closed  [task] Review guideline: python-to-rust-cli-porting.md (CLI patterns)
fmr-4l3z      P3  ✓ closed  [task] Review guideline: python-to-rust-porting-rules.md (core porting rules)
fmr-forn      P3  ✓ closed  [task] Review guideline: rust-cli-app-patterns.md (error handling, logging)
fmr-y5hc      P3  ✓ closed  [task] Review guideline: rust-general-rules.md (Edition 2024, LazyLock, resolver)
fmr-qjz0      P3  ✓ closed  [task] Review guideline: rust-project-setup.md (Cargo.toml, CI, clippy)
fmr-r7q9      P3  ✓ closed  [task] Review guideline: test-coverage-for-porting.md (coverage tools, targets)
fmr-ddki      P3  ✓ closed  [task] Review reference: python-to-rust-playbook.md (8-phase methodology)
fmr-6mrh      P3  ✓ closed  [task] Review reference: rust-code-review-checklist.md (apply to codebase)
fmr-xm1h      P3  ✓ closed  [task] Review reference: rust-cli-best-practices.md (CI/CD, release, cross-compilation)
fmr-u2cw      P3  ✓ closed  [task] Review reference: python-to-rust-test-coverage-playbook.md (check 90%+ target)
fmr-zp2k      P3  ✓ closed  [task] Review reference: port-checklist-initial-template.md (verify completion gates)
fmr-xl8l      P3  ✓ closed  [task] Review case study: flowmark-port-decision-log.md (verify 10 decisions)
fmr-jl96      P3  ✓ closed  [task] Review case study: flowmark-port-comrak-bug.md (fence bug, workaround status)
fmr-v2bt      P3  ✓ closed  [task] Review playbook fixes: plan-2026-02-08-playbook-review-fixes.md (53+ fixes)
fmr-c8l1      P3  ✓ closed  [task] P3: CLI polish - ValueEnum, color, BufWriter, verbose flag
fmr-z8nr      P3  ✓ closed  [task] P3: Replace unwrap() with expect() in library code
fmr-75kh      P3  ✓ closed  [task] Update DEFAULT_WRAP_WIDTH comment to remove Python Black reference
fmr-nbp4      P3  ✓ closed  [task] Add doc-tests (examples) for reformat_text and fill_markdown
fmr-nswa      P3  ✓ closed  [task] Remove || true from mapping check in CI to enforce completeness
fmr-zvbe      P3  ✓ closed  [task] Add .github/dependabot.yml for automated dependency updates
fmr-wjjm      P3  ✓ closed  [bug] O(n*m) placeholder restoration performance in restore_atomic_constructs
fmr-xado      P3  ✓ closed  [task] has_frontmatter parses entire document unnecessarily (frontmatter.rs:42)
fmr-zygd      P3  ✓ closed  [bug] Misleading 'Permission denied' error message in install_skill
fmr-hd1c      P3  ✓ closed  [bug] Fix contradictory/stale comments (lines 1345-1350 and line 32 in filling.rs)
fmr-rtfd      P3  ✓ closed  [task] DRY: refactor extract_link_ref_defs to use transform_outside_code_fences
fmr-rbe9      P3  ✓ closed  [task] DRY: extract loop-replace helper in encode_ref_links
fmr-8bn8      P3  ✓ closed  [task] Remove #[allow(clippy::nonminimal_bool)] with named variable for clarity
fmr-fzth      P3  ✓ closed  [bug] D2: Plaintext mode sentence detection differs on 'St.' abbreviation
fmr-bzra      P3  ✓ closed  [bug] D3: Narrow width (60) wraps differently around <sup> HTML tags
fmr-vpg4      P3  ✓ closed  [bug] D5: Loose list spacing missing blank lines in footnote embedded lists
fmr-el2i      P3  ✓ closed  [bug] P9: Smart quote conversion after inline code backtick
fmr-jcsj      P4  ✓ closed  [task] simple_word_split is pub but unused in production (text_wrapping.rs:75)
fmr-wonk      P4  ✓ closed  [task] first_sentence/first_sentences appear unused (sentence.rs:61)
fmr-kqxb      P4  ✓ closed  [task] in_heading threaded as mut bool through deep call chain — use context struct
fmr-m7o8      P4  ✓ closed  [bug] markdown_escape_word uses byte indexing, fragile if regex broadened

235 issues: 225 closed, 10 open
```
