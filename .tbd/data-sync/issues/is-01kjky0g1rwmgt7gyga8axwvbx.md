---
type: is
id: is-01kjky0g1rwmgt7gyga8axwvbx
title: "3.5: Verify installation (uvx, uv tool install, pip install)"
kind: task
status: open
priority: 1
version: 11
spec_path: docs/project/specs/done/plan-2026-03-01-pypi-distribution.md
labels: []
dependencies:
  - type: blocks
    target: is-01kjky0vrtsy1yxg44beqgwdkz
  - type: blocks
    target: is-01kjky0w72xfevdsnbenrdj7b6
  - type: blocks
    target: is-01kjky0wne5akwt6zh4nj8bjjx
  - type: blocks
    target: is-01kjky0xm8hnjqgr0h69h8jvh9
  - type: blocks
    target: is-01kjky0yjx7zqaxw5rkpzf915z
parent_id: is-01kjkxzp9z0zwwmq8tqa3yk8ax
created_at: 2026-03-01T05:30:17.528Z
updated_at: 2026-03-02T18:22:14.585Z
---
## Notes

2026-03-02 validation: PyPI package exists at 0.2.5 and installs, but `flowmark --version` from `uvx flowmark-rs==0.2.5` reports `flowmark 0.2.5-dev.unknown+g784b6d0 (Rust port of flowmark-py 0.6.4; base v0.2.5)`, not a stable release string. Keep this open until release metadata/version embedding is corrected and install checks pass for uvx, uv tool install, and pip install paths.

## Notes

Updated on 2026-03-02.

Verification summary:
- Published package flowmark-rs 0.2.5 installs via uvx, uv tool install, and pip install.
- Current published binaries still report a dev-flavored version string:
  flowmark 0.2.5-dev.unknown+g784b6d0 (...)
- Branch fix for release-tag version embedding is implemented and validated locally (stable string when FLOWMARK_RELEASE_TAG is set during build).

This bead remains open until the next tagged release is published and post-publish install checks confirm a stable non-dev version string across uvx, uv tool install, and pip install.
