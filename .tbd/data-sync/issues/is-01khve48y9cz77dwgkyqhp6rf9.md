---
type: is
id: is-01khve48y9cz77dwgkyqhp6rf9
title: "Loose mode Rules 3/4: paragraph→list and paragraph→code block blank line suppression incorrectly active in loose mode"
kind: bug
status: closed
priority: 1
version: 2
labels:
  - parity
dependencies: []
created_at: 2026-02-19T17:10:57.736Z
updated_at: 2026-02-19T17:11:27.187Z
closed_at: 2026-02-19T17:11:27.186Z
close_reason: "Fixed: added list_spacing != ListSpacing::Loose guard to Rules 3 and 4 in render_block_children(). Loose mode now correctly inserts blank separators for paragraph→list and paragraph→code block transitions."
---
