---
title: Incremental Cache and Performance Roadmap
description: Design review and roadmap for making flowmark-rs fastest on reruns via incremental caching and targeted bottleneck elimination
author: Joshua Levy (github.com/jlevy) with Codex assistance
---
# Feature: Incremental Cache and Performance Roadmap

**Date:** 2026-02-27 (last updated 2026-02-27)

**Author:** Joshua Levy with Codex assistance

**Status:** Draft

## Executive Summary

The core path to "fastest markdown formatter on reruns" is:

1. Add a correctness-safe incremental cache that skips unchanged files.
2. Add low-overhead stage timing to identify remaining hot spots.
3. Use benchmark gates to decide unresolved design choices, not intuition.

From first principles, rerun performance is won by minimizing work before parse/format.
Today flowmark always executes the full formatting pipeline for every file; dprint can
skip almost all work on unchanged files.
The design below is intentionally focused on that gap first.

## Current Baseline (Single Corpus)

### Baseline Corpus and Method

- Corpus: `benchmarks/corpus` (928 files, current harness corpus)
- Method: first run and immediate second run on the same directory
- Platform: local macOS arm64
- Commands:
  - flowmark-rs: `flowmark --auto .`
  - dprint (no incremental): `dprint fmt --config dprint.json --incremental=false .`
  - dprint (incremental default): `dprint fmt --config dprint.json .`

### First Run vs Second Run Snapshot

| Tool / Mode | Run 1 (fresh) | Run 2 (same corpus) | Rerun Speedup |
| --- | --- | --- | --- |
| flowmark-rs `--auto` | 0.671 s | 0.608 s | 1.1x |
| dprint `--incremental=false` | 0.433 s | 0.289 s | 1.5x |
| dprint incremental default | 0.316 s | 0.029 s | 10.9x |

### Key Implications

- Fresh-run performance is already strong (top tier), but flowmark reruns are
  effectively still full-work runs.
- dprint's incremental cache is the dominant reason for its rerun lead.
- The biggest opportunity is unchanged-file skip behavior, not file discovery.

## First-Principles Performance Model

Approximate total wall time for one run:

`T = T_discovery + T_metadata + T_read + T_hash + T_parse_transform_render + T_write + T_cache_io`

Without incremental cache, unchanged reruns still pay most of:

- `T_read`
- `T_parse_transform_render` (largest)
- partial `T_write` (even if unchanged check skips final write)

To win reruns, flowmark must skip before parse/transform/render, and ideally before file
reads when safe.

## Research Findings: Other Implementations

### dprint (source-backed)

- Incremental file is keyed by project base path hash under cache dir:
  `crates/dprint/src/incremental/mod.rs`
- Cache data stores plugin fingerprint + set of known-formatted content hashes:
  `crates/dprint/src/incremental/incremental_file.rs`
- Plugin fingerprint includes plugin version + sorted config + associations + global
  config: `crates/dprint/src/resolution.rs` (`incremental_hash`, `plugins_hash`)
- Hashing uses fast xxHash64: `crates/dprint/src/utils/get_bytes_hash.rs`
- In format path, dprint reads file bytes then checks incremental hash set before
  formatting: `crates/dprint/src/format.rs`

Important inference: dprint still reads each file to hash it on reruns; this is fast,
but a metadata fast path could potentially beat this for unchanged trees.

### Prettier / mdformat / markdownfmt / flowmark-py (CLI behavior)

- Prettier supports explicit cache controls: `--cache`, `--cache-location`,
  `--cache-strategy` (`metadata|content`)
- mdformat CLI has no incremental cache flag.
- flowmark-py CLI has no incremental cache flag.
- markdownfmt behavior in our environment does not expose an incremental cache mode.

## Flowmark-rs Bottleneck Review (Current Code)

1. No early skip path before formatting
- `src/main.rs` parallel loop calls `opts.reformat_file(...)` on every file.
- `src/lib.rs::FormatOptions::reformat_file` reads full file and always computes full
  `reformat_text` before deciding whether write is needed.

2. Formatting pipeline is intentionally heavy (feature-rich)
- `src/formatter/filling.rs::fill_markdown` does many pre/post passes and full comrak
  parse.
- This is expected cost for semantic features, but unchanged files should bypass it.

3. No stage-level timing in production path
- We currently cannot quantify per-stage cost split during real corpus runs.
- This blocks informed tradeoffs after cache lands.

4. File discovery is already optimized and not the lead bottleneck
- Existing profiling/bench data shows formatting dominates.

## Design Goals

- Preserve formatting correctness and parity guarantees.
- Make unchanged reruns dramatically faster.
- Keep fresh-run regressions near zero.
- Keep design measurable and reversible.

### API Changes

CLI additions (proposed):

- `--incremental` / `--no-incremental` (default enabled)
- `--incremental-cache-dir <PATH>` (optional override)
- `--incremental-strategy <content|hybrid|metadata>` (optional, see below)
- `--perf-stats[=json]` (timing diagnostics)

Library API:

- No breaking changes
- Incremental behavior remains a CLI concern

## Proposed Architecture

### Cache Fingerprint (Invalidation)

Fingerprint must include all formatting-affecting inputs:

- flowmark-rs version
- formatter options affecting output (`width`, `semantic`, `cleanups`, `smartquotes`,
  `ellipses`, `list_spacing`, plaintext mode)
- config-derived values that affect output
- formatting engine schema version (for future cache migrations)

### Two-Layer Incremental Model

Layer A: content-hash skip (safe default)
- Compute content hash for file bytes.
- If `(fingerprint, content_hash)` is known-formatted, skip formatting.
- Correctness-safe; hash collisions are the only theoretical risk.

Layer B: metadata fast path (optional first, then default if proven)
- Track per-path metadata from last successful run (`size`, `mtime_ns`, plus stored
  content hash).
- If metadata unchanged and fingerprint unchanged, skip file read + skip hash + skip
  format.
- If metadata changed or uncertain, fall back to Layer A.

Why both:
- Layer A gives safe wins immediately and mirrors dprint behavior.
- Layer B is the path to potentially beating dprint reruns by skipping reads.

### Storage Shape

Proposed `src/incremental_cache.rs` manifest model:

- `schema_version`
- `fingerprint`
- `formatted_hashes: HashSet<u64 or u128>`
- `path_index: HashMap<relative_path, PathEntry>`
  - `size`
  - `mtime_ns`
  - `content_hash`

Write strategy:
- Merge thread-local updates at end.
- Single atomic write (temp + rename).
- Corrupt/missing file => warn and recreate.

### Concurrency Model

- Read cache once at startup (`Arc` shared read-only across workers).
- Workers append updates to thread-local buffers.
- Main thread merges and writes once.
- Avoid per-file global mutex contention.

## Unresolved Decisions and How to Decide

| Decision | Options | What to Measure | Decision Gate |
| --- | --- | --- | --- |
| Hash function | `xxhash64` vs `xxh3_128` vs `blake3` | hash throughput, collision risk profile, rerun wall time | pick fastest option with acceptable collision envelope |
| Manifest backend | JSON vs binary/SQLite | parse/serialize overhead at 1k/10k/100k files | keep JSON unless it adds >5% rerun cost |
| Metadata fast path default | off by default vs hybrid default | correctness tests on coarse-mtime FS + rerun speed delta | default on only if zero correctness regressions in test matrix |
| Path keys | absolute vs relative-to-base | portability across checkout path changes | use relative keys for cache reuse and portability |
| Cache size policy | unbounded vs bounded/LRU | manifest growth over large repos | add pruning if cache write/read exceeds target budget |

## Measurement Plan (Before Locking Decisions)

### Workload Matrix

Single corpus, repeated for each candidate design:

- Fresh run: 100% files changed
- Warm unchanged: 0% changed (immediate rerun)
- Warm partial: 1%, 10%, 50% files changed
- Cache cold start: cache exists but new process

### Metrics

- Wall clock total
- Files/sec
- Stage timings (`read`, `hash`, `format`, `write`, `cache_lookup`)
- Cache hit rate and skip counts
- Cache read/write overhead

### Acceptance Criteria

Phase 1 (content-hash incremental):
- unchanged rerun at least 5x faster than current flowmark baseline
- fresh-run regression <= 5%
- zero formatting correctness regressions

Phase 2 (metadata fast path):
- unchanged rerun reaches dprint-incremental range on benchmark corpus
- no false skips in correctness suite
- fallback behavior verified on metadata ambiguity

## Benchmark Reporting Plan (Post-Implementation)

README should stay simple and single-corpus:

- one table, one corpus, same tools
- columns: `Run 1 (fresh)` and `Run 2 (unchanged rerun)`
- include a direct flowmark-rs vs flowmark-py statement from that same table

Full details and variant runs stay in `benchmarks/REPORT.md`.

## Implementation Phases

1. Add instrumentation (`--perf-stats`) with negligible overhead.
2. Land content-hash incremental cache (`content` strategy).
3. Add metadata fast path behind opt-in (`hybrid` or `metadata` strategy).
4. Promote fast path to default only after test + benchmark gates pass.
5. Update README summary table and full report with first-run vs second-run numbers.

## Risks and Mitigations

- False cache hits (hash collisions):
  - Mitigate with stronger hash option and optional secondary validation.
- Metadata-only false skips:
  - Keep metadata fast path gated; fallback to content hash when uncertain.
- Cache corruption:
  - Atomic writes + tolerant read/recreate behavior.
- Manifest growth:
  - Track size; add pruning if thresholds exceeded.
- Fresh-run slowdown from cache overhead:
  - Gate on <=5% regression.

## References

- Flowmark code paths:
  - `src/main.rs`
  - `src/lib.rs`
  - `src/formatter/filling.rs`
- Current benchmark harness:
  - `benchmarks/run_comparison.sh`
  - `benchmarks/generate_corpus.sh`
- dprint incremental and scheduler internals:
  - `attic/dprint/crates/dprint/src/incremental/mod.rs`
  - `attic/dprint/crates/dprint/src/incremental/incremental_file.rs`
  - `attic/dprint/crates/dprint/src/format.rs`
  - `attic/dprint/crates/dprint/src/resolution.rs`
  - `attic/dprint/crates/dprint/src/utils/get_bytes_hash.rs`
- Prettier CLI cache flags:
  - https://prettier.io/docs/cli
