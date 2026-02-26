# Feature: Comprehensive Tryscript Golden Test Suite

**Date:** 2026-02-17 (last updated 2026-02-17)

**Author:** Joshua Levy with LLM assistance

**Status:** Draft

## Overview

Create a comprehensive, **binary-agnostic** tryscript golden test suite that
systematically exercises every CLI flag, every formatting feature, and every file
discovery behavior of flowmark through a reusable set of fixture files and directories.

**The same tryscript files must produce identical results against both the Rust
`flowmark` binary and the Python `flowmark` binary.** The tests are developed and
validated first in the Rust repo (`flowmark-rs`), then upstreamed to the Python repo
(`flowmark`) where they serve as the authoritative cross-language parity contract.
Any test that passes for one binary but fails for the other is a parity bug that must be
fixed.

The current tryscript tests (`tests/tryscript/cli-golden.tryscript.md`) have 31
scenarios, but they are relatively shallow — most use one-liner `printf` inputs that
test only basic behavior.
This spec defines a fixture-first, matrix-driven approach where a rich set of Markdown
fixtures is combined with a systematic flag coverage table to ensure every CLI
capability is tested end-to-end.

## Goals

- **Systematic flag coverage**: Every CLI flag tested at least once, with a coverage
  matrix proving completeness.
- **Fixture-based testing**: Reusable fixture files and directories that exercise
  diverse Markdown constructs, replacing ad hoc `printf` commands.
- **Combination testing**: Key flag combinations tested together (e.g.,
  `--semantic --smartquotes --width 60`), not just flags in isolation.
- **Binary-agnostic design**: Every tryscript file is written to be completely
  independent of the implementation language.
  The same files, fixtures, and expected output work identically against both the Rust
  and Python `flowmark` binaries.
  The binary under test is selected solely via `PATH` — no test file references Rust,
  Python, cargo, or pip.
- **Cross-repo portability**: The test suite is developed in `flowmark-rs`, then
  upstreamed to the Python `flowmark` repo as a shared parity contract.
  Both repos run the same tests in CI.
- **File discovery coverage**: Full fixture directory tree exercising gitignore,
  `.flowmarkignore`, excluded dirs, glob patterns, max size, and config file loading.
- **Error path coverage**: Every documented error message triggered and validated.
- **Config interaction coverage**: TOML config files exercising the three-way merge
  (defaults, config, CLI), including `--auto` mode locking.
- **Idempotency validation**: Running flowmark twice on already-formatted output
  produces identical output.

## Non-Goals

- Performance benchmarking (covered separately by fmr-aq8o).
- Testing internal library APIs — this spec covers CLI-level end-to-end behavior only.
- Replacing the existing unit/integration test suite — tryscript golden tests complement
  (not replace) the 339 Rust tests.
- Testing implementation-specific internals (Rust module structure, Python import paths,
  etc.) — every test must be meaningful for both binaries.

## Background

### Current Tryscript Test Coverage

The existing `tests/tryscript/cli-golden.tryscript.md` has 31 tests:

| Category | Tests | What’s Covered |
| --- | --- | --- |
| Version/help | 1 | `--version` |
| Error handling | 5 | No args, `--auto` no args, `--list-files` no args, nonexistent file |
| File discovery | 8 | `--list-files`, `--extend-include`, `--extend-exclude`, `--no-respect-gitignore`, `--force-exclude`, `--files-max-size`, `.flowmarkignore` |
| Skills/docs | 2 | `--skill`, `--docs` |
| Stdin formatting | 6 | Default, `--semantic`, `--width 30`, `--width 0`, explicit `-`, `--semantic -` |
| Plaintext | 1 | `--plaintext` |
| Typography | 1 | `--smartquotes --ellipses` |
| List spacing | 1 | `--list-spacing loose` |
| File operations | 4 | File to stdout, `--inplace`, `--inplace --nobackup`, `--auto` single file |
| Auto mode | 1 | `--auto` on directory |
| Config | 1 | Width from TOML |

### Key Gaps in Current Tests

1. **No diverse fixture content**: All tests use trivial 1-3 line inputs created inline
   with `printf`. No testing of complex Markdown structures (nested lists, blockquotes,
   code blocks, alerts, frontmatter, tables, footnotes, math).
2. **No combination testing**: Flags tested in isolation.
   `--semantic --width 60 --cleanups` never tested together.
3. **No `--cleanups` test**: The `--cleanups` flag has zero tryscript coverage.
4. **No `--list-spacing tight` test**: Only `loose` tested.
5. **No `--list-spacing preserve` test**: Default not explicitly validated.
6. **No multi-file processing tests**: Never tests formatting multiple files at once.
7. **No `--output` flag tests**: The `-o` flag has zero tryscript coverage.
8. **No `--verbose` tests**: Verbose output never validated.
9. **No config file interaction tests**: Only width-from-TOML tested.
   Missing: `pyproject.toml`, `.flowmark.toml` precedence, `[file-discovery]` section,
   kebab-case keys, auto-mode locking, explicit CLI override of config.
10. **No `--install-skill` test**: Skill installation never tested end-to-end.
11. **No idempotency test**: Running flowmark on already-formatted output never
    validated.
12. **No `--exclude` test**: The `--exclude` flag (which replaces defaults) has zero
    tryscript coverage.
13. **No error test for `--output` with multiple files**: Expected error message not
    validated.

### Appendix D Bugs Not Covered by Any Test

From the exact parity spec’s Appendix D code review, several bugs have no test coverage
at all.
The comprehensive tryscript suite should include tests that would detect these if
regressions occur:

| Bug | Description | Tryscript Testable? |
| --- | --- | --- |
| C2 | Ellipsis+smartquotes interaction (`...` not converted near curly quotes) | Yes — `--auto` mode |
| C3 | Inplace mode loses file permissions (755 → 600) | Yes — `--auto` with chmod |
| H1 | `usize` underflow with very small width | Yes — `--width 2` |
| M4 | CRLF line endings not preserved | Partially — can test output correctness |

## Design

### Approach: Fixture-First, Matrix-Driven Testing

Rather than creating individual tryscript tests that each construct their own input, we
define:

1. **A fixture directory tree** (`tests/tryscript/fixtures/`) with carefully designed
   Markdown files covering all formatting features.
2. **A flag coverage matrix** mapping every CLI flag to the fixture(s) and tryscript
   test(s) that exercise it.
3. **Tryscript test files** organized by feature area, each using the shared fixtures.

### Fixture Directory Design

```
tests/tryscript/fixtures/
├── content/                          # Markdown content fixtures
│   ├── simple.md                     # Basic heading + paragraphs
│   ├── paragraphs-long.md            # Long paragraphs that need wrapping
│   ├── semantic-sentences.md         # Multi-sentence paragraphs for semantic mode
│   ├── lists-tight.md                # Tight lists (no blank lines between items)
│   ├── lists-loose.md                # Loose lists (blank lines between items)
│   ├── lists-nested.md               # Deeply nested lists with mixed markers
│   ├── lists-ordered.md              # Ordered lists
│   ├── blockquotes.md                # Blockquotes with nested content
│   ├── code-blocks.md                # Fenced code blocks (backtick and tilde)
│   ├── code-inline.md                # Inline code and code spans
│   ├── frontmatter.md                # YAML frontmatter
│   ├── headings.md                   # All heading levels, bold headings (for cleanups)
│   ├── typography.md                 # Straight quotes, apostrophes, ellipses
│   ├── escapes.md                    # Backslash escapes (\*, \#, \-, \[, etc.)
│   ├── links-emphasis.md             # Links, images, bold, italic, strikethrough
│   ├── alerts.md                     # GitHub-flavored alert blocks
│   ├── html-blocks.md                # HTML blocks and inline HTML
│   ├── tables.md                     # Markdown tables
│   ├── footnotes.md                  # Footnote references and definitions
│   ├── math.md                       # LaTeX math (inline $...$ and display $$...$$)
│   ├── comprehensive.md              # Kitchen sink: all features in one doc
│   └── plaintext.txt                 # Plain text (not Markdown)
│
├── project/                          # File discovery fixture tree
│   ├── README.md                     # Root markdown file
│   ├── .flowmark.toml                # Config file (highest priority)
│   ├── docs/
│   │   ├── guide.md                  # Guide document
│   │   ├── api.md                    # API document
│   │   └── tutorial.md               # Tutorial
│   ├── src/
│   │   └── README.md                 # Source readme
│   ├── pages/
│   │   ├── index.mdx                 # MDX file (for extend-include testing)
│   │   └── about.mdx                 # Another MDX file
│   ├── drafts/
│   │   └── wip.md                    # Draft (for extend-exclude testing)
│   ├── node_modules/
│   │   └── pkg/
│   │       └── README.md             # Should be excluded by default
│   ├── .venv/
│   │   └── lib/
│   │       └── README.md             # Should be excluded by default
│   ├── build/
│   │   └── output.md                 # Should be excluded by default
│   ├── vendor/
│   │   └── lib.md                    # Should be excluded by default
│   ├── .git/
│   │   └── config                    # Should be excluded by default
│   ├── nested/
│   │   ├── .gitignore                # Nested gitignore: "generated/"
│   │   ├── deep/
│   │   │   └── file.md               # Deep nested file
│   │   └── generated/
│   │       └── output.md             # Excluded by nested gitignore
│   ├── skip/
│   │   └── ignored.md                # Excluded by .flowmarkignore
│   ├── .gitignore                    # Root gitignore: "skip/"
│   └── .flowmarkignore               # Tool ignore: "skip/"
│
├── config-tests/                     # Config file interaction fixtures
│   ├── dot-flowmark/                 # .flowmark.toml takes precedence
│   │   ├── .flowmark.toml            # width = 50
│   │   ├── flowmark.toml             # width = 60 (should be ignored)
│   │   └── test.md
│   ├── flowmark-toml/                # flowmark.toml used
│   │   ├── flowmark.toml             # width = 60, semantic = true
│   │   └── test.md
│   ├── pyproject/                    # pyproject.toml [tool.flowmark]
│   │   ├── pyproject.toml            # [tool.flowmark] width = 70
│   │   └── test.md
│   ├── pyproject-no-section/         # pyproject.toml without [tool.flowmark]
│   │   ├── pyproject.toml            # No flowmark section
│   │   └── test.md
│   ├── kebab-case/                   # Kebab-case config keys
│   │   ├── flowmark.toml             # list-spacing = "loose", extend-include = ["*.mdx"]
│   │   └── test.md
│   ├── sections/                     # Nested config sections
│   │   ├── flowmark.toml             # [formatting] and [file-discovery] sections
│   │   ├── test.md
│   │   └── page.mdx
│   ├── auto-lock/                    # Auto mode overrides config formatting
│   │   ├── flowmark.toml             # semantic = false (should be overridden by --auto)
│   │   └── test.md
│   └── cli-overrides-config/         # CLI flags take precedence over config
│       ├── flowmark.toml             # width = 50
│       └── test.md
│
├── large-file/                       # Max size testing
│   └── create-large.sh              # Script to create >1MB file on demand
│
└── multi-file/                       # Multi-file processing
    ├── a.md
    ├── b.md
    └── c.md
```

### Complete CLI Flag Coverage Matrix

Every CLI flag mapped to the tryscript test file(s) that exercise it:

| # | Flag | Type | Default | Tryscript Test File(s) | Fixture(s) Used |
| --- | --- | --- | --- | --- | --- |
| **Formatting Options** |  |  |  |  |  |
| 1 | `--width <N>` | usize | 88 | `formatting.tryscript.md` | `paragraphs-long.md` |
| 2 | `--width 0` | usize | — | `formatting.tryscript.md` | `paragraphs-long.md` |
| 3 | `--plaintext` | flag | false | `formatting.tryscript.md` | `plaintext.txt` |
| 4 | `--semantic` | flag | false | `formatting.tryscript.md` | `semantic-sentences.md` |
| 5 | `--cleanups` | flag | false | `formatting.tryscript.md` | `headings.md` |
| 6 | `--smartquotes` | flag | false | `typography-tests.tryscript.md` | `typography.md` |
| 7 | `--ellipses` | flag | false | `typography-tests.tryscript.md` | `typography.md` |
| 8 | `--list-spacing preserve` | enum | preserve | `list-spacing.tryscript.md` | `lists-tight.md`, `lists-loose.md` |
| 9 | `--list-spacing loose` | enum | — | `list-spacing.tryscript.md` | `lists-tight.md` |
| 10 | `--list-spacing tight` | enum | — | `list-spacing.tryscript.md` | `lists-loose.md` |
| 11 | `--auto` | preset | — | `auto-mode.tryscript.md` | `comprehensive.md`, `project/` |
| **File Processing** |  |  |  |  |  |
| 12 | `--inplace` | flag | false | `file-ops.tryscript.md` | `simple.md` |
| 13 | `--nobackup` | flag | false | `file-ops.tryscript.md` | `simple.md` |
| 14 | `--output <PATH>` | path | stdout | `file-ops.tryscript.md` | `simple.md` |
| 15 | `-` (stdin) | positional | — | `stdin.tryscript.md` | (inline) |
| **File Discovery** |  |  |  |  |  |
| 16 | `--extend-include <PAT>` | vec | [] | `file-discovery.tryscript.md` | `project/` |
| 17 | `--exclude <PAT>` | vec | None | `file-discovery.tryscript.md` | `project/` |
| 18 | `--extend-exclude <PAT>` | vec | [] | `file-discovery.tryscript.md` | `project/` |
| 19 | `--no-respect-gitignore` | flag | false | `file-discovery.tryscript.md` | `project/` |
| 20 | `--force-exclude` | flag | false | `file-discovery.tryscript.md` | `project/` |
| 21 | `--list-files` | flag | false | `file-discovery.tryscript.md` | `project/` |
| 22 | `--files-max-size <N>` | usize | 1048576 | `file-discovery.tryscript.md` | `project/` + large file |
| **Verbose & Docs** |  |  |  |  |  |
| 23 | `--verbose` | flag | false | `verbose-docs.tryscript.md` | `project/` |
| 24 | `--skill` | flag | false | `verbose-docs.tryscript.md` | (none) |
| 25 | `--install-skill` | flag | false | `verbose-docs.tryscript.md` | (none) |
| 26 | `--agent-base <DIR>` | path | None | `verbose-docs.tryscript.md` | (none) |
| 27 | `--docs` | flag | false | `verbose-docs.tryscript.md` | (none) |
| 28 | `--version` | flag | — | `errors-version.tryscript.md` | (none) |
| **Error Cases** |  |  |  |  |  |
| 29 | (no args) | — | — | `errors-version.tryscript.md` | (none) |
| 30 | `--auto` (no files) | — | — | `errors-version.tryscript.md` | (none) |
| 31 | `--list-files` (no files) | — | — | `errors-version.tryscript.md` | (none) |
| 32 | nonexistent file | — | — | `errors-version.tryscript.md` | (none) |
| 33 | `--output` + multiple files | — | — | `errors-version.tryscript.md` | `multi-file/` |

### Tryscript Test File Organization

Tests organized into **10 separate tryscript files** by feature area.
This multi-file organization has several benefits:
- **Easier to write and review**: Each file focuses on one feature area, making it
  easier to understand and modify.
- **Parallel development**: Different files can be worked on independently.
- **Faster iteration**: Run a single file during development instead of the full suite
  (`npx tryscript run tests/tryscript/formatting.tryscript.md`).
- **Better failure isolation**: A failing test in one area doesn’t block progress in
  others.
- **Clearer organization**: The file name immediately signals what’s being tested.

The 10-file split keeps each file manageable (4-14 scenarios each), while the total
suite is comprehensive:

| # | Test File | Scenarios | Description |
| --- | --- | --- | --- |
| 1 | `formatting.tryscript.md` | ~12 | Width, plaintext, semantic, cleanups, combinations |
| 2 | `typography-tests.tryscript.md` | ~8 | Smart quotes, ellipses, combined, edge cases |
| 3 | `list-spacing.tryscript.md` | ~6 | Preserve, loose, tight on various list types |
| 4 | `auto-mode.tryscript.md` | ~6 | Auto mode on files and directories, idempotency |
| 5 | `file-ops.tryscript.md` | ~8 | Inplace, backup, nobackup, output, multi-file |
| 6 | `stdin.tryscript.md` | ~4 | Stdin with various flags |
| 7 | `file-discovery.tryscript.md` | ~14 | List-files, extend-include/exclude, gitignore, flowmarkignore, max-size, force-exclude, glob |
| 8 | `config-interaction.tryscript.md` | ~10 | TOML loading, precedence, pyproject, auto-lock, CLI override |
| 9 | `verbose-docs.tryscript.md` | ~6 | Verbose output, skill, install-skill, docs |
| 10 | `errors-version.tryscript.md` | ~8 | All error cases, version |

**Total: ~82 scenarios** across 10 files (vs 31 in current single-file suite).

**File naming convention:** All files use `.tryscript.md` extension and live in
`tests/tryscript/`. Feature-area prefix makes alphabetical listing logical.

### Detailed Test Scenarios

#### 1. `formatting.tryscript.md` — Core Formatting

Uses `content/` fixtures to test formatting options in isolation and combination.

| # | Scenario | Command | Fixture | Validates |
| --- | --- | --- | --- | --- |
| F1 | Default width (88) | `flowmark fixtures/content/paragraphs-long.md` | `paragraphs-long.md` | Lines wrap at width 88 |
| F2 | Custom width (60) | `flowmark --width 60 fixtures/content/paragraphs-long.md` | `paragraphs-long.md` | Lines wrap at width 60 |
| F3 | Custom width (30) | `flowmark --width 30 fixtures/content/paragraphs-long.md` | `paragraphs-long.md` | Very narrow wrapping |
| F4 | Width zero (no wrap) | `flowmark --width 0 fixtures/content/paragraphs-long.md` | `paragraphs-long.md` | No line wrapping at all |
| F5 | Plaintext mode | `flowmark --plaintext fixtures/content/plaintext.txt` | `plaintext.txt` | No Markdown parsing |
| F6 | Semantic line breaks | `flowmark --semantic fixtures/content/semantic-sentences.md` | `semantic-sentences.md` | Sentence-based breaks |
| F7 | Cleanups (unbold headings) | `flowmark --cleanups fixtures/content/headings.md` | `headings.md` | Bold removed from headings |
| F8 | Semantic + width 60 | `flowmark --semantic --width 60 fixtures/content/semantic-sentences.md` | `semantic-sentences.md` | Combination behavior |
| F9 | Semantic + cleanups | `flowmark --semantic --cleanups fixtures/content/headings.md` | `headings.md` | Combination behavior |
| F10 | Comprehensive default | `flowmark fixtures/content/comprehensive.md` | `comprehensive.md` | All Markdown structures preserved |
| F11 | Width 2 (edge case) | `flowmark --width 2 fixtures/content/simple.md` | `simple.md` | Very small width doesn’t crash |
| F12 | Idempotency | Run twice, diff output | `comprehensive.md` | Second run is identity |

#### 2. `typography-tests.tryscript.md` — Typography

| # | Scenario | Command | Fixture | Validates |
| --- | --- | --- | --- | --- |
| T1 | Smart quotes only | `flowmark --smartquotes fixtures/content/typography.md` | `typography.md` | Straight → curly quotes |
| T2 | Ellipses only | `flowmark --ellipses fixtures/content/typography.md` | `typography.md` | `...` → `…` |
| T3 | Smart quotes + ellipses | `flowmark --smartquotes --ellipses fixtures/content/typography.md` | `typography.md` | Both transformations |
| T4 | Smart quotes in code blocks | `flowmark --smartquotes fixtures/content/code-blocks.md` | `code-blocks.md` | Quotes NOT converted in code |
| T5 | Ellipses in code blocks | `flowmark --ellipses fixtures/content/code-blocks.md` | `code-blocks.md` | Ellipses NOT converted in code |
| T6 | Smart quotes + escapes | `flowmark --smartquotes fixtures/content/escapes.md` | `escapes.md` | Backslash-escaped quotes preserved |
| T7 | Apostrophes and contractions | `flowmark --smartquotes fixtures/content/typography.md` | `typography.md` | it’s, don’t, '90s, etc. |
| T8 | Typography edge cases (inline) | (inline stdin) | — | Specific edge cases from Appendix D |

#### 3. `list-spacing.tryscript.md` — List Spacing Modes

| # | Scenario | Command | Fixture | Validates |
| --- | --- | --- | --- | --- |
| LS1 | Preserve tight list | `flowmark --list-spacing preserve fixtures/content/lists-tight.md` | `lists-tight.md` | Tight list stays tight |
| LS2 | Preserve loose list | `flowmark --list-spacing preserve fixtures/content/lists-loose.md` | `lists-loose.md` | Loose list stays loose |
| LS3 | Tight → loose | `flowmark --list-spacing loose fixtures/content/lists-tight.md` | `lists-tight.md` | Tight list made loose |
| LS4 | Loose → tight | `flowmark --list-spacing tight fixtures/content/lists-loose.md` | `lists-loose.md` | Loose list made tight |
| LS5 | Nested lists | `flowmark --list-spacing loose fixtures/content/lists-nested.md` | `lists-nested.md` | Nested list spacing |
| LS6 | Ordered lists | `flowmark --list-spacing tight fixtures/content/lists-ordered.md` | `lists-ordered.md` | Ordered list spacing |

#### 4. `auto-mode.tryscript.md` — Auto Mode

| # | Scenario | Command | Validates |
| --- | --- | --- | --- |
| A1 | Auto on single file | `flowmark --auto file.md && cat file.md` | In-place with all enhancements |
| A2 | Auto on directory | `flowmark --auto dir/ && cat dir/*.md` | All `.md` files formatted |
| A3 | Auto is idempotent | Run `--auto` twice, diff | Second run produces identical output |
| A4 | Auto with config width | Config `width = 60` + `--auto` | Config width respected |
| A5 | Auto skips non-md | `--auto dir/` with `.py` and `.txt` | Only `.md` files touched |
| A6 | Auto with verbose | `--auto --verbose dir/` | Verbose output shows files |

#### 5. `file-ops.tryscript.md` — File Operations

| # | Scenario | Command | Validates |
| --- | --- | --- | --- |
| FO1 | File to stdout | `flowmark file.md` | Output on stdout |
| FO2 | Inplace with backup | `flowmark --inplace file.md` | `.bak` file created |
| FO3 | Inplace no backup | `flowmark --inplace --nobackup file.md` | No `.bak` file |
| FO4 | Output to file | `flowmark -o output.md input.md` | Output written to file |
| FO5 | Output to stdout (explicit) | `flowmark -o - input.md` | Output on stdout |
| FO6 | Multiple files to stdout | `flowmark a.md b.md` | Both outputs concatenated |
| FO7 | Multiple files inplace | `flowmark --inplace --nobackup a.md b.md` | Both files modified |
| FO8 | Permissions preserved | `chmod 755 file.md && flowmark --auto file.md && stat` | Permissions unchanged |

#### 6. `stdin.tryscript.md` — Stdin Processing

| # | Scenario | Command | Validates |
| --- | --- | --- | --- |
| S1 | Basic stdin | `echo "..." \| flowmark -` | Stdin processing |
| S2 | Stdin with semantic | `echo "..." \| flowmark --semantic -` | Flag applies to stdin |
| S3 | Stdin with smartquotes | `echo 'He said "hello."' \| flowmark --smartquotes -` | Typography on stdin |
| S4 | Stdin with width | `echo "..." \| flowmark --width 30 -` | Width on stdin |

#### 7. `file-discovery.tryscript.md` — File Discovery

Uses the `project/` fixture tree for all tests.

| # | Scenario | Command | Validates |
| --- | --- | --- | --- |
| D1 | List files in directory | `flowmark --list-files project/` | Finds `.md` files |
| D2 | Excludes default dirs | `--list-files project/` | `node_modules/`, `.venv/`, `build/`, `vendor/`, `.git/` excluded |
| D3 | Extend include MDX | `--list-files --extend-include "*.mdx" project/` | MDX files included |
| D4 | Extend exclude drafts | `--list-files --extend-exclude "drafts/" project/` | Drafts excluded |
| D5 | Exclude replaces defaults | `--list-files --exclude "docs/" project/` | Only `docs/` excluded, defaults removed |
| D6 | Respect gitignore | `--list-files project/` | `skip/` excluded (in `.gitignore`) |
| D7 | No respect gitignore | `--list-files --no-respect-gitignore project/` | `skip/` included |
| D8 | Nested gitignore | `--list-files project/` | `nested/generated/` excluded by nested `.gitignore` |
| D9 | Flowmarkignore | `--list-files project/` | `skip/` excluded by `.flowmarkignore` |
| D10 | Force exclude | `--list-files --force-exclude project/node_modules/pkg/README.md` | Explicitly named file excluded |
| D11 | Force exclude off (default) | `--list-files project/node_modules/pkg/README.md` | Explicitly named file passes through |
| D12 | Files max size | `--list-files --files-max-size 100 project/` | Large files filtered |
| D13 | Glob pattern | `flowmark --list-files "project/docs/*.md"` | Glob expansion works |
| D14 | Deduplication | `--list-files project/README.md project/README.md` | Same file listed once |

#### 8. `config-interaction.tryscript.md` — Config File Interaction

Uses `config-tests/` fixture directories.

| # | Scenario | Directory | Validates |
| --- | --- | --- | --- |
| C1 | `.flowmark.toml` used | `dot-flowmark/` | Width from `.flowmark.toml` |
| C2 | `.flowmark.toml` takes precedence | `dot-flowmark/` | `.flowmark.toml` over `flowmark.toml` |
| C3 | `flowmark.toml` used | `flowmark-toml/` | Width and semantic from config |
| C4 | `pyproject.toml` with section | `pyproject/` | Width from `[tool.flowmark]` |
| C5 | `pyproject.toml` without section | `pyproject-no-section/` | Config not loaded (default width) |
| C6 | Kebab-case keys | `kebab-case/` | `list-spacing`, `extend-include` work |
| C7 | Nested sections | `sections/` | `[formatting]` and `[file-discovery]` parsed |
| C8 | Auto mode locks formatting | `auto-lock/` | `--auto` overrides `semantic=false` from config |
| C9 | CLI overrides config | `cli-overrides-config/` | `--width 80` beats config `width = 50` |
| C10 | Config walks up | Run from `dot-flowmark/subdir/` | Config found in parent |

#### 9. `verbose-docs.tryscript.md` — Verbose Output, Skills, Docs

| # | Scenario | Command | Validates |
| --- | --- | --- | --- |
| V1 | Verbose single file | `flowmark --verbose file.md` | “Formatting file.md” on stderr |
| V2 | Verbose directory | `flowmark --verbose --auto dir/` | Lists all files being formatted |
| V3 | Skill print | `flowmark --skill` | SKILL.md content with metadata |
| V4 | Install skill | `flowmark --install-skill --agent-base tmpdir/` | Creates `skills/flowmark/SKILL.md` |
| V5 | Docs print | `flowmark --docs` | Documentation content |
| V6 | Install skill creates dirs | `--install-skill --agent-base deep/nested/path/` | Nested directories created |

#### 10. `errors-version.tryscript.md` — Error Cases and Version

| # | Scenario | Command | Exit | Validates |
| --- | --- | --- | --- | --- |
| E1 | Version | `flowmark --version` | 0 | Version string format |
| E2 | No args | `flowmark` | 1 | “No input specified” message |
| E3 | Auto no args | `flowmark --auto` | 1 | “--auto requires” message |
| E4 | List-files no args | `flowmark --list-files` | 1 | “--list-files requires” message |
| E5 | Auto + list-files no args | `flowmark --auto --list-files` | 1 | Auto message takes priority |
| E6 | Nonexistent file | `flowmark nonexistent.md` | 1 | “failed to format” message |
| E7 | Output + multiple files | `flowmark -o out.md a.md b.md` | 1 | “Cannot specify output file” |
| E8 | Malformed config | `flowmark file.md` (with bad TOML) | 0 | Warning on stderr, formatting proceeds |

### Fixture Content Design

Each fixture file is designed to exercise specific Markdown constructs.
Below are content guidelines for key fixtures:

#### `comprehensive.md` — Kitchen Sink

Must include all of the following in a single file:
- YAML frontmatter (3+ fields)
- All 6 heading levels
- Bold headings (for `--cleanups` testing)
- Long paragraphs requiring wrapping at various widths
- Multiple sentences per paragraph (for `--semantic` testing)
- Tight list (no blank lines)
- Loose list (blank lines)
- Nested list (3 levels deep)
- Ordered list
- Blockquote with nested paragraph
- Fenced code block (backtick)
- Fenced code block (tilde)
- Inline code with special characters
- Straight double quotes and single quotes (for `--smartquotes`)
- Apostrophes and contractions (it’s, don’t, '90s)
- Triple dots `...` (for `--ellipses`)
- Backslash escapes (`\*`, `\#`, `\-`)
- Links (inline, reference-style)
- Images
- Bold, italic, strikethrough
- HTML block
- HTML inline
- Alert block (`> [!NOTE]`)
- Horizontal rule
- Table
- Footnote reference and definition
- Math (inline and display)

#### `typography.md` — Typography Test Cases

Specific cases for smart quote and ellipsis conversion:
- `"double quoted"` → curly double quotes
- `'single quoted'` → curly single quotes
- `it's`, `don't`, `won't` → curly apostrophes
- `"nested 'quotes' inside"` → correct nesting
- `...` → `…` (ellipsis)
- `"word..."` → ellipsis + curly quote (tests C2 bug)
- Quotes around emphasis: `"*bold*"` → curly quotes preserved
- Quotes in headings
- Quotes NOT converted in code blocks or inline code

#### `paragraphs-long.md` — Width Testing

3-4 paragraphs of varying length:
- One paragraph exactly 88 chars wide (at default width boundary)
- One very long paragraph (200+ chars)
- One short paragraph (under 88 chars, no wrapping needed)
- A paragraph with inline formatting (bold, links) that affects wrapping

#### `semantic-sentences.md` — Sentence Break Testing

Paragraphs with clear sentence boundaries:
- Simple two-sentence paragraph
- Three sentences with abbreviations (Mr., Dr., e.g.) that should NOT be treated as
  sentence boundaries
- Sentences ending in various punctuation (., !, ?)
- Sentences with parenthetical content
- Sentences with links that end with a period

### Binary-Agnostic Design Principles

The tryscript files must work identically for both Rust and Python binaries.
This imposes the following constraints on test authoring:

1. **Binary selection via PATH only.** The test files never reference `cargo`,
   `target/`, `uvx`, `pip`, or any build system.
   The `path:` frontmatter points to wherever the built binary lives — this is the ONLY
   thing that differs between Rust and Python invocations.

2. **No implementation-specific output.** Tests must not assert on output that differs
   between implementations for non-behavioral reasons:
   - Version strings: use `[VERSION]` pattern (matches `flowmark X.Y.Z` regardless of
     version number).
   - `--help` output: NOT tested (argparse vs clap produce different layouts — this is
     an accepted non-parity).
   - Error message wording: must be identical across both binaries (this IS a parity
     requirement).

3. **No language-specific file paths.** Fixtures and test commands never reference
   `site-packages`, `target/debug`, `.venv`, or similar paths.

4. **Fixtures are self-contained.** The `tests/tryscript/fixtures/` directory is the
   complete test corpus.
   Both repos include an identical copy (or symlink/submodule).
   Changes to fixtures must be synced across repos.

5. **SKILL.md content divergence is accepted.** The `--skill` output differs between
   Python (`uvx flowmark@latest`) and Rust (`cargo install flowmark`) install
   instructions. Skill content tests use `[..]` for install-method-specific lines but
   assert on shared structural elements (frontmatter keys, section headings).

6. **`--docs` content divergence is accepted.** Documentation may differ between repos.
   Tests assert on the presence of key sections, not exact content.

### Tryscript Configuration

All test files share common YAML frontmatter:

```yaml
---
sandbox: true
env:
  NO_COLOR: "1"
path:
  - $FLOWMARK_BIN_DIR
patterns:
  VERSION: 'flowmark \d+\.\d+\.\d+\S*'
before: |
  cp -r $TRYSCRIPT_FIXTURES_DIR/. fixtures/
---
```

The binary-agnostic design uses two environment variables set by the caller (CI script,
Makefile, or shell wrapper), NOT hardcoded in the test files:

- **`FLOWMARK_BIN_DIR`**: Directory containing the `flowmark` binary to test.
  - Rust: `target/debug` or `target/release`
  - Python: the virtualenv `bin/` dir, or wherever `uvx` installs it
- **`TRYSCRIPT_FIXTURES_DIR`**: Path to the fixtures directory.
  - Both repos: `tests/tryscript/fixtures`

**Example invocations:**

```bash
# Rust (from flowmark-rs repo)
FLOWMARK_BIN_DIR=target/debug \
TRYSCRIPT_FIXTURES_DIR=tests/tryscript/fixtures \
  npx tryscript run tests/tryscript/

# Python (from flowmark repo)
FLOWMARK_BIN_DIR=$(dirname $(which flowmark)) \
TRYSCRIPT_FIXTURES_DIR=tests/tryscript/fixtures \
  npx tryscript run tests/tryscript/
```

Key configuration choices:
- **`sandbox: true`**: Isolate all file operations in a temp directory.
- **`NO_COLOR: "1"`**: Disable color output for deterministic matching.
- **`path`**: `$FLOWMARK_BIN_DIR` adds the correct binary to PATH regardless of
  language.
- **`before`**: Copy fixtures into the sandbox so file modifications don’t affect the
  source tree.
- **`[VERSION]`**: Pattern to match version strings (avoids asserting on specific
  version numbers that differ between repos).

## Implementation Plan

### Phase 1: Fixture Creation

- [ ] Create `tests/tryscript/fixtures/content/` directory with all 22 content fixture
  files
- [ ] Create `tests/tryscript/fixtures/project/` directory tree with all file discovery
  fixtures (including `.gitignore`, `.flowmarkignore`, excluded dirs)
- [ ] Create `tests/tryscript/fixtures/config-tests/` with all 8 config test directories
  and their TOML files
- [ ] Create `tests/tryscript/fixtures/multi-file/` with 3 simple `.md` files
- [ ] Verify fixtures are valid by running `flowmark` (Python) on each content fixture
  and inspecting output

### Phase 2: Tryscript Test Files (Written Against Python)

Write all tryscript files and **validate against the Python binary first.** Python
flowmark v0.6.4 is the reference implementation — its output defines the golden
baseline. This approach has two benefits:
1. Python validates the test design: if a test fails against Python, the test itself is
   likely wrong (not a bug).
2. The golden output captured from Python becomes the authoritative expected output that
   Rust must match.

- [ ] Write `formatting.tryscript.md` — 12 scenarios covering width, plaintext,
  semantic, cleanups, combinations
- [ ] Write `typography-tests.tryscript.md` — 8 scenarios covering smart quotes,
  ellipses, edge cases
- [ ] Write `list-spacing.tryscript.md` — 6 scenarios covering preserve, loose, tight
- [ ] Write `auto-mode.tryscript.md` — 6 scenarios covering auto mode on files and
  directories
- [ ] Write `file-ops.tryscript.md` — 8 scenarios covering inplace, backup, output,
  multi-file
- [ ] Write `stdin.tryscript.md` — 4 scenarios covering stdin with various flags
- [ ] Write `file-discovery.tryscript.md` — 14 scenarios covering all discovery flags
- [ ] Write `config-interaction.tryscript.md` — 10 scenarios covering config loading and
  merge
- [ ] Write `verbose-docs.tryscript.md` — 6 scenarios covering verbose, skill,
  install-skill, docs
- [ ] Write `errors-version.tryscript.md` — 8 scenarios covering all error paths and
  version
- [ ] Run all tryscript tests against the **Python** binary
- [ ] Fix any test authoring bugs (tests that fail against the reference implementation)
- [ ] Add `[..]` patterns for platform-dependent output (paths, etc.)
- [ ] Add `[..]` patterns for binary-specific content (SKILL.md install instructions,
  docs) — never for formatting output, error messages, or file discovery behavior
- [ ] Review all golden output for correctness — this IS the expected behavior
- [ ] **All tests pass against Python.** This is the gate before proceeding.

### Phase 3: Rust Validation and Reconciliation

Run the same tryscript files (now validated against Python) against the Rust binary.
Every failure is either:

- A **Rust bug** to fix (Rust output differs from Python golden baseline), or

- A **Python bug** discovered by the more thorough test suite (rare, but possible — file
  an issue in the Python repo)

- [ ] Run all tryscript tests against the Rust binary

- [ ] Catalogue every failure: Rust bug vs Python bug vs test issue

- [ ] Fix Rust bugs that cause test failures

- [ ] File issues for any Python bugs discovered

- [ ] Verify all tests pass on both macOS and Linux

- [ ] **All tests pass against Rust.** Both binaries produce identical output on all 82
  scenarios.

### Phase 4: CI Integration (Rust repo)

- [ ] Add tryscript run step to Rust CI workflow (`.github/workflows/`)
- [ ] Set `FLOWMARK_BIN_DIR` and `TRYSCRIPT_FIXTURES_DIR` in CI environment
- [ ] Ensure tryscript tests run after `cargo build` produces the binary
- [ ] Configure CI to fail on any tryscript test mismatch
- [ ] Verify tests pass on both Ubuntu and macOS CI runners

### Phase 5: Backfill to Python Repo

The tryscript test files and fixtures are designed to be **identical** across both
repos. This phase copies the entire test suite to the Python `flowmark` repo and
integrates it into that repo’s CI pipeline.
Since the tryscript files are binary-agnostic, they should work without modification —
the only difference is which binary is on `PATH`.

#### 5.1: Copy Test Files and Fixtures

Copy the entire `tests/tryscript/` directory tree from `flowmark-rs` to the Python
`flowmark` repo, preserving the exact same directory structure:

```bash
# From the flowmark-rs repo root:
PYTHON_REPO=../flowmark  # adjust path to your Python flowmark checkout

# Copy all tryscript test files (10 .tryscript.md files)
cp tests/tryscript/formatting.tryscript.md "$PYTHON_REPO/tests/tryscript/"
cp tests/tryscript/typography-tests.tryscript.md "$PYTHON_REPO/tests/tryscript/"
cp tests/tryscript/list-spacing.tryscript.md "$PYTHON_REPO/tests/tryscript/"
cp tests/tryscript/auto-mode.tryscript.md "$PYTHON_REPO/tests/tryscript/"
cp tests/tryscript/file-ops.tryscript.md "$PYTHON_REPO/tests/tryscript/"
cp tests/tryscript/stdin.tryscript.md "$PYTHON_REPO/tests/tryscript/"
cp tests/tryscript/file-discovery.tryscript.md "$PYTHON_REPO/tests/tryscript/"
cp tests/tryscript/config-interaction.tryscript.md "$PYTHON_REPO/tests/tryscript/"
cp tests/tryscript/verbose-docs.tryscript.md "$PYTHON_REPO/tests/tryscript/"
cp tests/tryscript/errors-version.tryscript.md "$PYTHON_REPO/tests/tryscript/"

# Copy all fixtures (preserving directory structure)
cp -r tests/tryscript/fixtures/ "$PYTHON_REPO/tests/tryscript/fixtures/"
```

Files that need `.gitignore` handling in the Python repo (same as Rust repo):
- `tests/tryscript/fixtures/project/skip/ignored.md` — caught by the fixture’s own
  `.gitignore`; must be force-added with `git add -f`
- `tests/tryscript/fixtures/project/nested/generated/output.md` — caught by the nested
  `.gitignore`; must be force-added with `git add -f`

#### 5.2: Adapt Tryscript Frontmatter for Python

The tryscript files use a `path:` frontmatter field to locate the binary.
In the Rust repo, this points to `$TRYSCRIPT_GIT_ROOT/target/debug`. For the Python
repo, this must point to wherever the `flowmark` binary is installed.

**Option A: Use the system PATH (simplest)**

If `flowmark` is already installed in the Python repo’s CI environment (via
`pip install -e .` or `uv pip install -e .`), it will be on PATH by default.
In this case, the `path:` frontmatter can either be removed or set to the virtualenv bin
directory.

Update each tryscript file’s frontmatter from:

```yaml
path:
  - $TRYSCRIPT_GIT_ROOT/target/debug
```

To:

```yaml
path:
  - $TRYSCRIPT_GIT_ROOT/.venv/bin
```

Or, if using a system-wide install, simply remove the `path:` field entirely and rely on
the flowmark binary being on the default PATH.

**Option B: Use an environment variable (more flexible)**

Set `FLOWMARK_BIN_DIR` in CI and reference it in frontmatter:

```yaml
path:
  - $FLOWMARK_BIN_DIR
```

This is the most portable approach and matches the design in the “Tryscript
Configuration” section above.

#### 5.3: Add CI Job to Python Repo

Add a tryscript job to the Python repo’s CI workflow (e.g., `.github/workflows/ci.yml`
or equivalent):

```yaml
tryscript:
  name: Tryscript golden tests
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-python@v5
      with:
        python-version: "3.12"
    - uses: actions/setup-node@v4
      with:
        node-version: "20"
    - name: Install flowmark
      run: pip install -e .
    - name: Force-add gitignored fixtures
      run: |
        # These fixtures are intentionally gitignored by their parent dirs
        # but need to exist for tests to work
        git add -f tests/tryscript/fixtures/project/skip/ignored.md
        git add -f tests/tryscript/fixtures/project/nested/generated/output.md
    - name: Run tryscript golden tests
      run: npx tryscript@latest run tests/tryscript/
```

**Key CI considerations:**
- Node.js is required for `npx tryscript@latest`
- The Python binary must be installed and on PATH before tryscript runs
- `NO_COLOR=1` is set in the tryscript frontmatter, so no CI-level env override needed
- Force-adding gitignored fixtures may not be needed if they’re committed with `-f`
  during the initial copy

#### 5.4: Validate All Tests Pass in Python CI

- [ ] Run the full tryscript suite locally against the Python binary:
  ```bash
  cd $PYTHON_REPO
  pip install -e .
  npx tryscript@latest run tests/tryscript/
  ```
- [ ] Verify all 79 scenarios pass (same count as the Rust repo)
- [ ] Push and confirm the CI job passes in the Python repo’s CI pipeline
- [ ] If any tests fail, investigate whether it’s a frontmatter/path issue or a real
  parity difference

#### 5.5: Establish Sync Protocol

Since the test files are identical across repos, changes must be synced:

- **Upstream direction**: The Rust repo (`flowmark-rs`) is the upstream source for
  tryscript test changes.
  All new test scenarios, fixture changes, and golden output updates originate here.
- **Sync trigger**: After any tryscript change merges to `main` in `flowmark-rs`, the
  same change should be copied to the Python repo.
- **Sync verification**: After copying, run the tryscript suite against the Python
  binary to confirm parity.
- **Divergence handling**: If a test must differ between repos (e.g., due to an accepted
  parity gap), document the divergence in both repos and use `[..]` or `...` patterns to
  accommodate both outputs.

#### 5.6: Update Test Mapping

Add entries for each tryscript file to `port-coverage-mapping/test-mapping.yaml`,
`port-coverage-mapping/python-tests.yaml`, and `port-coverage-mapping/rust-tests.yaml`
so the mapping framework tracks that these golden tests exist in both repos.
One entry per tryscript file is sufficient since the files are identical.

#### Phase 5 Checklist

- [ ] Copy all 10 tryscript test files to Python repo
- [ ] Copy all fixture directories to Python repo
- [ ] Force-add any gitignored fixture files
- [ ] Adapt `path:` frontmatter for Python binary location
- [ ] Add tryscript CI job to Python repo’s workflow
- [ ] Validate all 79 scenarios pass against the Python binary locally
- [ ] Validate CI passes in the Python repo
- [ ] Document the sync protocol in both repos
- [ ] Add tryscript entries to the test mapping YAML files

### Phase 6: Retire Old Tryscript File

- [ ] Verify all scenarios from `cli-golden.tryscript.md` are covered by the new suite
- [ ] Remove or archive `cli-golden.tryscript.md`
- [ ] Update references in other specs and docs

## Testing Strategy

The tryscript golden tests ARE the testing strategy — they validate end-to-end CLI
behavior. The process is:

1. **Python first**: Write tests and validate against the Python reference binary.
   Python output defines the golden baseline.
   If a test fails against Python, the test is probably wrong.
2. **Rust second**: Run the same tests against Rust.
   Every failure is a parity bug to fix.
3. **CI enforced**: Both repos run the same tests in CI. A test failure in either repo
   blocks merge.

**Verification checklist for each test file:**
1. All scenarios pass against Python with `npx tryscript run <file>`
2. All scenarios pass against Rust with `npx tryscript run <file>`
3. Output matches expected golden content exactly (or with documented `[..]` patterns)
4. Exit codes correct for all error cases
5. No platform-specific failures (test on macOS and Linux)

## Cross-Binary Validation Walkthrough

This section describes the concrete process for running the tryscript suite against both
binaries. Since the Python flowmark is already checked out at `attic/flowmark/`,
validation is straightforward.

### Setting Up the Python Binary

```bash
# From the flowmark-rs repo root:
# The Python source is already at attic/flowmark/ (cloned at v0.6.4)
cd attic/flowmark
pip install -e .      # or: uv pip install -e .
which flowmark        # → should point to the Python binary
flowmark --version    # → flowmark 0.6.4
cd ../..              # back to flowmark-rs root
```

### Running Against Python (Phase 2)

```bash
# Set env vars to point at the Python binary
export FLOWMARK_BIN_DIR=$(dirname $(which flowmark))
export TRYSCRIPT_FIXTURES_DIR=tests/tryscript/fixtures

# Run all tryscript tests against Python
npx tryscript run tests/tryscript/

# Or run a single test file for iterative development
npx tryscript run tests/tryscript/formatting.tryscript.md
```

When a test fails against Python, there are two possibilities:
1. **The test is wrong** (most likely during initial development) — fix the test.
2. **Python has a bug** — see “Handling Python Bugs” below.

### Running Against Rust (Phase 3)

```bash
# Build Rust binary
cargo build

# Set env vars to point at the Rust binary
export FLOWMARK_BIN_DIR=target/debug
export TRYSCRIPT_FIXTURES_DIR=tests/tryscript/fixtures

# Run all tryscript tests against Rust
npx tryscript run tests/tryscript/
```

When a test fails against Rust but passes against Python:
1. **Rust has a bug** (most common) — fix the Rust implementation.
2. **The test exposes a difference that needs `[..]` pattern** — only for accepted
   divergences (SKILL.md, docs content).

### Side-by-Side Comparison

For investigating specific failures, run the same command against both binaries and
diff:

```bash
# Python output
PATH=$(dirname $(which flowmark)):$PATH flowmark --semantic fixtures/content/comprehensive.md > /tmp/python-out.md

# Rust output
PATH=target/debug:$PATH flowmark --semantic fixtures/content/comprehensive.md > /tmp/rust-out.md

# Diff
diff /tmp/python-out.md /tmp/rust-out.md
```

### Handling Python Bugs

If the comprehensive test suite reveals bugs in Python flowmark (e.g., incorrect
formatting, inconsistent behavior), these are recorded separately:

1. **Create a spec in the Python repo**: File a new spec under
   `docs/project/specs/active/` in the Python `flowmark` repo documenting all bugs found
   during tryscript validation.
   This spec becomes a standalone follow-up for the Python maintainer.

2. **Track in the Rust repo**: Create a bead in the Rust repo referencing the Python
   bug, so we know the tryscript test is intentionally testing the corrected behavior
   (not the buggy Python behavior).

3. **Decide on golden output**: For each Python bug, choose one of:
   - **Python is wrong, Rust is right**: Use Rust’s output as golden.
     Add `[..]` or adjust test to pass for Python until fixed.
     Mark the test with a comment noting the Python bug.
   - **Both are wrong**: Fix Rust first, then use the corrected output as golden.
     File the Python fix separately.
   - **Behavior is ambiguous**: Document as an open question and use `[..]` to accept
     either output until resolved.

4. **Template for the Python bug spec:**
   ```
   # Bugs Found During Comprehensive Tryscript Validation
   Date: YYYY-MM-DD
   Source: flowmark-rs tryscript test suite (Phase 2 validation)

   ## Bug N: [description]
   - Tryscript file: [which test file]
   - Scenario: [which scenario ID]
   - Input: [fixture file or inline input]
   - Expected: [what the correct output should be]
   - Actual (Python): [what Python produces]
   - Rust behavior: [what Rust produces — correct or also wrong]
   ```

## Open Questions

1. ~~**Should the existing `cli-golden.tryscript.md` be kept alongside or fully
   replaced?**~~ **RESOLVED**: Replace entirely.
   The new suite covers all 31 existing scenarios and adds ~50 more.
   Remove `cli-golden.tryscript.md` in Phase 6 after verifying full coverage.

2. ~~**Should config interaction tests be tryscript or Rust integration tests?**~~
   **RESOLVED**: Tryscript for all config interaction tests.
   Config behavior depends on `cwd` and file system state, which tryscript handles
   perfectly via `sandbox: true` and `before:` setup.
   Tryscript tests are portable (same tests run against both binaries) and maintainable
   (easier to read and modify than Rust code).
   Keep the existing 20 Rust unit tests in `tests/test_config.rs` for internal config
   parsing logic (TOML deserialization, field validation), but all end-to-end config
   interaction scenarios (file discovery, precedence, auto-lock, CLI override) belong in
   tryscript.

## References

- Exact parity spec: `docs/project/specs/active/plan-2026-02-17-exact-parity.md` (Phase
  10.4)
- Current tryscript tests: `tests/tryscript/cli-golden.tryscript.md` (31 tests)
- Tryscript documentation: `npx tryscript@latest docs`
- Golden testing guidelines: `tbd guidelines golden-testing-guidelines`
- Appendix D (bugs with no test coverage): `plan-2026-02-17-exact-parity.md` Appendix D
- Python flowmark repo: https://github.com/jlevy/flowmark (v0.6.4)
- Local Python checkout: `attic/flowmark/` (gitignored)

## Appendix: Fixture File Cross-Reference

Which fixtures are used by which test files:

| Fixture | formatting | typography | list-spacing | auto-mode | file-ops | stdin | file-discovery | config | verbose | errors |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `simple.md` | F11 |  |  |  | FO1-FO5,FO8 |  |  |  | V1 |  |
| `paragraphs-long.md` | F1-F4 |  |  |  |  |  |  |  |  |  |
| `semantic-sentences.md` | F6,F8 |  |  |  |  |  |  |  |  |  |
| `headings.md` | F7,F9 |  |  |  |  |  |  |  |  |  |
| `typography.md` |  | T1-T3,T7 |  |  |  |  |  |  |  |  |
| `lists-tight.md` |  |  | LS1,LS3 |  |  |  |  |  |  |  |
| `lists-loose.md` |  |  | LS2,LS4 |  |  |  |  |  |  |  |
| `lists-nested.md` |  |  | LS5 |  |  |  |  |  |  |  |
| `lists-ordered.md` |  |  | LS6 |  |  |  |  |  |  |  |
| `code-blocks.md` |  | T4,T5 |  |  |  |  |  |  |  |  |
| `escapes.md` |  | T6 |  |  |  |  |  |  |  |  |
| `comprehensive.md` | F10,F12 |  |  | A1-A3 |  |  |  |  |  |  |
| `plaintext.txt` | F5 |  |  |  |  |  |  |  |  |  |
| `project/` |  |  |  | A2,A5,A6 |  |  | D1-D14 |  | V2 |  |
| `config-tests/` |  |  |  | A4 |  |  |  | C1-C10 |  |  |
| `multi-file/` |  |  |  |  | FO6,FO7 |  |  |  |  | E7 |
