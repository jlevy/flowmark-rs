---
type: is
id: is-01kss3qmp1rgjwy2c2mrn1jjze
title: Review and resolve 34 CommonMark shared-corpus divergences
kind: task
status: open
priority: 2
version: 2
labels:
  - parity
dependencies: []
parent_id: is-01kss3qk7et1jycwp3snxgs7zd
created_at: 2026-05-29T05:36:23.744Z
updated_at: 2026-08-27T05:06:08.024Z
---
The native Rust CommonMark spec gate is implemented and bidirectional. At upstream contract 19c840e, 34 active CommonMark 0.31.2 cases remain in tests/parity_corpus_known_divergences.toml. Review each against CommonMark semantics and Flowmark's formatter policy, resolve high-impact semantic, fixed-point, or cross-port failures first, and retain an exact ledger entry only when the divergence is explicitly approved. The test must reject unlisted failures, missing/deferred entries, and stale passing entries. Close only when every remaining case is fixed or has explicit reviewed approval consistent with the upstream language-neutral disposition.
