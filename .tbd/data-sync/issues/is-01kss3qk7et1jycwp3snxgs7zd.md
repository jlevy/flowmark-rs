---
type: is
id: is-01kss3qk7et1jycwp3snxgs7zd
title: "EPIC: Exact CLI parity with Python flowmark v0.7.0 (v0.3.0 gate)"
kind: epic
status: open
priority: 1
version: 7
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
created_at: 2026-05-29T05:36:22.254Z
updated_at: 2026-05-29T06:27:05.984Z
---
Close all known CLI-output divergences from Python flowmark v0.7.0. Found via 2026-05-29 corpus sweep (162 files, 272->82 +/- lines so far). FIXED on branch claude/modest-edison-TfyfF (tests in tests/test_known_parity_gaps.rs, all green): gap_b html-comment blank line, gap_c1 tasklist->thematic break, gap_c2 listitem code-block spacing, gap_f para->blockquote, gap_g code->para, loose-list per-item tightness, gap_h list->list + list->blockquote. REMAINING (see children).
