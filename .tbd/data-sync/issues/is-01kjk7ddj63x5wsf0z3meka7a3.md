---
type: is
id: is-01kjk7ddj63x5wsf0z3meka7a3
title: "CLI: add --show-cache and --clear-cache execution paths"
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-02-27-incremental-cache-and-performance-roadmap.md
labels: []
dependencies:
  - type: blocks
    target: is-01kjk7ddpzzz1p0h5f1s8v6pbs
  - type: blocks
    target: is-01kjk7ddvqc0zcrcg3am8zeyhd
parent_id: is-01kjk7d0kwsp6g521d1crnecr3
created_at: 2026-02-28T22:55:23.718Z
updated_at: 2026-02-28T22:55:49.751Z
closed_at: 2026-02-28T22:55:49.750Z
close_reason: Implemented CLI behavior and clippy-safe size formatting; verified help output exposes both flags.
---
Add non-interactive cache lifecycle command handling in src/main.rs, including cache root resolution reuse and cache size/file reporting.

## Notes

Implemented in src/main.rs: added --show-cache/--clear-cache flags, cache root resolution helper reuse, cache usage reporting, and non-interactive lifecycle command path before file-args validation. Landed in commits 5830a45 and 2035f8c.
