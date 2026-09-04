---
type: is
id: is-01m12n1dykp62bzxefadjy3y95
title: Add corpus-wide idempotence gate and known-divergence ledger
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-27-idempotence-verification.md
labels:
  - idempotence
  - testing
dependencies: []
parent_id: is-01m12n1d12xj4jjmaczmn9etzm
created_at: 2026-08-27T22:21:52.979Z
updated_at: 2026-08-27T22:35:50.035Z
---
Add tests/test_idempotence_corpus.rs asserting fixed-point (P1) and golden stability (P2) over every shipped corpus across the six-mode matrix, plus tests/idempotence_known_divergences.toml seeded from the audit. Assert the ledger exactly in both directions so it shrinks and cannot rot. Use the library directly rather than spawning the CLI; measure runtime and move behind an ignored gate only if the default suite gets too slow.
