---
type: is
id: is-01m1nycvvwjz6bdr0bxheph5q8
title: Audit 0.4.0 crate payload and packaged CLI behavior
kind: task
status: closed
priority: 1
version: 3
labels:
  - release
  - packaging
dependencies: []
parent_id: is-01m1nn49g1bds1ekgvg22r297p
created_at: 2026-09-04T10:10:59.067Z
updated_at: 2026-09-04T10:11:10.989Z
closed_at: 2026-09-04T10:11:10.972Z
close_reason: Exact crate payload and standalone packaged behavior pass; composition has no new release blocker relative to v0.3.2.
resolution: null
duplicate_of: null
---
Run cargo publish --dry-run on the exact 0.4.0 candidate; inventory the crate; prove repositories/shared submodules are excluded; build from the unpacked package with no VCS or submodule; smoke --version, --help, --docs, --skill, stdin, and an isolated in-place file operation. Compare package composition with published 0.3.2 and disposition any inherited development-only files.

## Notes

FLOWMARK_RELEASE_TAG=v0.4.0 cargo publish --dry-run --locked --allow-dirty passed: 154 files, 1.2 MiB unpacked / 298.1 KiB compressed. The payload has no repos/ submodule or shared upstream fixtures; bundled docs and skill are present. Built the unpacked source with no Git/submodule and verified stable 'flowmark 0.4.0 (Rust port of flowmark-py 0.8.0; base v0.4.0)', help, 754-line docs, 0.4.0/0.8.0 skill pins, stdin wrapping, and in-place no-backup formatting. The package includes the same development helpers/tests already present in published 0.3.2; this is inherited, contains no secrets, adds no compatibility risk, and is left unchanged to avoid unrelated release-scope churn.
