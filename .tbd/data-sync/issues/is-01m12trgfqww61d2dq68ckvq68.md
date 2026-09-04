---
type: is
id: is-01m12trgfqww61d2dq68ckvq68
title: Normalize corpus idempotence ledger paths on Windows
kind: bug
status: closed
priority: 1
version: 4
delegate: codex@spud10.local
labels:
  - testing
  - windows
dependencies: []
parent_id: is-01m12r94hb1se4rttk097t6zea
hold: null
hold_until: null
created_at: 2026-08-28T00:01:52.118Z
updated_at: 2026-08-28T00:06:31.475Z
started_at: 2026-08-28T00:01:58.660Z
closed_at: 2026-08-28T00:06:31.475Z
close_reason: "Implemented and verified: fixed list-specific pipe-continuation idempotence, restored strict shared discovery coverage, restored Rust-only cache help assertions, hardened the ledger gate, and passed the complete Linux, macOS, and Windows matrix."
resolution: null
duplicate_of: null
---
Hosted Windows CI exposed that the new corpus idempotence gate rendered observed repository paths with backslashes while the platform-neutral ledger uses forward slashes. All 67 known divergences were therefore misclassified as new failures. Normalize relative path components to slash-separated ledger keys, add a cross-platform assertion, and require the complete hosted matrix to pass.

## Notes

Normalized repository-relative ledger keys by path components and added a portable separator assertion. Local two-test idempotence target passes; awaiting the complete hosted Windows rerun.
