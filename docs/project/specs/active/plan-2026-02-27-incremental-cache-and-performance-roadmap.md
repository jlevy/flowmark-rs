---
title: Incremental Cache and Performance Roadmap
description: Add persistent incremental formatting cache and deeper performance instrumentation to close remaining gap to dprint
author: Joshua Levy (github.com/jlevy) with Codex assistance
---
# Feature: Incremental Cache and Performance Roadmap

**Date:** 2026-02-27 (last updated 2026-02-27)

**Author:** Joshua Levy with Codex assistance

**Status:** Draft

## Overview

Implement a persistent incremental cache for `flowmark-rs` and add stage-level
performance instrumentation to identify and remove remaining bottlenecks. The
goal is to beat dprint on fresh and reformat workloads while preserving
formatting behavior and parity guarantees.

## Goals

- Add persistent incremental cache for unchanged-file fast paths
- Preserve correctness by invalidating cache on options/version/behavior changes
- Add low-overhead timing instrumentation to isolate remaining hot stages
- Produce reproducible benchmark evidence against dprint and prior flowmark baselines
- Keep CLI/library behavior backward compatible by default

## Non-Goals

- Rewriting the markdown engine away from comrak in this iteration
- Introducing a plugin system or WASM runtime in flowmark
- Changing formatting semantics to chase benchmark numbers

## Background

### Current Performance Snapshot (Local Validation)

On a local corpus of 893 markdown files (`/tmp/fm-corpus-work`):

- `flowmark --auto --threads 1`: ~1.969s
- `flowmark --auto` (all cores): ~449ms
- `dprint fmt --incremental=false`: ~430ms (fresh)
- `dprint fmt --incremental=false`: ~216ms (reformat)
- `dprint fmt` (incremental default): ~23ms (reformat)

These measurements confirm:

- Flowmark is now roughly competitive with dprint for fresh runs
- The largest remaining practical gap is reformat/unchanged workloads
- File discovery is not the bottleneck (`flowmark --list-files` is single-digit ms)

### Why Incremental Cache Matters

The dprint source shows that incremental cache stores hashes of file content
known to be formatted, keyed under project base path and plugin/config hash.
When a file hash matches cache, dprint skips full formatting work for that file.
This does not require duplicate files; it primarily benefits unchanged files
across repeated runs.

### Research: Cache Behavior in Other Formatters

- **Prettier:** supports explicit cache mode (`--cache`) and configurable cache
  location/strategy
- **ESLint:** supports `--cache` and uses a cache file (`.eslintcache`) by
  default
- **Black:** uses an on-disk cache under the user cache directory by default
- **dprint:** uses a user cache directory with configurable override
  (`DPRINT_CACHE_DIR`) plus per-project incremental state

This pattern is common across modern format/lint tools: skip unchanged files
with persisted metadata instead of paying full parse/format cost every run.

## Design

### Approach

Add a persistent incremental cache layer in the CLI formatting path:

1. Read file bytes
2. Compute stable cache key (`content_hash + formatter_fingerprint`)
3. If cache hit, skip formatting and writing
4. If miss, format as today and update cache with resulting formatted hash

In parallel, add stage timing around `fill_markdown` pipeline segments to expose
real cost distribution:

- preprocessing workarounds
- comrak parse
- AST transforms
- rendering
- postprocessing normalization/restoration

### Components

- `src/main.rs`
  - Initialize cache manager for CLI runs
  - Wire cache checks into per-file processing
- `src/lib.rs`
  - Optionally split read/format/write path to support cache-first flow
- `src/formatter/filling.rs`
  - Add optional stage timers and aggregate counters
- `src/config.rs` / CLI args
  - Add incremental cache controls (enable/disable/path/strategy if needed)
- New module (proposed): `src/incremental_cache.rs`
  - Manifest model + serialization
  - Key/fingerprint computation
  - Atomic writes and corruption-safe load behavior

### API Changes

CLI additions (proposed):

- `--incremental` / `--no-incremental` (default enabled)
- `--incremental-cache-dir <PATH>` (optional override)
- `--perf-stats` (optional timing diagnostics output for profiling sessions)

Library API:

- No breaking changes
- Incremental behavior remains a CLI concern

## Implementation Plan

### Phase 1: Incremental Cache + Perf Instrumentation

- [ ] Add cache data model and persistence (JSON/TOML) in a user cache location
- [ ] Define formatter fingerprint that invalidates on version/config changes
- [ ] Integrate cache hit/miss logic into CLI file processing
- [ ] Ensure cache update occurs only after successful format path
- [ ] Add `--no-incremental` escape hatch and cache-dir override
- [ ] Add stage timing instrumentation behind `--perf-stats`
- [ ] Add tests for cache correctness and invalidation behavior
- [ ] Benchmark fresh/reformat workloads with and without cache
- [ ] Update `benchmarks/REPORT.md` and README performance section

## Testing Strategy

- Unit tests:
  - cache read/write/merge behavior
  - corruption handling (graceful reset)
  - fingerprint invalidation on option/version changes
- Integration tests:
  - unchanged files are skipped
  - changed files are reformatted
  - `--no-incremental` forces full formatting
  - output parity unchanged
- Benchmark validation:
  - fresh corpus timings
  - repeated-run timings (warm incremental cache)
  - thread scaling checks

## Rollout Plan

1. Land cache with conservative invalidation and `--no-incremental` fallback
2. Enable by default in CLI
3. Publish updated benchmark report with reproducible commands
4. Monitor for correctness regressions and cache edge cases

## Open Questions

- Should cache key use raw input hash only, or include output hash for stronger
  safety checks?
- Should cache be file-based manifest only, or sharded for large repos?
- Should perf stats be human-readable only, or optionally machine-readable JSON?

## References

- Existing perf spec:
  - `docs/project/specs/active/plan-2026-02-26-perf-comparison-profiling.md`
- Existing parallelization spec:
  - `docs/project/specs/active/plan-2026-02-27-parallel-file-processing.md`
- dprint source (incremental and scheduler):
  - `attic/dprint/crates/dprint/src/incremental/`
  - `attic/dprint/crates/dprint/src/format.rs`
- dprint markdown plugin source:
  - `attic/dprint-plugin-markdown/src/`
- Prettier CLI cache documentation:
  - https://prettier.io/docs/next/cli/
- ESLint CLI cache documentation:
  - https://eslint.org/docs/latest/use/command-line-interface
- Black cache behavior documentation:
  - https://black.readthedocs.io/en/stable/usage_and_configuration/file_collection_and_discovery.html
