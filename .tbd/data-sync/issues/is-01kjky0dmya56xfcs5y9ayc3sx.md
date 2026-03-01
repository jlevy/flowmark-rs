---
type: is
id: is-01kjky0dmya56xfcs5y9ayc3sx
title: "2.2: Test pypi.yml workflow via workflow_dispatch (dry run)"
kind: task
status: open
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-03-01-pypi-distribution.md
labels: []
dependencies:
  - type: blocks
    target: is-01kjky0e3tve5arqcd4nxjvzra
parent_id: is-01kjkxzp9z0zwwmq8tqa3yk8ax
created_at: 2026-03-01T05:30:15.069Z
updated_at: 2026-03-01T19:39:42.902Z
---
## Notes

Blocked operationally until `pypi.yml` lands on the default branch.

Revalidated on 2026-03-01:
- `gh workflow run pypi.yml --ref claude/research-rust-cli-packaging-h4oT3`
- Result: `HTTP 404: workflow pypi.yml not found on the default branch`

This task can be completed immediately after merging this branch to `main`.
