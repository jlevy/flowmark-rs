---
type: is
id: is-01kjf287g1fwtffhra2g7wqpaa
title: Add stage-level perf instrumentation and refresh cross-formatter benchmarks
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-02-27-incremental-cache-and-performance-roadmap.md
labels: []
dependencies:
  - type: blocks
    target: is-01kjgke0dpm5zem627y8nxrmq6
  - type: blocks
    target: is-01kjgke6rv2bcxb1shkfcpk6fe
parent_id: is-01kjf287ax0e8s1bqrcamkb2m0
created_at: 2026-02-27T08:08:13.056Z
updated_at: 2026-02-27T22:50:19.571Z
closed_at: 2026-02-27T22:50:19.570Z
close_reason: Implemented stage-level fill_markdown perf instrumentation and CLI perf output.
---
Add optional stage timing in src/formatter/filling.rs (preprocess, parse, transforms, render, postprocess) and aggregate/report via src/main.rs --perf-stats with near-zero overhead when disabled.
