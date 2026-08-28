---
type: is
id: is-01m12n1d12xj4jjmaczmn9etzm
title: "[follow-on PR] Verify and enforce flowmark idempotence corpus-wide"
kind: epic
status: open
priority: 1
version: 13
spec_path: docs/project/specs/active/plan-2026-08-27-idempotence-verification.md
labels:
  - idempotence
  - testing
dependencies: []
child_order_hints:
  - is-01m12n1dykp62bzxefadjy3y95
  - is-01m12n1ebwzyywcj2240p6zz33
  - is-01m12n1syzaz29j4vpngg4rf7n
  - is-01m12n1ta492s6xjy0sag092fk
  - is-01m11x43tnahwvqq0174fym25x
  - is-01m11x7nqfc6m3vkxqf2f3wjq0
  - is-01m11w0mtm43p57cn6as2shhws
  - is-01m12netgv484csz23xj4xzsgc
  - is-01m12nrbwk79bcy86t143b0nhb
  - is-01m13409cj1fx6f1bx2fykcccf
created_at: 2026-08-27T22:21:52.018Z
updated_at: 2026-08-28T02:43:24.178Z
---
FOLLOW-ON PR SCOPE. The one idempotence regression PR #81 introduced (fmr-0pxh) is fixed and that PR is clean on this axis: a full audit found 0 output-parity regressions and 0 cases that moved farther from Python, against 1,458 parity checks across 297 files that moved closer. Everything tracked under this epic is a LONGER-STANDING defect that also fails on the v0.3.2 release, so it does not block PR #81 and belongs in its own change. 67 ledger entries across 17 files remain in tests/idempotence_known_divergences.toml, gated by tests/test_idempotence_corpus.rs. Python is the less stable implementation (138 failing checks over 28 files against Rust's 67 over 17), so each defect is agreed and fixed upstream first, then replicated exactly in Rust. Spec: docs/project/specs/active/plan-2026-08-27-idempotence-verification.md
