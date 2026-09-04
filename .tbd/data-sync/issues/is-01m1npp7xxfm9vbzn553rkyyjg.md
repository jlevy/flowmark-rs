---
type: is
id: is-01m1npp7xxfm9vbzn553rkyyjg
title: Coordinate synchronized release ordering and rollback
kind: task
status: in_progress
priority: 1
version: 5
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m1npp7awqk8zn0t0xvds366j
parent_id: is-01m1nn49g1bds1ekgvg22r297p
created_at: 2026-09-04T07:56:17.725Z
updated_at: 2026-09-04T11:34:21.528Z
---
Keep Rust 0.4.0 and Python 0.8.0 version correspondence exact, prove the pinned Python commit is remotely fetchable, sequence crate/PyPI/GitHub/Homebrew publication before Python advertises the Rust pin, and stop on any real gate failure or collision.

## Notes

Release ordering executed safely through Rust channels: Python prep merged first, Rust prep merged second, mandatory Rust dry-run green, crates.io published/verified, tagged Rust PyPI+GitHub release published/verified, then Homebrew updated and tested. Homebrew initially exposed current brew strict-audit rejection of the inherited explicit version line; corrected via follow-up tap commit 2ac6ca2, then strict audit and brew test passed. Python 0.8.0 publication remains last.
