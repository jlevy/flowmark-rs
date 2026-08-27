---
type: is
id: is-01m10zhbvmnfgq1zwtdbzey628
title: "PR #81 review P5: compute GFM table-line classification once"
kind: bug
status: closed
priority: 3
version: 4
labels:
  - pr-review
  - performance
dependencies: []
parent_id: is-01m10zgx85zvg3r07e73xj2733
created_at: 2026-08-27T06:46:52.018Z
updated_at: 2026-08-27T08:13:32.926Z
closed_at: 2026-08-27T08:13:32.925Z
close_reason: The shared expensive GFM classification is cached while the two intentionally different semantic policies remain separate.
resolution: null
duplicate_of: null
---
PR #81 follow-up finding P5 at scanner.rs scan_pandoc_line_blocks and inline_scopes. Both paths repeat the same has_structural_pipe plus is_table_delimiter classification, including inline scans and arbitration. Compute the table-line bitmap once and reuse it without changing shared outputs.

## Notes

P5 disposition: the two GFM consumers intentionally differ in exclusion and container matching, so one shared bitmap would change semantics. The expensive structural-pipe result is now cached once per line and delimiter classification runs first, eliminating duplicate inline scans while retaining exact output. Full local and hosted suites are green at f833ce8.
