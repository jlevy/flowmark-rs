---
type: is
id: is-01kjk7ddpzzz1p0h5f1s8v6pbs
title: "Tests: cover cache lifecycle commands in integration and golden suites"
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-02-27-incremental-cache-and-performance-roadmap.md
labels: []
dependencies:
  - type: blocks
    target: is-01kjk7de0m2z4325z4fvws37zr
parent_id: is-01kjk7d0kwsp6g521d1crnecr3
created_at: 2026-02-28T22:55:23.870Z
updated_at: 2026-02-28T22:55:50.002Z
closed_at: 2026-02-28T22:55:50.001Z
close_reason: Integration and golden tests added and passing under cargo test --features cli.
---
Add/extend tests for --show-cache and --clear-cache behavior in tests/test_incremental_cache.rs and tryscript golden files.

## Notes

Added integration coverage in tests/test_incremental_cache.rs for show/clear behavior; extended tryscript help and cache-behavior golden sessions to cover new flags and lifecycle behavior.
