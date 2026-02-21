---
type: is
id: is-01khzgtceq57c9ca2xj4936yv6
title: "Phase 5.1: Create release.yml workflow for cross-platform binary builds"
kind: task
status: closed
priority: 1
version: 5
labels: []
dependencies:
  - type: blocks
    target: is-01khzgtr5kqr3e8nk492nwssfn
  - type: blocks
    target: is-01khzgv06kkdjrwdtjny5tq9we
parent_id: is-01khq6kjwwq12m46jr9e3v2hfw
created_at: 2026-02-21T07:14:57.109Z
updated_at: 2026-02-21T11:18:30.350Z
closed_at: 2026-02-21T11:18:30.348Z
close_reason: Created .github/workflows/release.yml with prerelease detection, 6-target matrix build, and SHA256SUMS checksum job. Modeled on casey/just.
---
Create .github/workflows/release.yml with three jobs modeled on casey/just's release.yaml:

1. **prerelease** job: Detect if tag is stable release vs prerelease (tags matching ^[0-9]+\.[0-9]+\.[0-9]+$ are releases; all others are prereleases).

2. **package** matrix job: Build 6 targets:
   - x86_64-unknown-linux-musl (ubuntu-latest, native with musl-tools)
   - aarch64-unknown-linux-musl (ubuntu-latest, cross via gcc-aarch64-linux-gnu)
   - x86_64-apple-darwin (macos-latest, --target on ARM runner)
   - aarch64-apple-darwin (macos-latest, native)
   - x86_64-pc-windows-msvc (windows-latest, native)
   - aarch64-pc-windows-msvc (windows-latest, rustup target add)

   Each job: install deps, cargo build --release --locked --target $TARGET, create archive (.tar.gz for Unix, .zip for Windows) containing binary + LICENSE + README.md, upload to GitHub Release via softprops/action-gh-release@v2.

3. **checksum** job: Download all artifacts, generate SHA256SUMS, upload to release.

Global RUSTFLAGS: --deny warnings --codegen target-feature=+crt-static
Trigger: push: tags: ['*']
Archive naming: flowmark-vX.Y.Z-TARGET.{tar.gz,zip} (cargo-binstall compatible)

Reference: casey/just release.yaml and Phase 5 of plan-2026-02-17-build-publishing.md
