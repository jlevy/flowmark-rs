---
type: is
id: is-01m10zhd2vdfhe24m306wqxsvb
title: "PR #81 review P7: preserve measured scanner improvements"
kind: task
status: closed
priority: 3
version: 4
labels:
  - pr-review
  - performance
dependencies: []
parent_id: is-01m10zgx85zvg3r07e73xj2733
created_at: 2026-08-27T06:46:53.273Z
updated_at: 2026-08-27T08:13:33.272Z
closed_at: 2026-08-27T08:13:33.272Z
close_reason: The P7 improvements are retained and covered by focused, corpus, and hosted validation.
resolution: null
duplicate_of: null
---
PR #81 follow-up finding P7 is positive evidence: the preservation scanner eliminates pre-existing quadratics for unmatched backticks, closed wikilinks, definition lists, and callouts. Record an explicit no-regression disposition and retain deterministic coverage while fixing P1-P4.

## Notes

P7 explicitly preserved. Focused scanner tests, full all-feature and no-default suites, 998-file output differential, and hosted CI confirm the prior backtick, closed-wikilink, definition-list, and callout improvements remain after the P1-P6 fixes.
