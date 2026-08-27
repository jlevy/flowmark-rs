---
type: is
id: is-01m10q7fa0mmt8ym0mac5yrthq
title: Upgrade repository tbd integration to 0.8.1
kind: chore
status: closed
priority: 1
version: 3
labels: []
dependencies: []
created_at: 2026-08-27T04:21:39.263Z
updated_at: 2026-08-27T04:33:29.252Z
closed_at: 2026-08-27T04:33:29.248Z
close_reason: "Upgraded the Flowmark-rs root managed integration and exact fallback to tbd 0.8.1 in PR #81; full Rust, mapping, workflow, and all 15 CI checks passed."
resolution: null
duplicate_of: null
---
Run the official tbd 0.8.1 setup upgrade on the active Flowmark Rust-port PR branch. Refresh checked-in launchers, skills, agent integration, repository metadata, and the complete Rust guideline catalog; verify every root executable fallback pins get-tbd@0.8.1; review the generated diff; run tbd doctor plus the repository Rust validation gates; commit and push to PR #81. Do not modify the rust-porting-playbook submodule's own repository setup in this root upgrade. The maintainer explicitly approved a first-party exception to the normal 14-day cool-off for this maintained package; record the exact package version, registry integrity, advisory checks, and approval in the PR.
