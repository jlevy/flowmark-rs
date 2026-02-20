---
type: is
id: is-01khve4kzgzqvg372z88kvjm2v
title: "Nested blockquote blank separator: source position tracking needed in render_block_children_quoted"
kind: bug
status: closed
priority: 2
version: 2
labels:
  - parity
dependencies: []
created_at: 2026-02-19T17:11:09.039Z
updated_at: 2026-02-19T17:11:38.602Z
closed_at: 2026-02-19T17:11:38.601Z
close_reason: "Fixed: added prev_source_end_line and originally_tight tracking to render_block_children_quoted. Only suppresses blank lines before nested blockquotes when source was originally tight. Tests: test_d6_nested_blockquotes_no_extra_blanks, test_d6_nested_blockquote_preserves_blank_separator"
---
