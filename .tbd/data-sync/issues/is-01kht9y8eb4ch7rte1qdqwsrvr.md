---
type: is
id: is-01kht9y8eb4ch7rte1qdqwsrvr
title: "D6: Nested blockquotes get extra blank separator lines"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-02-18-parity-discrepancies.md
labels: []
dependencies:
  - type: blocks
    target: is-01kht9xy41xv7hb3wch5nz4bds
created_at: 2026-02-19T06:38:31.882Z
updated_at: 2026-02-19T17:12:24.462Z
closed_at: 2026-02-19T17:12:24.461Z
close_reason: "Fixed: added source position tracking to render_block_children_quoted. Blank lines only suppressed when nested blockquotes were originally tight. Tests: test_d6_nested_blockquotes_no_extra_blanks, test_d6_two_level_blockquote, test_d6_nested_blockquote_preserves_blank_separator"
---
