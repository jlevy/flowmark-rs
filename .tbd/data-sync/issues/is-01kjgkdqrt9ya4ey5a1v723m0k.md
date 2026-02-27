---
type: is
id: is-01kjgkdqrt9ya4ey5a1v723m0k
title: "CLI/config wiring: incremental flags and merge precedence"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-02-27-incremental-cache-and-performance-roadmap.md
labels: []
dependencies:
  - type: blocks
    target: is-01kjgkdthfc5vt1q3rk45ym5b1
  - type: blocks
    target: is-01kjf287g1fwtffhra2g7wqpaa
parent_id: is-01kjf287ax0e8s1bqrcamkb2m0
created_at: 2026-02-27T22:27:33.784Z
updated_at: 2026-02-27T22:27:58.395Z
---
Update src/main.rs Args and src/config.rs FlowmarkConfig/merge_cli_with_config for --incremental, --no-incremental, --incremental-cache-dir, and --perf-stats with explicit CLI > config precedence.
