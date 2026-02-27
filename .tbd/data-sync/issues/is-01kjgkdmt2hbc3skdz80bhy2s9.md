---
type: is
id: is-01kjgkdmt2hbc3skdz80bhy2s9
title: "Cache core: add incremental manifest, fingerprint, and atomic persistence"
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-02-27-incremental-cache-and-performance-roadmap.md
labels: []
dependencies:
  - type: blocks
    target: is-01kjgkdqrt9ya4ey5a1v723m0k
  - type: blocks
    target: is-01kjgkdthfc5vt1q3rk45ym5b1
parent_id: is-01kjf287ax0e8s1bqrcamkb2m0
created_at: 2026-02-27T22:27:30.753Z
updated_at: 2026-02-27T22:50:19.181Z
closed_at: 2026-02-27T22:50:19.180Z
close_reason: Implemented incremental cache core module with persistence, fingerprinting, and tests.
---
Implement src/incremental_cache.rs with IncrementalCache::open/is_known_formatted/record_formatted/flush, formatter fingerprint computation, and corruption-safe atomic manifest load/save.
