# Port Sync Playbook

How to keep flowmark-rs in sync with the Python flowmark source, maintain exact parity,
and fix issues as they are discovered.

## Purpose

This document is the operational guide for anyone — human or AI agent — working on
flowmark-rs. Read this before starting any work that involves improving the port, fixing
parity issues, adding features from a new Python upstream release, or investigating
discrepancies.

It establishes the methodology, links to the foundational principles, and describes the
concrete procedures for keeping the Rust port aligned with Python flowmark.

## Background & Porting Methodology

flowmark-rs was built using the
[rust-porting-playbook](https://github.com/jlevy/rust-porting-playbook), available as a
git submodule at `repos/rust-porting-playbook/`. The playbook defines the methodology,
principles, and patterns that govern all work on this port.

### Essential reading

Before doing any porting, parity, or maintenance work, read or load these documents:

| Document | What it covers | Priority |
| --- | --- | --- |
| [Porting Principles and Anti-Patterns](../repos/rust-porting-playbook/guidelines/porting-principles-and-antipatterns.md) | **8 non-negotiable principles** for agent-driven porting — parity definition, active pursuit, test discipline, disparity handling | **Must read** |
| [Python-to-Rust Porting Rules](../repos/rust-porting-playbook/guidelines/python-to-rust-porting-rules.md) | Type mappings, module structure, dependency equivalences, pitfalls (regex anchoring, string indexing, comrak arena pattern) | **Must read** |
| [Test Coverage for Porting](../repos/rust-porting-playbook/guidelines/test-coverage-for-porting.md) | Test strategy, coverage targets, fixture organization, cross-validation | Must read |
| [Python-to-Rust CLI Porting](../repos/rust-porting-playbook/guidelines/python-to-rust-cli-porting.md) | argparse→clap mapping, SIGPIPE handling, exit codes, I/O parity | Read for CLI work |
| [Rust General Rules](../repos/rust-porting-playbook/guidelines/rust-general-rules.md) | Edition 2024 changes, ownership patterns, error handling, string safety | Read for Rust coding |
| [Rust CLI App Patterns](../repos/rust-porting-playbook/guidelines/rust-cli-app-patterns.md) | CLI project structure, logging, progress, config management | Read for CLI work |
| [Rust Project Setup](../repos/rust-porting-playbook/guidelines/rust-project-setup.md) | Cargo.toml, CI/CD, lint config, release workflow, security auditing | Read for infra work |

### Reference documents

For deeper context on the porting methodology and decision history:

| Document | What it covers |
| --- | --- |
| [8-Phase Playbook](../repos/rust-porting-playbook/reference/python-to-rust-playbook.md) | The complete porting process from assessment through ongoing sync |
| [Code Review Checklist](../repos/rust-porting-playbook/reference/rust-code-review-checklist.md) | Rust code review checklist for ports |
| [Mapping Reference](../repos/rust-porting-playbook/reference/python-to-rust-mapping-reference.md) | Comprehensive type/project/dependency mapping tables |
| [Test Coverage Playbook](../repos/rust-porting-playbook/reference/python-to-rust-test-coverage-playbook.md) | Pre-port test coverage strategy and tooling |

### Flowmark case study

The playbook includes a detailed case study of this project’s port at
`repos/rust-porting-playbook/case-studies/flowmark/`, covering library evaluation,
technical decisions, cross-validation results, comrak workarounds, and the custom
wrapping algorithm.

### Key principles (summary)

The
[Porting Principles](../repos/rust-porting-playbook/guidelines/porting-principles-and-antipatterns.md)
document defines 8 non-negotiable rules.
Each was learned from an actual mistake during agent-driven porting:

1. **Parity definition must be crisp** — never redefine scope without approval
2. **Agents must actively pursue parity** — every gap is a blocker, not a note
3. **Tests must always run in CI** — no orphaned test files
4. **Tests must never hide failures** — no massaging, truncating, or weakening
5. **Fix the process, not the test** — when tests fail, fix the code or CI
6. **Environment dependencies must be explicit** — CI installs everything
7. **Ignored tests must be tracked** — every `#[ignore]` needs a reason and issue
8. **Disparities must be tested before fixed** — write failing test first, then fix

## Repository Layout

Both upstream repos are git submodules under `repos/`:

```
repos/
├── flowmark/                # Python flowmark source (pinned to release tag)
└── rust-porting-playbook/   # Porting methodology and guidelines
```

The current Python version is tracked in `Cargo.toml`:

```toml
[package.metadata.parity]
version = "0.6.4"
```

## Initial Setup

After cloning, initialize submodules:

```bash
git submodule update --init --recursive
```

## Sync Process

When a new Python flowmark version is released:

### 1. Update the submodule

```bash
cd repos/flowmark
git fetch --tags
git checkout v0.X.Y
cd ../..
```

### 2. Copy test fixtures

```bash
cp repos/flowmark/tests/testdocs/testdoc.orig.md tests/testdocs/
cp repos/flowmark/tests/testdocs/testdoc.expected.*.md tests/testdocs/
```

### 3. Run tests

```bash
cargo test --all-features
```

If tests fail, review the diffs between `testdoc.actual.*.md` and
`testdoc.expected.*.md` in `tests/testdocs/`. Categorize each difference:

| Category | Action |
| --- | --- |
| **Porting bug** | Fix in Rust code |
| **Library difference** | Add workaround or accept and document |
| **Python bug fix** | Update Rust to match |
| **New Python feature** | Port to Rust |

### 4. Update test mapping

Re-discover Python tests, re-discover Rust tests, and check mapping completeness (see
the [Test Mapping and Parity Verification](#test-mapping-and-parity-verification)
section below for full details):

```bash
cd python
uv run flowmark-dev discover-python --local-path ../repos/flowmark
uv run flowmark-dev discover-rust
uv run flowmark-dev check-mapping
cd ..
```

The `check-mapping` command will fail if any new Python tests lack a mapping entry.
For each new test, add an entry to `admin/port-coverage-mapping/test-mapping.yaml` with
the appropriate Rust counterpart or `status: missing` until ported.

### 5. Update version correspondence

In `Cargo.toml`:

```toml
[package.metadata.parity]
version = "0.X.Y"
```

### 6. Commit everything

```bash
git add repos/flowmark Cargo.toml tests/testdocs/ python/ admin/port-coverage-mapping/
git commit -m "sync: update Python source to v0.X.Y"
```

## Test Mapping and Parity Verification

The [admin/port-coverage-mapping/](../admin/port-coverage-mapping/) directory contains a
CI-enforced system that tracks provenance between every Python test and its Rust
counterpart. This ensures that when Python upstream adds or changes tests, we know
exactly which Rust tests correspond and whether any are missing.

See the [full spec](project/specs/active/plan-2026-02-17-test-mapping-meta-test.md) for
design rationale, and the [admin README](../admin/README.md) for quick reference.

### How it works

Three YAML files form the mapping system:

| File | Role | Generated by |
| --- | --- | --- |
| `admin/port-coverage-mapping/python-tests.yaml` | All Python test functions (292 entries from v0.6.4) | `flowmark-dev discover-python` |
| `admin/port-coverage-mapping/rust-tests.yaml` | All Rust test functions (408+ entries) | `flowmark-dev discover-rust` |
| `admin/port-coverage-mapping/test-mapping.yaml` | Hand-maintained Python→Rust mapping with status | Manual / agent edits |

### Discovery procedure

**Python test discovery** (`flowmark-dev discover-python`):
- AST-parses the Python flowmark repo at the pinned release tag
- Extracts every `test_*` function, classifies by type (unit, integration, golden,
  infrastructure)
- Writes `python-tests.yaml` with idempotent merge — hand-added entries survive
  re-generation

**Rust test discovery** (`flowmark-dev discover-rust`):
- Runs `cargo test -- --list --format terse` (compiler-authoritative)
- Resolves file paths and line numbers
- Writes `rust-tests.yaml` with idempotent merge

### Mapping maintenance

`test-mapping.yaml` maps each Python test to one or more Rust tests.
Each entry has a status:

| Status | Meaning |
| --- | --- |
| `mapped` | Python test has verified Rust counterpart(s) |
| `excluded` | Intentionally not ported (Python-specific infrastructure) |
| `partial` | Rust test covers subset of Python test behavior |
| `missing` | No Rust equivalent yet (CI will fail) |

A single Python test can map to multiple Rust tests (`1:N` mapping).
For example, `test_ellipses` maps to 10 separate Rust test functions.

### CI enforcement

The `check-mapping` job in `.github/workflows/ci.yml` runs after all Rust tests pass.
It:

1. Runs 13 smoke tests (`pytest tests/test_smoke.py`) that verify YAML round-trip
   stability, deterministic ordering, discovery counts, and mapping completeness
2. Runs `flowmark-dev check-mapping` which validates:
   - Every Python test has a mapping entry
   - Every mapped Rust function exists in `rust-tests.yaml`
   - No entries have `status: missing`
3. **Fails the build** if any check fails

### Updating after a Python upstream release

When Python flowmark releases a new version with new or changed tests:

```bash
# 1. Update the Python submodule to the new tag
cd repos/flowmark && git checkout v0.X.Y && cd ../..

# 2. Re-discover Python tests (new tests appear in python-tests.yaml)
cd python
uv run flowmark-dev discover-python --local-path ../repos/flowmark

# 3. Re-discover Rust tests (after implementing new test counterparts)
uv run flowmark-dev discover-rust

# 4. Initialize mapping for any new entries (adds them as status: missing)
uv run flowmark-dev init-mapping

# 5. Check mapping — will fail until all new tests are mapped
uv run flowmark-dev check-mapping
cd ..
```

For each new `missing` entry, either:
- Port the corresponding test to Rust and set `status: mapped`
- Set `status: excluded` with a `notes:` field explaining why

CI will not pass until every Python test has a non-missing status.

## Test Fixture Files

| File | Role | Tracked |
| --- | --- | --- |
| `tests/testdocs/testdoc.orig.md` | Input fixture (from Python) | Yes |
| `tests/testdocs/testdoc.expected.*.md` | Expected golden outputs (from Python) | Yes |
| `tests/testdocs/testdoc.actual.*.md` | Actual outputs (generated by `cargo test`) | No (gitignored) |

## Updating the Playbook

The porting playbook itself is also a submodule.
To update it or push improvements back upstream:

```bash
cd repos/rust-porting-playbook
git checkout main
git pull
# Make edits, commit, push
cd ../..
git add repos/rust-porting-playbook
git commit -m "sync: update porting playbook"
```

## Markdown Formatting

All docs are auto-formatted with flowmark.
Run locally:

```bash
uvx flowmark@latest --auto --extend-exclude "tests/" --extend-exclude "attic/" --extend-exclude ".claude/" --extend-exclude "python/" .
```

This also runs in CI as a non-blocking check.
