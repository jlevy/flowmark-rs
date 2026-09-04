---
type: is
id: is-01m1nv6p9j3x0g54xw757xns6s
title: Refresh stale shared-conformance counts and release status
kind: bug
status: closed
priority: 1
version: 4
labels:
  - release
  - documentation
dependencies: []
parent_id: is-01m1nn49g1bds1ekgvg22r297p
created_at: 2026-09-04T09:15:11.025Z
updated_at: 2026-09-04T10:57:03.421Z
closed_at: 2026-09-04T10:57:03.420Z
close_reason: Release status and conformance counts now match the final candidate.
resolution: null
duplicate_of: null
---
docs/port-status.md still claims 34 ledgered CommonMark differences and 484 exact passes, but the bidirectional ledger contains 32 entries after two main-branch fixes. The synchronized release adds five active shared cases, yielding 527 active cases: 495 exact plus 32 ledgered. Update every count, source pin, version/date/status claim, and completion statement during 0.4.0 release prep so public parity evidence matches the executable contract.

## Notes

Finalized all public parity evidence at Rust v0.4.0 / Python v0.8.0 and exact merged Python commit 7dfd0421d483a42dee29edef999f866b04294720. Counts agree with executable data: 527 active shared cases, 495 exact passes, and 32 inherited ledgered CommonMark divergences. Updated date, source pins, completion criteria, release wording, README, and generated mirrors.
