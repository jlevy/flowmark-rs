---
type: is
id: is-01kss3qmp1rgjwy2c2mrn1jjze
title: Parity corpus too small; add CommonMark spec gate
kind: task
status: open
priority: 2
version: 1
labels:
  - parity
dependencies: []
parent_id: is-01kss3qk7et1jycwp3snxgs7zd
created_at: 2026-05-29T05:36:23.744Z
updated_at: 2026-05-29T05:36:23.744Z
---
Curated fixtures missed the gaps the corpus sweep found. Implement a CommonMark spec-parity gate that diffs both binaries over all spec examples against a shrinking known-divergences baseline.
