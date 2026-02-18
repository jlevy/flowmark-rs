# Flowmark Python-to-Rust Port: Master Plan

**Status: Complete** (2026-02-17)

All porting work is finished.
251 tests passing, 0 ignored, 100% of ported Python tests passing.
See the [exact parity spec](../active/plan-2026-02-17-exact-parity.md) for full details.

## Goal

Port [flowmark](https://github.com/jlevy/flowmark) (Python Markdown auto-formatter) to
Rust with **100% test conformance**. This is a fresh implementation based on the latest
Python version, using the previous `flowmark-rs-1` as reference for comrak workarounds.

## Source Analysis

### Python Codebase (flowmark)

- **26 source files**, ~4,433 lines (2,531 code lines)
- **20 test files**, ~5,619 lines (2,748 code lines)
- **5 test fixture files**, ~8,301 lines
- Core dependency: `marko` (CommonMark/GFM Markdown parser)

### Previous Rust Attempt (flowmark-rs-1)

- **21 source files**, ~4,939 lines
- Used `comrak` 0.29 for Markdown parsing
- ~95% cross-validation match with Python
- Missing: typography integration into pipeline, plaintext mode, multi-file batch
- Key learning: 14 post-processing workaround functions needed to match comrak to marko

### Current Port (flowmark-rs, this repo)

- **22 source files**, ~3,485 lines (2,610 code lines)
- **17 test files**, ~3,688 lines (2,674 code lines)
- Uses `comrak` 0.36 with a custom AST renderer (no workaround functions)
- 251 tests passing, 0 ignored
- 202 Python tests mapped, 79 excluded (infrastructure), 0 missing
- Rust/Python code lines ratio: 1.00x (5,284 vs 5,279)

## Architecture Decision

**Single package with feature-gated CLI** (same as flowmark-rs-1, per playbook
recommendation).

### Actual Module Layout

```
src/
  lib.rs              # Public API: reformat_text, reformat_file, fill_markdown, etc.
  main.rs             # CLI entry point + argument parsing (behind "cli" feature)
  config.rs           # Config, ListSpacing enum
  error.rs            # Error enum with thiserror
  formatter/
    mod.rs
    markdown.rs       # comrak-based Markdown parsing, custom AST renderer
    filling.rs        # fill_markdown pipeline, AST rendering (~1,000 lines)
  parser/
    mod.rs
    frontmatter.rs    # YAML frontmatter handling
  wrapping/
    mod.rs
    sentence.rs       # Sentence splitting regex
    text_wrapping.rs  # Word splitting, paragraph wrapping
    text_filling.rs   # Text filling (sentence-aware line wrapping)
    atomic_patterns.rs # Atomic construct patterns
    line_wrappers.rs  # LineWrapper implementations
    tag_handling.rs   # Jinja/Markdoc/HTML tag handling
    block_heuristics.rs # Table/list detection
  transform/
    mod.rs
    cleanups.rs       # unbold_headings, doc_cleanups
  typography/
    mod.rs
    quotes.rs         # Smart quotes
    ellipses.rs       # Ellipsis conversion
tests/
  test_alerts.rs
  test_cleanups.rs
  test_edge_cases.rs
  test_ellipses.rs
  test_escape_handling.rs
  test_fenced_code_blocks.rs
  test_filling.rs
  test_frontmatter.rs
  test_heading_spacing.rs
  test_list_spacing.rs
  test_ref_docs.rs
  test_sentences.rs
  test_smartquotes.rs
  test_strikethrough.rs
  test_tag_formatting.rs
  test_width_options.rs
  test_wrapping.rs
```

### Key Architectural Differences from v1

The original plan included `args.rs`, `transform/visitor.rs`, and
`transform/paragraph_wrapping.rs`. The actual implementation:

- **CLI args** are in `main.rs` (no separate `args.rs` — simple enough to inline)
- **No AST visitor/walker** — the custom renderer in `filling.rs` walks the AST directly
- **No AST-level paragraph wrapping** — wrapping is integrated into the rendering
  pipeline
- **Added `text_filling.rs`** — sentence-aware text filling, not in the original plan
- **Added `test_edge_cases.rs`** — additional edge case coverage not in original plan

## Dependency Mapping

| Python Package | Rust Crate | Risk | Status |
| --- | --- | --- | --- |
| marko | comrak 0.36 | **High** - custom AST renderer | Done |
| regex (Python) | regex 1.11 | Low | Done |
| argparse | clap 4.5 (derive) | Low | Done |
| strif (atomic file writes) | tempfile 3.10 | Low | Done |
| — | thiserror 2.0 | Low | Done |
| — | anyhow 1.0 (CLI-only) | Low | Done |
| — | libc 0.2 (SIGPIPE) | Low | Done |

**Not ported (not needed):** pathspec (multi-file batch not ported), tomli/tomllib
(config file loading not ported — config is via CLI args only).

## Module Porting Order (leaf-first)

The original plan called for 5 phases, 19 modules.
The actual port followed this order with minor deviations (noted above in Key
Architectural Differences).
All phases are complete.

### Phase 1: Foundation

1. error.rs - Error enum
2. config.rs - Config, ListSpacing
3. parser/frontmatter.rs - YAML frontmatter

### Phase 2: Text Processing (leaf modules)

4. wrapping/atomic_patterns.rs - Regex patterns for atomic constructs
5. wrapping/block_heuristics.rs - Table/list line detection
6. wrapping/sentence.rs - Sentence splitting
7. wrapping/text_wrapping.rs - Word splitting and paragraph wrapping
8. wrapping/text_filling.rs - Sentence-aware text filling (added during port)
9. wrapping/tag_handling.rs - Template tag handling
10. wrapping/line_wrappers.rs - LineWrapper factories

### Phase 3: Typography

11. typography/ellipses.rs - Ellipsis conversion
12. typography/quotes.rs - Smart quotes

### Phase 4: Markdown Pipeline

13. formatter/markdown.rs - comrak parsing, custom AST renderer
14. transform/cleanups.rs - Document cleanups
15. formatter/filling.rs - fill_markdown pipeline + AST rendering

### Phase 5: API & CLI

16. lib.rs - Public API (reformat_text, reformat_file)
17. main.rs - CLI entry point + argument parsing

## Acceptance Criteria

- [x] 100% of ported tests passing (251 tests, 0 ignored)
- [x] Cross-validation against Python on all test fixtures (202 mapped, 79 excluded)
- [x] All differences documented with explicit decisions
- [x] cargo fmt, cargo clippy, cargo test all passing

## Syncing with Python Flowmark

The Python flowmark source is tracked as a git submodule at `repos/flowmark`,
currently pinned to `v0.6.4`. The porting playbook (also a submodule) is at
`repos/rust-porting-playbook`.

See [docs/port-sync-playbook.md](../../port-sync-playbook.md) for the full
sync process.

## Key Pitfalls (from playbook + actual porting experience)

1. **Regex anchoring**: Python `re.match()` → add `^` in Rust
2. **comrak vs marko differences**: Solved in v2 via custom AST renderer rather than
   post-processing workarounds (v1 needed 14 fixup functions)
3. **Whitespace preservation**: Don’t add `.trim()` unless Python does
4. **Unicode width**: Use `chars().count()` not `.len()` for display width
5. **Arena/lifetime patterns**: comrak uses arena allocation — affects API design
6. **Sentence splitting**: Uses `regex` crate (not `fancy-regex`) with `\p{L}` Unicode
7. **Escape preservation**: comrak strips backslash escapes during parsing — solved via
   Unicode PUA (Private Use Area) placeholder system
