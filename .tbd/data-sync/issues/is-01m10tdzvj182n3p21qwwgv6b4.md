---
type: is
id: is-01m10tdzvj182n3p21qwwgv6b4
title: "PR #81 review R11: RegionKind has misleading Ord derive"
kind: task
status: closed
priority: 3
version: 3
labels:
  - pr-review
  - cleanup
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T05:17:38.545Z
updated_at: 2026-08-27T05:36:20.863Z
closed_at: 2026-08-27T05:36:20.862Z
close_reason: Fixed R11 by removing the misleading unused ordering derives.
resolution: null
duplicate_of: null
---
PR #81 R11. src/preservation/model.rs derives PartialOrd and Ord even though arbitration uses explicit priority, length, and stable name. Remove the unused derives or document a real ordering contract.

## Notes

Removed PartialOrd and Ord from RegionKind. Arbitration remains solely the explicit registry priority, longest span, and stable name.
