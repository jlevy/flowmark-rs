---
type: is
id: is-01m10tdyd9kam1n26s9z1sdr8q
title: "PR #81 review R6: reformat_text dedent change is silent"
kind: bug
status: closed
priority: 2
version: 3
labels:
  - pr-review
  - api
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T05:17:37.064Z
updated_at: 2026-08-27T05:36:18.552Z
closed_at: 2026-08-27T05:36:18.551Z
close_reason: Fixed R6 documentation and production-path coverage; retained direct true-mode tests because they exercise the explicit public opt-in rather than a dead path.
resolution: null
duplicate_of: null
---
PR #81 R6. src/lib.rs changed FormatOptions::reformat_text dedent_input from true to false while many tests still exercise true and docs/changelog omit the behavior change. Reconcile production parity, tests, and documentation.

## Notes

Confirmed false is the intentional Python parity contract: reformat_text never implicitly dedents, while direct fill_markdown(true) remains an explicit docstring API. Added Rust API docs, a shipped FormatOptions regression test, and the Unreleased changelog entry.
