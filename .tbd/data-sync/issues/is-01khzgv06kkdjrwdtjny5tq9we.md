---
type: is
id: is-01khzgv06kkdjrwdtjny5tq9we
title: "Phase 5.4: Test full release cycle with a patch release"
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01khq6kjwwq12m46jr9e3v2hfw
created_at: 2026-02-21T07:15:17.330Z
updated_at: 2026-02-21T07:15:17.330Z
---
Test the complete release pipeline end-to-end:

1. Merge release workflow PR to main
2. Bump version in Cargo.toml (e.g. to 0.2.2)
3. Update CHANGELOG.md with release entry
4. Commit, push, merge to main
5. Tag and push: git tag v0.2.2 && git push origin v0.2.2
6. Watch release.yml: gh run list --workflow=release.yml --limit 1
7. Verify GitHub Release has all 6 archives + SHA256SUMS
8. Watch publish.yml: gh run list --workflow=publish.yml --limit 1
9. Verify crates.io: https://crates.io/crates/flowmark
10. Test installation:
    - cargo install flowmark
    - cargo binstall flowmark
    - Direct binary download
11. Verify flowmark --version shows correct version and parity info

This is a manual step that requires all previous Phase 5 work to be merged.

Reference: Phase 5 Step 5.4 of plan-2026-02-17-build-publishing.md
