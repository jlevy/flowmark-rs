---
title: Incremental Cache and Performance Roadmap
description: Add persistent incremental formatting cache and deeper performance instrumentation to close remaining gap to dprint
author: Joshua Levy (github.com/jlevy) with Codex assistance
---
# Feature: Incremental Cache and Performance Roadmap

**Date:** 2026-02-27 (last updated 2026-02-27)

**Author:** Joshua Levy with Codex assistance

**Status:** In Review

## Overview

Implement a persistent incremental cache for `flowmark-rs` and add stage-level
performance instrumentation to identify and remove remaining bottlenecks. The
goal is to match or beat dprint on fresh runs and decisively beat non-cached
reruns when files are unchanged.

## Goals

- Add persistent incremental cache for unchanged-file fast paths
- Add cache lifecycle UX commands for visibility and cleanup
- Preserve correctness by invalidating cache on options/version/behavior changes
- Add low-overhead timing instrumentation to isolate remaining hot stages
- Produce reproducible benchmark evidence against dprint and prior flowmark baselines
- Keep CLI/library behavior backward compatible by default

## Non-Goals

- Rewriting markdown formatting away from comrak in this iteration
- Introducing a plugin runtime in flowmark-rs
- Changing formatting semantics to chase benchmark numbers
- Making cache behavior part of library API in this iteration

## Background

### Current Snapshot (Single Corpus)

From `benchmarks/REPORT.md`, same 928-file corpus for all tools:

- `dprint --incremental=false`: ~0.37s (fresh)
- `flowmark-rs --auto` (parallel): ~0.73s (fresh)
- `flowmark-rs --auto --threads 1`: ~2.42s (fresh)

Interpretation:

- Concurrency gave flowmark-rs a major speedup and moved it to #2 overall.
- Remaining gap vs dprint on fresh runs is mostly per-file CPU cost.
- Biggest practical gap is unchanged reruns, where dprint skips work via cache.

### Why Incremental Cache Matters

Incremental cache does not depend on duplicate files. It helps when the same
files are seen again and content is unchanged. That is why dprint can move from
sub-second fresh runs to near-instant reruns.

### Industry Pattern (Where Caches Live)

Common formatter/linter behavior:

- Prettier: optional cache (`--cache`), typically under project-local cache path
  (for example `node_modules/.cache/prettier` unless overridden).
- ESLint: optional cache (`--cache`), default cache file in project (`.eslintcache`
  or configured cache location).
- Black: on by default, stored under user cache directory (for example
  `~/.cache/black/...` on Unix-like systems).
- dprint: cache directory under user cache root, with per-project incremental
  data keyed from base path and plugin/config fingerprint.

Takeaway: persisted hash metadata is now standard behavior for fast reruns.

### dprint Performance Characteristics We Can Borrow

From `attic/dprint/crates/dprint/src/incremental/*` and
`attic/dprint/crates/dprint/src/format.rs`:

- Hash-based skip before full format pass (`is_file_known_formatted`)
- Plugin/config fingerprint invalidation (`plugins_hash`)
- In-memory write set and single flush at end of command (`write()` once)
- Atomic file writes for cache durability
- Blocking work off async control plane (in flowmark, rayon already provides this)

Native/process plugin support in dprint is not the main advantage here; the
incremental skip path is.

## First-Principles Bottleneck Model

For flowmark-rs today (no incremental cache):

`total ~= file_discovery + per_file(read + decode + preprocess + parse + ast_transforms + render + postprocess + compare + maybe_write)`

Likely dominant costs:

- `parse + render` in comrak plus AST walk
- preprocess/postprocess string allocations and regex passes in
  `src/formatter/filling.rs`
- unavoidable per-file read/decode on reruns when cache miss logic is absent

Because discovery is already fast and concurrency is implemented, the next
high-leverage win is to skip parse/render entirely on unchanged files.

## Design

### High-Level Approach

1. Read file bytes once
2. Compute content hash and formatter fingerprint
3. Cache hit -> skip formatting/write
4. Cache miss -> format normally, then store post-format content hash
5. Flush cache manifest atomically at end of run

### File and Function-Level Implementation Map

1. `src/incremental_cache.rs` (new)
- `IncrementalCache::open(cache_dir, project_root, fingerprint) -> Result<Self>`
- `IncrementalCache::is_known_formatted(path: &Path, input_bytes: &[u8]) -> bool`
- `IncrementalCache::record_formatted(path: &Path, formatted_bytes: &[u8])`
- `IncrementalCache::flush() -> Result<()>`
- `compute_formatter_fingerprint(opts: &FormatOptions, binary_version: &str, config_path: Option<&Path>) -> u64`
- `load_manifest` / `save_manifest_atomic`

2. `src/main.rs`
- `Args`: add cache controls with explicit UX:
  `--incremental`, `--no-cache`, `--cache-dir`, `--show-cache`,
  `--clear-cache`, `--perf-stats`
- `run`: initialize cache once for command execution
- `run`: replace `opts.reformat_file(...)` in file loop with cache-aware path:
  read -> cache check -> format -> write -> cache record
- `run`: aggregate and print perf/cache summary when `--perf-stats`
- `run`: support cache lifecycle operations:
  - `--show-cache`: print resolved cache root, total cache file count, and
    human-readable total cache size
  - `--clear-cache`: delete resolved cache root recursively and report result
  - both commands respect `--cache-dir` override

3. `src/lib.rs`
- Add a non-I/O formatting helper to avoid duplicated read/write logic in CLI path:
  `FormatOptions::format_str(&self, text: &str) -> String` (or equivalent wrapper)
- Keep public API compatibility; current `reformat_file` remains behaviorally stable

4. `src/config.rs`
- `FlowmarkConfig`: add optional incremental controls
- `VALID_FIELDS` and `set_config_field`: parse both canonical and friendly keys:
  `incremental`, `cache`, `cache-dir`, `incremental-cache-dir`
- `merge_cli_with_config`: merge incremental settings with explicit CLI precedence

5. `src/settings.rs` (new)
- Centralize cache path constants:
  `FALLBACK_CACHE_DIR`, `APP_CACHE_DIR`, `INCREMENTAL_CACHE_SUBDIR`
- `default_cache_root()` for consistent path resolution

6. `src/formatter/filling.rs`
- Introduce timing hooks around:
  - preprocess block
  - comrak parse
  - AST transforms
  - render
  - postprocess
- Keep `fill_markdown(...) -> String` stable
- Add internal timed variant used only when perf stats enabled

7. Tests
- `tests/test_config.rs`: new config merge coverage for incremental flags
- `tests/test_cli_file_discovery.rs` or new `tests/test_incremental_cache.rs`:
  unchanged file skip, changed file miss, disabled incremental behavior
- `tests/tryscript/help.tryscript.md`: help text snapshots for new flags
  (including `--no-cache`, `--cache-dir`)
- corruption reset / invalid fingerprint unit tests for cache manifest

8. Benchmark/Docs
- `benchmarks/run_comparison.sh`: add explicit first-run and second-run modes
- `benchmarks/REPORT.md`: add cached rerun section after implementation
- `README.md`: concise first-run + second-run summary using same corpus
- `docs/cache.md`: full cache settings and location documentation

### Cache Key and Invalidation

Cache validity requires all to match:

- file content hash
- formatter fingerprint (version + relevant CLI/config options)
- project identity (root path hash) to avoid cross-project collisions

Conservative invalidation is preferred initially.

## Implementation Beads (Mapped)

Epic:

- `fmr-leqz` - Spec: Incremental cache and performance roadmap

Child beads with file/function scope and blockers:

- `fmr-qb08` - Cache core: add incremental manifest, fingerprint, and atomic persistence
  - Files/functions: `src/incremental_cache.rs` (`IncrementalCache::open`,
    `is_known_formatted`, `record_formatted`, `flush`, fingerprint/hash helpers)
  - Blocked by: none
- `fmr-ynyg` - CLI/config wiring: incremental flags and merge precedence
  - Files/functions: `src/main.rs` (`Args`, `run` flag handling, `--show-cache`,
    `--clear-cache` command path), `src/config.rs`
    (`FlowmarkConfig`, `set_config_field`, `merge_cli_with_config`),
    `src/settings.rs` (`default_cache_root`, cache path constants)
  - Blocked by: `fmr-qb08`
- `fmr-m4z9` - Integrate cache-aware file processing path in formatter loop
  - Files/functions: `src/main.rs` (`run` per-file pipeline), `src/lib.rs`
    (minimal non-I/O helper extraction if needed)
  - Blocked by: `fmr-qb08`, `fmr-ynyg`
- `fmr-8tpy` - Stage-level perf instrumentation
  - Files/functions: `src/formatter/filling.rs` (timed pipeline sections),
    `src/main.rs` (`--perf-stats` aggregation/output)
  - Blocked by: `fmr-ynyg`
- `fmr-2z00` - Validation: cache correctness, invalidation, and CLI coverage
  - Files/functions: `tests/test_config.rs`, new cache integration/unit tests,
    `tests/tryscript/help.tryscript.md`, cache lifecycle command coverage
  - Blocked by: `fmr-m4z9`, `fmr-8tpy`
- `fmr-ysne` - Hotspot follow-up: optimize dominant `fill_markdown` stages
  - Files/functions: targeted optimizations in `src/formatter/filling.rs`
    based on measured stage costs
  - Blocked by: `fmr-8tpy`
- `fmr-unp8` - Benchmark + docs: first-run and second-run performance reporting
  - Files/functions: `benchmarks/run_comparison.sh`, `benchmarks/REPORT.md`,
    `README.md`
  - Blocked by: `fmr-2z00`, `fmr-ysne`

### Current Implementation Status (2026-02-27)

- `fmr-qb08`: complete
- `fmr-ynyg`: complete
- `fmr-m4z9`: complete
- `fmr-8tpy`: complete
- `fmr-2z00`: complete
- `fmr-ysne`: complete
- `fmr-unp8`: complete

## Measurement Plan and Decision Gates

### Core Metrics

- Cache hit ratio (`hits / total`) on rerun
- Fresh-run wall time (cache enabled should not regress materially)
- Rerun wall time (target substantial win vs current)
- Stage timing breakdown in `fill_markdown`

### Gates Before Merge

- Correctness: no parity regressions in existing suite
- Stability: cache corruption cannot crash command
- Fresh-run guardrail: <= 5% regression allowed (or require explicit signoff)
- Rerun improvement: meaningful speedup on unchanged corpus

## Testing Strategy

- Unit tests:
  - cache read/write/flush behavior
  - fingerprint invalidation on version/option change
  - corruption handling fallback
- Integration tests:
  - unchanged files skipped
  - changed files reformatted
  - `--no-cache` forces full path
  - `--show-cache` reports path, file count, and size
  - `--clear-cache` removes cache root and reports cleanly when cache is absent
- Benchmark validation:
  - fresh run (cold cache)
  - second run (warm cache)
  - thread scaling remains intact

## Rollout Plan

1. Land cache core + CLI/config gates with tests
2. Land cache-aware format pipeline and perf stats
3. Publish first-run and second-run benchmark results
4. Tune based on stage-level metrics

## Open Questions

- Manifest structure for very large repos: single file vs sharded files
- Hash choice tradeoff: speed vs collision margin
- Whether to include file path in key (for safety) or only content hash (for max dedupe)
- Whether `--perf-stats` should support machine-readable JSON output
- Whether `--clear-cache` should require an explicit confirmation flag in non-interactive contexts

## References

- Existing perf spec:
  - `docs/project/specs/active/plan-2026-02-26-perf-comparison-profiling.md`
- Existing concurrency spec:
  - `docs/project/specs/active/plan-2026-02-27-parallel-file-processing.md`
- Benchmark report:
  - `benchmarks/REPORT.md`
- dprint source (incremental and scheduler):
  - `attic/dprint/crates/dprint/src/incremental/`
  - `attic/dprint/crates/dprint/src/format.rs`
