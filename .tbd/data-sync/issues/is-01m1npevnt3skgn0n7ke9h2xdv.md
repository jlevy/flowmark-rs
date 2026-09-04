---
type: is
id: is-01m1npevnt3skgn0n7ke9h2xdv
title: Preserve fenced-code info escapes and literal numbered content in Rust
kind: bug
status: closed
priority: 1
version: 5
labels:
  - release-blocker
  - parity
dependencies:
  - type: blocks
    target: is-01m1npp71fxvqkntedbzbtxpqq
parent_id: is-01m1nn49g1bds1ekgvg22r297p
created_at: 2026-09-04T07:52:15.800Z
updated_at: 2026-09-04T08:11:16.371Z
closed_at: 2026-09-04T08:11:16.371Z
close_reason: Rust-only fenced-code corruption and parity gaps are fixed locally with shared, unit, differential, and full-suite evidence; ready for the Rust release-prep PR.
resolution: null
duplicate_of: null
---
The on-demand deterministic property harness exposed Rust-only parity gaps against current Python that also exist in published Rust v0.3.2. Rust strips backslash escapes throughout a valid fence info string even though Python only decodes the first language token and preserves the extra field; Rust also applies normalize_numbered_lists inside literal fenced-code bodies, removing one authored space per pass. Add shared Python-generated conformance cases first, prove current Rust red, fix the behavior, and make the property evidence non-stale before release.

## Notes

Evidence (2026-09-04): implemented one container-aware fence scan shared by pre/post-processing passes; info-string ranges preserve suffix escapes while leaving the language token parser-visible; numbered-list normalization no longer edits fenced literals. Shared FM-FENCED-CODE-002/003/004 pass twice. A generated differential matrix of 840 combinations (6 containers, 4 fence runs, 7 info strings, 5 modes) matched current Python exactly and was Rust-idempotent. Focused formatter tests and the all-feature suite passed.
