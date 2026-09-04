---
type: is
id: is-01m15b4krz8tx27sqgs7qqt39c
title: Advance the upstream pin to the merged Python fixes
kind: chore
status: closed
priority: 1
version: 2
labels: []
dependencies: []
created_at: 2026-08-28T23:26:34.782Z
updated_at: 2026-08-28T23:45:54.180Z
closed_at: 2026-08-28T23:45:54.179Z
close_reason: null
resolution: null
duplicate_of: null
---
jlevy/flowmark#75 is merged (c2c4b5fe). Advance repos/flowmark and admin/port-coverage-mapping/shared-conformance.toml upstream_commit together — shared_traceability_matches_the_pinned_upstream enforces they move in lockstep.

Then remove commonmark.default.0532 from tests/parity_corpus_known_divergences.toml: it was listed because Python rewrote an inline link that shared a definition URL, which #75 fixes. Rust already produced the correct bytes, and the shared golden was moved to match in that PR. The conformance run will fail on a stale entry until it is removed.
