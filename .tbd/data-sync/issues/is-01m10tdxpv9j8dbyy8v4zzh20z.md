---
type: is
id: is-01m10tdxpv9j8dbyy8v4zzh20z
title: "PR #81 review R4: protected wrapping implementation duplication"
kind: task
status: open
priority: 2
version: 2
labels:
  - pr-review
  - refactor
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T05:17:36.345Z
updated_at: 2026-08-27T05:36:21.202Z
---
PR #81 R4. src/wrapping/line_wrappers.rs and src/wrapping/text_wrapping.rs duplicate protected and ordinary algorithms and have already drifted in indentation, whitespace replacement, and splitter options. Consolidate behind a measurement strategy or explicitly defer with bounded follow-up scope.

## Notes

Deferred after correctness fixes. The duplication and drift are real, but consolidating the ordinary and protected wrappers changes a broad parity-sensitive surface. Follow up with a measurement-strategy refactor and characterization tests; the new fail-soft path contains preservation failures meanwhile. Review suggestion S4 (options struct for the boolean-heavy fill_markdown API) belongs in this API/refactor phase.
