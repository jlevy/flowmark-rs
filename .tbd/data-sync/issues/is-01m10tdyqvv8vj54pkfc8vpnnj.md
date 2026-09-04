---
type: is
id: is-01m10tdyqvv8vj54pkfc8vpnnj
title: "PR #81 review R7: changelog and CLI contracts are missing"
kind: task
status: closed
priority: 2
version: 3
labels:
  - pr-review
  - docs
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T05:17:37.403Z
updated_at: 2026-08-27T05:36:19.090Z
closed_at: 2026-08-27T05:36:19.088Z
close_reason: Fixed R7 in canonical documentation sources and generated surfaces.
resolution: null
duplicate_of: null
---
PR #81 R7. CHANGELOG Unreleased does not describe backup naming, missing-path behavior, invalid UTF-8, stdin plus output, dedent, or preservation features. Document user-visible changes and the stable 0/1/2 exit-code contract; include recursive-clone test requirements.

## Notes

Added comprehensive Unreleased formatter/CLI/testing notes, the required common-doc footer, and the 0/1/2 exit contract in the upstream shared README source. Regenerated Python README, Rust README, and the bundled Rust docs; recursive submodule setup was already documented in docs/development.md.
