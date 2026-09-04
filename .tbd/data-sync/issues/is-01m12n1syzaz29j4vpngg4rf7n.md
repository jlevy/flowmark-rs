---
type: is
id: is-01m12n1syzaz29j4vpngg4rf7n
title: "Defects B/C/D: setext, empty list item and link-label instability"
kind: task
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-27-idempotence-verification.md
labels:
  - idempotence
  - parity
dependencies: []
parent_id: is-01m12n1d12xj4jjmaczmn9etzm
created_at: 2026-08-27T22:22:05.278Z
updated_at: 2026-08-28T03:15:24.104Z
---
Three CommonMark shapes that do not reach a fixed point. B (0081, 0082, 0095): a setext heading whose content spans lines splits on the second pass, the trailing line escaping into a paragraph; both ports. C (0283): an empty ordered-list item is dropped on the first pass and the list renumbers on the second; both ports. D (0552): a link label containing a newline becomes a collapsed reference on the second pass; RUST ONLY, Python is already correct. For B and C the first pass is already arguably wrong, so decide intended bytes before targeting idempotence. Python first for B and C; D needs only a shared case pinning Python's existing bytes plus a Rust fix.

## Notes

Python twin: fm-k7yk (defects B and C). Rust additionally carries CommonMark 0552, the link label containing a newline, which Python does not.
