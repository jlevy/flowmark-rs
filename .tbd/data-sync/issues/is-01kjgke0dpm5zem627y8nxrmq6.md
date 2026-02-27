---
type: is
id: is-01kjgke0dpm5zem627y8nxrmq6
title: "Validation: cache correctness, invalidation, and CLI coverage"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-02-27-incremental-cache-and-performance-roadmap.md
labels: []
dependencies:
  - type: blocks
    target: is-01kjgke312artqxqyeb075d8r0
parent_id: is-01kjf287ax0e8s1bqrcamkb2m0
created_at: 2026-02-27T22:27:42.645Z
updated_at: 2026-02-27T22:50:19.733Z
closed_at: 2026-02-27T22:50:19.732Z
close_reason: Added validation coverage for cache behavior, config merge, and help/tryscript flags.
---
Add tests in tests/test_config.rs plus new cache-focused integration/unit tests (and tryscript help snapshots) covering unchanged hits, changed misses, no-incremental path, fingerprint invalidation, and corruption recovery.
