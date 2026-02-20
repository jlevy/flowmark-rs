# Feature: Parity Review and Playbook Sync

**Date:** 2026-02-19 (last updated 2026-02-19)

**Author:** Joshua Levy

**Status:** Draft

## Overview

This spec consolidates all remaining work to achieve full CLI/feature parity with Python
flowmark v0.6.4 and to reconcile the project’s documentation with the
rust-porting-playbook.

It covers two major work streams:

1. **Part A: CLI & Feature Parity** — Port file resolver, config loading, CLI flags, and
   skill system (79 previously excluded tests).
   Establish tryscript golden tests as the cross-language parity contract.
2. **Part B: Playbook Review & Sync** — Update the porting playbook case study with
   current metrics, integrate lessons learned, fix stale documentation, and backfill 13
   pending observations.

Both work streams are prerequisites for claiming flowmark-rs is a true drop-in
replacement for Python flowmark.

**Epic bead:** fmr-7mmt (CLI parity)

## Goals

- Port all remaining Python CLI features: file resolver, config loading, 11 missing CLI
  flags, skill system
- Port all 79 previously excluded tests (31 file resolver + 20 config + 19 CLI + 9
  skill)
- Achieve `check-mapping` target: **281 mapped, 0 excluded, 0 missing, 0 partial**
- Establish tryscript-based golden tests for end-to-end CLI validation
- Update playbook case study to reflect the current port (not the old flowmark-rs-1)
- Integrate 13 Phase 7C observations into the playbook
- Backfill all lessons from the porting log into appropriate playbook documents
- Fix all stale metrics, contradictions, and documentation gaps

## Non-Goals

- Performance benchmarking (tracked separately as fmr-aq8o)
- Adding features beyond Python v0.6.4
- Restructuring the playbook’s 8-phase methodology
- Adding entirely new playbook documents (only updating existing ones)

## Background

### Part A: CLI & Feature Parity

The exact parity spec (Phases 1-9) achieved byte-for-byte formatting parity: 481 tests
pass, 0 ignored, all formatting modes match exactly.
However, 79 Python tests remain excluded because the features they test are not yet
ported:

| File | # Tests | Feature |
| --- | --- | --- |
| `test_cli_file_discovery.py` | 19 | CLI arg handling, `--auto` mode, file discovery, error messages |
| `test_config.py` | 20 | TOML config loading, pyproject.toml, three-way merge |
| `test_file_resolver.py` | 31 | Directory recursion, glob expansion, gitignore, exclude patterns |
| `test_skill.py` | 9 | Claude Code skill installation (`--skill`, `--install-skill`, `--docs`) |

### Part B: Playbook Review & Sync

The rust-porting-playbook was built primarily from the **first** flowmark-rs port
(flowmark-rs-1, in `attic/flowmark-rs-1/`). That port achieved:
- 141 tests (93 unit + 42 integration + 6 doctests), 2 ignored
- ~95% cross-validation match
- 14 library workarounds, 3 accepted differences
- Rust/Python LOC ratio: ~1.7x app code

The **current** flowmark-rs (this repo) is a fresh reimplementation that achieved:
- 481 tests, 0 ignored
- 202 mapped Python tests + 79 excluded (infrastructure)
- 100% of ported tests passing, 0 partial mappings
- Rust/Python code lines ratio: 1.00x (5,284 vs 5,279)

The playbook case study docs still describe the **old** port.
This spec addresses that gap.

### Key Discrepancies Found

#### D1: Stale Case Study Metrics (CRITICAL)

The playbook README and case study docs describe the **old** flowmark-rs-1 port:

| Metric | Playbook Says | Actual (Current Port) |
| --- | --- | --- |
| Test count | 141 (93+42+6), 2 ignored | 481, 0 ignored |
| Python test mapping | Not tracked | 202 mapped, 79 excluded, 0 missing |
| Rust/Python LOC ratio (app) | ~1.7x | 1.03x (2,610 vs 2,531) |
| Rust/Python LOC ratio (total) | ~1.8x | 0.69x (6,909 vs 10,052) |
| Code lines ratio | Not stated | 1.00x (5,284 vs 5,279) |
| Cross-validation | ~95% match | 100% of ported tests |
| Workarounds | “14 fixable, 3 unfixable” | Different set (new implementation) |

#### D2: Workaround Count Inconsistencies

Workaround counts differ across documents: 13, 14, 15, 17 appear in different places.
The current port has a different set of workarounds than the old port.

#### D3: `porting-checklist.md` is a Stale Duplicate

`docs/project/specs/active/porting-checklist.md` in this repo is a copy of the
playbook’s `reference/python-to-rust-playbook.md`. It may be out of sync.

#### D5: Phase 7C Observations Not Integrated

13 observations were recorded in
`attic/rust-porting-playbook/case-studies/flowmark/flowmark-port-observations-2.md` but
not yet integrated into the playbook.

#### D6: Code Review Findings vs Playbook Best Practices

| Code Review Issue | Playbook Coverage |
| --- | --- |
| P0: Clippy failures (9 errors) | Playbook says clippy pedantic — but as warn not deny |
| P0: Formatting violations | Playbook says `cargo fmt` — not enforced |
| P0.5: Lint config gaps (warn vs deny) | Playbook says pedantic warn, code review says deny |
| P1: Dead dependencies | Playbook has no guidance on dependency pruning |
| P1: Dead error variants | Playbook has no dead-code-detection guidance |
| P2: Code duplication (fence tracking) | Code review checklist has nothing on duplication |
| P2: Unnecessary allocations | Code review checklist covers “hot path allocation” |
| P2: Boolean parameter overload | Playbook recommends options structs |
| P3: Stale comments ("Same as Black") | Playbook has no Python-reference-cleanup guidance |

#### D7-D10: Other Issues

- **D7**: `XXX:` comment convention → should be `HACK:`/`FIXME:` per updated playbook
- **D8**: Non-compiling code examples in playbook porting guide
- **D9**: 53+ playbook fixes (plan-2026-02-08) — unclear if all applied
- **D10**: Playbook references archived/stale crates (`serde_yaml`, `once_cell`,
  `actions/checkout@v5`, `color-eyre`)

### New Lessons for Playbook (from Current Port)

| Lesson | Playbook Impact | Category |
| --- | --- | --- |
| Cross-language test mapping system (YAML-based) | Major — reusable for any port | ADD |
| `flowmark-dev` CLI for test discovery + mapping | Major — tool pattern for ports | ADD |
| Test mapping as CI hard gate | Important — enforcement pattern | ADD |
| Golden test with 4 formatting modes | Important — testing pattern | ADD |
| Exact parity spec as tracking document | Important — project management | ADD |
| `pub(crate)` visibility audit as post-port step | Medium — code quality | ADD |
| Unicode PUA placeholder pattern for escape preservation | Niche — text processing | ADD (case study) |
| Lint configuration: pedantic as deny, not warn | Important — stricter than playbook | CLARIFY |
| `warnings = "deny"` in Cargo.toml (not just CI) | Important — consistency | ADD |
| Edition 2024 `resolver = "3"` | Important — already identified | FIX |

### Document Inventory

#### flowmark-rs docs (this repo: `docs/`)

| Document | Lines | Current Status | Notes |
| --- | --- | --- | --- |
| `project/specs/done/porting-plan.md` | 137 | Complete | Updated and moved to specs/done |
| `porting-checklist.md` | 643 | Duplicate | Copy of playbook’s `python-to-rust-playbook.md` |
| `code-review-2026-02-17.md` | 471 | Current | Fresh review with P0-P3 issues |
| `specs/active/plan-2026-02-17-exact-parity.md` | 751 | Complete | Comprehensive parity spec |
| `specs/active/plan-2026-02-17-test-mapping-meta-test.md` | 482 | Implemented | Test mapping infrastructure |

#### rust-porting-playbook (attic: `attic/rust-porting-playbook/`)

**Reference docs (11 files, ~5,289 lines):**

| Document | Lines | Status |
| --- | --- | --- |
| `reference/python-to-rust-playbook.md` | ~619 | Core doc — needs metric updates |
| `reference/python-to-rust-mapping-reference.md` | ~788 | Comprehensive — review for accuracy |
| `reference/python-to-rust-porting-guide.md` | ~807 | Detailed methodology — check code examples |
| `reference/rust-cli-best-practices.md` | ~832 | Extensive — check versions/deps |
| `reference/rust-code-review-checklist.md` | ~285 | Good shape — validate against code review |
| `reference/python-to-rust-test-coverage-playbook.md` | ~312 | Review against actual test strategy |
| `reference/port-checklist-initial-template.md` | ~546 | Template — validate completeness |
| `reference/port-checklist-update-template.md` | ~379 | Template — validate completeness |
| `reference/meta-improving-this-playbook.md` | ~236 | Process doc — current |
| `reference/case-study-observations-template.md` | ~252 | Template — current |
| `reference/case-study-improvement-triage-template.md` | ~148 | Template — current |

**Guidelines (6 files, ~2,246 lines):**

| Document | Lines | Status |
| --- | --- | --- |
| `guidelines/python-to-rust-porting-rules.md` | ~360 | Core rules — multiple fixes identified |
| `guidelines/python-to-rust-cli-porting.md` | ~285 | CLI porting — validate |
| `guidelines/rust-general-rules.md` | ~286 | General Rust — review |
| `guidelines/rust-cli-app-patterns.md` | ~403 | CLI patterns — review |
| `guidelines/rust-project-setup.md` | ~626 | Project setup — critical for accuracy |
| `guidelines/test-coverage-for-porting.md` | ~286 | Test coverage — review |

**Case studies (8 files, ~5,400 lines):**

| Document | Lines | Status |
| --- | --- | --- |
| `case-studies/flowmark/flowmark-port-analysis.md` | ~326 | STALE — describes old port |
| `case-studies/flowmark/flowmark-port-library-choices.md` | ~257 | Partially current |
| `case-studies/flowmark/flowmark-port-decision-log.md` | ~523 | STALE — old decisions |
| `case-studies/flowmark/flowmark-port-migration-plan-v1.md` | ~3,339 | Renamed — v1 port |
| `case-studies/flowmark/flowmark-port-migration-plan-v2.md` | ~400 | NEW — v2 port |
| `case-studies/flowmark/flowmark-port-cross-validation.md` | ~189 | STALE — old validation |
| `case-studies/flowmark/flowmark-port-comrak-bug.md` | ~211 | Partially current |
| `case-studies/flowmark/flowmark-port-wrapping-solution.md` | ~155 | STALE — old approach |

## Design

### Approach

Part A (CLI parity) follows the same methodology as the exact parity spec: port each
Python module, write tests that match the Python test suite, validate via cross-language
test mapping, enforce in CI.

Part B (playbook sync) is a documentation reconciliation: inventory every document,
verify against current state, fix discrepancies, backfill lessons.

Both parts are independent and can proceed in parallel.

### New Dependencies for Part A

| Rust Crate | Replaces (Python) | Purpose |
| --- | --- | --- |
| `ignore` | `pathspec` + `os.walk` | Gitignore-aware directory walking and glob matching |
| `toml` | `tomllib` / `tomli` | TOML config file parsing |
| `serde` | — | Deserialization for TOML config struct |
| `glob` | `pathlib.Path.glob()` | Glob pattern expansion (if `ignore` doesn’t cover all cases) |
| `dirs` | — | Home directory resolution (for skill installation) |

All should be feature-gated under `cli` except `serde`/`toml` if config loading is in
the library.

## Implementation Plan

### Phase 1: File Resolver Module

**Bead:** fmr-t834 | **Python source:** 402 lines across 4 files | **Tests:** 31 |
**Estimated Rust LOC:** 400-500

Port `file_resolver/` (4 Python files → Rust module).
This is the largest gap.

**Python source files (all in `attic/flowmark/src/flowmark/file_resolver/`):**
- `resolver.py` (222 lines) — `FileResolver` class: `resolve()`, `_walk_directory()`,
  `_should_include_explicit()`, `_is_dir_excluded()`, `_expand_glob()`,
  `_exceeds_max_size()`, `_get_gitignore()`, `_get_gitignore_chain()`,
  `_get_tool_ignore()`
- `types.py` (39 lines) — `FileResolverConfig` dataclass with `effective_include` and
  `effective_exclude` properties
- `defaults.py` (57 lines) — `DEFAULT_INCLUDES` and `DEFAULT_EXCLUDES` constants
- `gitignore.py` (54 lines) — `_read_ignore_file()`, `load_gitignore()`,
  `load_tool_ignore()`

**Python data structures:**

`FileResolverConfig` fields (all with defaults):
- `tool_name: str = "flowmark"` — determines ignore file name (`.flowmarkignore`)
- `include: list[str] = ["*.md"]` — base include patterns
- `extend_include: list[str] = []` — additional include patterns
- `exclude: list[str] | None = None` — `None` = use `DEFAULT_EXCLUDES`; list replaces
  them
- `extend_exclude: list[str] = []` — added to effective excludes
- `respect_gitignore: bool = True`
- `force_exclude: bool = False` — apply exclusions to explicitly named files
- `files_max_size: int = 1_048_576` — 1 MiB; 0 = no limit

Properties:
- `effective_include` → `include + extend_include`
- `effective_exclude` → `(exclude ?? DEFAULT_EXCLUDES) + extend_exclude`

`DEFAULT_EXCLUDES` (30 patterns — must match exactly):

```
.git/, .hg/, .svn/, .bzr/, _darcs/,
.venv/, venv/, __pycache__/, .tox/, .nox/, .mypy_cache/, .ruff_cache/, .pytest_cache/,
.eggs/, *.egg-info/,
build/, dist/,
node_modules/, .next/, .nuxt/, .output/, .cache/, .parcel-cache/, .turbo/,
.idea/, .vscode/, .vs/, .fleet/,
coverage/, htmlcov/, .coverage/,
vendor/, third_party/, Pods/, target/, .terraform/
```

**Python behavior (exact algorithm):**

`FileResolver.resolve(paths)` dispatches each input path:
- **Existing file** → pass through (apply `force_exclude` and `max_size` checks via
  `_should_include_explicit`)
- **Existing directory** → recursive walk with all filters (`_walk_directory`)
- **Contains glob chars** (`*`, `?`, `[`) → expand via `Path.glob()`, filter results
  (`_expand_glob`)
- **Otherwise** → raise `FileNotFoundError(f"Path not found: {raw_path}")`

Result: deduplicated by `Path.resolve()` into `seen: set[Path]`, sorted
lexicographically.

`_walk_directory(root)`:
1. Load tool ignore once per walk root: `_get_tool_ignore(root)` — walks up from root
   looking for `.flowmarkignore`, caches per `resolved` directory
2. `os.walk(root)` loop: a. Compute `rel_to_root = current.relative_to(root)` b. Prune
   directories in-place:
   `dirnames[:] = [d for d if not _is_dir_excluded(d, rel/d, current, tool_ignore, root)]`
   c. Collect gitignore specs: `_get_gitignore_chain(current, root)` if
   `respect_gitignore` d. For each file: check `include_spec` → check `max_size` → check
   gitignore chain → check tool ignore → yield

`_is_dir_excluded(dirname, rel_path, current_dir, tool_ignore, walk_root)`:
1. Check `exclude_spec.match_file(dirname + "/")` — bare directory name
2. Check `exclude_spec.match_file(str(rel_path) + "/")` — relative path
3. If `respect_gitignore`: check all specs from
   `_get_gitignore_chain(current_dir, root)` against `dirname + "/"`
4. Check `tool_ignore.match_file(dirname + "/")` and
   `tool_ignore.match_file(str(rel) + "/")` if tool_ignore exists

`_should_include_explicit(path)`:
1. If `force_exclude`: check `filename` and each `parent.parts[:-1]` component + “/”
   against `exclude_spec`
2. Check `_exceeds_max_size(path)` (0 = no limit; `OSError` on `stat()` → return False,
   i.e., include)

`_get_gitignore_chain(directory, walk_root)`:
- Walk from `walk_root.resolve()` down to `directory.resolve()`, collecting
  `_get_gitignore(dir)` for each ancestor directory (inclusive)
- Returns `list[PathSpec]`

`_read_ignore_file(path)`:
- Read text; return `None` on `OSError` or `UnicodeDecodeError`
- Strip comments (`#`) and blank lines
- Return `PathSpec.from_lines("gitignore", lines)` or `None` if no active rules

`load_tool_ignore(tool_name, start_dir)`:
- Walk up from `start_dir.resolve()` to filesystem root
- Look for `.{tool_name}ignore` (e.g., `.flowmarkignore`) in each directory
- Return first found via `_read_ignore_file`, or `None`

**Rust implementation plan:**

New dependency: `ignore` crate (gitignore parsing; may also use `glob` crate for pattern
expansion).

Module structure:
- [ ] `src/file_resolver/mod.rs` — public API re-exports
- [ ] `src/file_resolver/config.rs` — `FileResolverConfig` struct with
  `effective_include()` and `effective_exclude()` methods
- [ ] `src/file_resolver/defaults.rs` — `DEFAULT_INCLUDES` and `DEFAULT_EXCLUDES`
  constants (exact match with Python)
- [ ] `src/file_resolver/gitignore.rs` — `read_ignore_file()`, `load_gitignore()`,
  `load_tool_ignore()` using `ignore::gitignore::GitignoreBuilder` or custom parsing
- [ ] `src/file_resolver/resolver.rs` — `FileResolver` struct with `resolve()`,
  `walk_directory()`, `should_include_explicit()`, `is_dir_excluded()`, `expand_glob()`,
  `exceeds_max_size()`, `get_gitignore_chain()`, `get_tool_ignore()`
- [ ] Register module in `src/lib.rs`: `pub mod file_resolver;`
- [ ] Port all 31 tests to `tests/test_file_resolver.rs` (use `tempfile` crate for temp
  directories)
- [ ] Update `test-mapping.yaml`: change 31 entries from `excluded` → `mapped`

**Rust test mapping (31 tests from `test_file_resolver.py`):**

Config tests (4):
- `test_config_effective_include` — include pattern merging
- `test_config_effective_include_custom_base` — custom base patterns
- `test_config_effective_exclude_replaced` — exclude replacement
- `test_config_effective_exclude_extended` — exclude extension

Resolver core (7):
- `test_resolver_single_file` — explicit file pass-through
- `test_resolver_directory_recursion` — recursive walk, only `*.md` found
- `test_resolver_excludes_default_dirs` — `node_modules/`, `.venv/` excluded
- `test_resolver_respects_gitignore` — `.gitignore build/` respected
- `test_resolver_no_respect_gitignore` — `respect_gitignore=False` override
- `test_resolver_force_exclude_filters_explicit_files` — `force_exclude=True` filters
  `node_modules/README.md`
- `test_resolver_explicit_files_bypass_exclusions_by_default` — `force_exclude=False`
  passes through

Filter options (5):
- `test_resolver_extend_include` — `extend_include=["*.mdx"]` adds patterns
- `test_resolver_exclude_replaces_defaults` — `exclude=["custom_dir/"]` replaces
  defaults
- `test_resolver_extend_exclude` — `extend_exclude=["drafts/"]` adds to defaults
- `test_resolver_files_max_size` — 2MB file excluded with default 1MB limit
- `test_resolver_files_max_size_zero_disables` — `max_size=0` disables limit

Glob and deduplication (4):
- `test_resolver_glob_pattern` — `docs/*.md` glob expansion
- `test_resolver_mixed_inputs` — explicit file + directory together
- `test_resolver_deduplication` — same file listed twice → one result
- `test_resolver_sorted_output` — results are sorted

Error handling (1):
- `test_resolver_file_not_found` — nonexistent path → `FileNotFoundError`

Tool ignore (2):
- `test_resolver_flowmarkignore` — `.flowmarkignore` with `drafts/`
- `test_resolver_tool_ignore_per_walk_root` — separate `.flowmarkignore` per walk root

Gitignore specifics (5):
- `test_resolver_nested_gitignore` — nested `.gitignore` in subdirectory
- `test_resolver_nested_gitignore_combines_parent_rules` — parent `*.log` rule applies
  in child
- `test_resolver_gitignore_file_patterns` — `draft.md` file pattern (not just
  directories)
- `test_resolver_gitignore_wildcard_file_pattern` — `temp.*` wildcard pattern

Ignore file internals (3):
- `test_read_ignore_file_missing` — missing file → `None`
- `test_read_ignore_file_unreadable` — unreadable file (chmod 000) → `None`
- `test_read_ignore_file_non_utf8` — non-UTF-8 bytes → `None`

Flowmarkignore positive (1):
- `test_resolver_flowmarkignore_positive_assertion` — verify exactly which files kept

### Phase 2: Config Loading

**Bead:** fmr-z8j5 | **Python source:** 184 lines | **Tests:** 20 | **Estimated Rust
LOC:** 250-350

Port `config.py` (TOML-based config file loading with three-way merge).
The existing `src/config.rs` (83 lines) has `ListSpacing` and `FormatOptions` but no
TOML loading — extend it or create `src/config/` module.

**Python source:** `attic/flowmark/src/flowmark/config.py` (184 lines)

**Python data structures:**

`FlowmarkConfig` — all fields are `Option<T>` (Python `None`) to distinguish “not
configured” from “explicitly set to default”:

```
# Formatting
width: int | None = None
semantic: bool | None = None
cleanups: bool | None = None
smartquotes: bool | None = None
ellipses: bool | None = None
list_spacing: str | None = None

# File discovery
include: list[str] | None = None
extend_include: list[str] | None = None
exclude: list[str] | None = None
extend_exclude: list[str] | None = None
files_max_size: int | None = None
respect_gitignore: bool | None = None
force_exclude: bool | None = None
```

`_CONFIG_FILENAMES = [".flowmark.toml", "flowmark.toml", "pyproject.toml"]`

Kebab-to-snake mapping table (6 entries):

```
"list-spacing" → "list_spacing"
"extend-include" → "extend_include"
"extend-exclude" → "extend_exclude"
"files-max-size" → "files_max_size"
"respect-gitignore" → "respect_gitignore"
"force-exclude" → "force_exclude"
```

Auto-locked fields (not overridable by config in `--auto` mode):
`{"semantic", "cleanups", "smartquotes", "ellipses", "inplace", "nobackup"}`

**Python behavior (exact algorithm):**

`find_config_file(start_dir)`:
1. `current = start_dir.resolve()`
2. Loop: for each of `.flowmark.toml`, `flowmark.toml`, `pyproject.toml`:
   - If file exists: for `pyproject.toml`, check
     `_pyproject_has_flowmark_section(candidate)` first; for others, return immediately
3. `parent = current.parent`; if `parent == current`, break (filesystem root)
4. `current = parent`; repeat
5. Return `None`

`_pyproject_has_flowmark_section(path)`:
- Parse TOML; check `"flowmark" in data.get("tool", {})`
- Catch `TOMLDecodeError` and `OSError` → return `False`

`load_config(config_path)`:
1. Parse TOML text; on `TOMLDecodeError`/`OSError`:
   `eprintln!("Warning: could not parse config file {config_path}")` and return empty
   config
2. If `config_path.name == "pyproject.toml"`: extract `data["tool"]["flowmark"]`
   subsection
3. Call `_parse_config_data(data)`: a. Flatten nested sections: any `dict` value’s
   sub-keys merge to top level b. Map kebab-case → snake_case via lookup table
   (fallback: `key.replace("-", "_")`) c. Validate against `FlowmarkConfig` field names
   d. Unrecognized keys: `eprintln!("Warning: unrecognized config key '{key}'")` e.
   Construct `FlowmarkConfig(**mapped)`

`merge_cli_with_config(cli_opts, config, is_auto, explicit_flags)`:
1. If `config` is `None`, return `cli_opts` unchanged
2. For each field in `FlowmarkConfig`: a. If field value is `None` → skip (not set in
   config) b. If field name in `explicit_flags` → skip (CLI takes precedence) c. If
   `is_auto` and field name in `auto_locked` → skip d. Set
   `cli_opts.{field} = config_value`
3. Return `cli_opts`

**Rust implementation plan:**

New dependency: `toml` crate (add to `[dependencies]` with `optional = true` under `cli`
feature, or unconditional if config loading belongs in lib).

- [ ] Add `serde` and `toml` crates as dependencies
- [ ] Create `FlowmarkConfig` struct with all-`Option<T>` fields (use
  `#[derive(Default, Deserialize)]`)
- [ ] Implement `find_config_file(start_dir: &Path) -> Option<PathBuf>` — directory walk
  with per-directory search order
- [ ] Implement `pyproject_has_flowmark_section(path: &Path) -> bool`
- [ ] Implement `load_config(config_path: &Path) -> FlowmarkConfig` — TOML parsing with
  section flattening, kebab→snake mapping, and warning messages to stderr
- [ ] Implement `merge_cli_with_config(cli_opts, config, is_auto, explicit_flags)` —
  three-way merge
- [ ] Decide: extend existing `src/config.rs` or create `src/config/` module with
  `mod.rs` + `toml_config.rs`
- [ ] Port 20 tests to `tests/test_config.rs` (use `tempfile` for temp directories)
- [ ] Update `test-mapping.yaml`: change 20 entries from `excluded` → `mapped`

**Rust test mapping (20 tests from `test_config.py`):**

Config file discovery (6):
- `test_find_config_flowmark_toml` — finds `flowmark.toml`
- `test_find_config_dot_flowmark_toml_takes_precedence` — `.flowmark.toml` >
  `flowmark.toml`
- `test_find_config_pyproject_toml` — finds `pyproject.toml` with `[tool.flowmark]`
- `test_find_config_pyproject_without_section_skipped` — skips `pyproject.toml` without
  section
- `test_find_config_walks_up` — finds config in parent directory
- `test_find_config_none_when_missing` — returns `None` when no config exists

Config loading (5):
- `test_load_config_flowmark_toml` — loads formatting options, unset fields are `None`
- `test_load_config_pyproject_toml` — extracts `[tool.flowmark]` section
- `test_load_config_kebab_case` — kebab-case keys mapped correctly (all 6 mappings)
- `test_load_config_file_discovery_section` — `[file-discovery]` section parsed
- `test_load_config_partial` — partial config, unset fields remain `None`

Config merge (7):
- `test_merge_no_config` — `None` config returns defaults unchanged
- `test_merge_config_overrides_defaults` — config values override defaults
- `test_merge_explicit_cli_overrides_config` — explicit CLI flag beats config
- `test_merge_auto_mode_overrides_formatting` — `--auto` locks formatting on
- `test_merge_auto_mode_width_from_config` — width comes from config even in auto mode
- `test_merge_file_discovery_from_config` — file discovery settings from config
- `test_merge_extend_include_from_config` — `extend_include` from config applied

Error handling (2):
- `test_load_config_malformed_toml` — malformed TOML returns empty config (no crash)
- `test_parse_config_warns_unknown_keys` — unknown keys produce
  `"unrecognized config key"` warning to stderr

### Phase 3: CLI Flag Parity

**Bead:** fmr-4sc5 | **Python source:** 527 lines (`cli.py`) | **Tests:** 19 |
**Estimated Rust LOC:** 200-300 | **Depends on:** Phase 1, Phase 2

Add all missing Python CLI flags to `main.rs`, integrate file resolver and config
loading, port argument validation and error messages.

**Current Rust CLI state (`src/main.rs`, 143 lines):**
- `Args` struct has 13 fields (files, output, width, plaintext, semantic, cleanups,
  smartquotes, ellipses, list_spacing, inplace, nobackup, auto, verbose)
- `run()` function: parse args → auto expansion → build `FormatOptions` → simple file
  loop (stdin or file, no directory support)
- No file discovery, no config loading, no `--list-files`, no skill flags
- Default `files = ["-"]` (stdin) — Python changed to `files = []` (empty, requires
  explicit input)

**Missing flags (11 — Python has, Rust doesn’t):**

| Flag | clap Type | Default | Purpose |
| --- | --- | --- | --- |
| `--extend-include PATTERN` | `Vec<String>` (append) | `[]` | Additional file include patterns |
| `--exclude PATTERN` | `Option<Vec<String>>` (append) | `None` | Replace default exclusions |
| `--extend-exclude PATTERN` | `Vec<String>` (append) | `[]` | Add to default exclusions |
| `--no-respect-gitignore` | `bool` (flag) | `false` | Disable .gitignore integration |
| `--force-exclude` | `bool` (flag) | `false` | Apply exclusions to explicit files |
| `--list-files` | `bool` (flag) | `false` | Print resolved paths, don’t format |
| `--files-max-size BYTES` | `usize` | `1_048_576` | Skip files larger than N bytes |
| `--skill` | `bool` (flag) | `false` | Print SKILL.md content to stdout |
| `--install-skill` | `bool` (flag) | `false` | Install skill to `~/.claude/` |
| `--agent-base DIR` | `Option<String>` | `None` | Custom agent config dir |
| `--docs` | `bool` (flag) | `false` | Print documentation to stdout |

**Already present in Rust (no action needed):**
- `--version` — clap `version` derive (auto-generated from Cargo.toml)
- `--verbose` (`-v`) — Rust-only addition (doesn’t break drop-in compatibility)

**Critical behavior change: default files argument**

Python changed from `default=["-"]` (stdin) to `default=[]` (empty, requires explicit
input). Rust must match:
- Change `#[arg(default_value = "-")]` to no default
- Add validation: if `files.is_empty()`, print error and exit 1

**Explicit-flag tracking (for config merge precedence):**

Python uses a sentinel parser to detect which flags the user explicitly passed (even if
the value matches the default).
Rust approach options:
- Use `clap`'s `value_source()` method to check if a value came from CLI vs default
- Or: parse with `Option<T>` for tracked fields, then apply defaults after detection

Tracked flags (12): `width`, `semantic`, `cleanups`, `smartquotes`, `ellipses`,
`list_spacing`, `extend_include`, `exclude`, `extend_exclude`, `respect_gitignore`,
`force_exclude`, `files_max_size`

**Error messages (must match Python exactly):**

1. `--auto` without files (exit 1):

   ```
   Error: --auto requires at least one file or directory argument (use '.' for current directory, --help for more options)
   ```

2. `--list-files` without files (exit 1):

   ```
   Error: --list-files requires at least one file or directory argument (use '.' for current directory, --help for more options)
   ```

3. No input at all (exit 1):

   ```
   Error: No input specified. Provide files, directories (use '.' for current directory), or '-' for stdin. Use --help for more options.
   ```

4. `--auto --list-files` without files → `--auto` message takes priority (exit 1)

**`_needs_file_resolution` logic:**
- Skip stdin marker `"-"`
- Return true if any path `is_dir()` or contains glob chars `*?[`
- If returns false and not `--list-files`, pass files through unchanged

**`_resolve_files` logic:**
- Filter out `"-"` before passing to `FileResolver`
- Track whether stdin was present; re-insert at position 0 if so
- Create `FileResolverConfig` from CLI options
- Call `resolver.resolve(resolvable)` and convert results to strings

**Multi-file processing (`reformat_files` behavior):**
- Single stdin: pass through to `reformat_file`
- Multiple files with `--output` (not `-`): error
  `"Cannot specify output file when processing multiple files (use --inplace instead)"`
- Multiple files with `--inplace`: process each in-place
- Multiple files without `--inplace`: process each to stdout

**Main function flow (must match Python `main()`):**
1. Parse args, detect explicit flags, detect `is_auto`
2. Handle `--auto` expansion (set inplace, nobackup, semantic, cleanups, smartquotes,
   ellipses)
3. Early exits: `--version` → print and exit 0; `--install-skill` → install and exit 0;
   `--skill` → print SKILL.md and exit 0; `--docs` → print docs and exit 0
4. Validate: files required (with mode-specific error messages)
5. Load config: `find_config_file(cwd)` → `load_config()` → `merge_cli_with_config()`
6. Resolve files: `_resolve_files()` (conditionally invoke `FileResolver`)
7. Handle `--list-files`: print resolved paths and exit 0
8. Format files via loop (existing behavior, extended for multi-file)

**Rust implementation plan:**
- [ ] Change `files` default from `"-"` to empty (no default)
- [ ] Add all 11 missing flags to `Args` struct with clap derive attributes
- [ ] Implement explicit-flag tracking via `clap::ArgMatches::value_source()`
- [ ] Implement file resolution integration (`_needs_file_resolution` +
  `_resolve_files`)
- [ ] Implement config loading integration (`find_config_file` + `load_config` +
  `merge_cli_with_config`)
- [ ] Implement `--list-files` mode (resolve → print → exit)
- [ ] Port all 4 error messages with exact wording
- [ ] Port multi-file processing with `--output` validation
- [ ] Port early exit handlers (`--version`, `--skill`, `--install-skill`, `--docs`)
- [ ] Port 19 tests to `tests/test_cli_file_discovery.rs`
- [ ] Update `test-mapping.yaml`: change 19 entries from `excluded` → `mapped`

**Rust test mapping (19 tests from `test_cli_file_discovery.py`):**

File discovery via `--list-files` (7):
- `test_list_files_directory` — `--list-files .` lists `README.md`, `api.md`, `guide.md`
- `test_list_files_skips_excluded_dirs` — `node_modules/`, `.venv/` not in output
- `test_list_files_extend_include` — `--extend-include *.mdx` finds `.mdx` files
- `test_list_files_extend_exclude` — `--extend-exclude drafts/` excludes directory
- `test_list_files_no_respect_gitignore` — `--no-respect-gitignore` overrides
- `test_list_files_force_exclude` — `--force-exclude` filters explicit
  `node_modules/README.md`
- `test_list_files_max_size` — `--files-max-size 100` skips 2MB file

Error cases (4):
- `test_auto_no_args_errors` — `--auto` with no files → exit 1,
  `"--auto requires at least one file or directory argument"`
- `test_list_files_no_args_errors` — `--list-files` with no files → exit 1
- `test_no_args_errors` — bare `flowmark` → exit 1, `"No input specified"` +
  `"'-' for stdin"` + `"'.' for current directory"` + `"--help"`
- `test_auto_list_files_no_args_errors` — `--auto --list-files` → exit 1, auto message
  takes priority

Formatting integration (4):
- `test_auto_with_dot_formats_cwd` — `--auto .` formats files in cwd
- `test_explicit_file_still_works` — explicit file path → stdout
- `test_stdin_still_works` — `-` reads stdin → stdout
- `test_auto_with_explicit_file` — `--auto file.md` formats single file in-place

Tool ignore (1):
- `test_flowmarkignore` — `.flowmarkignore` respected in `--list-files` mode

Edge cases (3):
- `test_list_files_stdin_does_not_crash` — `--list-files - /dir` doesn’t crash
- `test_stdin_explicit_dash` — explicit `-` reads stdin
- `test_explicit_flag_detection_with_default_value` — `--width 88` (default value) still
  detected as explicit flag

### Phase 4: Skill System

**Bead:** fmr-qa6p | **Python source:** 158 lines (`skill.py`) | **Tests:** 9 |
**Estimated Rust LOC:** 150-200 | **Depends on:** Phase 3

Port `skill.py` and `skills/` — Claude Code skill installation.

**Python source:** `attic/flowmark/src/flowmark/skill.py` (158 lines)

**Python functions:**

`get_skill_content() -> str`:
- Loads `SKILL.md` from package data via `importlib.resources.files("flowmark")`
- Returns content as string
- Raises `ImportError` / `FileNotFoundError` if unavailable

`get_docs_content() -> str`:
- Finds `README.md` relative to `skill.py` (up 3 levels to repo root)
- Falls back to basic help text with link to GitHub if not found

`install_skill(agent_base: str | None = None)`:
- Default (`None`): install to `~/.claude/skills/flowmark/SKILL.md`
- Custom: install to `{agent_base}/skills/flowmark/SKILL.md`
- Creates directories with `mkdir(parents=True, exist_ok=True)`
- Writes SKILL.md content
- Prints success message with location
- If custom base: prints tip “Commit .claude/skills/ to share with team”
- On `PermissionError`: `"Permission denied: {e}"` to stderr, exit 1
- On `OSError`: `"Installation failed: {e}"` to stderr, exit 1

**SKILL.md content** (118 lines, in `attic/flowmark/src/flowmark/skills/SKILL.md`):
- YAML-style frontmatter: `name: flowmark`, `description:`, `allowed-tools:`
- Usage instructions with `uvx flowmark@latest`
- Key options table, common workflows, semantic line breaks explanation

**Rust implementation plan:**

Resources to embed at compile time:
- `SKILL.md` — copy from Python’s `skills/SKILL.md` into a Rust-accessible location
  (e.g., `src/skills/SKILL.md`), embed via `include_str!("skills/SKILL.md")`
- Documentation content — embed README.md or equivalent via `include_str!()`

Note: Python’s `SKILL.md` references `uvx flowmark@latest` (Python distribution).
The Rust binary will need its own SKILL.md that references the Rust binary installation
method (e.g., `cargo install flowmark`). This is an acceptable adaptation, not a parity
violation.

Module structure:
- [ ] Create `src/skill.rs` with `get_skill_content()`, `get_docs_content()`,
  `install_skill(agent_base: Option<&str>)`
- [ ] Create `src/skills/SKILL.md` — adapted from Python version (update install
  instructions for Rust binary)
- [ ] Embed SKILL.md via `include_str!()` in `skill.rs`
- [ ] Embed documentation content via `include_str!()` (either README.md or dedicated
  docs file)
- [ ] Register module in `src/lib.rs`: `pub mod skill;`
- [ ] Wire in CLI (`main.rs`): `--skill` → `print!(get_skill_content())`;
  `--install-skill` → `install_skill(args.agent_base)`; `--docs` →
  `print!(get_docs_content())`
- [ ] Handle errors: permission denied, OS errors
- [ ] Add `dirs` crate for `home_dir()` or use `std::env::var("HOME")` on Unix
- [ ] Port 9 tests to `tests/test_skill.rs`
- [ ] Update `test-mapping.yaml`: change 9 entries from `excluded` → `mapped`

**Rust test mapping (9 tests from `test_skill.py`):**

Skill content loading (3):
- `test_skill_content_loads` — `get_skill_content()` returns non-empty string
- `test_skill_content_has_metadata` — contains `name: flowmark`, `description:`,
  `allowed-tools:`
- `test_skill_content_has_usage` — contains `# Flowmark` and install command

Docs content loading (2):
- `test_docs_content_loads` — `get_docs_content()` returns non-empty string
- `test_docs_content_is_readme` — contains distinctive sections (`# flowmark`,
  `## Installation`, `## Semantic Line Breaks`)

Skill installation (4):
- `test_install_skill_default` — installs to `~/.claude/skills/flowmark/SKILL.md` (mock
  home dir)
- `test_install_skill_custom_base` — installs to
  `{custom_base}/skills/flowmark/SKILL.md`
- `test_install_skill_creates_directories` — creates nested
  `deep/nested/path/skills/...`
- `test_install_skill_overwrites_existing` — overwrites old SKILL.md content

### Phase 5: Tryscript CLI Golden Tests

**Bead:** fmr-t3va | **Depends on:** Phase 3, Phase 4

Establish tryscript-based end-to-end golden tests as the authoritative cross-language
CLI validation.

**Workflow:**

1. **Audit**: Enumerate every CLI feature/flag (done — see Phase 3 flag table)
2. **Baseline**: Write tryscript tests against the Python `flowmark` CLI to capture
   expected behavior as golden output
3. **Replicate**: Run the same tryscript tests against the Rust `flowmark` binary and
   verify identical output
4. **Map**: Add all tryscript test scenarios to the test mapping system
5. **Review**: Manually review every golden file for accuracy

**Prerequisites — review before implementing:**
- [ ] Read `tbd guidelines golden-testing-guidelines` for the full golden testing
  methodology, session modeling, and tryscript integration patterns
- [ ] Run `npx tryscript@latest readme` for tryscript overview
- [ ] Run `npx tryscript@latest docs` for tryscript syntax quick reference (patterns,
  elisions, config, YAML frontmatter)
- [ ] Run `npx tryscript@latest --help` for CLI options

**Tryscript setup:**
- [ ] Install tryscript: `npx tryscript@latest`
- [ ] Create `tests/tryscript/` directory structure
- [ ] Add tryscript CI job (runs after build, validates CLI golden output)
- [ ] Use `[..]` for variable output (paths, timestamps), `...` for multi-line elision
- [ ] Define `[PATTERN]` regex patterns in YAML frontmatter for platform-specific paths

**Concrete tryscript test matrix (24 scenarios):**

Each scenario below becomes a tryscript test file in `tests/tryscript/`. First run
against Python `flowmark` to capture golden baseline, then verify Rust binary produces
identical output.

| # | Scenario | Test File | What It Validates |
| --- | --- | --- | --- |
| 1 | `flowmark file.md` (single file → stdout) | `basic-file.try` | Basic file formatting |
| 2 | `echo "..." \| flowmark -` (stdin → stdout) | `basic-stdin.try` | Stdin processing |
| 3 | `flowmark --inplace file.md` (backup created) | `inplace-backup.try` | In-place with `.bak` backup |
| 4 | `flowmark --inplace --nobackup file.md` | `inplace-nobackup.try` | In-place without backup |
| 5 | `flowmark --auto .` (dir with mixed files) | `auto-directory.try` | Auto mode on directory |
| 6 | `flowmark --auto file.md` (single file) | `auto-single-file.try` | Auto mode on explicit file |
| 7 | `flowmark --width 60 file.md` | `width-custom.try` | Custom width |
| 8 | `flowmark --width 0 file.md` | `width-zero.try` | Width 0 (no wrapping) |
| 9 | `flowmark --plaintext file.txt` | `plaintext.try` | Plaintext mode |
| 10 | `flowmark --semantic file.md` | `semantic.try` | Semantic line breaks |
| 11 | `flowmark --smartquotes --ellipses file.md` | `typography.try` | Smart quotes + ellipses |
| 12 | `flowmark --list-spacing loose file.md` | `list-spacing.try` | List spacing modes |
| 13 | `flowmark --list-files .` | `list-files-dir.try` | File discovery + listing |
| 14 | `flowmark --list-files --extend-include "*.mdx" .` | `list-files-extend.try` | Extended include patterns |
| 15 | `flowmark --list-files --extend-exclude "drafts/" .` | `list-files-exclude.try` | Extended exclude patterns |
| 16 | `flowmark --list-files --no-respect-gitignore .` | `list-files-gitignore.try` | Gitignore override |
| 17 | `flowmark --list-files --force-exclude nm/README.md` | `list-files-force.try` | Force exclude on explicit files |
| 18 | `flowmark --list-files --files-max-size 100 .` | `list-files-maxsize.try` | Max file size filtering |
| 19 | `flowmark` (no args) | `error-no-args.try` | Error: no input specified |
| 20 | `flowmark --auto` (no files) | `error-auto-no-args.try` | Error: --auto requires files |
| 21 | `flowmark nonexistent.md` | `error-not-found.try` | Error: file not found |
| 22 | `flowmark --version` | `version.try` | Version output format |
| 23 | `flowmark --skill` | `skill-print.try` | Print SKILL.md content |
| 24 | `flowmark --docs` | `docs-print.try` | Print documentation |

**Test fixture directory (`tests/tryscript/fixtures/`):**
- [ ] `simple.md` — basic Markdown (heading + paragraphs)
- [ ] `with-frontmatter.md` — YAML frontmatter + content
- [ ] `with-code.md` — code blocks, inline code
- [ ] `large.md` — >1MB file (for max-size testing)
- [ ] `page.mdx` — MDX file (for extend-include testing)
- [ ] `nested/` directory structure:
  - `docs/guide.md`, `docs/api.md`
  - `node_modules/pkg/README.md` (should be excluded)
  - `.venv/lib/README.md` (should be excluded)
  - `drafts/wip.md` (for extend-exclude testing)
- [ ] `.gitignore` with `ignored/` pattern
- [ ] `ignored/found.md` (for gitignore testing)
- [ ] `.flowmarkignore` with `skip/` pattern
- [ ] `skip/nope.md` (for flowmarkignore testing)
- [ ] `.flowmark.toml` with `[formatting]` and `[file-discovery]` sections
- [ ] `pyproject.toml` with `[tool.flowmark]` section

**Implementation steps:**
- [ ] Create fixture directory and files
- [ ] Write all 24 tryscript test files against Python `flowmark`
- [ ] Run `npx tryscript@latest tests/tryscript/` to capture golden output
- [ ] Build Rust binary and run same tryscript tests against it
- [ ] Diff output — fix any discrepancies in Rust implementation
- [ ] Add tryscript CI job to `.github/workflows/`
- [ ] Iterate until all 24 scenarios pass for both Python and Rust

### Phase 6: Update Test Mapping and CI

**Bead:** fmr-v2de | **Depends on:** Phase 5

Update the test mapping system and CI gates to reflect the new scope.

- [ ] Update `test-mapping.yaml`: all 79 previously excluded entries change from
  `excluded` → `mapped`
- [ ] Update `check-mapping` expected counts: 281 mapped, 0 excluded, 0 missing, 0
  partial
- [ ] Update Rust test count assertion in CI (will increase as new tests are added)
- [ ] Add tryscript CI job to `.github/workflows/`
- [ ] Run `flowmark-dev discover-rust` to refresh `rust-tests.yaml`
- [ ] Run `flowmark-dev check-mapping` — verify exit code 0

### Phase 7: Upstream Contributions

**Bead:** fmr-03xy | **Priority:** P2 | **Depends on:** Phase 5

PR tryscript tests and any needed end-to-end tests to the Python flowmark repo
(`github.com/jlevy/flowmark`) to ensure parity.

- [ ] PR tryscript tests to the Python flowmark repo (if not already present)
- [ ] PR any missing CLI test coverage discovered during the audit
- [ ] Bump the Python source pin from `v0.6.4` to the version that includes the new
  tests (once merged)
- [ ] Update `flowmark-dev discover-python` to pick up new test functions

### Phase 8: Playbook Review & Sync

Systematically reconcile all documentation in the flowmark-rs project against the
rust-porting-playbook.
This is bidirectional: flowmark-rs docs inform playbook improvements, and playbook best
practices inform remaining flowmark-rs cleanup.

#### 8.1: Verify Playbook Fix Status

Before making new changes, verify what has already been done.

- [ ] Check if the 53+ fixes from `plan-2026-02-08-playbook-review-fixes.md` were
  actually applied to the playbook documents (beads closed ≠ changes committed)
- [ ] Check the status of the comprehensive review from
  `plan-2026-02-12-comprehensive-playbook-review.md`
- [ ] Identify which playbook spec changes are already implemented vs still pending
- [ ] Check `XXX:` → `HACK:`/`FIXME:` convention change status in playbook docs
- [ ] Check for non-compiling code examples identified in the review
- [ ] Grep playbook for archived crate references (`serde_yaml`, `once_cell`,
  `actions/checkout@v5`, `actions/create-release@v1`)
- [ ] Create a status matrix: {fix-id, target-file, applied-or-not}

#### 8.2: Review flowmark-rs Docs Against Current State

Ensure this project’s own docs are accurate.

- [x] **`porting-plan.md`**: Updated with “Status: Complete” header, checked acceptance
  criteria, verified module mapping against actual `src/` layout, moved to
  `docs/project/specs/done/porting-plan.md`.
- [ ] **`porting-checklist.md`**: Determine if this should be removed (it’s a duplicate
  of the playbook). If kept, verify it matches the current playbook version.
  Decision: remove or convert to a project-specific checklist with checked items.
- [ ] **`code-review-2026-02-17.md`**: Cross-reference all findings against playbook
  best practices. For each finding, note whether the playbook covers it, and if not, flag
  as a playbook gap (ADD).
- [ ] **Exact parity spec**: Verify “Complete” status is accurate.
  Check all appendices for correctness.
- [ ] **Test mapping spec**: Verify “Implemented” status.
  Check that workflow descriptions match actual `flowmark-dev` CLI behavior.
- [ ] Check for any `XXX:` comments in flowmark-rs source code that should be
  `HACK:`/`FIXME:` per updated playbook convention
- [ ] Verify `HACK:` and `FIXME:` comments exist where playbook says they should (all
  library workarounds documented)

#### 8.3: Review Playbook Case Study Against Current Port

Update all 7 case study documents to reflect the current port.

- [ ] **`flowmark-port-analysis.md`**: Update metrics (LOC, test counts, ratios).
  Update “what’s automatable” assessment based on this port’s experience.
  Note that the new port used a cross-language test mapping system not in the original.

- [ ] **`flowmark-port-library-choices.md`**: Verify library choices match current
  `Cargo.toml`. Update comrak version references.
  Note any new library decisions.

- [ ] **`flowmark-port-decision-log.md`**: Update or add entries for decisions made in
  the new port (e.g., test mapping infrastructure, CI hardening, lint configuration).
  Fix the D7 wrapping solution contradiction identified in plan-2026-02-08.

- [ ] **`flowmark-port-migration-plan.md`**: This is the longest doc (3,339 lines).
  Decide: update in place, add a “v2 port” appendix, or create a separate doc for the
  new port’s migration narrative.

- [ ] **`flowmark-port-cross-validation.md`**: Update with current cross-validation
  results (100% mapped tests passing, 0 ignored).
  Update escape handling table.

- [ ] **`flowmark-port-comrak-bug.md`**: Verify still relevant.
  Check if any comrak bugs were fixed upstream since the original doc.

- [ ] **`flowmark-port-wrapping-solution.md`**: Update with current wrapping approach.
  The doc describes two approaches — verify which one the current port uses and update.

- [ ] **Reconcile workaround counts** across all 7 case study docs.
  Establish a single authoritative count for the current port by grepping `HACK:`
  comments in source.

#### 8.4: Review Playbook Reference Docs

Review each reference doc against this port’s experience.

- [ ] **`python-to-rust-playbook.md`** (core playbook):
  - [ ] Verify effort allocation table sums to 100% (identified as 105% in review)
  - [ ] Update “Key insight” with data from both ports
  - [ ] Check Phase 4.3 (submodule setup) — the current port uses `attic/` clones
    instead of submodules; document both approaches
  - [ ] Check Phase 4.6 (version tracking) — verify recommendation matches practice
  - [ ] Validate Phase 7 (finalize) CLI parity section against actual CLI state
  - [ ] Check Phase 8 (sync) — not yet exercised; note this

- [ ] **`python-to-rust-mapping-reference.md`**:
  - [ ] Verify type mappings against actual code translations
  - [ ] Check `dict` → `HashMap` insertion-order warning is present
  - [ ] Check `assert` → `debug_assert!` dangerous mapping is fixed
  - [ ] Verify version constraint mappings (identified as swapped)
  - [ ] Check for `Cow<'_, str>` in type mappings
  - [ ] Check for `re.search()` and `re.fullmatch()` regex mappings
  - [ ] Check `str.find()` byte-offset warning
  - [ ] Verify dunder methods → traits table exists

- [ ] **`python-to-rust-porting-guide.md`**:
  - [ ] Verify `build.rs` code examples compile
  - [ ] Check version tracking recommendations against actual practice
  - [ ] Validate cross-validation script template
  - [ ] Check 9 critical pitfalls against this port’s experience

- [ ] **`rust-cli-best-practices.md`**:
  - [ ] Verify recommended crate versions are current
  - [ ] Check `color-eyre` maintenance status note
  - [ ] Verify CI workflow uses current GitHub Actions versions
  - [ ] Check `cargo-dist` mentioned alongside `cargo-release`
  - [ ] Validate lint configuration against code review recommendations

- [ ] **`rust-code-review-checklist.md`**:
  - [ ] Run the checklist against the code-review-2026-02-17.md findings
  - [ ] Identify any findings the checklist would NOT have caught
  - [ ] Flag checklist gaps as ADD items

- [ ] **`python-to-rust-test-coverage-playbook.md`**:
  - [ ] Compare recommended test strategy against actual test mapping approach
  - [ ] Note that flowmark-rs developed a cross-language test mapping system beyond what
    the playbook describes — flag as ADD

- [ ] **Checklist templates** (`port-checklist-initial-template.md`,
  `port-checklist-update-template.md`):
  - [ ] Walk through each checklist item against the flowmark-rs port
  - [ ] Mark items that were done, skipped, or done differently
  - [ ] Flag missing checklist items discovered during this port

#### 8.5: Review Playbook Guidelines

Review each guideline against this port’s experience.

- [ ] **`python-to-rust-porting-rules.md`**:
  - [ ] Check Pitfall #6 (identical wrong/correct examples — identified in review)
  - [ ] Verify `assert` → `debug_assert!` fix applied
  - [ ] Check `frozenset` note accuracy
  - [ ] Check acceptance criteria include clippy
  - [ ] Verify comment convention uses `HACK:`/`FIXME:` (not `XXX:`)

- [ ] **`python-to-rust-cli-porting.md`**:
  - [ ] Validate argparse → clap mappings against actual CLI
  - [ ] Check SIGPIPE handling recommendation matches implementation
  - [ ] Verify exit code guidance

- [ ] **`rust-general-rules.md`**:
  - [ ] Check Edition 2024 guidance completeness
  - [ ] Verify `LazyLock` recommendation (not `once_cell`)
  - [ ] Check ownership patterns against actual code

- [ ] **`rust-cli-app-patterns.md`**:
  - [ ] Validate project structure recommendation against actual structure
  - [ ] Check error handling pattern (main() contradiction identified in review)
  - [ ] Verify `ExitCode` recommendation matches implementation

- [ ] **`rust-project-setup.md`**:
  - [ ] Validate Cargo.toml recommendations against actual Cargo.toml
  - [ ] Check lint configuration (warn vs deny)
  - [ ] Verify CI workflow recommendations against actual `.github/workflows/`
  - [ ] Check `deny.toml` recommendations against actual `deny.toml`
  - [ ] Verify release profile against actual profile
  - [ ] Check `resolver = "3"` for Edition 2024

- [ ] **`test-coverage-for-porting.md`**:
  - [ ] Compare coverage targets against actual coverage
  - [ ] Check `insta` snapshot testing mention
  - [ ] Validate cross-validation CI example

#### 8.6: Integrate Phase 7C Observations

Complete the pending meta-playbook Phase C work.

- [ ] Read `flowmark-port-observations-2.md` (13 observations from exact-parity spec)
- [ ] Triage each observation using the improvement-triage-template categories
  (FIX/ADD/CLARIFY/GENERALIZE/VALIDATE)
- [ ] Draft specific text changes for each non-VALIDATE observation
- [ ] Prioritize by impact and severity
- [ ] Create implementation list organized by target file

#### 8.7: Update Playbook README and Cross-References

- [ ] Update README.md case study metrics table with current port data
- [ ] Update the “Case studies completed” table
- [ ] Verify all cross-references between docs are correct
- [ ] Check all internal links resolve
- [ ] Update “validated by N case studies” if applicable

#### 8.8: Consolidate Findings into Action Items

- [ ] Compile all FIX/ADD/CLARIFY/GENERALIZE items from Phases 8.1-8.7
- [ ] Organize by target file for efficient editing
- [ ] Prioritize: factual errors first, then missing content, then clarity improvements
- [ ] Create beads for each actionable change
- [ ] Determine which changes go to the playbook repo vs this repo

### Phase 9: Final Acceptance

**Bead:** fmr-h01s | **Depends on:** Phase 6

- [ ] **Every** Python CLI flag has a Rust equivalent with identical behavior — no
  exceptions
- [ ] `flowmark --auto .` works identically in both Python and Rust (same files
  discovered, same output produced)
- [ ] `flowmark --list-files .` produces identical sorted file lists
- [ ] Config loading from `.flowmark.toml` and `pyproject.toml [tool.flowmark]` works
- [ ] `.flowmarkignore` patterns are respected
- [ ] Gitignore integration works (and `--no-respect-gitignore` disables it)
- [ ] Skill system works: `--skill`, `--install-skill`, `--agent-base`, `--docs`
- [ ] All 79 previously-excluded tests are ported and passing
- [ ] Tryscript golden tests pass in CI for both Python and Rust
- [ ] `check-mapping` passes: **281 mapped, 0 excluded, 0 missing, 0 partial**
- [ ] Every mapping entry manually reviewed for accuracy
- [ ] Tryscript tests contributed upstream to the Python repo
- [ ] All existing 481+ tests continue to pass (no regressions)

## Testing Strategy

- **Part A**: Each phase adds tests ported from the corresponding Python test file.
  Total: 79 new tests.
  CI enforces via `check-mapping` (281 mapped, 0 excluded).
  Tryscript golden tests provide end-to-end CLI validation.
- **Part B**: Each phase produces a deliverable document or set of changes.
  Validation by grep/search of playbook files, verification against current
  `cargo test`, `cargo clippy`, CI status.

## Decisions Made

1. **`porting-checklist.md`: Remove.** It’s a stale duplicate of the playbook’s
   `python-to-rust-playbook.md`. No backward compatibility needed for docs.

2. **Case study versioning: Add “v2 port” sections.** Keep old port data and add v2
   sections to each case study doc.

3. **Test mapping system: New reference doc.** Create a new reference doc in the
   playbook (e.g., `reference/cross-language-test-mapping.md`) and link from the
   playbook’s Phase 5, the test coverage playbook, and the README.

4. **`porting-plan.md`: Updated and moved to `specs/done/`.** Updated with accurate
   module layout, checked acceptance criteria, current metrics, and “Status: Complete”
   header.

5. **Migration plan: Renamed v1, created v2.** Renamed the existing 3,339-line migration
   plan to `flowmark-port-migration-plan-v1.md` with a note pointing to v2. Created new
   `flowmark-port-migration-plan-v2.md` documenting the current port’s architecture.

## Open Questions

None remaining. All decisions resolved.

## Future Work (tracked separately)

| Item | Priority | Bead | Notes |
| --- | --- | --- | --- |
| **Performance optimization + benchmarks** | P1 | fmr-aq8o | File resolver `--list-files` 4x slower than Python due to excessive syscalls; fix with `ignore::WalkBuilder`, then benchmark |
| **Property-based testing** (proptest) | P3 | — | Idempotency, width invariants, round-trip properties |
| **justfile** for common dev workflows | P3 | — | `just test`, `just lint`, `just check-mapping` |
| **Release workflow** (GitHub Actions) | P3 | — | Automated binary builds + crates.io publish (see build-publishing spec) |
| **README and CHANGELOG** | P3 | — | Public-facing documentation (see build-publishing spec) |
| **`clap_complete` shell completions** | P4 | — | Generate bash/zsh/fish completions |
| **Color flag** (`--color auto/always/never`) | P4 | — | Standard CLI convention |

## References

- Exact parity spec: `docs/project/specs/active/plan-2026-02-17-exact-parity.md`
- Test mapping spec:
  `docs/project/specs/active/plan-2026-02-17-test-mapping-meta-test.md`
- Code review: `docs/project/specs/active/code-review-2026-02-17.md`
- Porting plan: `docs/project/specs/done/porting-plan.md`
- Porting log: `docs/porting-log-review.md`
- Port sync playbook: `docs/port-sync-playbook.md`
- Playbook repo: `attic/rust-porting-playbook/`
- Playbook README: `attic/rust-porting-playbook/README.md`
- Meta-playbook: `attic/rust-porting-playbook/reference/meta-improving-this-playbook.md`
- Playbook review fixes spec:
  `attic/rust-porting-playbook/docs/project/specs/active/plan-2026-02-08-playbook-review-fixes.md`
- Comprehensive review spec:
  `attic/rust-porting-playbook/docs/project/specs/active/plan-2026-02-12-comprehensive-playbook-review.md`
- Phase 7C observations:
  `attic/rust-porting-playbook/case-studies/flowmark/flowmark-port-observations-2.md`
- Comprehensive tryscript spec:
  `docs/project/specs/active/plan-2026-02-17-comprehensive-tryscript-golden-tests.md`
- **Python CLI source**: `attic/flowmark/src/flowmark/cli.py`
- **Python config source**: `attic/flowmark/src/flowmark/config.py`
- **Python file resolver**: `attic/flowmark/src/flowmark/file_resolver/`
- **Python test files (in scope)**: `attic/flowmark/tests/test_cli_file_discovery.py`,
  `attic/flowmark/tests/test_config.py`, `attic/flowmark/tests/test_file_resolver.py`
- **Golden testing methodology**: `tbd guidelines golden-testing-guidelines`
- **Tryscript documentation**: `npx tryscript@latest readme` (overview),
  `npx tryscript@latest docs` (syntax reference)
- **Tryscript repo**: https://github.com/jlevy/tryscript
