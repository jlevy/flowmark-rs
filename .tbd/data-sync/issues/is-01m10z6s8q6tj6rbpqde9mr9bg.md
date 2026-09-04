---
type: is
id: is-01m10z6s8q6tj6rbpqde9mr9bg
title: "PR #81: instrument and guard preservation performance"
kind: task
status: in_progress
priority: 1
version: 3
labels:
  - preservation
  - performance
  - testing
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T06:41:05.302Z
updated_at: 2026-08-27T08:13:25.368Z
---
Current stage statistics begin after normalization, scan/protect, and construction/cloning of ProtectedSource, so they report fill_markdown as unchanged even while end-to-end wall time regresses by 37%. Add explicit preservation timing that covers normalization, scanning, protection, and integration overhead; remove or account for the deep ProtectedSource clone (prefer borrowing or Arc if compatible); and add reproducible Rust-native performance validation.

Acceptance: benchmark the existing 1,000-file corpus against the documented pre-preservation baseline; include deterministic adversarial cases for many protected blocks, many pipes plus atomics, and unmatched nested delimiters; use operation-count/structural assertions or a stable benchmark job rather than flaky wall-clock unit-test thresholds; document the <=5% representative throughput budget and report preservation time separately; keep shared goldens as the behavioral authority.

## Notes

P4 and P6 from comment 5435358838 and the release addendum are fixed at the implementation level in ae6b736 and f833ce8: marker-free Cow fast path, precise zero-preservation admission, conditional CR work, direct UTF-8 token parsing without Vec char, Arc protected state, allocation-reduced restore, parallel large-document scanning, and preservation included in reported preprocessing time. Final v0.3.2 comparison is effectively flat on 5.9 MB construct-free prose and about 4-5 percent typical overhead on testdoc x64. Keep this bead open for the remaining durable operation-count or benchmark job, distinct scan/protect counters, and the recorded 17 percent single-thread tax on the generated 999-small-file feature-heavy corpus.
