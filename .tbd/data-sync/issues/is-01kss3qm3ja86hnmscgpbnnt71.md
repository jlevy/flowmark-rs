---
type: is
id: is-01kss3qm3ja86hnmscgpbnnt71
title: "PARITY GAP: wide/irregular (malformed) GFM table reflow"
kind: bug
status: in_progress
priority: 2
version: 2
labels:
  - parity
dependencies: []
parent_id: is-01kss3qk7et1jycwp3snxgs7zd
created_at: 2026-05-29T05:36:23.154Z
updated_at: 2026-05-29T06:03:24.593Z
---
convex-rules.md ~40 lines: a malformed pseudo-table embedded in a list item parses differently in comrak vs marko. Parity on malformed input is ill-defined. Candidate for tolerated variation.
