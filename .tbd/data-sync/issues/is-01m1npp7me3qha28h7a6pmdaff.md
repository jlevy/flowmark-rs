---
type: is
id: is-01m1npp7me3qha28h7a6pmdaff
title: Dry-run, publish, and verify flowmark-rs 0.4.0
kind: task
status: closed
priority: 1
version: 5
labels:
  - release
dependencies: []
parent_id: is-01m1nn49g1bds1ekgvg22r297p
created_at: 2026-09-04T07:56:17.421Z
updated_at: 2026-09-04T11:34:15.760Z
closed_at: 2026-09-04T11:34:15.750Z
close_reason: "Completed: mandatory dry run green; crate 0.4.0, flowmark-rs PyPI 0.4.0, GitHub v0.4.0 assets, and Homebrew 0.4.0 all published and independently verified. Release runs 33866506716, 33867147690, and 33867336678; tap commits 9ec1dd9 and strict-audit correction 2ac6ca2."
resolution: null
duplicate_of: null
---
Run the documented release workflow dry-run first; only if green, publish crates.io, then PyPI and GitHub artifacts, verify versioned binaries and aliases on supported channels, update and verify Homebrew, and record artifact provenance and post-publication smoke results.

## Notes

Rust release channels verified. crates.io publish run https://github.com/jlevy/flowmark-rs/actions/runs/33867147690 succeeded; crates.io 0.4.0 is unyanked, checksum 6813b0bfb90978490915fa0d16e70ecdb54745aa41d93486207f2d52d02871e8, rust_version 1.85, trustpub SHA f1e9337e2d87ba614c231f0d17a2c181a3634117. Tagged release run https://github.com/jlevy/flowmark-rs/actions/runs/33867336678 succeeded. GitHub https://github.com/jlevy/flowmark-rs/releases/tag/v0.4.0 has tag at f1e9337 and six archives plus SHA256SUMS; independent download checksum verification passed for all six. PyPI flowmark-rs 0.4.0 exposes five platform wheels plus sdist, all unyanked. Fresh uvx installs of both flowmark and flowmark-rs entrypoints succeeded and report the synchronized 0.4.0 / Python 0.8.0 version.
