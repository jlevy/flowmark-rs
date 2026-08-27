---
type: is
id: is-01m10tcw81ta6bfvxa5xkj7707
title: "Address review: PR #81 — source-exact Markdown preservation"
kind: task
status: open
priority: 1
version: 14
labels:
  - pr-review
  - preservation
dependencies: []
child_order_hints:
  - is-01m10tdwjjbephf0kvjfdx00zs
  - is-01m10tdx0hzjkcbpsqpkn0gsx5
  - is-01m10tdxbw30eyj8xvqf7asmtk
  - is-01m10tdxpv9j8dbyy8v4zzh20z
  - is-01m10tdy1fgztpm44wttyy5ag5
  - is-01m10tdyd9kam1n26s9z1sdr8q
  - is-01m10tdyqvv8vj54pkfc8vpnnj
  - is-01m10tdz0v6w74nxzm80brdq9v
  - is-01m10tdzb5cq8bedqvm3qqe90w
  - is-01m10tdzkg4ct3eydzvh3jz06r
  - is-01m10tdzvj182n3p21qwwgv6b4
created_at: 2026-08-27T05:17:02.079Z
updated_at: 2026-08-27T05:41:53.269Z
---
Track and explicitly disposition every finding R1-R11 in the senior engineering review posted as PR #81 issue comment 5434439321. Cross-language observable fixes begin in the shared Python corpus; Rust-only API and error-path fixes use focused native tests. Close only after a published per-finding disposition map and final hosted CI.

## Notes

Disposition map published at PR #81 comment 5434858115. Fix commits: 26dbd7e and 366932b. Local full all-feature and no-default-feature suites, fmt, clippy, rustdoc, 484 exact shared cases, and generated-doc checks passed. Hosted CI is fully green across Ubuntu, macOS, Windows, library-only, MSRV, semver, coverage, audit, docs, formatting, mapping, README sync, and workflow tests. Parent remains open only for deferred R4 fmr-wksl, R5 fmr-xyd1, and R10 fmr-sh2b.
