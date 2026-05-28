# Port Sync Playbook

> **Doc status:** Rust port-specific (no upstream equivalent).
> Documents the Rust port lifecycle: parity verification, sync workflow, and port
> history.

How to keep flowmark-rs in sync with the Python flowmark source, maintain exact parity,
and fix issues as they are discovered.

## Purpose

This document is the operational guide for syncing with Python upstream, porting new
features, fixing parity issues, and maintaining the test mapping.
Read this before starting any sync or parity work.

For general build/test/lint instructions, see [`docs/development.md`](development.md).
For the full port overview and release status, see
[`docs/port-status.md`](port-status.md).

This document is **flowmark-rs-specific** (fixtures, mapping files, local scripts).
For reusable process, use the canonical playbook docs first:

- [`../repos/rust-porting-playbook/playbooks/python-to-rust-sync-release-workflow.md`](../repos/rust-porting-playbook/playbooks/python-to-rust-sync-release-workflow.md)
- [`../repos/rust-porting-playbook/playbooks/port-checklist-update-template.md`](../repos/rust-porting-playbook/playbooks/port-checklist-update-template.md)
- [`../repos/rust-porting-playbook/playbooks/auto-sync-agent-prompt-template.md`](../repos/rust-porting-playbook/playbooks/auto-sync-agent-prompt-template.md)

For a full manual validation run (automated tests + tryscript sanity review + docs/help
equivalence checks), use:
[tests/qa/rust-python-parity-e2e.qa.md](../tests/qa/rust-python-parity-e2e.qa.md).

## Background & Porting Methodology

flowmark-rs was built using the
[rust-porting-playbook](https://github.com/jlevy/rust-porting-playbook), available as a
git submodule at `repos/rust-porting-playbook/`. The playbook defines the methodology,
principles, and patterns that govern all work on this port.

### Essential reading

Before doing any porting, parity, or maintenance work, read or load these documents:

| Document | What it covers | Priority |
| --- | --- | --- |
| [Porting Principles and Anti-Patterns](../repos/rust-porting-playbook/guidelines/porting-principles-and-antipatterns.md) | **8 non-negotiable principles** for agent-driven porting, parity definition, active pursuit, test discipline, disparity handling | **Must read** |
| [Python-to-Rust Porting Rules](../repos/rust-porting-playbook/guidelines/python-to-rust-porting-rules.md) | Type mappings, module structure, dependency equivalences, pitfalls (regex anchoring, string indexing, comrak arena pattern) | **Must read** |
| [Test Coverage for Porting](../repos/rust-porting-playbook/guidelines/test-coverage-for-porting.md) | Test strategy, coverage targets, fixture organization, cross-validation | Must read |
| [Python-to-Rust CLI Porting](../repos/rust-porting-playbook/guidelines/python-to-rust-cli-porting.md) | argparse→clap mapping, SIGPIPE handling, exit codes, I/O parity | Read for CLI work |
| [Rust General Rules](../repos/rust-porting-playbook/guidelines/rust-general-rules.md) | Edition 2024 changes, ownership patterns, error handling, string safety | Read for Rust coding |
| [Rust CLI App Patterns](../repos/rust-porting-playbook/references/rust-cli-app-patterns.md) | CLI project structure, logging, progress, config management | Read for CLI work |
| [Rust Project Setup](../repos/rust-porting-playbook/guidelines/rust-project-setup.md) | Cargo.toml, CI/CD, lint config, release workflow, security auditing | Read for infra work |

### Reference documents

For deeper context on the porting methodology and decision history:

| Document | What it covers |
| --- | --- |
| [Python-to-Rust Playbook](../repos/rust-porting-playbook/playbooks/python-to-rust-playbook.md) | The complete porting process from assessment through ongoing sync |
| [Sync Release Workflow](../repos/rust-porting-playbook/playbooks/python-to-rust-sync-release-workflow.md) | Two-stage release refresh: Rust-only stabilization, then upstream sync |
| [Code Review Checklist](../repos/rust-porting-playbook/references/rust-code-review-checklist.md) | Rust code review checklist for ports |
| [Mapping Reference](../repos/rust-porting-playbook/references/python-to-rust-mapping-reference.md) | Comprehensive type/project/dependency mapping tables |
| [Test Coverage Playbook](../repos/rust-porting-playbook/playbooks/python-to-rust-test-coverage-playbook.md) | Pre-port test coverage strategy and tooling |

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

1. **Parity definition must be crisp**: never redefine scope without approval
2. **Agents must actively pursue parity**: every gap is a blocker, not a note
3. **Tests must always run in CI**: no orphaned test files
4. **Tests must never hide failures**: no massaging, truncating, or weakening
5. **Fix the process, not the test**: when tests fail, fix the code or CI
6. **Environment dependencies must be explicit**: CI installs everything
7. **Ignored tests must be tracked**: every `#[ignore]` needs a reason and issue
8. **Disparities must be tested before fixed**: write failing test first, then fix

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
version = "0.7.0"
```

## Version Convention

Each release documents which Python version it targets, and dev builds include
commits-ahead and git hash metadata:

> flowmark 0.2.5-dev.<N>+g<hash> (Rust port of flowmark-py 0.7.0; base v0.2.5)

The Rust version follows its own semver independently.
The port note indicates which Python version’s behavior is fully covered.

## Initial Setup

> **First thing every session:** initialize submodules.
> Most sync work fails early without them and the failure modes are confusing (missing
> `repos/flowmark` paths, empty fixtures, “playbook not found”).

```bash
git submodule update --init --recursive
```

## Checking for Upstream Updates

To check whether the Python upstream has a newer release than the current parity target,
run:

```bash
BASELINE=$(grep -A1 '\[package.metadata.parity\]' Cargo.toml | grep version | sed 's/.*"\(.*\)"/\1/')
git -C repos/flowmark fetch --tags >/dev/null 2>&1
LATEST=$(git -C repos/flowmark tag -l 'v[0-9]*' | sort -V | tail -1)
echo "Current parity target: v${BASELINE}"
echo "Latest upstream tag:   ${LATEST}"
```

If `LATEST` is greater than `v${BASELINE}`, run a Mode B sync (below).
Otherwise, no sync work is needed.

## Sync Process (Mode B: Upstream baseline change)

Use this section when updating from one published Python release to a newer one (for
example, `v0.6.5 -> v0.7.0`).

### Canonical checklist translation for flowmark-rs

The reusable playbook checklist uses generic names.
For this repo, use these concrete equivalents:

| Canonical checklist item | flowmark-rs equivalent |
| --- | --- |
| `scripts/sync-from-python.sh` | Update `repos/flowmark` submodule tag + copy `tests/testdocs` fixtures |
| `scripts/validate-parity.sh` | `cargo test --all-features`, `FLOWMARK_PARITY_PYTHON=1 cargo test --test test_parity_cross_binary`, `scripts/corpus-parity-check.sh` |
| `[package.metadata.python_source]` | `[package.metadata.parity]` in `Cargo.toml` |
| `docs/version-history.md` / `docs/python-sync-log.md` | `docs/sync-artifacts/*.md`, `README.md` “Last sync”, `docs/port-status.md` |
| `test-fixtures/` | `tests/testdocs/` and `tests/parity/` |

### End-to-end runbook

1. **Declare baseline and target**

   ```bash
   BASELINE=$(grep -A1 '\[package.metadata.parity\]' Cargo.toml | grep version | sed 's/.*"\(.*\)"/\1/')
   TARGET="0.X.Y"
   echo "Syncing Python baseline v${BASELINE} -> v${TARGET}"
   ```

2. **Create a diff artifact (required before coding)**

   ```bash
   mkdir -p docs/sync-artifacts
   ARTIFACT="docs/sync-artifacts/$(date +%F)-sync-v${BASELINE}-to-v${TARGET}.md"
   ```

3. **Update Python submodule and summarize upstream changes**

   ```bash
   cd repos/flowmark
   git fetch --tags
   git checkout "v${TARGET}"
   git log --oneline "v${BASELINE}..v${TARGET}"
   git diff --name-status "v${BASELINE}..v${TARGET}"
   cd ../..
   ```

4. **Sync fixtures and generated README inputs**

   ```bash
   cp repos/flowmark/tests/testdocs/testdoc.orig.md tests/testdocs/
   cp repos/flowmark/tests/testdocs/testdoc.expected.*.md tests/testdocs/
   ./scripts/generate_rust_readme.py
   ```

   The generator pulls `last_sync_date` from `docs/port-status.md` and `parity_version`
   from `Cargo.toml`, so update those before running it.

5. **Triage mapping gaps early (temp manifest)**

   ```bash
   cd python
   TMP_PYTHON=$(mktemp)
   uv run flowmark-dev discover-python --local-path ../repos/flowmark --output "$TMP_PYTHON"
   uv run flowmark-dev check-mapping \
     --python-yaml "$TMP_PYTHON" \
     --rust-yaml ../admin/port-coverage-mapping/rust-tests.yaml \
     --mapping-yaml ../admin/port-coverage-mapping/test-mapping.yaml || true
   cd ..
   ```

   This gives an immediate list of new/changed upstream tests to port or classify.

6. **Port behavior and tests, then refresh checked-in mapping files**

   ```bash
   cd python
   uv run flowmark-dev discover-python --local-path ../repos/flowmark
   uv run flowmark-dev discover-rust
   uv run flowmark-dev init-mapping
   uv run flowmark-dev check-mapping
   cd ..
   ```

   For each `missing` mapping entry:
   - add/update Rust tests and set `status: mapped`, or
   - set `status: excluded` with explicit `notes:` rationale.

7. **Update all parity version references**

   - `Cargo.toml` `[package.metadata.parity].version`
   - `.github/workflows/ci.yml` `FLOWMARK_PY_VERSION`
   - `python/src/flowmark_dev_tools/cli.py` `DEFAULT_REF`
   - `README.md` last sync line
   - `docs/port-status.md` parity target/status text
   - `python/tests/test_smoke.py` expected discovery counts when they change

8. **Run full validation gates**

   ```bash
   cargo fmt --all -- --check
   cargo build --locked --all-features
   cargo clippy --locked --all-targets --all-features -- -D warnings
   cargo test --locked --all-features
   FLOWMARK_PARITY_PYTHON=1 cargo test --locked --test test_parity_cross_binary

   cd python
   uv run pytest tests/test_smoke.py -q
   uv run flowmark-dev check-mapping
   cd ..

   ./scripts/generate-parity-golden.sh
   cargo build --release
   ./scripts/corpus-parity-check.sh
   ```

9. **Commit and release**

   ```bash
   git add repos/flowmark Cargo.toml .github/workflows/ci.yml README.md docs/port-status.md tests/testdocs/ tests/parity/ python/ admin/port-coverage-mapping/
   git commit -m "sync: update Python source to v${TARGET}"
   ```

   Then follow [`docs/publishing.md`](publishing.md) to cut the Rust release.

## Test Mapping and Parity Verification

The [admin/port-coverage-mapping/](../admin/port-coverage-mapping/) directory contains a
CI-enforced system that tracks provenance between every Python test and its Rust
counterpart. This ensures that when Python upstream adds or changes tests, we know
exactly which Rust tests correspond and whether any are missing.

See the [full spec](project/specs/done/plan-2026-02-17-test-mapping-meta-test.md) for
design rationale.

### How it works

Three YAML files form the mapping system:

| File | Role | Generated by |
| --- | --- | --- |
| `admin/port-coverage-mapping/python-tests.yaml` | All Python test functions (347 entries from v0.7.0; 323 mapped + 24 excluded) | `flowmark-dev discover-python` |
| `admin/port-coverage-mapping/rust-tests.yaml` | All Rust test functions (408+ entries) | `flowmark-dev discover-rust` |
| `admin/port-coverage-mapping/test-mapping.yaml` | Hand-maintained Python→Rust mapping with status | Manual / agent edits |

### Discovery procedure

**Python test discovery** (`flowmark-dev discover-python`):

- AST-parses the Python flowmark repo at the pinned release tag
- Extracts every `test_*` function, classifies by type (unit, integration, golden,
  infrastructure)
- Writes `python-tests.yaml` with idempotent merge, hand-added entries survive
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

# 5. Check mapping, will fail until all new tests are mapped
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
FLOWMARK_PY_VERSION=$(grep -A1 '\[package.metadata.parity\]' Cargo.toml | grep version | sed 's/.*"\(.*\)"/\1/')
uvx "flowmark@${FLOWMARK_PY_VERSION}" --auto --extend-exclude "tests/" --extend-exclude "attic/" --extend-exclude ".claude/" --extend-exclude "python/" .
git checkout -- README.md
```

This also runs in CI as a non-blocking check.
