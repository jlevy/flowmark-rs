---
type: is
id: is-01kss3qm3ja86hnmscgpbnnt71
title: "PARITY GAP: wide/irregular (malformed) GFM table reflow"
kind: bug
status: closed
priority: 2
version: 3
labels:
  - parity
dependencies: []
parent_id: is-01kss3qk7et1jycwp3snxgs7zd
created_at: 2026-05-29T05:36:23.154Z
updated_at: 2026-05-29T06:27:05.280Z
closed_at: 2026-05-29T06:27:05.280Z
close_reason: "Fixed: ported flowmark table-row preservation (block_heuristics + add_tag_newline_handling); convex-rules now byte-identical, gap_d green"
---
convex-rules.md ~40 lines: a malformed pseudo-table embedded in a list item parses differently in comrak vs marko. Parity on malformed input is ill-defined. Candidate for tolerated variation.
