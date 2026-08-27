---
type: is
id: is-01m12n1ebwzyywcj2240p6zz33
title: "Defect A: wrapped line beginning with '=' becomes a setext underline"
kind: bug
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-27-idempotence-verification.md
labels:
  - idempotence
  - parity
  - wrapping
dependencies: []
parent_id: is-01m12n1d12xj4jjmaczmn9etzm
created_at: 2026-08-27T22:21:53.403Z
updated_at: 2026-08-27T22:29:24.403Z
---
Content corruption, present in BOTH Python and Rust. A wrapped line starting with = is reparsed as a setext H1 underline: list text is promoted to a heading, the = is consumed, and following text escapes the list. Reproducer: printf -- '- alpha beta gamma delta epsilon word =\n' | flowmark --width 20 - run twice. Also fires on testdoc.orig.md and on docs/project/specs/done/plan-2026-02-19-parity-review-and-playbook-sync.md at narrow widths. Likely cause is the wrap-time escape set: both ports guard -, *, +, > and # (Rust MD_SPECIALS_PAT is ^([-*+>]|#+)$) but not =, and neither guards runs like --- or ===. DO NOT FIX YET: agree intended bytes, pin a shared case, fix in Python, then replicate exactly in Rust.
