---
type: is
id: is-01m1npp6ekdnvppnhj4w0ayp0m
title: Adversarially review Rust changes since v0.3.2
kind: task
status: closed
priority: 1
version: 6
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m1npp71fxvqkntedbzbtxpqq
parent_id: is-01m1nn49g1bds1ekgvg22r297p
created_at: 2026-09-04T07:56:16.210Z
updated_at: 2026-09-04T10:57:02.983Z
closed_at: 2026-09-04T10:57:02.982Z
close_reason: Adversarial review complete; every new release blocker is fixed and independently gated.
resolution: null
duplicate_of: null
---
Audit every production, test, CI, packaging, submodule, and dependency change since v0.3.2. Check exact Python parity, CLI and filesystem compatibility, Unicode and byte offsets, preservation/scanner bounds, panic/error paths, performance, unsafe code, and release workflow gates. Record each finding as a child bead and resolve all release blockers.

## Notes

Completed the full v0.3.2-to-candidate audit across production code, public API, CLI/filesystem behavior, tests, CI, packaging, dependencies, submodules, Unicode/byte offsets, scanner bounds, fixed-point behavior, and performance. Verified exact Python 0.8.0 parity over 1,677 tracked Markdown files in six modes with zero unexplained mismatches or regressions. Release blockers were isolated and fixed under child beads: archive tag propagation/version validation, path-aware invalid UTF-8, fenced/indented-code rewrite safety, setext boundaries, sentinel width, and performance. The exact package builds outside Git/submodules. Only inherited bidirectional divergence ledgers remain; hosted PR validation is tracked separately.
