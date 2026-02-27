# Flowmark Performance Report: Python vs Rust

**Date:** 2026-02-26

## Benchmark Setup

- **Platform:** Linux 4.4.0, x86_64
- **Python:** flowmark v0.6.4
- **Rust:** flowmark v0.2.4 (release: `opt-level=3`, LTO, `codegen-units=1`, `panic=abort`)
- **Corpus:** 1,080 Markdown files (11 MB) — 90 unique repo `.md` files duplicated 12x in
  a 4–5 level deep directory tree
- **Benchmarking tool:** hyperfine (with warmup, multiple runs)
- **Profiling tool:** valgrind callgrind (instruction-level, single file and batch)

Scripts to reproduce: `benchmarks/generate_corpus.sh`, `benchmarks/run_benchmarks.sh`,
`benchmarks/profile_rust.sh`.

## Headline Results

Rust flowmark is **9–14x faster** than Python flowmark across all workloads.

| Benchmark | Python | Rust | Speedup |
| --- | --- | --- | --- |
| Single file (1,734 lines, stdout) | 471.7 ms | 34.7 ms | **13.6x** |
| Batch `--auto` (1,080 files in-place) | 32.1 s | 3.7 s | **8.8x** |
| Batch `--semantic` (1,080 files in-place) | 27.2 s | 2.5 s | **10.9x** |
| File discovery `--list-files` (1,080 files) | 1.31 s | 169 ms | **7.8x** |

### Per-File Throughput

| Mode | Python | Rust |
| --- | --- | --- |
| `--auto` (batch) | 29.7 ms/file, 33 files/sec | 3.4 ms/file, 294 files/sec |
| `--semantic` (batch) | 25.2 ms/file, 39 files/sec | 2.3 ms/file, 432 files/sec |

### Notes

- Python startup overhead (~300 ms) inflates single-file times; in batch mode this is
  amortized and the per-file speedup drops to 9–11x.
- Semantic mode is slightly faster than auto for both implementations (fewer line-wrap
  iterations).
- File discovery (`--list-files`) shows 7.8x speedup, reflecting Rust `ignore` crate vs
  Python `pathspec`/`os.walk`.

## Profiling: Where Does Rust Spend Its Time?

Profiled with `valgrind --tool=callgrind` on `tests/testdocs/testdoc.orig.md` (1,734
lines).
Total: 155.7M instructions.

### Call Hierarchy (Inclusive Cost)

```
fill_markdown (entry)                           99.4%   (154.7M)
├── render_block (comrak AST → Markdown)        69.3%   (107.9M)
│   └── render_block recursive                  55.9%   ( 87.0M)
│       └── line wrapping pipeline              37.8%   ( 58.8M)
│           └── tag newline handling            37.4%   ( 58.2M)
│               └── line_wrap_to_width          35.6%   ( 55.4M)
│                   └── wrap_paragraph          35.1%   ( 54.6M)
│                       └── wrap_paragraph_lines 34.4%  ( 53.5M)
│                           └── html_md_word_split 27.6% ( 43.0M)
└── pre/post-processing workarounds             30.1%   ( 46.8M)
```

The wrapping pipeline (word splitting → paragraph wrapping → line breaking) is the
dominant cost at ~35% inclusive.
Pre- and post-processing workarounds for comrak account for another ~30%.

### Self-Time Breakdown (Exclusive Cost)

| Category | % | Instructions | What's happening |
| --- | --- | --- | --- |
| **String searching** (`core::str::pattern`) | **~30%** | ~46.7M | `StrSearcher::new` 15.2%, `TwoWaySearcher::next` 10.4% |
| **Memory allocation** (malloc/free/realloc) | ~18.5% | ~28.8M | Allocation churn from string operations |
| **Memory ops** (memcpy/memcmp/memset) | ~6.7% | ~10.4M | Copying strings during replace/concat |
| **Regex** (regex-automata hybrid DFA) | ~5.5% | ~8.6M | Sentence detection, atomic construct extraction |
| **flowmark functions** (direct self-time) | ~4.5% | ~7.0M | `fill_markdown` 0.6%, `remove_period_escapes` 0.5% |
| **str::replace** (alloc + search) | ~2.8% | ~4.4M | Each `.replace()` allocates a new String |
| **Comrak parser** | ~2.4% | ~3.7M | `parse_inline`, `process_line`, `open_new_blocks` |

### Key Finding

**String pattern searching is the #1 bottleneck at ~30% of total instructions.** This is
not from the comrak parser or regex — it's from Rust's `str::replace()`, `str::contains()`,
and related methods that use `core::str::pattern::StrSearcher` (Two-Way string search
algorithm).

## Root Causes

### 1. O(N×M) Placeholder Restoration in `restore_atomic_constructs`

**File:** `src/wrapping/text_wrapping.rs:56–71`

```rust
fn restore_atomic_constructs(tokens: &[String], constructs: &[String], placeholders: &[String]) -> Vec<String> {
    tokens.iter().map(|token| {
        let mut result = token.clone();
        for (placeholder, construct) in placeholders.iter().zip(constructs.iter()) {
            result = result.replace(placeholder.as_str(), construct);  // N×M string scans
        }
        result
    }).collect()
}
```

For each token, this scans the full string M times (once per placeholder). Each
`.replace()` call invokes `StrSearcher::new` (builds a Two-Way searcher) and
`TwoWaySearcher::next` (scans the string). With many tokens and many placeholders, this is
expensive.

### 2. 32× Sequential `.replace()` for Escape Placeholders

**File:** `src/formatter/filling.rs:2200–2203`

```rust
for (escaped, placeholder) in &escape_placeholders {
    result = result.replace(placeholder.as_str(), escaped.as_str());
}
```

This runs 32 `.replace()` calls over the entire document (one per escapable ASCII
punctuation character). Each call scans the full document and allocates a new `String`.
The same pattern appears in the pre-processing direction at lines 750–755.

### 3. Per-Line Character Scanning in `remove_period_escapes_preserving_code`

**File:** `src/formatter/filling.rs:807–850`

Called on every non-fenced line. Character-by-character processing with
`String::with_capacity` + push. Not algorithmically bad, but the sheer call volume makes
it visible at 0.5% self-time.

## Optimization Opportunities

| # | Optimization | Estimated Impact | Complexity |
| --- | --- | --- | --- |
| 1 | Single-pass `restore_atomic_constructs`: scan each token once for `\x00AC` prefix instead of M `.replace()` calls | 10–15% | Low |
| 2 | Single-pass PUA escape restoration: scan document once for PUA chars in `\u{E000}..=\u{E07E}` instead of 32 `.replace()` calls | 5–10% | Low |
| 3 | Buffer reuse / `Cow<str>` in wrapping pipeline to reduce allocation churn | 3–5% | Medium |
| 4 | Pre-built regex `Cache` for hybrid DFA | 1–2% | Low |

## Optimization Experiments

Two optimizations were implemented and tested.
All 430 tests pass after each change.

### Optimization 1: Single-pass `restore_atomic_constructs`

**Change:** Replace the O(N×M) `.replace()` loop in `restore_atomic_constructs`
(`src/wrapping/text_wrapping.rs`) with a fast-path check: if the token doesn't contain
the placeholder prefix byte (`\x00`), skip entirely.
If the entire token is a placeholder (common case), do a HashMap lookup instead of
M sequential `.replace()` calls.

**Result (alone):** Within measurement noise — no significant improvement on test
document.
This makes sense: the testdoc has relatively few atomic constructs (HTML tags,
code spans), so the placeholder restoration isn't the dominant contributor.
The optimization would show more benefit on documents heavy with inline HTML/code.

### Optimization 2: Single-pass PUA Escape Processing

**Change:** Replace two sets of 32× sequential `.replace()` calls:

- **Pre-processing** (`replace_escapes_in_line`): Instead of calling
  `.replace(escaped, placeholder)` for each of 32 escape chars, scan the line once
  for `\` and check if the next char is in the escape set.
- **Post-processing** (`restore_pua_escape_placeholders`): Instead of 32×
  `.replace(placeholder, escaped)` over the full document, scan once for any char in
  the PUA range `\u{E000}..=\u{E0FF}` followed by filler `\u{E100}` and emit the
  original `\<char>`.

Both directions now process the text in a single pass with O(N) time per call instead
of O(32×N).

### Combined Results (Optimizations 1+2)

Benchmarked with `hyperfine` (warmup + 10 runs for single file, 5 for batch).

**Single file (`testdoc.orig.md`, 1,734 lines):**

| | Mean | Range |
| --- | --- | --- |
| Before | 31.5 ms +/- 2.2 ms | 28.4 – 39.6 ms |
| After | 27.3 ms +/- 2.5 ms | 24.2 – 34.9 ms |
| **Improvement** | **13.3% faster** | |

Verified across 3 independent runs: 27.0, 27.2, 27.4, 27.8 ms (consistent).

**Batch `--auto` (1,080 files):**

| | Mean | Range |
| --- | --- | --- |
| Before | 3.21 s +/- 0.11 s | 3.09 – 3.34 s |
| After | 2.69 s +/- 0.15 s | 2.58 – 3.02 s |
| **Improvement** | **16.2% faster** | |

Verified across 3 independent runs: 2.71, 2.73, 2.63 s (consistent).

### Profiling After Optimization

Re-profiled with callgrind after optimizations:

| Metric | Before | After | Change |
| --- | --- | --- | --- |
| **Total instructions** | 155.7M | 89.0M | **-42.8%** |
| String searching (`str::pattern`) | ~30% (46.7M) | ~7.4% (6.6M) | **-85.9%** |
| Memory allocation (malloc/free) | ~18.5% (28.8M) | ~19% (16.9M) | -41.3% |
| Regex (regex-automata) | ~5.5% (8.6M) | ~5.8% (5.2M) | -39.5% |
| Comrak parser | ~2.4% (3.7M) | ~2.6% (2.3M) | -37.8% |

The string searching cost dropped from the dominant bottleneck (30%) to a minor
contributor (7.4%).
All other categories decreased in absolute terms by ~40%, reflecting the removal of
the unnecessary work that string-search-heavy replace loops were causing.

### What's Left After Optimization

Post-optimization, the remaining cost is spread across:

1. **Memory allocation** (~19%) — inherent to string manipulation; would require
   `Cow<str>` or arena allocation (medium complexity)
2. **String searching** (~7%) — remaining uses are necessary `.contains()` and
   `.find()` calls
3. **Regex** (~6%) — already well-optimized with `LazyLock`; hybrid DFA is the
   regex crate's efficient path
4. **Comrak parser** (~3%) — external dependency, not directly optimizable
5. **memcpy/memset** (~7%) — inherent to string operations

Further optimization would yield diminishing returns for increasing complexity.

### Optimization 3: Allocation Reduction

**Status:** Not implemented — the profiling after optimizations 1+2 shows that
allocation cost dropped 41% in absolute terms (from 28.8M to 16.9M instructions)
as a side effect of eliminating the string-replace churn.
The remaining allocations are spread across many small sites in the wrapping
pipeline, and reducing them would require introducing `Cow<str>` throughout the
call chain — medium complexity for an estimated 3-5% further improvement.

## Updated Headline Numbers (With Optimizations)

After applying optimizations 1+2:

| Benchmark | Python | Rust (before) | Rust (after) | Speedup vs Python |
| --- | --- | --- | --- | --- |
| Single file (1,734 lines) | 471.7 ms | 31.5 ms | 27.3 ms | **17.3x** |
| Batch `--auto` (1,080 files) | 32.1 s | 3.21 s | 2.69 s | **11.9x** |

Per-file throughput after optimization: **401 files/sec** in `--auto` mode (was 294).
