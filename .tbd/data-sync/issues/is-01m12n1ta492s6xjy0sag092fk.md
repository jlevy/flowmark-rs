---
type: is
id: is-01m12n1ta492s6xjy0sag092fk
title: Promote the generated fixed-point sweep to a CI gate
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-27-idempotence-verification.md
labels:
  - idempotence
  - testing
dependencies: []
parent_id: is-01m12n1d12xj4jjmaczmn9etzm
created_at: 2026-08-27T22:22:05.635Z
updated_at: 2026-08-27T22:22:05.635Z
---
generated_documents_reach_a_fixed_point in tests/test_preservation_properties.rs currently ships #[ignore] because the property fails on 60 of 10,000 generated cases, all traced to fmr-c6xs and fmr-uao3. Once those land and the corpus ledger is empty, restore the excluded generator fragments and promote the sweep from an on-demand harness to a gate.
