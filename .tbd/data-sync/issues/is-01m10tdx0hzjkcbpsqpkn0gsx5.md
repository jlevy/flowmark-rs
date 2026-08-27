---
type: is
id: is-01m10tdx0hzjkcbpsqpkn0gsx5
title: "PR #81 review R2: preservation failures panic"
kind: bug
status: closed
priority: 1
version: 3
labels:
  - pr-review
  - error-handling
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T05:17:35.632Z
updated_at: 2026-08-27T05:36:17.289Z
closed_at: 2026-08-27T05:36:17.288Z
close_reason: Fixed R2 by making preservation failures fail soft to normalized source instead of panicking.
resolution: null
duplicate_of: null
---
PR #81 R2. src/formatter/filling.rs converts PreservationError results into process-aborting expect calls. Choose and test a fail-soft or fail-explicit policy so third-party renderer disagreement cannot abort library or CLI callers.

## Notes

Chose fail-soft to preserve the public String-returning API. Scanner, protection, typography, wrapping, and restoration errors no longer use expect. A restoration fault returns normalized source unchanged; focused fault-injection and public callout tests pass.
