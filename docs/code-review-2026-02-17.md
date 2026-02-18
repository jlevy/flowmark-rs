# Code Review: flowmark-rs (Python-to-Rust Port)

**Date:** 2026-02-17
**Reviewer:** Claude Opus 4.6
**Scope:** Full top-to-bottom review of Rust port against Python original (github.com/jlevy/flowmark)
**Commit:** 0dda5de (branch: claude/setup-tbd-tool-tBXpO)

## Executive Summary

This is a high-quality port of the Python flowmark Markdown formatter to Rust. All 250
tests pass, the module structure is clean, and the codebase demonstrates strong Rust
practices overall (zero unsafe in library code, thiserror/anyhow split, feature-gated
CLI, LazyLock regex patterns, deny.toml supply chain security). The test mapping is
complete with 202 mapped and 79 intentionally excluded tests.

However, there are several issues that should be addressed before release, ranging from
dead dependencies that inflate compile times to clippy/formatting failures and code
duplication.

---

## Build & Test Status

| Check | Result | Notes |
|-------|--------|-------|
| `cargo build` | PASS | |
| `cargo build --no-default-features` | PASS | Library-only without CLI |
| `cargo test` (250 tests) | PASS | 27 unit + 223 integration |
| `cargo clippy -- -D warnings` | **FAIL** | 9 `inefficient_to_string` errors |
| `cargo fmt --check` | **FAIL** | Extensive formatting diffs |
| End-to-end CLI (stdin, --semantic, --cleanups, --smartquotes) | PASS | |
| Test mapping coverage | Complete | 202 mapped, 79 excluded, 0 missing |

---

## Issues by Priority

### P0: Blocking (CI would fail)

#### 1. Clippy Failures: 9 `inefficient_to_string` Errors

**Files:** `src/formatter/filling.rs` (6 occurrences), `src/wrapping/tag_handling.rs` (3 occurrences)

**Problem:** Calling `.to_string()` on `&&str` uses a slower blanket `ToString` impl
instead of the specialized `str::to_string()`.

**Locations in `src/formatter/filling.rs`:**
- Line 147: `result.push(line.to_string());`
- Line 168: `result.push(line.to_string());`
- Line 171: `let mut processed = line.to_string();`
- Line 201: `result.push(line.to_string());`
- Line 223: `result.push(line.to_string());`
- Line 257: `result.push(line.to_string());`

**Locations in `src/wrapping/tag_handling.rs`:**
- Line 226: `fixed_lines.push(line.to_string());`
- Line 244: `result_lines.push(line.to_string());`
- Line 277: `result_lines.push(line.to_string());`

**Fix:** In all cases, change `line.to_string()` to `(*line).to_string()` or preferably
`line.to_owned()` (which auto-derefs). Since these are all iterating `&lines` where
`lines: Vec<&str>`, the loop variable is `&&str`.

#### 2. Formatting Violations

**Problem:** `cargo fmt --check` shows diffs in nearly every source and test file.
The project has `rustfmt.toml` configured (`max_width = 100`, `use_small_heuristics = "Max"`)
but the code was not formatted with it.

**Fix:** Run `cargo fmt` across the entire project. Key areas with diffs:
- `src/formatter/filling.rs` — import ordering, closure formatting, long conditionals
- `src/wrapping/tag_handling.rs` — import ordering
- `src/wrapping/line_wrappers.rs` — import ordering
- `tests/test_wrapping.rs` — long string literals, multi-line asserts
- `tests/test_tag_formatting.rs` — assert formatting
- Multiple other test files

---

### P1: Should Fix Before Release

#### 3. Dead Dependencies (compile-time bloat)

**File:** `Cargo.toml` lines 28-31

Three dependencies are declared but **never imported or used** anywhere in `src/`:

```toml
# DEAD — zero imports anywhere in source code:
unicode-segmentation = "1.11"
toml = "0.8"
serde = { version = "1.0", features = ["derive"] }
```

**Verification:** `grep -r 'use (toml|serde|unicode.segmentation)' src/` returns zero
matches. No `#[derive(Serialize, Deserialize)]` or `#[serde(...)]` attributes exist.

These were likely carried over from the porting plan for config file loading (Python's
`.flowmark.toml` / `pyproject.toml` merging), which was intentionally not ported.

**Impact:** These add significant compile time (serde alone pulls in proc-macros).

**Fix:** Remove all three from `[dependencies]` in `Cargo.toml`.

#### 4. Dead Error Variants

**File:** `src/error.rs` lines 9-13

```rust
#[error("Configuration error: {0}")]
Config(String),

#[error("{0}")]
Other(String),
```

Neither `Error::Config` nor `Error::Other` is constructed anywhere in the codebase.
`grep -r 'Error::(Config|Other)' src/` returns zero matches.

Only `Error::Io` (via `#[from] std::io::Error`) is ever used.

**Fix:** Remove both variants. If config loading is added later, re-add `Config` then.

---

### P2: Recommended Improvements

#### 5. Code Duplication: Fence-Tracking Pattern (3x in filling.rs)

**File:** `src/formatter/filling.rs`

The pattern "iterate lines, track whether inside a fenced code block using fence_str
state, process lines differently inside vs outside code" is copy-pasted three times:

| Function | Lines | Purpose |
|----------|-------|---------|
| `collapse_blank_lines_outside_code` | 83-134 | Collapse multiple blank lines |
| `protect_escapes_outside_code` | 138-185 | Replace escape chars with PUA placeholders |
| `postprocess_period_escapes` | 192-270 | Remove unnecessary `\.` escapes |

All three share ~20 lines of identical fence-detection logic:
```rust
let trimmed = line.trim();
let is_backtick_fence = trimmed.starts_with("```");
let is_tilde_fence = trimmed.starts_with("~~~");
if is_backtick_fence || is_tilde_fence {
    let fence_char = if is_backtick_fence { '`' } else { '~' };
    let fence_len = trimmed.chars().take_while(|&c| c == fence_char).count();
    fence_str = std::iter::repeat_n(fence_char, fence_len).collect();
    in_code = true;
    // ...
}
// And the matching closing-fence check
```

**Fix:** Extract a helper function like:
```rust
fn process_lines_outside_code<F>(text: &str, process_line: F) -> String
where F: Fn(&str) -> String
```
This would reduce ~60 lines of duplication and make the fence-tracking logic a single
source of truth.

#### 6. Unnecessary Allocation in Ellipsis Character Check

**File:** `src/typography/ellipses.rs` lines 42, 56, 67

```rust
if !WORD_CHAR_RE.is_match(&nc.to_string()) {  // allocates String for 1 char
```

This compiles a regex, allocates a `String`, and runs a regex match — just to check if a
single `char` is a word character. This happens in a hot loop over every ellipsis match.

**Fix:** Replace with `nc.is_alphanumeric() || nc == '_'` (which is what `\w` matches
for ASCII). If full Unicode `\w` semantics are needed, use
`nc.is_alphanumeric() || nc == '_'` which covers the relevant cases. This eliminates
both the allocation and the regex overhead.

#### 7. Vec<char> Allocation in remove_period_escapes_preserving_code

**File:** `src/formatter/filling.rs` line 275

```rust
let chars: Vec<char> = line.chars().collect();
```

Collects every character in the line into a heap-allocated vector, then indexes into it.
This is called for every non-code-block line in the output.

**Fix:** Use a `Peekable<CharIndices>` iterator or manual byte-offset tracking to avoid
the allocation while maintaining the same lookahead behavior.

#### 8. Public API: Boolean Parameter Overload

**File:** `src/lib.rs` lines 24-58

`reformat_text` takes 8 parameters (5 booleans), `reformat_file` takes 11 parameters
(7 booleans). Both have `#[allow(clippy::fn_params_excessive_bools)]`.

This is a faithful port of the Python API, but idiomatic Rust would use an options struct:

```rust
#[derive(Debug, Clone, Default)]
pub struct FormatOptions {
    pub width: usize,
    pub plaintext: bool,
    pub semantic: bool,
    pub cleanups: bool,
    pub smartquotes: bool,
    pub ellipses: bool,
    pub list_spacing: ListSpacing,
}
```

The current API works but becomes unwieldy for callers:
```rust
// Current — which bool is which?
reformat_text(text, 88, false, true, true, true, true, ListSpacing::Preserve)

// With struct:
reformat_text(text, &FormatOptions { semantic: true, cleanups: true, ..Default::default() })
```

**Priority:** Consider for 1.0 release. Not urgent since the API is internal-facing for now.

#### 9. Unused `_name` Field in AtomicPattern

**File:** `src/wrapping/atomic_patterns.rs` line 16

```rust
pub(crate) struct AtomicPattern {
    pub(crate) _name: &'static str,  // underscore-prefix to suppress warning
    // ...
}
```

The `_name` field is assigned in all 6 static instances but never read. The underscore
prefix is a Rust convention for "intentionally unused," but if this is truly unused,
it should either:
- Be removed entirely, OR
- Be renamed to `name` with `#[allow(dead_code)]` on the field (if kept for debugging)

#### 10. Unnecessary String Clone in Code Block Rendering

**File:** `src/formatter/filling.rs` lines 605-609

```rust
let lang_text = if info.is_empty() {
    String::new()
} else {
    info.clone()  // info is &String from AST — unnecessary heap allocation
};
```

**Fix:** Use `info` directly in the `writeln!` call:
```rust
let _ = writeln!(output, "{prefix}{fence}{info}");
// (empty info just produces no extra text)
```
Since `info` is already a `String` reference and `writeln!` takes `Display` impls,
no clone is needed.

#### 11. Repeated `.expect()` Calls in line_wrappers.rs

**File:** `src/wrapping/line_wrappers.rs` lines 115-140

`lines.last().expect("non-empty lines")` is called 4 times in the same code block,
repeatedly computing the same value.

**Fix:** Extract to a local: `let last_line = lines.last().expect("non-empty lines");`

---

### P3: Nice to Have

#### 12. DEFAULT_WRAP_WIDTH Comment

**File:** `src/config.rs` line 44

```rust
/// Default wrap width. Same as Black (88 characters).
pub const DEFAULT_WRAP_WIDTH: usize = 88;
```

"Same as Black" is a Python formatter reference that's confusing in a Rust context.
Consider: `/// Default wrap width (88 characters).`

#### 13. No Doc-Tests

The library has doc comments but no `/// # Examples` with runnable code. Adding one
doc-test for `reformat_text` or `fill_markdown` would improve API discoverability
and serve as living documentation.

#### 14. simple_word_split Returns Owned Strings Unnecessarily

**File:** `src/wrapping/text_wrapping.rs` line 78

```rust
pub fn simple_word_split(text: &str) -> Vec<String> {
    text.split_whitespace().map(String::from).collect()
}
```

This allocates a new `String` for each word when `Vec<&str>` would suffice. However,
this matches the `html_md_word_split` return type for interface compatibility, so
changing it would require updating the `WordSplitter` trait/function pointer signature.

**Verdict:** Leave as-is unless the splitter interface is refactored.

---

## Things Done Well

These are worth calling out as strong practices to maintain:

1. **Zero unsafe code** — `unsafe_code = "deny"` in Cargo.toml; one necessary exception
   in `main.rs` for SIGPIPE handling, properly annotated with `#[allow(unsafe_code)]`

2. **Test coverage** — 250 tests (27 unit + 223 integration), 4-mode golden test on a
   1,416-line reference document, complete Python test mapping

3. **Error handling split** — `thiserror` for typed library errors, `anyhow` for CLI
   context chains. No `unwrap()` in library code (all replaced with `expect()` with
   descriptive messages)

4. **Feature-gated CLI** — Library usable without clap/anyhow/tempfile via
   `--no-default-features`. Binary correctly requires `cli` feature.

5. **Release profile** — LTO, single codegen unit, panic=abort, strip — well-optimized

6. **Supply chain security** — `deny.toml` with license allowlist and source restrictions

7. **Idiomatic patterns** — `LazyLock<Regex>` for compiled regexes, `Box<dyn Fn + Send + Sync>`
   for composable wrappers, `Arc` for shared closure state

8. **MSRV** — Rust 1.85 specified with Edition 2024

9. **CI pipeline** — 8 parallel GitHub Actions jobs: fmt, clippy, test (Ubuntu+macOS),
   test-lib-only, MSRV, deny, docs, check-mapping

10. **Atomic file writes** — Correct tempfile + persist pattern preventing corruption

---

## Test Coverage Assessment

The test mapping is **complete and verified**:

| Category | Count |
|----------|-------|
| Mapped (Python test → Rust equivalent) | 202 |
| Excluded (CLI/config/file-resolver/skill — not ported) | 79 |
| Missing | 0 |
| Partial | 0 |
| Extra Rust-only tests | 48 |
| **Total Rust tests** | **250** |

The golden test (`tests/test_ref_docs.rs`) validates all 4 formatting modes (plain,
cleaned, semantic, auto) against a 1,416-line reference document, catching regressions
that individual unit tests might miss.

No test coverage gaps were identified for ported functionality.

---

## P0.5: Strict Lint & Build Configuration

The project should fail the build on all warnings, both locally and in CI.
Currently there is a gap between local and CI strictness.

### Current State

- **Cargo.toml** sets clippy pedantic to `warn` (not `deny`), so `cargo clippy` alone
  succeeds even with lint violations
- **No `warnings` deny** — compiler warnings (unused imports, dead code, etc.) don't
  fail the build locally
- **CI is stricter than local** — `ci.yml` line 32 passes `-D warnings` to clippy,
  but a developer running `cargo clippy` locally sees warnings, not errors. This
  disconnect means issues accumulate between CI runs.
- **Test compilation not strict** — CI doesn't set `RUSTFLAGS="-D warnings"`, so test
  code can have compiler warnings without failing CI

### Recommended Cargo.toml `[lints]` Section

Replace the current `[lints]` section with:

```toml
[lints.rust]
unsafe_code = "deny"
# Fail build on all compiler warnings locally, not just in CI
warnings = "deny"

[lints.clippy]
# Pedantic as deny, not warn — catch issues locally, not just in CI
pedantic = { level = "deny", priority = -1 }

# Intentional selective exceptions (with justification):
missing_errors_doc = "allow"       # Not writing exhaustive public docs yet
missing_panics_doc = "allow"       # Not writing exhaustive public docs yet
module_name_repetitions = "allow"  # e.g., tag_handling::TagPattern is acceptable
must_use_candidate = "allow"       # Too noisy for this codebase
too_many_lines = "allow"           # filling.rs has long functions by design
```

### Additional Strict Lints Worth Enabling

These catch real bugs and are standard in production Rust projects:

```toml
[lints.rust]
unsafe_code = "deny"
warnings = "deny"
# Visibility hygiene:
unreachable_pub = "warn"                # Catch pub items only reachable internally

[lints.clippy]
# ... (pedantic as deny above, plus):
unwrap_used = "deny"                    # Already avoided — now enforce it
clone_on_ref_ptr = "warn"              # Catch accidental Arc/Rc clones
redundant_closure_for_method_calls = "warn"
```

Note: `unwrap_used = "deny"` will require `#[allow(clippy::unwrap_used)]` on test
modules (where unwrap/expect is acceptable), but enforces the existing zero-unwrap
discipline in library code.

### CI Improvements

**1. Add `RUSTFLAGS` for strict test compilation:**

```yaml
  test:
    # ...
    steps:
      # ...
      - run: cargo test --locked --all-features
        env:
          RUSTFLAGS: "-D warnings"
```

**2. Make mapping check non-informational:**

Line 97 of `ci.yml`: `uv run flowmark-dev check-mapping || true` swallows failures
with `|| true`. If mapping completeness should be enforced, remove the `|| true`.

**3. Consider adding `cargo-audit` or `cargo-vet`** alongside the existing `cargo-deny`
for defense-in-depth on supply chain security.

---

## Recommended Fix Order

1. `cargo fmt` (trivial, fixes all formatting)
2. Tighten `[lints]` in Cargo.toml: `warnings = "deny"`, clippy pedantic to `deny`
3. Remove dead dependencies from Cargo.toml (toml, serde, unicode-segmentation)
4. Fix 9 clippy `inefficient_to_string` errors (mechanical: `line.to_string()` → `(*line).to_string()`)
5. Remove dead error variants `Error::Config` and `Error::Other`
6. Add `RUSTFLAGS="-D warnings"` to CI test jobs
7. Extract fence-tracking helper to eliminate 3x duplication in filling.rs
8. Fix `nc.to_string()` allocation in ellipses.rs
9. Remove `_name` field from AtomicPattern (or justify with `#[allow(dead_code)]`)
10. Remove unnecessary `info.clone()` in code block rendering
11. Consider `FormatOptions` struct for public API (can be deferred to pre-1.0)

Items 1-6 are mechanical and low-risk — do these first and verify all 250 tests still
pass. Items 7-10 are refactors that should each be a separate commit with a full test
run. Item 11 is a design decision for pre-1.0.

**Validation after each change:** `cargo fmt --check && cargo clippy -- -D warnings && cargo test`
