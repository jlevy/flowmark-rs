---
type: is
id: is-01khve4rjfkamvhpdjzaffnmv7
title: "Golden test regression: item_needs_child_spacing complex sublist check incorrectly applied in Preserve mode"
kind: bug
status: closed
priority: 1
version: 2
labels:
  - parity
dependencies: []
created_at: 2026-02-19T17:11:13.742Z
updated_at: 2026-02-19T17:11:42.562Z
closed_at: 2026-02-19T17:11:42.561Z
close_reason: "Fixed: gated has_complex_sublist check in item_needs_child_spacing to only fire in ListSpacing::Tight mode, not Preserve mode. Golden test test_reference_doc_formats now passes without regenerating expected files."
---
