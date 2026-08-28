---
type: is
id: is-01kss3qk7et1jycwp3snxgs7zd
title: "EPIC: Exact CLI parity with Python flowmark v0.7.0 (v0.3.0 gate)"
kind: epic
status: open
priority: 1
version: 9
labels:
  - parity
dependencies: []
child_order_hints:
  - is-01kss3qkgs9sapttg0636pgegd
  - is-01kss3qkt64d0bcxvw4jkaqekh
  - is-01kss3qm3ja86hnmscgpbnnt71
  - is-01kss3qmd08pep1hrspzhggbth
  - is-01kss3qmp1rgjwy2c2mrn1jjze
  - is-01kss6mfm0qsvbnvxb02scjtsh
  - is-01m136m4xkr207cmyp7113kk95
created_at: 2026-05-29T05:36:22.254Z
updated_at: 2026-08-28T03:29:12.115Z
---
Close all known CLI-output divergences from Python flowmark v0.7.0. 2026-05-29 deep parity push: corpus 272 -> 24 +/- lines. ALL 9 fixable corpus classes FIXED on branch claude/modest-edison-TfyfF (tests/test_known_parity_gaps.rs gap_a..gap_h all green; full suite 582/0; CI parity gating enabled). Remaining: (1) fmr-qmd8 one UPSTREAM Python/marko bug (escaped-backtick mis-pairs subsequent backticks, strips spaces) - Rust is correct, fix belongs upstream, 24 corpus lines; (2) fmr-rz9f CommonMark spec gate (process); (3) fmr-5u8i Python plaintext md-awareness quirk (intentionally matched). Epic effectively complete for v0.3.0 modulo the upstream-bug decision.
