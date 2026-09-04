---
type: is
id: is-01m1npp7awqk8zn0t0xvds366j
title: Prepare flowmark-rs 0.4.0 release metadata and PR
kind: task
status: closed
priority: 1
version: 6
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m1npp7me3qha28h7a6pmdaff
parent_id: is-01m1nn49g1bds1ekgvg22r297p
created_at: 2026-09-04T07:56:17.115Z
updated_at: 2026-09-04T11:07:47.549Z
closed_at: 2026-09-04T11:07:47.548Z
close_reason: Release metadata and fixes landed on main only after every required PR gate passed.
resolution: null
duplicate_of: null
---
Advance the exact Python submodule pin to the final flowmark 0.8.0 release commit, bump Cargo and parity metadata to 0.4.0/0.8.0, update lockfile, changelog, version history, sync checklist and release notes, commit, push, open the release-prep PR, and land only after all required checks pass.

## Notes

PR #97 passed the complete hosted matrix on immutable head 7b0430d with no inline review findings and DeepSource grade A, then squash-merged to main as f1e9337e2d87ba614c231f0d17a2c181a3634117. A fresh recursive clone from GitHub resolved Python 7dfd0421d483a42dee29edef999f866b04294720, Homebrew 6567a9f, and the porting playbook d24760a from their configured remotes.
