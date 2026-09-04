---
type: is
id: is-01m1npp71fxvqkntedbzbtxpqq
title: Run full Rust source, security, semver, and artifact validation
kind: task
status: closed
priority: 1
version: 5
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m1npp7awqk8zn0t0xvds366j
parent_id: is-01m1nn49g1bds1ekgvg22r297p
created_at: 2026-09-04T07:56:16.814Z
updated_at: 2026-09-04T11:07:47.157Z
closed_at: 2026-09-04T11:07:47.156Z
close_reason: Every source, security, semver, platform, mapping, and artifact validation gate passed.
resolution: null
duplicate_of: null
---
Run locked formatting, clippy, all-feature/no-default/doc tests, rustdoc, build/release, admin mapping tests, dependency audit/deny, cargo-semver-checks against crates.io, package listing/dry-run, archive and wheel entrypoint smoke tests, platform CI, and performance/binary-size checks.

## Notes

All local and hosted validation is green on PR #97 commit 7b0430d. Hosted gates passed: Ubuntu, macOS, Windows, all-feature, no-default-feature, Rust 1.85 MSRV, semver compatibility, coverage, Clippy, rustfmt, Markdown, rustdoc, dependency audit, mapping, README sync, workflow scripts, and DeepSource secrets (grade A, no inline findings). Local Cargo registry dry-run and isolated crate build/smoke also passed.
