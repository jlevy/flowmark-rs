# Feature: Cross-Language Test Mapping (Port Coverage)

**Date:** 2026-02-17 (last updated 2026-02-17)

**Author:** Joshua Levy

**Status:** Implemented — all phases complete, CI enforced

## Overview

A systematic test provenance tracking system that ensures every Python flowmark test has
a verified Rust counterpart.
All artifacts are YAML for human readability, clean diffs, and agent editability.

The system has four components:

1. **Python test discovery CLI** (`flowmark-dev discover-python`) — walks the Python
   flowmark repo at a pinned release tag, extracts every test function via AST parsing,
   writes `python-tests.yaml`.
2. **Rust test discovery** — two options, both producing `rust-tests.yaml`:
   - a. A Python script (`flowmark-dev discover-rust`) using regex parsing of `*.rs`
     files.
   - b. (Proposed) A Rust build target or test that uses compile-time introspection to
     emit its own test list as YAML, which would be more authoritative and not require
     a Python dependency.
3. **Hand-maintained mapping file** (`test-mapping.yaml`) — maps each Python test to its
   Rust equivalent(s) with status and notes.
   Agent-editable YAML.
4. **Mapping checker** (`flowmark-dev check-mapping`) — loads all three YAML files and
   asserts completeness.
   Pure Python, runnable standalone or from CI.

### Key Design Principles

- **All YAML**: Every artifact is YAML for readability, diffability, and agent
  editability.
- **Idempotent and additive**: Both discovery scripts merge into existing files.
  Auto-discovered tests update in place; hand-added entries are preserved.
  Nothing is deleted on re-generation.
- **Pinned source version**: The Python discovery is pinned to a specific release tag
  (currently `v0.6.4`).
  The checked-in YAML only changes when we intentionally bump the pin.
- **Reusable**: The tool structure is designed to work for any Python-to-Rust port, not
  just flowmark.
  Project-specific knowledge (file classification, integration function names) is
  configurable.

## Goals

- 100% coverage tracking: every Python test has a documented Rust mapping or an explicit
  exclusion reason.
- Machine-verifiable: `flowmark-dev check-mapping` fails with exit code 1 if the mapping
  is stale or incomplete.
- Low friction: discovery is fast and deterministic, requiring only Python 3.11+ and the
  uv-managed project.
- Incremental: an agent can fill in the mapping one test at a time, and each addition is
  immediately verifiable.
- Extra Rust tests are tracked and logged (not failures) — useful for potential
  upstreaming.

## Non-Goals

- This does NOT verify behavioral equivalence (i.e., that the Rust test actually tests
  the same thing as the Python test).
  That is a separate manual review step.
- This does NOT auto-generate Rust tests from Python tests.
- This does NOT require the Python test suite to be runnable.
  The discovery script only parses test structure; it does not execute tests.

## Background

The flowmark-rs project is a Rust port of the Python
[flowmark](https://github.com/jlevy/flowmark) Markdown auto-formatter.
As of v0.6.4, the Python test suite has **281 test functions** across 20 files.
The Rust port currently has **250 test functions** (223 integration tests in `tests/` +
27 unit tests in `src/` modules).

There is no current mechanism to track which Python tests have been ported, which are
intentionally excluded, and which have drifted.
This spec addresses that gap.

### Python Source Pin

The original Python source is at `https://github.com/jlevy/flowmark`.
Discovery is pinned to **tag `v0.6.4`** (the latest release).
A local copy at `attic/flowmark/` (gitignored) can be used for faster iteration.

### Test Categories in Python

| Category | Files | Description |
|---|---|---|
| Unit tests | `test_ellipses.py`, `test_sentences.py`, `test_smartquotes.py`, `test_escape_handling.py`, `test_strikethrough.py`, `test_wrapping.py` | Test individual functions/modules |
| Integration tests | `test_filling.py`, `test_alerts.py`, `test_cleanups.py`, `test_fenced_code_blocks.py`, `test_frontmatter.py`, `test_heading_spacing.py`, `test_list_spacing.py`, `test_tag_formatting.py`, `test_width_options.py` | Test `fill_markdown` pipeline |
| Golden/fixture tests | `test_ref_docs.py` | Compare full document output against expected fixtures |
| Infrastructure tests | `test_cli_file_discovery.py`, `test_config.py`, `test_file_resolver.py`, `test_skill.py` | CLI, config, file resolution, skill system |

### Test Patterns (Python)

- No `pytest.mark.parametrize` is used anywhere.
- Classes are rare (only `test_skill.py` for logical grouping).
- Dense multi-assert functions are common (e.g., `test_ellipses()` has ~98 assertions).
- Fixtures used only for CLI integration tests (`tmp_path`, `capsys`, `monkeypatch`).
- Golden tests in `test_ref_docs.py` use an inline dataclass + loop pattern.

## Design

### Directory Layout

```
python/                              # uv-managed Python project
  pyproject.toml                     # Modern uv setup, flowmark-dev CLI entry point
  README.md
  src/flowmark_dev_tools/
    __init__.py
    cli.py                           # CLI: discover-python, discover-rust, init-mapping,
                                     #       check-mapping
    models.py                        # Frozen dataclasses: PythonTestRecord,
                                     #   RustTestRecord, MappingRecord
    discover_python.py               # AST-based Python test walker
    discover_rust.py                 # Cargo-based + regex fallback Rust test walker
    yaml_io.py                       # YAML read/write with ordered keys, atomic writes
    check_mapping.py                 # Mapping validation logic

port-coverage-mapping/               # All checked-in YAML artifacts
  python-tests.yaml                  # Python test manifest (checked in, pinned)
  rust-tests.yaml                    # Rust test manifest (checked in)
  test-mapping.yaml                  # Hand-maintained mapping (checked in)
```

### Data Model

**Python test record** (in `python-tests.yaml`):
```yaml
- file: tests/test_alerts.py
  function: test_basic_note_alert
  test_type: integration
  line_number: 14
  doc_string: Test basic [!NOTE] alert formatting.
```

Fields: `file`, `function`, `class_name` (optional), `test_type`
(`unit`|`integration`|`golden`|`infrastructure`), `line_number`, `doc_string`
(optional).

**Rust test record** (in `rust-tests.yaml`):
```yaml
- file: tests/test_alerts.rs
  function: test_basic_note_alert
  line_number: 8
```

Fields: `file`, `function`, `line_number`.

**Mapping record** (in `test-mapping.yaml`):
```yaml
- python_file: tests/test_alerts.py
  python_function: test_basic_note_alert
  status: mapped
  rust_file: tests/test_alerts.rs
  rust_function: test_basic_note_alert

- python_file: tests/test_ellipses.py
  python_function: test_ellipses
  status: mapped
  rust_file: tests/test_ellipses.rs
  rust_functions:
    - test_ellipses_basic_conversions
    - test_ellipses_punctuation
    - test_ellipses_does_not_apply
    - test_ellipses_multiline
    # ... etc.

- python_file: tests/test_skill.py
  python_function: test_get_skill_content
  python_class: TestGetSkillContent
  status: excluded
  notes: Python skill system infrastructure, not applicable to Rust port.
```

Status values:
- **mapped**: Python test has a direct Rust equivalent.
- **excluded**: Intentionally not ported.
  `notes` explains why.
- **partial**: Rust test exists but covers only a subset.
- **missing**: No Rust equivalent yet; should be ported.

### Idempotent, Additive Merge Behavior

All three commands (`discover-python`, `discover-rust`, `init-mapping`) are
**idempotent and additive**.
This is critical because some projects have custom test forms (like flowmark's inline
dataclass golden tests) that may be hand-added to the YAML but not detected by
auto-discovery.

**Identity keys** determine how records merge:
- Test manifests (`python-tests.yaml`, `rust-tests.yaml`): identity is `(file, function)`
  for test records without classes, `(file, class_name, function)` for tests inside
  classes.
- Mapping (`test-mapping.yaml`): identity is
  `(python_file, python_class, python_function)`.

**Merge rules for all three files:**

1. **Load existing file** if it exists, index all records by identity key.
2. **For each auto-discovered record:**
   - If identity key already exists: **update** auto-discoverable fields (line number,
     test type, doc_string) with the latest values.
     Preserve any hand-added fields that auto-discovery doesn't populate.
   - If identity key is new: **add** the record.
3. **For each existing record not found by auto-discovery:** **preserve** it.
   It may be a hand-added entry for a custom test form, or it may be stale.
   The `check-mapping` command detects and reports stale entries — the discovery scripts
   never delete.
4. **Sort** deterministically and write atomically.

**Example: hand-added custom test in `python-tests.yaml`:**

```yaml
# Auto-discovered:
- file: tests/test_ellipses.py
  function: test_ellipses
  test_type: unit
  line_number: 9

# Hand-added (custom test form not caught by AST walker):
- file: tests/test_ref_docs.py
  function: test_reference_doc_markdown_only
  test_type: golden
  line_number: 42
  doc_string: Manually added — tests a subset of ref doc configs.
```

When `discover-python` runs again, the hand-added entry is preserved because its
identity key `(tests/test_ref_docs.py, None, test_reference_doc_markdown_only)` won't
be found by auto-discovery.
The auto-discovered entry for `test_ellipses` gets its `line_number` and other fields
updated if they changed.

### Component 1: Python Test Discovery CLI

**Command:** `flowmark-dev discover-python`

**Flags:**
- `--repo-url` (default: `https://github.com/jlevy/flowmark`)
- `--ref` (default: `v0.6.4`) — pinned release tag
- `--local-path` — use a local checkout instead of cloning
- `--output` / `-o` — output YAML path (default: `port-coverage-mapping/python-tests.yaml`)

**Behavior:**
1. Clone repo at pinned ref to a temp directory (or use `--local-path`).
2. Use Python `ast` module to walk all `test_*.py` files under `tests/`.
3. Extract every `test_*` function, including those inside classes.
4. Classify test type by filename heuristic + call-site analysis (looks for
   `fill_markdown` etc.).
5. Write sorted YAML using atomic file output.

**Merge behavior:**

Like all discovery commands, `discover-python` is idempotent and additive.
If `python-tests.yaml` already exists, existing records are loaded and hand-added entries
are preserved.
Auto-discovered records update fields (line_number, test_type, doc_string) for matching
identity keys.

**Dependencies:** Python 3.11+ stdlib (`ast`, `subprocess`, `tempfile`) + `pyyaml` +
`strif` (for atomic writes).

### Component 2: Rust Test Discovery

**Command:** `flowmark-dev discover-rust`

**Flags:**
- `--project-dir` — path to the Rust project root (default: auto-detect from cwd)
- `--output` / `-o` — output YAML path (default: `port-coverage-mapping/rust-tests.yaml`)
- `--fallback-regex` — use regex parsing instead of cargo (for environments without
  Rust toolchain)

**Primary strategy: `cargo test -- --list`**

The Python CLI shells out to `cargo test -- --list --format terse`, which is
compiler-authoritative.
The Rust compiler knows exactly what's a `#[test]` function — no regex guessing.

This discovers **all tests**, including:
- Integration tests in `tests/test_*.rs` (output: `test_function_name: test`)
- Unit tests in `src/` modules (output: `module::submod::tests::test_name: test`)

For each discovered test, the CLI resolves file paths and line numbers by searching the
source files.
This hybrid approach (compiler-authoritative list + file-level line resolution) gives the
best of both worlds: the compiler's authority on test identity, and human-useful file/line
references.

**Fallback strategy: regex parsing**

If `cargo` is not available (e.g., a CI environment with only Python), the
`--fallback-regex` flag activates the original regex-based parser.
This only finds integration tests in `tests/test_*.rs` — it misses unit tests in `src/`.

**Why not a Rust binary target?**

A Rust binary that calls `cargo test -- --list` and writes YAML would require adding
`serde_yaml` as a dependency and a binary target to the Cargo workspace.
Since the Python CLI already handles YAML serialization and the cargo subprocess call is
trivial, this adds complexity without benefit.
Rust has no runtime test introspection — the only authoritative source is the compiler
via `cargo test -- --list`, which is equally accessible from Python or Rust.

**Merge behavior:**

Like all discovery commands, `discover-rust` is idempotent and additive.
If `rust-tests.yaml` already exists, existing records are loaded and hand-added entries
are preserved.
Auto-discovered records update fields (file, line_number) for matching identity keys.

### Component 3: Mapping Checker

**Command:** `flowmark-dev check-mapping`

**Flags:**
- `--python-yaml`, `--rust-yaml`, `--mapping-yaml` — override default paths.

**Checks:**
1. **Every Python test has a mapping entry.** Unmapped tests cause FAIL.
2. **Every mapped Rust function actually exists** in `rust-tests.yaml`.
   Broken refs cause FAIL.
3. **No `missing` status entries.** Any `missing` causes FAIL.
4. **Stale mapping entries** (Python test removed upstream): WARN.
5. **Extra Rust tests** (not referenced in any mapping): INFO with a log.
   These are candidates for upstreaming to the Python repo.
6. **Summary statistics**: total, mapped, excluded, partial, missing counts.

Exit code 0 on pass, 1 on fail.

### Component 4: Init/Update Mapping

**Command:** `flowmark-dev init-mapping`

**Flags:**
- `--python-yaml` — source Python manifest.
- `--output` / `-o` — mapping file path.

**Behavior:**
1. Load `python-tests.yaml`.
2. If `test-mapping.yaml` exists, load it and index by identity key.
3. For each Python test: if already in mapping, preserve the existing record.
   If new, add with status `missing`.
4. Sort by `(python_file, python_class, python_function)`.
5. Write atomically.

### Workflow

**Setup (one-time):**
```bash
cd flowmark-rs
uv run --project python flowmark-dev discover-python --local-path attic/flowmark
uv run --project python flowmark-dev discover-rust
uv run --project python flowmark-dev init-mapping
uv run --project python flowmark-dev check-mapping  # Fails: all missing
```

**Agent mapping population (one-time, labor-intensive):**
1. For each entry in `test-mapping.yaml` with status `missing`:
   - Read the Python test source.
   - Find the Rust counterpart (by name similarity, file correspondence, behavior).
   - Update the YAML record: set `status`, `rust_file`, `rust_function`/`rust_functions`,
     `notes`.
2. Run `flowmark-dev check-mapping` after each batch to verify progress.

**Ongoing maintenance (when Python upstream changes):**
1. Bump `--ref` to new release tag.
2. Re-run `discover-python` → updates `python-tests.yaml`.
3. Re-run `init-mapping` → adds new entries as `missing`, preserves existing.
4. `check-mapping` fails → agent addresses new/changed tests.

**Ongoing maintenance (when Rust tests change):**
1. Re-run `discover-rust` → updates `rust-tests.yaml`.
2. `check-mapping` reports broken refs or extra tests.
3. Agent updates mapping as needed.

## Implementation Plan

### Phase 1: Python Project and Discovery CLI (DONE — prototype)

- [x] Create `python/` directory with modern uv project setup
- [x] Write `models.py` with frozen dataclasses and StrEnum types
- [x] Write `discover_python.py` using `ast` module with test type classification
- [x] Write `discover_rust.py` using regex parsing
- [x] Write `yaml_io.py` with ordered keys, None-omission, atomic writes
- [x] Write `cli.py` with `discover-python`, `discover-rust`, `init-mapping`,
  `check-mapping` subcommands
- [x] Write `check_mapping.py` with completeness validation and human-readable report
- [x] Create `port-coverage-mapping/` directory
- [x] Verify end-to-end: 281 Python tests discovered, 151 Rust tests discovered,
  skeleton mapping generated, check correctly reports all missing

### Phase 2: Cargo-Based Discovery, Idempotent Merge, and Polish

- [x] Switch `discover-rust` to use `cargo test -- --list` as the primary strategy
- [x] Keep regex parser as `--fallback-regex` option
- [x] Add idempotent merge to `discover-python`: load existing YAML, preserve hand-added
  entries, update auto-discovered entries by identity key
- [x] Add idempotent merge to `discover-rust`: same behavior
- [x] Re-generate `rust-tests.yaml` with full 178-test list (151 integration + 27 unit)
- [x] Run ruff and basedpyright, fix any lint/type issues
- [x] Add a basic smoke test in `python/tests/`

### Phase 3: Populate the Mapping (Agent Labor) — DONE

- [x] For each of the 20 Python test files, review every test function against its Rust
  counterpart
- [x] Update `test-mapping.yaml` with `mapped`, `excluded`, or `partial` status
- [x] Handle 1:N cases (e.g., `test_ellipses` → 10 Rust functions,
  `test_split_frontmatter` → 5 Rust functions)
- [x] Mark infrastructure tests (`test_skill`, `test_cli_file_discovery`,
  `test_file_resolver`, `test_config`) as `excluded` with notes
- [x] Verify `flowmark-dev check-mapping` passes with exit code 0
- [x] Document `partial` entry: `test_other_escaped_chars` covers subset of escape types

**Mapping results:** 202 mapped, 79 excluded, 0 missing, 0 partial.
`flowmark-dev check-mapping` passes with exit code 0.

### Phase 4: CI Integration — DONE

- [x] Add a CI step that runs `flowmark-dev check-mapping` and fails the build if
  incomplete
- [x] Run all 13 smoke tests (including `TestMappingCompleteness`) as hard CI gates
- [x] Exact Rust test count assertion (250) instead of lower bound
- [ ] Optionally: CI re-runs discovery scripts and checks for drift between committed
  YAML and actual test trees

## Testing Strategy

- `flowmark-dev check-mapping` is the primary verification mechanism.
- The discovery scripts are tested by running them against the current codebase and
  verifying the output YAML contains expected entries and is valid.
- A basic smoke test in `python/tests/` validates round-trip YAML serialization.

## Resolved Questions

- **YAML over JSON**: YAML chosen for readability, diffability, and agent editability.
  All three artifact files use YAML.

- **Python-based mapping checker over Rust meta-test**: Chosen for simplicity. The
  checker is a Python CLI command, avoiding the need for Python in the Rust test
  environment.
  Can be wrapped by CI directly.

- **1:N mappings**: Supported via `rust_functions: [...]` list field alongside
  `rust_function` for the 1:1 case.

- **Extra Rust tests**: Logged at INFO level, not failures.
  Useful for identifying candidates to upstream to the Python repo.

- **Idempotent merge**: All three commands (`discover-python`, `discover-rust`,
  `init-mapping`) preserve hand-edits.
  Identity keys: `(file, function)` for test manifests,
  `(python_file, python_class, python_function)` for mapping.
  Auto-discovered fields are updated; hand-added entries are never deleted.

- **Rust discovery via cargo vs binary target**: The Python CLI shells out to
  `cargo test -- --list` rather than building a Rust binary.
  Rust has no runtime test introspection — the compiler is the authoritative source
  regardless.
  The Python CLI already handles YAML serialization and subprocess calls, so a Rust
  binary would add complexity (serde_yaml dep, binary target) without benefit.

## Open Questions

- **Reusability for other ports**: The current tool has some flowmark-specific
  assumptions (test type classification heuristics, `fill_markdown` as integration
  indicator).
  If we want to reuse this for other Python→Rust ports, we'd need to make the
  classification configurable (e.g., via a config file or CLI flags).

## References

- Original Python repo: https://github.com/jlevy/flowmark (pinned: `v0.6.4`)
- Porting plan: `docs/project/specs/done/porting-plan.md`
- Python project: `python/pyproject.toml`
- YAML artifacts: `port-coverage-mapping/`
