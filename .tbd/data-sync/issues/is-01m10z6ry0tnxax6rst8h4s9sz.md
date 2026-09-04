---
type: is
id: is-01m10z6ry0tnxax6rst8h4s9sz
title: "PR #81: restore linear preservation scanning and pre-PR throughput"
kind: bug
status: closed
priority: 1
version: 4
labels:
  - preservation
  - performance
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T06:41:04.958Z
updated_at: 2026-08-27T08:13:32.575Z
closed_at: 2026-08-27T08:13:32.574Z
close_reason: The P1-P3 regressions are linear, exact-output parity is preserved, and representative release throughput is restored; residual durable performance work remains in fmr-95v6.
resolution: null
duplicate_of: null
---
The preservation implementation demonstrably violates the shared spec's linear-time claim and the project's historical <=5% fresh-run regression guardrail. On the existing 1,000-file/11 MB benchmark corpus, release --check --no-cache --threads 1 regressed from 0.905 s at pre-preservation commit c24284d6 to 1.238 s at PR head 366932b7 (+37%). Confirmed superlinear paths include the per-line scan over all block candidates in scanner.rs, per-pipe scan over all atomic ranges, repeated suffix scans for unmatched nested wikilinks, and comparison-sort arbitration instead of the specified/Python 8-pass radix sort. A 16,000-block adversarial file took 688.5 ms versus 63.2 ms at baseline; 8,000 unmatched wikilink openers took 112.7 ms versus 7.3 ms.

Acceptance: replace the confirmed nested joins/restarts with linear sweeps or stacks; port linear arbitration or reconcile the spec; audit remaining recognizers for repeated suffix rescans; retain exact shared-corpus behavior; show approximately linear N/2N/4N scaling on the adversarial shapes; and keep representative fresh-run throughput within 5% of the pre-preservation baseline on the same machine and release profile.

## Notes

Follow-up comments 5435358838 and 5435430891 addressed by ae6b736 and f833ce8. Replaced block exclusion with a two-pointer sweep, restored fixed-pass radix arbitration, made angle failure scanning amortized linear, and made unmatched wikilink scanning linear. Final exact v0.3.2 comparisons: 32k blocks 106 ms versus 115 ms release, angle pathology 33 ms versus 30 ms, wikilink pathology 11 ms versus 11 ms; ordinary testdoc x64 is about 4-5 percent typical overhead and construct-free prose is effectively flat. Exact pre-optimization output matched on 998 generated corpus files plus earlier seeded and parallel-path differentials.
