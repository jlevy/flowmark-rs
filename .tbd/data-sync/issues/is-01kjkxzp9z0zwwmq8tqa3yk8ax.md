---
type: is
id: is-01kjkxzp9z0zwwmq8tqa3yk8ax
title: Distribute flowmark-rs on PyPI via maturin
kind: epic
status: open
priority: 1
version: 30
spec_path: docs/project/specs/done/plan-2026-03-01-pypi-distribution.md
labels: []
dependencies: []
parent_id: is-01khq6kjwwq12m46jr9e3v2hfw
child_order_hints:
  - is-01kjky00nw931d7mv070g97ery
  - is-01kjky019g5pj2zbtg0t2chpzd
  - is-01kjky01sb6xg8c1se8q39g0np
  - is-01kjky029k4md2et72xtnyys4q
  - is-01kjky02sk6yjmdkdbfv9s3qyx
  - is-01kjky0d6j0qz8zh8mzf6ddf42
  - is-01kjky0dmya56xfcs5y9ayc3sx
  - is-01kjky0e3tve5arqcd4nxjvzra
  - is-01kjky0ek4xgdb322nchwga7bs
  - is-01kjky0f1rjbx4bmpwphsx548j
  - is-01kjky0fhs90zn6d0k13649xtk
  - is-01kjky0g1rwmgt7gyga8axwvbx
  - is-01kjky0vrtsy1yxg44beqgwdkz
  - is-01kjky0w72xfevdsnbenrdj7b6
  - is-01kjky0wne5akwt6zh4nj8bjjx
  - is-01kjky0x4kgbky2j8przbjzqz7
  - is-01kjky0xm8hnjqgr0h69h8jvh9
  - is-01kjky0y33stb0zm66gj3c7wyb
  - is-01kjky0yjx7zqaxw5rkpzf915z
  - is-01kjky0z1x73jj3gfj1374bpfw
  - is-01kjky0zgvdr0t91c2pn13jdfa
created_at: 2026-03-01T05:29:51.166Z
updated_at: 2026-03-02T18:22:14.454Z
---
## Notes

Snapshot 2026-03-01: 13/22 tasks closed, 9 open.

Validation on branch `claude/research-rust-cli-packaging-h4oT3` at commit `08ca8e3` and follow-up workflow/docs updates in progress:
- `maturin build --release --locked` succeeded (macOS arm64 wheel)
- `maturin sdist` succeeded
- local `pip install --no-index --find-links ... flowmark-rs` succeeded
- both `flowmark` and `flowmark-rs` commands run after install
- `twine check` passed for wheel and sdist
- `pypi.yml` includes Linux x86_64 smoke test and wheel entrypoint validation

Release framework direction:
- single `release.yml` orchestrator with dry-run support
- reusable channel workflows (`publish.yml`, `pypi.yml`) invoked via `workflow_call`
- rerun-safe channel publishing (duplicate-safe behavior)
- homebrew update gated after successful crates.io + PyPI publish

Operational blockers remain:
- final workflow behavior can only be end-to-end validated once merged to default branch
- first real PyPI publish via release tag is still pending

Configuration decision retained: no explicit GitHub environment is required for trusted publishing.

## Notes

Snapshot updated on 2026-03-02.

Current status: 18/22 child tasks closed; 4 open.
Open tasks are:
- fmr-foeq: final install verification for published release artifacts
- fmr-en1o: optional TestPyPI preflight
- fmr-sgqv: optional musllinux targets for PyPI
- fmr-b11z: optional python -m flowmark_rs wrapper

Operational state now differs from the original notes:
- pypi.yml is present on default branch and active
- release orchestration has been validated on main
- flowmark-rs 0.2.5 exists on PyPI and crates.io
- Homebrew tap formula is updated to 0.2.5

Remaining non-optional completion gate for this epic is fmr-foeq tied to next stable tag publish verification.
