---
type: is
id: is-01m10tdwjjbephf0kvjfdx00zs
title: "PR #81 review R1: callout plus protected inline aborts"
kind: bug
status: closed
priority: 0
version: 3
labels:
  - pr-review
  - preservation
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T05:17:35.183Z
updated_at: 2026-08-27T05:36:16.680Z
closed_at: 2026-08-27T05:36:16.678Z
close_reason: Fixed R1 with a shared Python golden and Rust parser/scanner ownership alignment; the reproducer no longer aborts or loses content.
resolution: null
duplicate_of: null
---
PR #81 R1. src/preservation/scanner.rs is_obsidian_callout, src/formatter/markdown.rs alerts, and src/formatter/filling.rs restoration disagree for forms such as > [!NOTE]<v>, causing SIGABRT. Add shared callout-plus-inline cases first, make scanner and renderer agree, and require non-aborting exact output.

## Notes

Shared-first fix at upstream 6f74a02: preservation.extension.callout.adjacent-inline covers HTML, code, math, wikilink, role, spaced-title, and no-space quote variants. Rust disables comrak alert parsing so the preservation scanner is the sole callout owner. The shared case passes twice and the native alert suite passes.
