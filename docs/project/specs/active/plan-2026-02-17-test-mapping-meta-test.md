# Feature: Cross-Language Test Mapping (Port Coverage)

**Date:** 2026-02-17 (last updated 2026-02-17)

**Author:** Joshua Levy

**Status:** Draft — prototype implemented, spec under review

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
The Rust port currently has **151 test functions** across 16 files.

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
    discover_rust.py                 # Regex-based Rust test walker
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

Both discovery commands and `init-mapping` are **idempotent and additive**:

- **Discovery scripts** (`discover-python`, `discover-rust`): Write out the full
  discovered set.
  Since these are auto-generated (not hand-edited), they are fully overwritten each time.
  The header comment marks them as auto-generated.

- **`init-mapping`**: If `test-mapping.yaml` already exists, it loads existing entries,
  preserves all manual edits (status, rust refs, notes), and only adds new `missing`
  entries for newly discovered Python tests.
  It never deletes entries — stale entries are detected by `check-mapping` instead.

- **Hand-added entries**: If a user or agent manually adds a test record to any YAML file
  (e.g., a custom test type the auto-discovery missed), it is preserved across
  re-generation.
  The identity key for merging is `(python_file, python_class, python_function)` for
  mapping and `(file, function)` for test manifests.

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

**Dependencies:** Python 3.11+ stdlib (`ast`, `subprocess`, `tempfile`) + `pyyaml` +
`strif` (for atomic writes).

### Component 2: Rust Test Discovery

#### Option A: Python-based (implemented)

**Command:** `flowmark-dev discover-rust`

**Flags:**
- `--tests-dir` (default: `tests/`)
- `--output` / `-o` — output YAML path (default: `port-coverage-mapping/rust-tests.yaml`)

**Behavior:**
1. Walk `test_*.rs` files.
2. Regex-based parser: find `#[test]` attribute followed by `fn name(`.
3. Record file, function, line number.
4. Write sorted YAML.

#### Option B: Rust build target (proposed, not yet implemented)

A Rust binary target (e.g., `cargo run --bin discover-tests`) or a test helper that uses
compile-time or runtime introspection to enumerate all `#[test]` functions and emit YAML.

**Advantages:**
- More authoritative — the Rust compiler knows exactly what's a test.
- No Python dependency needed for Rust-side discovery.
- Could use `cargo test --list` output parsing as a simple approach.

**Disadvantages:**
- `cargo test --list` output format is not stable/guaranteed.
- Adds a binary target to the Cargo workspace.

**Recommendation:** Start with Option A (Python regex parser) since it's already working
and sufficient.
Evaluate Option B if the regex parser misses edge cases or if we want to eliminate the
Python dependency for Rust-side discovery.
A practical middle ground: parse `cargo test -- --list` output in a Python wrapper.

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

### Phase 2: Idempotent Merge and Polish

- [ ] Add idempotent merge to `discover-python`: if existing `python-tests.yaml` exists,
  preserve hand-added entries not found by auto-discovery
- [ ] Add idempotent merge to `discover-rust`: same behavior
- [ ] Add `--format json` option for machine consumption alongside YAML
- [ ] Run ruff and basedpyright, fix any lint/type issues
- [ ] Add a basic smoke test in `python/tests/`

### Phase 3: Rust-Side Discovery (Optional Enhancement)

- [ ] Evaluate `cargo test -- --list` parsing as an alternative to regex
- [ ] If pursued: add a `discover-rust-native` command that wraps `cargo test --list`
- [ ] Or: add a Rust binary target `discover-tests` that emits YAML directly
- [ ] Compare output of Python regex parser vs Rust-native discovery for correctness

### Phase 4: Populate the Mapping (Agent Labor)

- [ ] For each of the 20 Python test files, review every test function against its Rust
  counterpart
- [ ] Update `test-mapping.yaml` with `mapped`, `excluded`, or `partial` status
- [ ] Handle 1:N cases (e.g., `test_ellipses` → 10 Rust functions)
- [ ] Mark infrastructure tests (`test_skill`, `test_cli_file_discovery`,
  `test_file_resolver`, `test_config`) as `excluded` with notes
- [ ] Verify `flowmark-dev check-mapping` passes with exit code 0
- [ ] Document any `partial` entries with notes on what's missing

### Phase 5: CI Integration (Future)

- [ ] Add a CI step that runs `flowmark-dev check-mapping` and fails the build if
  incomplete
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

- **Idempotent merge**: Both discovery scripts and `init-mapping` preserve hand-edits.
  Identity keys: `(file, function)` for test manifests,
  `(python_file, python_class, python_function)` for mapping.

## Open Questions

- **Rust-native discovery**: Should we add a Rust build target that emits its own test
  list?
  The Python regex parser works but `cargo test -- --list` would be more authoritative.
  Not blocking — can be added later.

- **Reusability for other ports**: The current tool has some flowmark-specific
  assumptions (test type classification heuristics, `fill_markdown` as integration
  indicator).
  If we want to reuse this for other Python→Rust ports, we'd need to make the
  classification configurable (e.g., via a config file or CLI flags).

## References

- Original Python repo: https://github.com/jlevy/flowmark (pinned: `v0.6.4`)
- Porting plan: `docs/porting-plan.md`
- Python project: `python/pyproject.toml`
- YAML artifacts: `port-coverage-mapping/`
