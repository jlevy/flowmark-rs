---
type: is
id: is-01m1npp7me3qha28h7a6pmdaff
title: Dry-run, publish, and verify flowmark-rs 0.4.0
kind: task
status: in_progress
priority: 1
version: 3
labels:
  - release
dependencies: []
parent_id: is-01m1nn49g1bds1ekgvg22r297p
created_at: 2026-09-04T07:56:17.421Z
updated_at: 2026-09-04T11:15:34.597Z
---
Run the documented release workflow dry-run first; only if green, publish crates.io, then PyPI and GitHub artifacts, verify versioned binaries and aliases on supported channels, update and verify Homebrew, and record artifact provenance and post-publication smoke results.

## Notes

Mandatory Rust release dry run succeeded: https://github.com/jlevy/flowmark-rs/actions/runs/33866506716 at merged main f1e9337e2d87ba614c231f0d17a2c181a3634117. All six native archives, checksum generation, cargo tests/package dry-run, five wheel builds, entrypoint validation, sdist, and PyPI no-upload validation passed; announce correctly skipped.
