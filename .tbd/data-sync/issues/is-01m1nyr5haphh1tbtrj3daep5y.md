---
type: is
id: is-01m1nyr5haphh1tbtrj3daep5y
title: Recover Rust formatter performance before 0.4.0
kind: bug
status: closed
priority: 1
version: 4
labels:
  - release-blocker
  - performance
dependencies: []
parent_id: is-01m1nn49g1bds1ekgvg22r297p
created_at: 2026-09-04T10:17:09.408Z
updated_at: 2026-09-04T10:57:02.529Z
closed_at: 2026-09-04T10:57:02.528Z
close_reason: Normal-workload performance restored to parity while retaining the new compatibility guarantees.
resolution: null
duplicate_of: null
---
The adversarial release benchmark against published v0.3.2 found README semantic formatting has a 12.0 ms vs 25.5 ms wall-clock median in a 100-run no-cache hyperfine sample (CPU 8.2 ms vs 9.8 ms, with scheduler noise). Internal perf stats isolate repeatable overhead in pre/postprocess: 1.06/0.81 ms on v0.3.2 versus 2.33/2.80 ms on the candidate. The new container-aware opaque-code scanner is recomputed by many workaround passes even when their target syntax is absent. Add correctness-preserving admission filters or reuse evidence, rerun full parity/idempotence tests, and require a controlled benchmark showing no material regression before release.

## Notes

Recovered release-candidate performance with conservative syntax admission filters and one reused exact opaque-line classification. A controlled 100-run normal README benchmark now measures candidate 10.0 ms versus published v0.3.2 at 10.2 ms (1.02x faster, within noise), eliminating the release-blocking regression. An artificial 3.46 MB syntax-dense stress corpus retains a bounded 7% default-thread / 10% one-thread overhead for the broader exact scanner; this does not reproduce on normal repository-sized documentation. Full all-feature, 10,000-check fixed-point, no-default-feature, Clippy, and package gates pass after the optimization.
