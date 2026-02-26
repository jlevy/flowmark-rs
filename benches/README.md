# Performance Benchmarks and Profiling

## Benchmark Results (2026-02-26)

**Environment:** Linux 4.4.0 (x86_64), Rust 1.85 (release profile), Python 3.11

**Corpus:** 1,040 Markdown files (26 unique files x 40 batches), 5.72 MB total.
Files range from 86 bytes to 77 KB and exercise all Markdown features (tables, footnotes,
code blocks, frontmatter, math, lists, blockquotes, headings, inline formatting, HTML, etc.)

### End-to-End Timing (5 iterations, median)

| Mode    | Rust (median) | Python (median) | Speedup | Rust throughput        |
| ------- | ------------- | --------------- | ------- | ---------------------- |
| default | 0.052s        | 0.519s          | **10x** | 20,000 files/s, 110 MB/s |
| --auto  | 0.057s        | 0.477s          | **8.4x**| 18,246 files/s, 100 MB/s |

The Rust version is approximately **8-10x faster** than Python for end-to-end formatting
including file discovery and I/O.

## Profiling Analysis

Profiled using `valgrind --tool=callgrind` on the 1,040-file corpus (default mode).

**Total instructions:** 9.84 billion

### High-Level Breakdown

| Category                        | Instructions   | % of total |
| ------------------------------- | -------------- | ---------- |
| Memory allocation (malloc/free) | 1,967M         | 20.0%      |
| String search (TwoWaySearcher)  | 1,358M         | 13.8%      |
| Memory copy/compare             | 652M           | 6.6%       |
| Regex engine                    | 421M           | 4.3%       |
| String replace                  | 364M           | 3.7%       |
| Comrak parser                   | 359M           | 3.6%       |
| flowmark formatting             | 267M           | 2.7%       |
| flowmark period escapes         | 184M           | 1.9%       |
| flowmark text wrapping          | 126M           | 1.3%       |
| String trim                     | 69M            | 0.7%       |
| Vec grow/allocation             | 65M            | 0.7%       |
| Format strings                  | 38M            | 0.4%       |

### Top 3 Hotspots (with `.replace()` call counts)

1. **`replace_escapes_in_line()`** — 25.06% of total (4.37M `.replace()` calls)
   - Located in `src/formatter/filling.rs:750`
   - Called once per line per escape character (32 escape chars x every line)
   - Each call does a full string scan with `TwoWaySearcher`

2. **`restore_atomic_constructs()`** — 13.52% of total (699K `.replace()` calls)
   - Located in `src/wrapping/text_wrapping.rs:56`
   - For each word token, iterates over all extracted atomic constructs
   - Each `.replace()` scans the full token string

3. **Escape placeholder restoration** — 6.72% of total (33K `.replace()` calls)
   - Located in `src/formatter/filling.rs:2201`
   - Post-render loop: 32 `.replace()` calls over the entire output string

**Combined:** These three `.replace()` loops account for ~45% of all instructions.

### Root Cause Analysis

The dominant performance cost comes from **repeated full-string scans via `str::replace()`**.
The pattern is:

```rust
for (needle, replacement) in pairs {
    result = result.replace(needle, replacement);
}
```

Each `str::replace()` call:
1. Creates a new `TwoWaySearcher` (12.6% of total just for initialization)
2. Scans the full string (12.4% for `TwoWaySearcher::next`)
3. Allocates a new `String` for the result

When called in a loop over 32 escape characters, this means 32 full-string scans per line
of input, even when no replacements are needed (which is the common case).

### Optimization Opportunities

1. **Single-pass replacement for escape placeholders** (largest win, ~32% reduction):
   Replace the 32-iteration `.replace()` loop in `replace_escapes_in_line()` and the
   post-render restoration loop with a single-pass character scanner that checks each
   character against the escape set. This eliminates 31 of 32 redundant full-string scans.

2. **Index-based atomic construct restoration** (~14% reduction):
   Instead of doing `token.replace(placeholder, construct)` for every placeholder on every
   token, use the placeholder format (`\x00AC{idx}\x01*\x00`) to find and replace by index
   in a single pass. The placeholder format already embeds the construct index.

3. **Reduce allocations** (~20% reduction potential):
   The profile shows 20% of time in malloc/free. Key areas:
   - `String::replace()` allocates a new String each time
   - `wrap_paragraph_lines()` clones words and joins with intermediate Strings
   - `tag_handling` functions allocate `Vec<String>` for line processing

   Using `Cow<str>` or pre-allocated buffers could help significantly.

4. **Comrak `process_email_autolinks`** (0.88%):
   Comrak's email autolink processing runs on every document. If email autolinks are
   rarely used, this could be disabled via comrak options.

## Scripts

- `generate-bench-corpus.sh` — Generate the benchmark corpus (~1000 markdown files)
- `run-bench.sh` — Run Python vs Rust comparison benchmark
- `profile-rust.sh` — Profile the Rust binary with callgrind

### Usage

```bash
# Generate corpus
./benches/generate-bench-corpus.sh

# Run benchmark
cargo build --release
./benches/run-bench.sh

# Profile (requires debug symbols)
CARGO_PROFILE_RELEASE_STRIP=false CARGO_PROFILE_RELEASE_DEBUG=2 cargo build --release
./benches/profile-rust.sh
```
