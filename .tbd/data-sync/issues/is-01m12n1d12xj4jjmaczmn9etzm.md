---
type: is
id: is-01m12n1d12xj4jjmaczmn9etzm
title: Verify and enforce flowmark idempotence corpus-wide
kind: epic
status: open
priority: 1
version: 11
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
created_at: 2026-08-27T22:21:52.018Z
updated_at: 2026-08-27T22:34:24.531Z
---
Idempotence is flowmark's core safety promise but is not verified across the option space. A first audit of 1,519 shipped documents across a six-mode matrix found 68 of 9,114 checks not reaching a fixed point, spanning 20 files including goldens, the reference testdoc, and ordinary CommonMark. Establish a corpus-wide gate with an exact known-divergence ledger, and fix or pin every defect. See the spec for the audit table and defect list.
