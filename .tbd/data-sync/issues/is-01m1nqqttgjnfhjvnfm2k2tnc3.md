---
type: is
id: is-01m1nqqttgjnfhjvnfm2k2tnc3
title: Reject release tags that do not match Cargo package version
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
created_at: 2026-09-04T08:14:38.415Z
updated_at: 2026-09-04T08:15:57.463Z
closed_at: 2026-09-04T08:15:57.462Z
close_reason: Release planning now fails before packaging or publication when the requested tag and Cargo package version differ.
resolution: null
duplicate_of: null
---
resolve_release_plan.py accepts any real tag without comparing it to Cargo.toml. A mismatched dispatch could publish the Cargo/PyPI version under a different GitHub tag and build dev-version archives because build.rs ignores nonmatching overrides. Make the plan job fail before packaging unless the normalized real tag exactly equals v plus package.version, while preserving dry-run behavior; cover dispatch and push mismatches.

## Notes

Evidence (2026-09-04): added failing dispatch and tag-push cases using temporary Cargo manifests. resolve_release_plan.py now parses package.version with stdlib tomllib and rejects every real tag whose normalized value is not exactly v plus that version; dry-run remains manifest-independent. Eight focused planner tests and all 30 release-script tests pass.
