---
type: is
id: is-01m13f89zd7hhz8cpsc95hndn6
title: Three test files hard-code target/debug, so cargo test --release fails spuriously
kind: bug
status: open
priority: 2
version: 1
labels:
  - testing
dependencies: []
parent_id: is-01khq6kjwwq12m46jr9e3v2hfw
created_at: 2026-08-28T06:00:01.261Z
updated_at: 2026-08-28T06:00:01.261Z
---
`tests/test_known_parity_gaps.rs`, `tests/test_cli_file_discovery.rs` and
`tests/test_skill_cli.rs` locate the CLI as `CARGO_MANIFEST_DIR/target/debug/flowmark`.
That path is only correct for a debug-profile run. `cargo test --release` — the fast
default for this repo, whose suite is slow in debug — fails 14 tests in
`test_known_parity_gaps.rs` alone with:

    Rust binary not found at .../target/debug/flowmark. Build with `cargo build --all-features`.

Two things make it worse than a stale message:

- **It hides real failures.** Cargo stops running further test binaries after a target
  fails, so the release run reported "334 passed, 14 failed" and never executed the
  remaining suites. Any genuine regression behind that point is invisible.
- **It looks like a product defect.** I hit this while verifying a Dependabot bump on a
  clean worktree and had to run a control against unpatched `main` to prove the
  dependency was not at fault. Anyone with a stale `target/debug/flowmark` lying around
  from earlier work will not reproduce it, which makes it worse to diagnose, not better.

CI is unaffected: it runs `cargo test --locked --all-features` in the debug profile,
after `cargo build --locked --all-features`, so the binary is always present.

## Fix

The repo already has the correct idiom two files over. `tests/test_conformance.rs` and
`tests/test_tryscript_golden.rs` both use:

    PathBuf::from(env!("CARGO_BIN_EXE_flowmark"))

Cargo sets that for integration tests and points it at the binary for the profile
actually being built, so it is correct in both debug and release and needs no
prerequisite build step. Switch the three stragglers to it and delete the
`bin.exists()` assertion, which only exists to explain the hard-coded path.

Keep the "fail loudly rather than skip" intent of the current comment — with
`CARGO_BIN_EXE_*` the binary cannot be missing, so the property is stronger, not weaker.
