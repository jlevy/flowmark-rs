---
type: is
id: is-01m11w0mtm43p57cn6as2shhws
title: Unterminated fence with escaped backtick is non-idempotent and diverges from Python
kind: bug
status: open
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-08-27-idempotence-verification.md
labels:
  - pr-review
  - idempotence
  - parity
dependencies: []
parent_id: is-01m12n1d12xj4jjmaczmn9etzm
created_at: 2026-08-27T15:04:32.852Z
updated_at: 2026-08-27T22:35:51.924Z
---
Found while recovering the exact reproducer for R10 (fmr-sh2b). This one is
NOT a PR #81 regression: it reproduces on the shipped v0.3.2 release binary,
so it predates the preservation work and is out of scope for that PR.

## Reproducer: 5 bytes

Input (no trailing newline), hex `60 60 60 5c 60`:

    ```\`

Behavior:

- Python 9e9fd7c: "```\`\n"            -> stable, backslash preserved
- Rust v0.3.2:    "````\n" -> "````\n````\n" -> stable only after 2 passes
- Rust f833ce8:   "````\n" -> "````\n````\n" -> stable only after 2 passes

Two defects in one input:

1. Idempotence: formatting the output again appends another ```` fence line,
   so `flowmark --check` exits 1 on flowmark's own output (breaks pre-commit
   and CI gates that format then verify).
2. Parity: Rust turns the escaped backtick into a fourth fence character and
   drops the backslash; Python preserves "```\`" exactly.

## Suggested handling

Add a shared malformed-fallback golden for unterminated fences containing
escapes so Python and Rust agree, then fix the Rust escape handling. The
sibling PR-regression case is tracked in fmr-sh2b; both belong to the same
"unterminated fence + escape" family and are probably one fix.

## Notes

Partial fix is active on PR #81 as commit 6b41887: the escape survives and output reaches a fixed point in one pass. Exact Python parity remains open because Rust still appends a closing fence to this incomplete construct. This residual overlaps token-system consolidation bead fmr-oe3g and was explicitly deferred in PR comment 5442270412.
