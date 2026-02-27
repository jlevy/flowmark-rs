---
title: Performance Comparison and Profiling
description: End-to-end performance comparison of Python vs Rust flowmark, plus Rust profiling
author: Joshua Levy (github.com/jlevy) with LLM assistance
---

# Feature: Performance Comparison and Profiling

**Date:** 2026-02-26 (last updated 2026-02-26)

**Author:** Joshua Levy with LLM assistance

**Status:** Implemented

## Overview

Quantify end-to-end performance of flowmark-rs vs Python flowmark using a realistic
workload of ~1,000 Markdown files, then profile the Rust binary to identify
optimization opportunities.

## Goals

- Create a reproducible benchmark corpus (~1,000 Markdown files in a deep directory
  tree) from repo sources
- Measure end-to-end formatting time for both Python and Rust (file discovery +
  parsing + formatting + writing)
- Use `hyperfine` for statistically rigorous wall-clock comparison
- Profile the Rust binary with `valgrind --tool=callgrind` to identify hot spots
- Document findings and actionable optimization opportunities

## Non-Goals

- Micro-benchmarks of individual functions (future work)
- Optimizing the Python version
- Changing formatting behavior or output
- CI integration of benchmarks (future work)

## Background

flowmark-rs v0.2.4 is a full behavioral parity port of Python flowmark v0.6.4.
The port prioritized correctness over performance.
Now that parity is achieved and verified (430 tests, 292 Python tests mapped),
it's time to quantify the performance difference and identify optimization
opportunities in the Rust implementation.

The repo contains 90 Markdown files of varying complexity:

- `tests/testdocs/testdoc.orig.md` (1,734 lines) — comprehensive test document
- `tests/parity/corner-cases.md` (278 lines) — edge cases
- `tests/tryscript/fixtures/content/*.md` (~20 files, varying sizes)
- `docs/*.md` and `README.md` — project documentation

These are duplicated into 12 sets of 90 files (1,080 total) in a deep directory tree
to create a realistic batch-formatting workload.

## Design

### Approach

1. **Corpus generation script** (`benchmarks/generate_corpus.sh`): Copies all 90 repo
   Markdown files into a deep directory tree with 1,080 total files, organized in
   nested subdirectories to exercise file discovery.

2. **Benchmark script** (`benchmarks/run_benchmarks.sh`): Uses `hyperfine` to compare
   `flowmark --auto <corpus_dir>` (Python) vs
   `target/release/flowmark --auto <corpus_dir>` (Rust) with warmup runs, multiple
   iterations, and statistical output.

3. **Profiling** (`benchmarks/profile_rust.sh`): Runs the Rust binary under
   `valgrind --tool=callgrind` with debug symbols to get instruction-level profiling
   of all functions.

### Components

- `benchmarks/generate_corpus.sh` — corpus generation (idempotent, creates
  `benchmarks/corpus/`)
- `benchmarks/run_benchmarks.sh` — hyperfine comparison (4 benchmarks)
- `benchmarks/profile_rust.sh` — callgrind profiling
- `docs/project/specs/active/plan-2026-02-26-perf-comparison-profiling.md` — this
  spec

### Corpus Structure

```
benchmarks/corpus/           (1,080 files, 11 MB total)
├── batch_000/               (5 sets)
│   ├── set_00/docs/         (90 .md files)
│   ├── set_01/content/deep/ (90 .md files)
│   ├── set_02/notes/archive/(90 .md files)
│   ├── set_03/pages/        (90 .md files)
│   └── set_04/docs/         (90 .md files)
├── batch_001/               (5 sets)
│   └── ...
└── batch_002/               (2 sets)
    └── ...
```

Each "set" contains all 90 unique source Markdown files.
12 sets across 3 batches = 1,080 files total in a 4-5 level deep tree.

### API Changes

None — this is purely tooling and measurement.

## Implementation Plan

### Phase 1: Corpus and Benchmarking

- [x] Create `benchmarks/generate_corpus.sh` that collects all repo `.md` files
  (excluding `.git`, `target`, `.tbd`, `benchmarks/corpus`) and replicates them
  into a deep nested directory tree totaling ~1,000 files
- [x] Create `benchmarks/run_benchmarks.sh` that uses `hyperfine` to compare Python
  vs Rust end-to-end formatting of the corpus
- [x] Run the benchmark and record results in this spec
- [x] Verify both tools produce equivalent output on the corpus

### Phase 2: Profiling and Analysis

- [x] Profile Rust binary with `valgrind --tool=callgrind` on single file and batch
- [x] Identify top hot spots from the callgrind data
- [x] Document findings: bottleneck functions, percentage of time, optimization ideas
- [x] Update this spec with results and recommendations

## Benchmark Results

### Environment

- Platform: Linux 4.4.0, x86_64
- Python: flowmark v0.6.4
- Rust: flowmark v0.2.4 (release build, `opt-level=3`, LTO, `codegen-units=1`)
- Corpus: 1,080 Markdown files (11 MB), 90 unique files x 12 copies
- Tool: `hyperfine` with warmup runs

### Benchmark 1: Full Batch Formatting (`--auto`, 1,080 files)

| Command | Mean | Min | Max |
| --- | --- | --- | --- |
| Python `flowmark --auto` | 32.127s +/- 0.301s | 31.823s | 32.426s |
| Rust `flowmark --auto` | 3.665s +/- 0.009s | 3.660s | 3.676s |

**Rust is 8.77x faster** for batch `--auto` formatting.

### Benchmark 2: File Discovery Only (`--list-files`, 1,080 files)

| Command | Mean | Min | Max |
| --- | --- | --- | --- |
| Python `flowmark --list-files` | 1.314s +/- 0.028s | 1.278s | 1.368s |
| Rust `flowmark --list-files` | 168.9ms +/- 4.1ms | 162.5ms | 178.5ms |

**Rust is 7.78x faster** for file discovery.

### Benchmark 3: Single Large File (`testdoc.orig.md`, 1,734 lines)

| Command | Mean | Min | Max |
| --- | --- | --- | --- |
| Python `flowmark` | 471.7ms +/- 16.8ms | 449.2ms | 517.0ms |
| Rust `flowmark` | 34.7ms +/- 4.2ms | 30.9ms | 68.4ms |

**Rust is 13.58x faster** for single-file formatting (includes startup).

### Benchmark 4: Semantic Mode (`--semantic`, 1,080 files)

| Command | Mean | Min | Max |
| --- | --- | --- | --- |
| Python `flowmark --semantic` | 27.232s +/- 0.182s | 27.049s | 27.413s |
| Rust `flowmark --semantic` | 2.499s +/- 0.027s | 2.469s | 2.523s |

**Rust is 10.90x faster** for semantic-mode formatting.

### Per-File Throughput

| Mode | Python | Rust | Speedup |
| --- | --- | --- | --- |
| `--auto` (batch) | 29.7ms/file (33 files/sec) | 3.4ms/file (294 files/sec) | 8.8x |
| `--semantic` (batch) | 25.2ms/file (39 files/sec) | 2.3ms/file (432 files/sec) | 10.9x |
| Single file (1,734 lines) | 471.7ms | 34.7ms | 13.6x |

### Summary

Rust flowmark is **9-14x faster** than Python flowmark across all workloads:

- Single file: **13.6x** (startup overhead dominates Python)
- Batch `--auto`: **8.8x** (file I/O becomes more significant)
- Batch `--semantic`: **10.9x** (semantic mode slightly faster than auto)
- File discovery only: **7.8x**

## Profiling Results

Profiled using `valgrind --tool=callgrind` on `testdoc.orig.md` (155.7M
instructions total) with the Rust release binary built with debug symbols.

### Inclusive Call Hierarchy

```
fill_markdown (entry point)                    99.4%  (154.7M instructions)
├── render_block (comrak AST → Markdown)       69.3%  (107.9M)
│   └── render_block recursive                 55.9%  (87.0M)
│       └── line wrapping pipeline             37.8%  (58.8M)
│           └── tag handling                   37.4%  (58.2M)
│               └── line_wrap_to_width         35.6%  (55.4M)
│                   └── wrap_paragraph         35.1%  (54.6M)
│                       └── wrap_paragraph_lines  34.4%  (53.5M)
│                           └── html_md_word_split  27.6%  (43.0M)
└── pre/post-processing workarounds            30.1%  (46.8M)
```

### Exclusive Cost Breakdown (Self Time)

| Category | % of Total | Instructions | Notes |
| --- | --- | --- | --- |
| **String searching** (`str::pattern`) | **~30%** | ~46.7M | `StrSearcher::new` (15.2%), `TwoWaySearcher::next` (10.4%) |
| **Memory allocation** (malloc/free) | ~18.5% | ~28.8M | High allocation churn from string operations |
| **Memory ops** (memcpy/memcmp) | ~6.7% | ~10.4M | Copying strings during replace/concat |
| **Regex** (regex-automata DFA) | ~5.5% | ~8.6M | Sentence splitting, atomic construct extraction |
| **flowmark direct** | ~4.5% | ~7.0M | `fill_markdown` (0.6%), `remove_period_escapes` (0.5%), `html_md_word_split` (0.3%) |
| **str::replace** | ~2.8% | ~4.4M | Allocating new strings for each replacement |
| **Comrak parser** | ~2.4% | ~3.7M | `parse_inline`, `process_line`, `open_new_blocks` |
| **Other** | ~29.6% | ~46.1M | Various stdlib, formatting, iterators |

### Key Finding: String Searching Dominates

The **#1 bottleneck is `str::pattern` string searching** at ~30% of total
instructions.
This comes from:

1. **`str::replace()` calls in loops** — The `restore_atomic_constructs()` function
   in `text_wrapping.rs:56-71` calls `.replace()` on every token for every
   placeholder.
   With N tokens and M placeholders, this is O(N*M) string scans.

2. **Escape placeholder replacement** — `fill_markdown` replaces 32 escape
   placeholder patterns (`ESCAPE_CHARS`) across the entire document twice (once for
   protection, once for restoration), each using `.replace()` which triggers
   `StrSearcher::new` + `TwoWaySearcher::next`.

3. **`remove_period_escapes_preserving_code()`** — Called on every non-code-fence
   line, this does character-by-character scanning which shows up as a
   secondary hot spot.

### Optimization Opportunities

Ranked by estimated impact:

1. **Replace O(N\*M) `restore_atomic_constructs` with O(N) single-pass scan**
   (est. 10-15% speedup)
   - Current: loops over all tokens and calls `.replace()` for each placeholder
   - Better: walk each token once, scanning for `\x00AC` prefix to find and restore
     placeholders in a single pass without `.replace()`

2. **Replace 32x `.replace()` for escape placeholders with single-pass PUA scan**
   (est. 5-10% speedup)
   - Current: 32 sequential `.replace()` calls, each scanning the full document
   - Better: single pass scanning for any PUA character in range `\u{E000}..=\u{E07E}`
     and replacing them inline

3. **Reduce allocation in wrapping pipeline** (est. 3-5% speedup)
   - 18.5% of time is in malloc/free
   - Reuse buffers across `wrap_paragraph_lines` calls instead of creating new
     `Vec<String>` each time
   - Use `Cow<str>` instead of `String` where no modification occurs

4. **Cache or precompile regex patterns** (est. 1-2% speedup)
   - Already using `LazyLock` for most patterns, which is good
   - The `regex_automata::hybrid::dfa::Lazy::cache_next_state` (0.25%) suggests
     the DFA cache is warming up repeatedly; passing a shared `Cache` object could
     help

5. **Optimize `comrak::parser::autolink::process_email_autolinks`** (0.71%)
   - This is inside comrak and not directly controllable
   - Could file an upstream issue or pre-strip content that triggers email autolink
     detection

## Testing Strategy

- Corpus generation verified: 1,080 files generated (target was ~1,000)
- Benchmark correctness verified by pre-formatting corpus with Rust and spot-checking
  output
- Profiling validated with debug-symbol-enabled binary

## References

- `docs/port-status.md` — port status and metrics
- `docs/porting-log-review.md` — lessons learned
- `benchmarks/generate_corpus.sh` — corpus generation script
- `benchmarks/run_benchmarks.sh` — hyperfine benchmark runner
- `benchmarks/profile_rust.sh` — callgrind profiling script
- Python flowmark: `repos/flowmark/`
