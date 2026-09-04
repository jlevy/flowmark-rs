---
type: is
id: is-01m1nqypq1r200m40scykbxgr0
title: Match path-aware invalid UTF-8 diagnostics in Rust
kind: bug
status: closed
priority: 1
version: 5
labels:
  - release-blocker
  - compatibility
dependencies:
  - type: blocks
    target: is-01m1npp71fxvqkntedbzbtxpqq
parent_id: is-01m1nn49g1bds1ekgvg22r297p
created_at: 2026-09-04T08:18:23.584Z
updated_at: 2026-09-04T08:22:28.148Z
closed_at: 2026-09-04T08:22:28.147Z
close_reason: Rust now matches the path-aware shared file diagnostic while retaining generic stdin behavior and atomicity.
resolution: null
duplicate_of: null
---
Consume the Python-owned fm-vc1q shared diagnostic for a named invalid UTF-8 input and restore at least the path information flowmark-rs v0.3.2 provided. Stdin must retain the generic parity diagnostic, file failures must remain atomic, and check/in-place/stdout paths must keep consistent exit status 2. This is the release-scoped diagnostic half of fmr-4wpn; batch continuation remains separately open.

## Notes

Evidence (2026-09-04): Rust was first demonstrated red against Python-owned preservation.core.invalid-file-no-mutation. Added a typed CLI annotation that carries paths through check, cached/uncached in-place, and stdout code paths without parsing anyhow display text; stdin stays generic. Shared named-file and stdin conformance cases pass, as does a native output/check integration test with exit status 2 and no mutation.
