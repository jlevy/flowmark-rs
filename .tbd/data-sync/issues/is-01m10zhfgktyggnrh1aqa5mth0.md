---
type: is
id: is-01m10zhfgktyggnrh1aqa5mth0
title: "PR #81 suggestion: add preservation property and fuzz testing"
kind: task
status: open
priority: 2
version: 1
labels:
  - testing
  - preservation
dependencies: []
parent_id: is-01m10zgx85zvg3r07e73xj2733
created_at: 2026-08-27T06:46:55.756Z
updated_at: 2026-08-27T06:46:55.756Z
---
Add a small Rust property/fuzz target that composes delimiter and block-construct fragments and asserts no panic plus idempotence-or-explicit-fallback. Promote every semantic discovery into the language-neutral shared corpus. Do not add a dependency until SUPPLY-CHAIN-SECURITY.md is reviewed and cool-off policy is satisfied; a dependency-free deterministic generator is acceptable for this PR.
