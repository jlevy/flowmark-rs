---
type: is
id: is-01m12r94hb1se4rttk097t6zea
title: "Audit: PR #81 test changes are net stricter, with two small losses"
kind: task
status: open
priority: 2
version: 1
labels:
  - testing
dependencies: []
created_at: 2026-08-27T23:18:31.211Z
updated_at: 2026-08-27T23:18:31.211Z
---
Audit of whether PR #81's test changes are strict improvements in strictness and
completeness, or whether anything was relaxed.

## Net: stronger

    Rust #[test] count        663 -> 690   (+27)
    shared tryscript asserts  117 -> 144   (+27), plus 10 retained locally
    conformance corpus        none -> 776 cases with exact bytes, stderr,
                              exit status, filesystem trees, timeouts and
                              per-case idempotence
    CommonMark examples       none -> 652
    divergence ledger         none -> 34 entries, asserted bidirectionally

## The 86 deleted test files are accounted for

    73  relocated to the pinned submodule at the same path
     9  corner-cases corpus and its three drivers, now 5 conformance cases
        (parity.corner-cases.{default,auto,tight,loose,plaintext}), all passing
     4  fixtures inside ignored directories, deliberately removed (see below)

test_parity_red_green.rs was removed. It required a v0.2.0 binary via
FLOWMARK_OLD_BINARY and printed SKIP and returned when absent, so it asserted
nothing in CI. Not a loss.

cache-behavior.tryscript.md is absent from the shared set but retained locally
with all 10 assertions, which is correct since the incremental cache is
Rust-only.

## Two real losses found

1. Three vacuous assertions in file-discovery.tryscript.md (fmr-mv3c). The
   before: block deletes .venv, build, skip and nested/generated, and the
   fixtures are gone from the corpus, yet the document still asserts each is
   excluded from discovery. The assertions pass trivially and can no longer
   fail. flowmark-rs main committed all four fixtures and had no rm -rf, so
   this is a reduction from previous behavior.

2. Five --help listing assertions dropped (fmr-qsfc). main asserted that
   --cache-dir, --clear-cache, --no-cache, --perf-stats and --show-cache each
   appear in --help. Correctly removed from the shared doc since Python lacks
   the flags, but no Rust-local replacement asserts the listing. Flag behavior
   remains well covered by test_incremental_cache.rs and the local
   cache-behavior tryscript, so this is narrow.

Neither blocks PR #81: both assertions still pass, and both are losses of test
power rather than failures.

## Disclosure about this reviewer's own changes

Two tests were removed or weakened on branch claude/pr-review-comment-9vmwd9,
both authored in the same change, so neither reduces coverage relative to the
pre-existing baseline:

- default_goldens_are_unchanged_by_formatting was removed. Its premise was
  wrong: 258 of 673 CommonMark cases are tagged deferred, so their goldens are
  aspirational rather than current output, and the check measured the deferred
  backlog instead of stability.
- generated_documents_reach_a_fixed_point ships #[ignore]. The property
  genuinely fails on 60 of 10,000 generated cases, all traced to fmr-c6xs and
  fmr-uao3, so gating on it would assert something false. The two shapes are
  pinned by explicit tests instead, and fmr-3j93 tracks promoting the sweep once
  the ledger empties.

Added in the same change: the corpus idempotence gate (9,114 checks with a
67-entry bidirectional ledger), a no-abort sweep and an output-normalization
sweep (10,000 cases each), and four regression tests.
