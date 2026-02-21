---
type: is
id: is-01khzgtr5kqr3e8nk492nwssfn
title: "Phase 5.3: Update README.md and docs/publishing.md for binary releases"
kind: task
status: closed
priority: 2
version: 4
labels: []
dependencies:
  - type: blocks
    target: is-01khzgv06kkdjrwdtjny5tq9we
parent_id: is-01khq6kjwwq12m46jr9e3v2hfw
created_at: 2026-02-21T07:15:09.104Z
updated_at: 2026-02-21T11:19:31.160Z
closed_at: 2026-02-21T11:19:31.157Z
close_reason: Updated README.md with cargo binstall and platform list. Updated docs/publishing.md with binary release workflow section, target table, checksum verification, and workflow coordination description.
---
Update documentation to reflect new binary release installation methods:

1. **README.md** — Add pre-built binary download instructions alongside existing cargo install:
   - cargo install flowmark (from crates.io)
   - Download pre-built binary from GitHub Releases
   - cargo binstall flowmark (auto-discovers pre-built binaries)

2. **docs/publishing.md** — Add section explaining:
   - How release.yml and publish.yml coordinate (tag push -> release.yml builds binaries -> GitHub Release published event -> publish.yml publishes to crates.io)
   - The 6 target platforms
   - SHA256SUMS verification instructions

Reference: Phase 5 Step 5.3 of plan-2026-02-17-build-publishing.md
