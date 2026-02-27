---
type: is
id: is-01kjgkdthfc5vt1q3rk45ym5b1
title: Integrate cache-aware file processing path in formatter loop
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-02-27-incremental-cache-and-performance-roadmap.md
labels: []
dependencies:
  - type: blocks
    target: is-01kjgke0dpm5zem627y8nxrmq6
parent_id: is-01kjf287ax0e8s1bqrcamkb2m0
created_at: 2026-02-27T22:27:36.622Z
updated_at: 2026-02-27T22:27:58.524Z
---
Refactor src/main.rs file processing path (and minimal src/lib.rs helpers if needed) to do read -> cache check -> format -> write -> cache record while preserving current CLI behavior and output parity.
