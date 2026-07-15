---
type: is
id: is-01kss6mfm0qsvbnvxb02scjtsh
title: "UPSTREAM (marko/flowmark) bug: escaped-backtick code span mis-pairs subsequent backticks, stripping spaces"
kind: bug
status: in_progress
priority: 2
version: 2
labels:
  - parity
  - upstream
dependencies: []
parent_id: is-01kss3qk7et1jycwp3snxgs7zd
created_at: 2026-05-29T06:27:05.984Z
updated_at: 2026-05-29T06:44:26.229Z
---
Root cause confirmed via marko parse: a line containing an escaped-backtick inline code span (e.g. the literal `\``) causes marko's CodeSpan regex to mis-pair the backticks — it parses '`\`` and updated `' as ONE code span, shifting every subsequent code-span boundary by one and absorbing the surrounding spaces (output: '`x`status from`y`' instead of '`x` status from `y`'). comrak (Rust) parses it correctly via the PUA escape pipeline, so Rust output is CORRECT and Python is buggy. Per the porting playbook + maintainer guidance, the fix belongs UPSTREAM in marko/flowmark (cannot push from flowmark-rs; tools scoped to this repo). ACTION FOR MAINTAINER: file a GitHub issue on jlevy/flowmark (and marko if applicable), add a test there asserting the space-preserving output, and that same test ports down to Rust where it already passes. Affects 24 corpus lines in plan-2026-02-17-exact-parity.md and 2026-05-28-sync-*.md. Rust-side documenting test: tests/test_known_parity_gaps.rs::gap_e2_python_backtick_space_bug_escalated (RED until upstream fixed; golden = Python's buggy output).
