---
type: is
id: is-01m10tdy1fgztpm44wttyy5ag5
title: "PR #81 review R5: custom public line wrappers are token-unaware"
kind: bug
status: open
priority: 2
version: 2
labels:
  - pr-review
  - api
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T05:17:36.686Z
updated_at: 2026-08-27T05:36:21.467Z
---
PR #81 R5. A caller-supplied public LineWrapper receives preservation tokens and may measure token spelling rather than authored width. Remove, adapt, deprecate, or document the unsafe combination with a tested public API contract.

## Notes

Deferred as a cross-port public-API design change. fill_markdown now documents that custom wrappers receive preservation tokens, requires unchanged token order, recommends FormatOptions/None, and fails soft if tokens are lost. The remaining authored-width issue requires either a shared measurement API or a breaking/deprecated API decision in Python and Rust, not a Rust-only semantic divergence.
