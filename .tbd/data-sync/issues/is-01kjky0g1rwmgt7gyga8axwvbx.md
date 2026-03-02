---
type: is
id: is-01kjky0g1rwmgt7gyga8axwvbx
title: "3.5: Verify installation (uvx, uv tool install, pip install)"
kind: task
status: open
priority: 1
version: 8
spec_path: docs/project/specs/active/plan-2026-03-01-pypi-distribution.md
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
updated_at: 2026-03-02T18:01:26.692Z
---
## Notes

2026-03-02 validation: PyPI package exists at 0.2.5 and installs, but `flowmark --version` from `uvx flowmark-rs==0.2.5` reports `flowmark 0.2.5-dev.unknown+g784b6d0 (Rust port of flowmark-py 0.6.4; base v0.2.5)`, not a stable release string. Keep this open until release metadata/version embedding is corrected and install checks pass for uvx, uv tool install, and pip install paths.
