---
type: is
id: is-01m12r8er7sz7h65gbjrq1n0ge
title: Restore --help listing assertions for the five Rust-only cache flags
kind: task
status: closed
priority: 3
version: 4
labels:
  - testing
dependencies: []
parent_id: is-01m12r94hb1se4rttk097t6zea
created_at: 2026-08-27T23:18:08.902Z
updated_at: 2026-08-28T00:06:31.468Z
closed_at: 2026-08-28T00:06:31.468Z
close_reason: "Implemented and verified: fixed list-specific pipe-continuation idempotence, restored strict shared discovery coverage, restored Rust-only cache help assertions, hardened the ledger gate, and passed the complete Linux, macOS, and Windows matrix."
resolution: null
duplicate_of: null
---
main's help.tryscript.md asserted that --cache-dir, --clear-cache, --no-cache, --perf-stats and --show-cache each appear in --help. Those lines were correctly dropped from the shared tryscript because Python has no such flags, but no Rust-local replacement asserts the help listing. Flag BEHAVIOR is still well covered (tests/test_incremental_cache.rs and the retained local tests/tryscript/cache-behavior.tryscript.md), so this is only the documentation-surface assertion. Add the five checks to tests/test_cli_help.rs or the local cache-behavior tryscript.

## Notes

Added a Rust-local CLI help assertion for cache-dir, clear-cache, no-cache, perf-stats, and show-cache. The focused test and full suite pass.
