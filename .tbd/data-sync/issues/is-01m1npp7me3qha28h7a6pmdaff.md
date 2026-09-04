---
type: is
id: is-01m1npp7me3qha28h7a6pmdaff
title: Dry-run, publish, and verify flowmark-rs 0.4.0
kind: task
status: in_progress
priority: 1
version: 2
labels:
  - release
dependencies: []
parent_id: is-01m1nn49g1bds1ekgvg22r297p
created_at: 2026-09-04T07:56:17.421Z
updated_at: 2026-09-04T11:07:47.741Z
---
Run the documented release workflow dry-run first; only if green, publish crates.io, then PyPI and GitHub artifacts, verify versioned binaries and aliases on supported channels, update and verify Homebrew, and record artifact provenance and post-publication smoke results.

## Notes

Source preparation is merged on main at f1e9337 after fully green PR #97. Starting the mandatory non-publishing release.yml dry run. Publication will stop on any dry-run failure or registry/tag collision.
