---
type: is
id: is-01m12nrbwk79bcy86t143b0nhb
title: "Regression: wrapped continuation line starting with a pipe flips list spacing"
kind: bug
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-27-idempotence-verification.md
labels:
  - idempotence
  - regression
  - preservation
dependencies: []
parent_id: is-01m12n1d12xj4jjmaczmn9etzm
created_at: 2026-08-27T22:34:24.531Z
updated_at: 2026-08-28T00:06:31.446Z
closed_at: 2026-08-28T00:06:31.445Z
close_reason: "Implemented and verified: fixed list-specific pipe-continuation idempotence, restored strict shared discovery coverage, restored Rust-only cache help assertions, hardened the ledger gate, and passed the complete Linux, macOS, and Windows matrix."
resolution: null
duplicate_of: null
---
THE ONLY IDEMPOTENCE REGRESSION PR #81 INTRODUCES. Everything else the corpus
audit found predates the PR. This is the one item blocking a no-regression land.

## Minimal reproducer (3 lines, --width 40)

    - Bead: fmr-hr43 | Scope: Phase 8.5/8.6 | Repo: playbook
    - Depends on: WI-1, WI-4
    - Findings: F1-F12 (12 lessons + anti-patterns)

Pass 1 wraps the first item so a continuation line begins with a pipe:

    - Bead: fmr-hr43 | Scope: Phase 8.5/8.6
      | Repo: playbook
    - Depends on: WI-1, WI-4
    - Findings: F1-F12 (12 lessons +
      anti-patterns)

Pass 2 inserts a blank line before the third item, partially converting the
tight list to loose.

## Status by implementation

    flowmark-rs v0.3.2:   STABLE
    Python flowmark 9e9fd7c: STABLE
    PR #81 head f833ce8:  NOT idempotent
    branch c00f74b:       NOT idempotent

So it is both a regression and a divergence from the reference implementation.

## Trigger

Confirmed by substitution:

- pipes replaced with commas -> stable, so the pipe is the trigger
- only two list items -> stable, so a third item is required
- an unrelated list whose continuation happens to contain a pipe -> stable

The specific shape is a continuation line that BEGINS with '|' inside a list,
followed by at least one more item. That is what the new pre-parse scanner's
structural-pipe and table detection reacts to (has_structural_pipe /
inline_scopes in src/preservation/scanner.rs), so the second pass classifies the
region differently and list spacing changes.

## Real-world hit

docs/project/specs/done/plan-2026-02-19-parity-review-and-playbook-sync.md at
--width 40; the file is stable on v0.3.2. Ledgered in
tests/idempotence_known_divergences.toml, which will fail once this is fixed.

## Ask

Fix before merging PR #81. Python is already correct here, so unlike the other
idempotence defects this needs no upstream decision: make Rust match Python.

## Notes

Integrated the active-PR blocker fix, then strengthened the shared adjacency golden after review exposed that the initial predicate was too broad. The final predicate is list-item-specific; both exact shared cases and the corpus-wide idempotence gate pass.
