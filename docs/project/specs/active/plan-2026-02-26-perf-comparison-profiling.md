---
title: Performance Comparison and Profiling
description: End-to-end benchmarking of Rust vs Python flowmark and profiling the Rust implementation
author: Joshua Levy (github.com/jlevy) with LLM assistance
---
# Feature: Performance Comparison and Profiling

**Date:** 2026-02-26

**Author:** Joshua Levy with LLM assistance

**Status:** In Progress

## Overview

Create a reproducible performance comparison between the Rust (flowmark-rs v0.2.4) and
Python (flowmark v0.6.5) implementations, measuring end-to-end formatting time including
file discovery and I/O.
Then profile the Rust version to identify optimization opportunities.

## Goals

- Measure real-world end-to-end performance (file discovery + formatting + I/O) at scale
- Produce a reproducible benchmark using ~1,000 copies of the project's own Markdown
  files, placed in a realistic directory tree
- Quantify the Rust vs Python speedup factor
- Profile the Rust binary to identify hotspots and bottlenecks
- Document findings for future optimization work

## Non-Goals

- Implementing optimizations (this spec is for measurement and analysis)
- Micro-benchmarks of individual functions (focus is end-to-end)
- Cross-platform benchmarking (Linux only for this iteration)

## Background

The Rust port (v0.2.4) achieves full parity with Python flowmark v0.6.4 across 430
tests.
The README notes Rust is "good for large repos and CI pipelines" but no benchmarks exist
yet.
This spec creates the first systematic performance measurement.

The release profile is already well-tuned (`opt-level = 3`, `lto = true`,
`codegen-units = 1`, `panic = "abort"`, `strip = true`).

## Design

### Approach

1. **Test corpus generation**: A shell script creates a directory tree with ~1,000 copies
   of all Markdown files from this repo (content fixtures, test documents, documentation).
   Files are distributed across a moderately deep directory structure to exercise file
   discovery.

2. **Benchmark runner**: A shell script that:
   - Runs the Python CLI (`flowmark`) and Rust binary on the same corpus
   - Measures wall-clock time for end-to-end processing (`--check` mode to avoid I/O of
     writing output files, focusing on parse + format time)
   - Runs multiple iterations and reports min/mean/median
   - Reports files/second and MB/second throughput

3. **Profiling**: Use `perf` / `flamegraph` to profile the Rust binary on the corpus and
   identify where time is spent.

### Components

- `benches/generate-bench-corpus.sh` - Corpus generation script
- `benches/run-bench.sh` - Benchmark runner (Python vs Rust)
- `benches/profile-rust.sh` - Profiling helper script
- `benches/README.md` - Results documentation

### Test Corpus Design

Source files (from this repo):
- 21 content fixtures (`tests/tryscript/fixtures/content/*.md`) - ~6.5 KB total
- 1 large test document (`tests/testdocs/testdoc.orig.md`) - 77 KB
- 5 parity test files (`tests/parity/corner-cases.md` + expected outputs) - ~26 KB
- Key documentation files (README.md, docs/*.md) - selected subset

Total source: ~25 unique files of varying complexity.

Target: 1,000 copies distributed across a directory tree like:
```
benches/corpus/
  batch-000/ through batch-039/
    each containing 25 copies of the source files
    with subdirectories: content/, docs/, tests/
```

This gives ~1,000 total files, ~3-5 MB total, exercising:
- Directory traversal across 40+ directories
- Mix of small (86 bytes) to large (77 KB) files
- All Markdown features (tables, footnotes, code blocks, frontmatter, math, etc.)

## Implementation Plan

### Phase 1: Benchmarking Infrastructure

- [x] Create `benches/generate-bench-corpus.sh` to generate the test corpus
- [x] Create `benches/run-bench.sh` to time both implementations
- [x] Run initial comparison and capture baseline numbers
- [x] Document results

### Phase 2: Profiling and Analysis

- [x] Profile Rust binary with `perf` or equivalent
- [x] Generate flamegraph or equivalent analysis
- [x] Identify top hotspots and bottleneck areas
- [x] Document profiling findings and optimization opportunities

## Testing Strategy

- Verify corpus generation produces expected file count and structure
- Verify both formatters produce identical output on the corpus (parity check)
- Ensure benchmark timing is reproducible (multiple runs, low variance)

## Open Questions

- Should we also measure `--auto` mode (semantic + cleanups + smartquotes + ellipses) in
  addition to default mode?
- What level of speedup warrants further optimization effort?

## References

- Port status: `docs/port-status.md`
- Test fixtures: `tests/tryscript/fixtures/content/`
- Large test doc: `tests/testdocs/testdoc.orig.md`
- Parity tests: `tests/parity/`
