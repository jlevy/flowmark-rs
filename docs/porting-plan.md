# Flowmark Python-to-Rust Port: Master Plan

## Goal

Port [flowmark](https://github.com/jlevy/flowmark) (Python Markdown auto-formatter) to
Rust with **100% test conformance**. This is a fresh implementation based on the latest
Python version, using the previous `flowmark-rs-1` as reference for comrak workarounds.

## Source Analysis

### Python Codebase (flowmark)
- **26 source files**, ~4,433 lines
- **20 test files**, ~5,619 lines
- **5 test fixture files**, ~8,301 lines
- Core dependency: `marko` (CommonMark/GFM Markdown parser)

### Previous Rust Attempt (flowmark-rs-1)
- **21 source files**, ~4,939 lines
- Used `comrak` for Markdown parsing
- ~95% cross-validation match with Python
- Missing: typography integration into pipeline, plaintext mode, multi-file batch
- Key learning: 20+ workaround functions needed to match comrak output to marko output

## Architecture Decision

**Single package with feature-gated CLI** (same as flowmark-rs-1, per playbook recommendation).

```
src/
  lib.rs              # Public API: reformat_text, reformat_file, fill_markdown, etc.
  main.rs             # CLI entry point (behind "cli" feature)
  args.rs             # CLI argument parsing (clap derive)
  config.rs           # Config, ListSpacing enum, config loading
  error.rs            # Error enum with thiserror
  formatter/
    mod.rs
    markdown.rs       # comrak-based Markdown parsing/rendering
    filling.rs        # fill_markdown pipeline + normalization fixups
  parser/
    mod.rs
    frontmatter.rs    # YAML frontmatter handling
  wrapping/
    mod.rs
    sentence.rs       # Sentence splitting regex
    text_wrapping.rs  # Word splitting, paragraph wrapping
    atomic_patterns.rs # Atomic construct patterns
    line_wrappers.rs  # LineWrapper implementations
    tag_handling.rs   # Jinja/Markdoc/HTML tag handling
    block_heuristics.rs # Table/list detection
  transform/
    mod.rs
    visitor.rs        # AST visitor/walker
    cleanups.rs       # unbold_headings, doc_cleanups
    paragraph_wrapping.rs # AST-level paragraph wrapping
  typography/
    mod.rs
    quotes.rs         # Smart quotes
    ellipses.rs       # Ellipsis conversion
tests/
  test_sentences.rs
  test_wrapping.rs
  test_filling.rs
  test_tag_formatting.rs
  test_list_spacing.rs
  test_smartquotes.rs
  test_ellipses.rs
  test_escape_handling.rs
  test_heading_spacing.rs
  test_fenced_code_blocks.rs
  test_alerts.rs
  test_frontmatter.rs
  test_cleanups.rs
  test_strikethrough.rs
  test_width_options.rs
  test_ref_docs.rs
  test_config.rs
```

## Dependency Mapping

| Python Package | Rust Crate | Risk |
| --- | --- | --- |
| marko | comrak | **High** - 20+ workarounds needed |
| regex (Python) | regex | Low |
| pathspec | ignore / glob | Low |
| tomli/tomllib | toml | Low |
| argparse | clap (derive) | Low |
| strif (atomic file writes) | tempfile | Low |

## Module Porting Order (leaf-first)

### Phase 1: Foundation
1. error.rs - Error enum
2. config.rs - Config, ListSpacing
3. parser/frontmatter.rs - YAML frontmatter

### Phase 2: Text Processing (leaf modules)
4. wrapping/atomic_patterns.rs - Regex patterns for atomic constructs
5. wrapping/block_heuristics.rs - Table/list line detection
6. wrapping/sentence.rs - Sentence splitting
7. wrapping/text_wrapping.rs - Word splitting and paragraph wrapping
8. wrapping/tag_handling.rs - Template tag handling
9. wrapping/line_wrappers.rs - LineWrapper factories

### Phase 3: Typography
10. typography/ellipses.rs - Ellipsis conversion
11. typography/quotes.rs - Smart quotes

### Phase 4: Markdown Pipeline
12. formatter/markdown.rs - comrak parsing/rendering
13. transform/visitor.rs - AST visitor
14. transform/cleanups.rs - Document cleanups
15. transform/paragraph_wrapping.rs - AST paragraph wrapping
16. formatter/filling.rs - fill_markdown pipeline

### Phase 5: API & CLI
17. lib.rs - Public API (reformat_text, reformat_file)
18. args.rs - CLI arguments
19. main.rs - CLI entry point

## Acceptance Criteria

- [ ] 100% of ported tests passing
- [ ] Cross-validation against Python on all test fixtures
- [ ] All differences documented with explicit decisions
- [ ] cargo fmt, cargo clippy, cargo test all passing

## Key Pitfalls (from playbook + flowmark-rs-1 experience)

1. **Regex anchoring**: Python `re.match()` → add `^` in Rust
2. **comrak vs marko differences**: ~20 normalization fixups needed
3. **Whitespace preservation**: Don't add `.trim()` unless Python does
4. **Unicode width**: Use `chars().count()` not `.len()` for display width
5. **Dict ordering**: Python preserves insertion order; use IndexMap if needed
6. **Arena/lifetime patterns**: comrak uses arena allocation
7. **Sentence splitting**: Uses `regex` crate (not `fancy-regex`) with `\p{L}` Unicode
