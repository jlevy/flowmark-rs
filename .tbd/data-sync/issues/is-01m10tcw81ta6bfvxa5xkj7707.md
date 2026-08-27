---
type: is
id: is-01m10tcw81ta6bfvxa5xkj7707
title: "Address review: PR #81 — source-exact Markdown preservation"
kind: task
status: open
priority: 1
version: 18
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
  - is-01m10z6ry0tnxax6rst8h4s9sz
  - is-01m10z6s8q6tj6rbpqde9mr9bg
  - is-01m11w0mtm43p57cn6as2shhws
created_at: 2026-08-27T05:17:02.079Z
updated_at: 2026-08-27T15:05:02.109Z
---
Track and explicitly disposition every finding R1-R11 in the senior engineering review posted as PR #81 issue comment 5434439321. Cross-language observable fixes begin in the shared Python corpus; Rust-only API and error-path fixes use focused native tests. Close only after a published per-finding disposition map and final hosted CI.

## Notes

Disposition map published at PR #81 comment 5434858115. Fix commits: 26dbd7e and 366932b. Local full all-feature and no-default-feature suites, fmt, clippy, rustdoc, 484 exact shared cases, and generated-doc checks passed. Hosted CI is fully green across Ubuntu, macOS, Windows, library-only, MSRV, semver, coverage, audit, docs, formatting, mapping, README sync, and workflow tests.

## Independent verification at head f833ce8 (reviewer, 2026-08-27)

Rebuilt v0.3.2, PR 04bd444, and PR f833ce8 in release mode in one session and re-ran the original reproducers and benchmark corpora.

Confirmed fixed:

- R1: all five callout-plus-inline inputs exit 0 AND retain content ("> [!NOTE]<v>" round-trips instead of dropping "<v>" as v0.3.2 did). The fix repairs the underlying data loss, not just the abort.
- R2: 1,500-case differential fuzz over a PUA-hostile alphabet in 4 modes produced 0 crashes (the same harness found a SIGABRT on the pre-fix head).
- R3: invalid UTF-8 exits 2. R8: zero matched files with --output exits 0 silently. R9: ENOTDIR reports "[Errno 20] Not a directory" with exit 2.
- P1 32k blocks 6,733 ms -> 254 ms (v0.3.2 241 ms). P2 angle prose 3,910 ms -> 41 ms (33 ms). P3 unclosed wikilinks 25,036 ms -> 74 ms (71 ms). P4 5 MB construct-free 436 ms -> 314 ms (287 ms). testdoc x64 1,055 ms -> 809 ms (772 ms), i.e. overhead down from about 37 percent to about 5 percent.
- Content retention: 0 cases in 1,500 where the head kept fewer input characters than v0.3.2.

Still open:

- R10 (fmr-sh2b) reproduces unchanged at f833ce8. The exact bytes the bead was waiting on are now recorded there, with a 7-byte minimal case and the --check impact.
- A related pre-existing (non-PR) idempotence and parity bug is filed as fmr-5vfu.
