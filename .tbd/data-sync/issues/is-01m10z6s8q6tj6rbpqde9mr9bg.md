---
type: is
id: is-01m10z6s8q6tj6rbpqde9mr9bg
title: "PR #81: instrument and guard preservation performance"
kind: task
status: open
priority: 1
version: 1
labels:
  - preservation
  - performance
  - testing
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T06:41:05.302Z
updated_at: 2026-08-27T06:41:05.302Z
---
Current stage statistics begin after normalization, scan/protect, and construction/cloning of ProtectedSource, so they report fill_markdown as unchanged even while end-to-end wall time regresses by 37%. Add explicit preservation timing that covers normalization, scanning, protection, and integration overhead; remove or account for the deep ProtectedSource clone (prefer borrowing or Arc if compatible); and add reproducible Rust-native performance validation.

Acceptance: benchmark the existing 1,000-file corpus against the documented pre-preservation baseline; include deterministic adversarial cases for many protected blocks, many pipes plus atomics, and unmatched nested delimiters; use operation-count/structural assertions or a stable benchmark job rather than flaky wall-clock unit-test thresholds; document the <=5% representative throughput budget and report preservation time separately; keep shared goldens as the behavioral authority.
