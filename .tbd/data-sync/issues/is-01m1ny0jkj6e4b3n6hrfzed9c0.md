---
type: is
id: is-01m1ny0jkj6e4b3n6hrfzed9c0
title: Map every new Python 0.8.0 shared change ID
kind: bug
status: closed
priority: 1
version: 4
labels:
  - release-blocker
  - traceability
dependencies: []
parent_id: is-01m1nn49g1bds1ekgvg22r297p
created_at: 2026-09-04T10:04:16.365Z
updated_at: 2026-09-04T10:05:10.057Z
closed_at: 2026-09-04T10:05:10.056Z
close_reason: The shared change-ID map, upstream commit, and submodule contract are exact and the bidirectional traceability/conformance gate is green.
resolution: null
duplicate_of: null
---
The synchronized all-feature suite passes all 527 behavior dispositions but shared_traceability_matches_the_pinned_upstream fails because the Rust administrative map omits FM-CODE-SPAN-002, FM-FENCED-CODE-002/003/004, and FM-PRESERVE-CORE-002. Add each Python/Rust bead pair, rerun the exact traceability gate, and keep the parent gitlink, upstream_commit, and change-ID set identical before release.

## Notes

Added five missing change records with exact owners: fm-eo8r/fmr-4phz for code-span setext scope; fm-obf8/fmr-k9gi for three fenced-code contracts; fm-0dmt/fmr-zl5s for sentinel logical width. cargo test --locked --all-features --test test_conformance now passes all 6 tests, including 527 behavior dispositions and exact gitlink/hash/change-ID traceability.
