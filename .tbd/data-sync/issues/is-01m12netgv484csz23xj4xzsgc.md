---
type: is
id: is-01m12netgv484csz23xj4xzsgc
title: "Defects E/F: Python-only blockquote and nested-list spacing oscillation"
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-27-idempotence-verification.md
labels:
  - idempotence
  - parity
dependencies: []
parent_id: is-01m12n1d12xj4jjmaczmn9etzm
created_at: 2026-08-27T22:29:11.834Z
updated_at: 2026-08-27T22:29:11.834Z
---
Two defect classes present in Python but NOT in Rust, found by the corpus audit. E (CommonMark 0228-0232): a blockquote holding a heading then prose oscillates between being split into two quotes and rejoined. F (0307, 0315): nested list indentation gains a blank line after the outer item on the second pass, turning tight into loose. Together these are most of the 84 checks and 14 files where Python is unstable and Rust is not. Because Python is the reference, fixing these will change bytes that Rust must then match, so they are sequenced before any Rust work. Tracked upstream as fm-3qkc.
