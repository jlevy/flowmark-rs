---
type: is
id: is-01m10zhfgktyggnrh1aqa5mth0
title: "PR #81 suggestion: add preservation property and fuzz testing"
kind: task
status: closed
priority: 2
version: 2
labels:
  - testing
  - preservation
dependencies: []
parent_id: is-01m10zgx85zvg3r07e73xj2733
created_at: 2026-08-27T06:46:55.756Z
updated_at: 2026-08-27T15:31:19.826Z
closed_at: 2026-08-27T15:31:19.826Z
close_reason: "Implemented in c00f74b as tests/test_preservation_properties.rs: a dependency-free fixed-seed generator over construct-mix inputs. No-abort and the output normalization contract are gated on every run; the fixed-point sweep ships as an on-demand harness because that property still fails on shapes Python reproduces too (fmr-c6xs, fmr-uao3), which are pinned as explicit divergence tests. The harness found fmr-00e2, fmr-c6xs and fmr-uao3 within minutes of being written."
resolution: null
duplicate_of: null
---
Add a small Rust property/fuzz target that composes delimiter and block-construct fragments and asserts no panic plus idempotence-or-explicit-fallback. Promote every semantic discovery into the language-neutral shared corpus. Do not add a dependency until SUPPLY-CHAIN-SECURITY.md is reviewed and cool-off policy is satisfied; a dependency-free deterministic generator is acceptable for this PR.
