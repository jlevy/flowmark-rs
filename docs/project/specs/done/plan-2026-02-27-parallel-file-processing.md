---
title: Parallel File Processing and Per-File Performance
description: Add rayon-based parallel file formatting and per-file optimizations
author: Joshua Levy (github.com/jlevy) with LLM assistance
---
# Feature: Parallel File Processing and Per-File Performance

**Date:** 2026-02-27 (last updated 2026-02-27)

**Author:** Joshua Levy with LLM assistance

**Status:** Implemented

## Overview

Add parallel file processing to flowmark-rs using rayon, and implement per-file
optimizations inspired by dprint’s architecture.
The goal is to bring batch formatting performance from ~2.7s (924 files) down to
~0.3-0.5s, competitive with dprint (0.23s).

## Goals

- Parallelize the file formatting loop in `src/main.rs` using rayon
- Achieve near-linear scaling with CPU core count for batch workloads
- Add a `--threads` CLI flag to control parallelism (default: all cores)
- Skip writing files that are already correctly formatted (unchanged content)
- Maintain identical formatting output (no behavioral changes)
- Pass all existing tests (430+ tests)
- Re-run cross-formatter benchmarks to measure improvement

## Non-Goals

- Parallelizing within a single file (formatting a single file stays single-threaded)
- Incremental caching with persistent hash files (dprint-style) — may be future work
- Async I/O or tokio integration (rayon is simpler and sufficient)
- Plugin architecture (not needed for flowmark)

## Background

### Current State

flowmark-rs v0.2.4 processes files sequentially in a `for` loop (`src/main.rs:379`):

```rust
for file in &resolved_files {
    opts.reformat_file(&path, output_path.as_deref(), args.inplace, args.nobackup)?;
}
```

Each call to `reformat_file` reads one file, formats it, and writes it back.
There is no shared mutable state between files — `FormatOptions` is `Clone`, all regex
patterns are `LazyLock<Regex>` (thread-safe), and `atomic_write` uses per-file
`tempfile::NamedTempFile`.

### Cross-Formatter Benchmark (924 files, 12 MB)

| Formatter | Mean | Relative |
| --- | --- | --- |
| dprint (parallel, Rust) | 0.23 s | 1.0x |
| markdownfmt (Go) | 0.80 s | 3.4x |
| **flowmark-rs (sequential, Rust)** | **2.74 s** | **11.7x** |
| prettier (JS) | 20.89 s | 89.4x |
| flowmark-py (Python) | 27.80 s | 119.0x |
| mdformat (Python) | 37.49 s | 160.4x |

dprint achieves 0.23s wall-clock with 3.3s user CPU time, indicating ~14x parallelism on
the benchmark machine.
flowmark-rs at 2.74s with single-threaded execution suggests that with equivalent
parallelism, it could achieve ~0.2-0.4s.

### dprint Architecture (Source Analysis)

Source analysis of dprint (cloned to `attic/dprint`, key file:
`crates/dprint/src/format.rs`) reveals:

1. **Tokio current_thread runtime** for async orchestration
2. **`spawn_blocking()`** dispatches all work to a multi-threaded blocking pool
3. **Semaphore-controlled concurrency** capped at CPU core count
4. **Adaptive CPU throttling** — reduces parallelism if system is busy
5. **Incremental caching** — hash-based skip for unchanged files
6. **Skip-unchanged optimization** — reads file, checks if formatting would change,
   skips write if identical

For flowmark-rs, rayon provides equivalent parallelism with far less complexity since we
have no plugin infrastructure or async requirements.

## Design

### Approach

Two complementary improvements:

1. **Parallel file processing** — Replace the sequential `for` loop with
   `rayon::par_iter()`. Rayon provides a work-stealing thread pool sized to
   `available_parallelism()` by default, with zero boilerplate.

2. **Skip-unchanged optimization** — After formatting, compare the output to the
   original content. If identical, skip the write entirely.
   This avoids unnecessary disk I/O and preserves file timestamps, which matters for
   build tools that use mtime.

### Components

**Files changed:**

- `Cargo.toml` — Add `rayon` dependency (v1.11, optional behind `cli` feature)
- `src/main.rs` — Parallel formatting loop, `--threads` flag, verbose output with
  thread-safe stderr
- `src/lib.rs` — Skip-unchanged check in `reformat_file`
- `benchmarks/REPORT.md` — Updated benchmark results

**No changes to:**

- `src/formatter/` — Formatting logic is untouched
- `src/wrapping/` — Wrapping logic is untouched
- `src/config.rs` — `FormatOptions` is already `Clone` and stateless

### Thread Safety Analysis

All components are already thread-safe:

- **`FormatOptions`** — `#[derive(Clone)]`, all methods take `&self`, no interior
  mutability
- **Regex patterns** — All `static LazyLock<Regex>` (25+ patterns across 8 files).
  `LazyLock` is `Sync`, and `Regex` is `Send + Sync`.
- **`atomic_write()`** — Uses per-call `tempfile::NamedTempFile::new_in()` with
  directory as scope. No shared state.
- **comrak** — `comrak::parse_document` and `comrak::format_commonmark` are stateless
  functions that take owned arenas.
- **File I/O** — Each file reads/writes independently; no cross-file dependencies.

No `Mutex`, `RefCell`, `static mut`, or `thread_local!` found anywhere in the codebase.

### API Changes

**CLI:**

```
--threads <N>    Number of parallel formatting threads (0 = all cores, default)
```

When `--threads 1` is specified, falls back to sequential processing (useful for
debugging or deterministic output ordering with `--verbose`).

**Library (`FormatOptions`):**

No changes to the library API. Parallelism is a CLI concern only.

### Detailed Code Changes

**`src/main.rs` — Parallel formatting loop:**

```rust
use rayon::prelude::*;

// Filter out stdin (must be handled sequentially)
let (stdin_files, regular_files): (Vec<_>, Vec<_>) =
    resolved_files.iter().partition(|f| *f == "-");

// Handle stdin sequentially (if present)
for file in &stdin_files {
    // ... existing stdin handling ...
}

// Format regular files in parallel
regular_files.par_iter().try_for_each(|file| {
    let path = PathBuf::from(file);
    if args.verbose {
        eprintln!("formatting {}", path.display());
    }
    opts.reformat_file(&path, None, args.inplace, args.nobackup)
        .with_context(|| format!("failed to format {}", path.display()))
})?;
```

**`src/lib.rs` — Skip-unchanged optimization in `reformat_file`:**

```rust
pub fn reformat_file(&self, path: &Path, output: Option<&Path>, inplace: bool, nobackup: bool) -> Result<()> {
    let content = std::fs::read_to_string(path)?;
    let formatted = self.reformat_text(&content);

    // Skip write if content is unchanged (preserves mtime, avoids I/O)
    if inplace && formatted == content {
        return Ok(());
    }

    // ... existing write logic ...
}
```

**`Cargo.toml`:**

```toml
rayon = { version = "1.11", optional = true }

[features]
cli = ["clap", "anyhow", "rayon"]
```

**Thread pool configuration:**

```rust
if let Some(threads) = args.threads {
    if threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .ok();
    }
}
```

### Error Handling

With `par_iter().try_for_each()`, the first error short-circuits execution and is
propagated. This matches the current sequential behavior where the first `?` exits the
loop. Remaining in-flight files finish their current work but no new files start.

### Verbose Output

With parallel execution, verbose output (`--verbose`) may interleave.
This is acceptable since:

- The output is on stderr (informational, not machine-parsed)
- `eprintln!` acquires the stderr lock per call (no torn lines)
- Users who need deterministic ordering can use `--threads 1`

## Implementation Plan

### Phase 1: Parallel Processing and Skip-Unchanged

- [ ] Add `rayon = { version = "1.11", optional = true }` to `Cargo.toml` under `cli`
  feature
- [ ] Add `--threads` CLI argument to `Args` struct (default: 0 = all cores)
- [ ] Configure rayon thread pool based on `--threads` value
- [ ] Partition `resolved_files` into stdin vs regular files
- [ ] Replace sequential `for` loop with `par_iter().try_for_each()` for regular files
- [ ] Add skip-unchanged check in `reformat_file` (compare formatted == content when
  inplace)
- [ ] Run all existing tests (`cargo test`) — must pass with no changes
- [ ] Run the cross-formatter benchmark (`benchmarks/run_comparison.sh`) to measure
  improvement
- [ ] Update `benchmarks/REPORT.md` with new results

## Testing Strategy

- **Correctness:** All 430+ existing tests pass unchanged.
  Formatting output is identical to sequential mode.
- **Thread safety:** Run with `--threads 1` (sequential), `--threads 2`, and default
  (all cores) on the benchmark corpus to verify identical output.
- **Benchmark:** Re-run `benchmarks/run_comparison.sh` with 3 runs each, verify CV% <
  5%.
- **Edge cases:**
  - Single file: `flowmark --auto README.md` (no parallelism needed, should not regress)
  - Stdin: `echo "# test" | flowmark -` (must still work, not parallelized)
  - Mixed: `flowmark --auto . -` (stdin + directory, handled correctly)
  - `--threads 1`: Verify sequential fallback works
  - Empty corpus: No files to format (should exit cleanly)

## Rollout Plan

1. Implement behind the `cli` feature flag (rayon is CLI-only, not in library)
2. Default to all cores (matches dprint, markdownfmt, and user expectations)
3. Library API unchanged — no breaking changes for library consumers
4. Bump version to 0.3.0 (minor version for new feature)

## Open Questions

None — the approach is well-understood from the dprint analysis and the thread-safety
audit shows no blockers.

## References

- `benchmarks/REPORT.md` — Cross-formatter benchmark results and dprint architecture
  analysis
- `docs/project/specs/done/plan-2026-02-26-perf-comparison-profiling.md` — Previous
  profiling spec
- [rayon crate](https://crates.io/crates/rayon) (v1.11, data parallelism library)
- [dprint source](https://github.com/dprint/dprint) — `crates/dprint/src/format.rs`
  (parallel formatting reference)
- `attic/dprint/` — Local clone of dprint source
