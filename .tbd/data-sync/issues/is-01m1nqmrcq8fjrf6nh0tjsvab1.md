---
type: is
id: is-01m1nqmrcq8fjrf6nh0tjsvab1
title: Propagate release tag into Rust archive builds
kind: bug
status: closed
priority: 1
version: 5
labels:
  - release-blocker
  - release
dependencies:
  - type: blocks
    target: is-01m1npp71fxvqkntedbzbtxpqq
parent_id: is-01m1nn49g1bds1ekgvg22r297p
created_at: 2026-09-04T08:12:57.617Z
updated_at: 2026-09-04T08:14:04.651Z
closed_at: 2026-09-04T08:14:04.650Z
close_reason: Tagged archive builds now receive the resolved tag and are guarded by a workflow regression test.
resolution: null
duplicate_of: null
---
The canonical runbook requires FLOWMARK_RELEASE_TAG to reach release builds so binaries report a stable version. release.yml resolves release_tag but its six package matrix builds do not export it. A documented manual v0.4.0 dispatch from main would therefore use the prior git tag plus commits-ahead metadata and package binaries reporting 0.4.0-dev.N+gHASH. Add exact workflow wiring and a regression test that distinguishes dry-run from tagged-release behavior.

## Notes

Evidence (2026-09-04): demonstrated the release.yml package Build step omitted FLOWMARK_RELEASE_TAG even though the plan job resolves it and build.rs needs it to suppress commits-ahead dev metadata. Added a structural workflow regression test first and observed it fail. The package matrix Build step now exports needs.plan.outputs.release_tag. Focused test is green and all 28 release-script unit tests pass (loopback-backed registry stubs included).
