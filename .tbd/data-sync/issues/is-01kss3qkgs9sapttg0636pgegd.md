---
type: is
id: is-01kss3qkgs9sapttg0636pgegd
title: "PARITY GAP: --plaintext glues adjacent {% %} tags (test gap_a)"
kind: bug
status: closed
priority: 1
version: 2
labels:
  - parity
dependencies: []
parent_id: is-01kss3qk7et1jycwp3snxgs7zd
created_at: 2026-05-29T05:36:22.552Z
updated_at: 2026-05-29T05:59:17.250Z
closed_at: 2026-05-29T05:59:17.250Z
close_reason: "Fixed: paired-tag atomic patterns accept non-letter first char; gap_a green, testdoc plaintext byte-identical"
---
Python Wrap.WRAP preserves the source newline before a standalone closing tag in --plaintext; Rust reflows them together. Related to historical fmr-5u8i. Failing test: tests/test_known_parity_gaps.rs::gap_a_plaintext_tag_wrap. Needs plaintext WRAP newline-preservation work.
