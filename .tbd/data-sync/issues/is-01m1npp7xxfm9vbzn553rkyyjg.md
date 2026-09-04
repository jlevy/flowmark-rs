---
type: is
id: is-01m1npp7xxfm9vbzn553rkyyjg
title: Coordinate synchronized release ordering and rollback
kind: task
status: in_progress
priority: 1
version: 4
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m1npp7awqk8zn0t0xvds366j
parent_id: is-01m1nn49g1bds1ekgvg22r297p
created_at: 2026-09-04T07:56:17.725Z
updated_at: 2026-09-04T11:07:47.943Z
---
Keep Rust 0.4.0 and Python 0.8.0 version correspondence exact, prove the pinned Python commit is remotely fetchable, sequence crate/PyPI/GitHub/Homebrew publication before Python advertises the Rust pin, and stop on any real gate failure or collision.

## Notes

Release sequence is fixed and the exact Python gitlink is remotely fetchable. Python source landed first; Rust 0.4.0 will publish crate, PyPI/GitHub assets, then Homebrew; Python 0.8.0 publishes last so its discovery pin never advertises an unavailable Rust release. Each immutable channel is collision-checked and verified before advancing; reruns use the documented idempotent workflows.
