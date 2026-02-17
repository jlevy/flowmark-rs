# Feature: Cross-Language Test Mapping Meta-Test

**Date:** 2026-02-17 (last updated 2026-02-17)

**Author:** Joshua Levy

**Status:** Draft

## Overview

A systematic test provenance tracking system that ensures every Python flowmark test has
a verified Rust counterpart.
The system consists of three components:

1. A **Python discovery script** that walks the Python test suite and emits a JSON
   manifest of every test.
2. A **Rust-side test discovery** mechanism that walks the Rust test files and emits a
   similar JSON manifest.
3. A **hand-maintained mapping file** (JSON) that maps each Python test to its Rust
   equivalent(s), with status and notes.
4. A **Rust meta-test** that loads all three artifacts and asserts completeness: every
   Python test must appear in the mapping, every mapping target must exist in the Rust
   manifest, and any unmapped tests cause a test failure.

When a Python test is added, renamed, or removed upstream, the meta-test breaks.
When a Rust test is added or removed, the meta-test breaks.
The mapping file must be manually updated to resolve failures, ensuring a human (or LLM
agent) has verified the correspondence.

## Goals

- 100% coverage tracking: every Python test has a documented Rust mapping or an explicit
  exclusion reason.
- Machine-verifiable: `cargo test test_mapping_completeness` fails if the mapping is
  stale.
- Low friction: the Python discovery script and Rust discovery are fast, deterministic,
  and require no special environment beyond Python 3.10+ and `cargo test`.
- Incremental: an agent can fill in the mapping one test at a time, and each addition is
  immediately verifiable.

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
The Python test suite has ~234 test functions across 20 files.
The Rust port currently has ~148 test functions across 16 files.
The gap is approximately 37% by count, concentrated in tag formatting (36 vs 16) and
wrapping (38 vs 15).

There is no current mechanism to track which Python tests have been ported, which are
intentionally excluded, and which have drifted.
This spec addresses that gap.

### Python Source Location

The original Python source is available at `https://github.com/jlevy/flowmark`.
A local copy exists at `attic/flowmark/` (gitignored).
The discovery script will work against a fresh checkout of a specified branch/tag to
ensure reproducibility.

### Test Categories in Python

| Category | Files | Description |
|---|---|---|
| Unit tests | `test_ellipses.py`, `test_sentences.py`, `test_smartquotes.py`, `test_escape_handling.py`, `test_strikethrough.py`, `test_wrapping.py` | Test individual functions/modules |
| Integration tests | `test_filling.py`, `test_alerts.py`, `test_cleanups.py`, `test_fenced_code_blocks.py`, `test_frontmatter.py`, `test_heading_spacing.py`, `test_list_spacing.py`, `test_tag_formatting.py`, `test_width_options.py` | Test `fill_markdown` pipeline with various options |
| Golden/fixture tests | `test_ref_docs.py` | Compare full document output against expected fixture files in `tests/testdocs/` |
| Infrastructure tests | `test_cli_file_discovery.py`, `test_config.py`, `test_file_resolver.py`, `test_skill.py` | Test CLI, config, file resolution, skill system |

## Design

### Data Model

Each test is represented as a record with the following fields:

```
TestRecord:
  file: str            # Relative path to test file (e.g., "tests/test_alerts.py")
  function: str        # Test function name (e.g., "test_basic_note_alert")
  class_name: str?     # Enclosing class name, if any (Python only)
  test_type: enum      # "unit" | "integration" | "golden" | "infrastructure"
  line_number: int     # Line number where the test function is defined
  doc_string: str?     # First line of docstring, if present
```

The mapping file uses a record per Python test:

```
MappingRecord:
  python_file: str         # e.g., "tests/test_alerts.py"
  python_function: str     # e.g., "test_basic_note_alert"
  python_class: str?       # e.g., null or "TestSkillInstallation"
  status: enum             # "mapped" | "excluded" | "partial" | "missing"
  rust_file: str?          # e.g., "tests/test_alerts.rs" (null if excluded)
  rust_function: str?      # e.g., "test_basic_note_alert" (null if excluded)
  rust_functions: [str]?   # If one Python test maps to multiple Rust tests
  notes: str?              # Why excluded, what's partial, etc.
```

Status values:
- **mapped**: Python test has a direct Rust equivalent.
  `rust_function` (or `rust_functions` for 1:N) is set.
- **excluded**: Python test is intentionally not ported.
  `notes` explains why (e.g., "Python CLI infrastructure, not applicable to Rust").
- **partial**: Rust test exists but covers only a subset of the Python test's
  assertions.
  `rust_function` is set, `notes` describes what's missing.
- **missing**: Python test has no Rust equivalent yet and should be ported.

### Component 1: Python Test Discovery Script

**Location:** `dev/test_mapping/discover_python_tests.py`

**Inputs:**
- `--repo-url` (default: `https://github.com/jlevy/flowmark`)
- `--ref` (default: `main`) — branch, tag, or commit
- `--output` (default: `dev/test_mapping/python_tests.json`)
- `--local-path` (optional) — use a local checkout instead of cloning

**Behavior:**
1. If `--local-path` is not provided, clone the repo to a temp directory at the
   specified ref.
2. Use Python's `ast` module to walk all `test_*.py` files under `tests/`.
3. For each file, extract every function whose name starts with `test_`.
4. Record: file path (relative to repo root), function name, enclosing class (if any),
   line number, first line of docstring.
5. Classify test type using a heuristic:
   - If file is `test_ref_docs.py` → `golden`
   - If file is in `{test_cli_file_discovery, test_config, test_file_resolver,
     test_skill}` → `infrastructure`
   - If the test calls `fill_markdown` or uses pipeline-level functions → `integration`
   - Otherwise → `unit`
6. Write sorted JSON array to output file.

**Dependencies:** Python 3.10+ standard library only (`ast`, `json`, `pathlib`,
`subprocess`, `tempfile`).
No pytest or third-party libraries needed since we're parsing AST, not running tests.

**Output format** (`python_tests.json`):
```json
[
  {
    "file": "tests/test_alerts.py",
    "function": "test_basic_note_alert",
    "class_name": null,
    "test_type": "integration",
    "line_number": 14,
    "doc_string": "Test basic [!NOTE] alert formatting."
  },
  ...
]
```

### Component 2: Rust Test Discovery

**Location:** `dev/test_mapping/discover_rust_tests.py`

This is also a Python script (for consistency and ease of text parsing).
It walks the Rust `tests/` directory.

**Inputs:**
- `--tests-dir` (default: `tests/`)
- `--output` (default: `dev/test_mapping/rust_tests.json`)

**Behavior:**
1. Walk all `test_*.rs` files under the tests directory.
2. Use a regex-based parser to find all `#[test]` annotated functions:
   - Pattern: `#[test]` followed by `fn <name>(` (possibly with intervening attributes
     or comments).
3. Record: file path, function name, line number.
4. Write sorted JSON array to output file.

**Output format** (`rust_tests.json`):
```json
[
  {
    "file": "tests/test_alerts.rs",
    "function": "test_basic_note_alert",
    "line_number": 8
  },
  ...
]
```

### Component 3: Hand-Maintained Mapping File

**Location:** `dev/test_mapping/test_mapping.json`

This file is checked into the repository.
It is the source of truth for test provenance.
An agent or human fills it in by:

1. Running both discovery scripts to generate current manifests.
2. For each Python test, examining the Python test code and the candidate Rust test code.
3. Adding a mapping record with the appropriate status.

The mapping file is sorted by `(python_file, python_class, python_function)` for
deterministic diffs.

### Component 4: Rust Meta-Test

**Location:** `tests/test_mapping_completeness.rs`

A Rust integration test that:

1. Shells out to run both discovery scripts (or reads pre-generated JSON files — see
   open questions).
2. Loads `python_tests.json`, `rust_tests.json`, and `test_mapping.json`.
3. Asserts:
   - **Every Python test has a mapping entry.** If a new Python test appears that isn't
     in the mapping, the test fails with a clear message listing the unmapped tests.
   - **Every mapped Rust test exists.** If a mapping says `rust_function:
     "test_foo"` in `test_bar.rs`, that function must appear in `rust_tests.json`.
   - **No stale mapping entries.** If a Python test was removed upstream but still has a
     mapping entry, the test warns (or fails, configurable).
   - **Summary statistics.** Print a coverage report: N mapped, N excluded, N partial, N
     missing out of N total Python tests.
4. The test passes only when every Python test is either `mapped`, `excluded`, or
   `partial` — never `missing`.

### Directory Layout

```
dev/
  test_mapping/
    discover_python_tests.py    # Python test discovery script
    discover_rust_tests.py      # Rust test discovery script
    test_mapping.json           # Hand-maintained mapping (checked in)
    python_tests.json           # Generated (gitignored)
    rust_tests.json             # Generated (gitignored)
    README.md                   # Usage instructions
tests/
  test_mapping_completeness.rs  # Rust meta-test
```

### Workflow

**Initial population (one-time, by LLM agent):**
1. Run `python dev/test_mapping/discover_python_tests.py` → generates
   `python_tests.json`.
2. Run `python dev/test_mapping/discover_rust_tests.py` → generates `rust_tests.json`.
3. Generate a skeleton `test_mapping.json` with every Python test listed as `missing`.
4. For each Python test, the agent reads the Python source and the candidate Rust source,
   determines the mapping, and updates the record.
   This is the labor-intensive step.
5. Commit `test_mapping.json`.
   The meta-test now passes.

**Ongoing maintenance:**
- When a Python test is added upstream: the meta-test fails → agent adds a mapping entry
  (either ports the test or marks it excluded).
- When a Rust test is added: the meta-test may fail if no mapping references it (this is
  fine — extra Rust tests are allowed but the mapping should be updated for traceability).
- When a Python test is removed upstream: the meta-test fails → agent removes or updates
  the mapping entry.
- Periodically: re-run discovery scripts and verify the mapping is current.

## Implementation Plan

### Phase 1: Scaffolding and Discovery Scripts

- [ ] Create `dev/test_mapping/` directory structure
- [ ] Write `discover_python_tests.py` using `ast` module
- [ ] Write `discover_rust_tests.py` using regex parsing
- [ ] Add `.gitignore` entries for generated JSON files (`python_tests.json`,
  `rust_tests.json`)
- [ ] Test both scripts produce correct output against current codebase
- [ ] Add a `--skeleton` flag to `discover_python_tests.py` that generates a starter
  `test_mapping.json` with all entries as `missing`

### Phase 2: Meta-Test and Mapping Infrastructure

- [ ] Write `tests/test_mapping_completeness.rs` that loads the three JSON files and
  asserts completeness
- [ ] Generate initial skeleton `test_mapping.json` using the `--skeleton` flag
- [ ] Verify the meta-test correctly fails (all entries are `missing`)
- [ ] Add `README.md` with usage instructions

### Phase 3: Populate the Mapping (Agent Labor)

- [ ] For each Python test file, review every test function against its Rust counterpart
- [ ] Update `test_mapping.json` with `mapped`, `excluded`, or `partial` status for each
  entry
- [ ] Verify `cargo test test_mapping_completeness` passes
- [ ] Document any `partial` entries with notes on what's missing

## Testing Strategy

- The meta-test itself is the primary test.
  It runs as part of `cargo test`.
- The discovery scripts should be tested with a small smoke test: run them and verify the
  output JSON is valid and contains expected entries.
- The meta-test should print a human-readable summary showing coverage statistics.

## Open Questions

- **Should the meta-test shell out to Python at test time, or read pre-committed JSON?**
  Recommendation: read pre-committed JSON files.
  The discovery scripts are run manually (or by CI) and their output is committed.
  This avoids requiring Python in the Rust test environment.
  The trade-off is that the JSON manifests could drift — but the mapping update workflow
  catches this since the agent re-runs discovery before updating the mapping.
  Alternatively: a CI job runs the discovery scripts and commits updated JSON, and the
  meta-test just reads those files.

- **Should the Rust meta-test also be implemented in Python for simplicity?**
  Since both discovery scripts are Python, a pure Python meta-test
  (`dev/test_mapping/check_mapping.py`) could complement or replace the Rust integration
  test.
  Recommendation: implement it as a Python script in `dev/test_mapping/` that is also
  callable from a Rust test via `Command::new("python")`.
  This keeps the tooling self-contained and makes debugging easier.

- **How to handle 1:N mappings (one Python test → multiple Rust tests)?**
  The Python ellipsis test is one monolithic function with 98 assertions, split into 10
  Rust tests.
  The mapping record should support `rust_functions: [...]` as an alternative to
  `rust_function`.

- **How to handle parameterized tests?**
  Some Python tests use inline parameterization (multiple assert blocks in one function).
  The Rust side may split these into separate `#[test]` functions.
  The mapping should treat the Python function as the atomic unit and list all
  corresponding Rust functions.

- **Should extra Rust-only tests cause a warning or be silently allowed?**
  Recommendation: silently allowed.
  Extra Rust tests (e.g., comrak-specific regression tests) are fine.
  The meta-test only ensures Python → Rust coverage, not the reverse.

## References

- Original Python repo: https://github.com/jlevy/flowmark
- Porting plan: `docs/porting-plan.md`
- Previous cross-validation assessment:
  `attic/flowmark-rs-1/docs/project/cross-validation-assessment.md`
