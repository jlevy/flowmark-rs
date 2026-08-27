---
type: is
id: is-01m11w0mtm43p57cn6as2shhws
title: Unterminated fence with escaped backtick is non-idempotent and diverges from Python
kind: bug
status: open
priority: 2
version: 2
labels:
  - pr-review
  - idempotence
  - parity
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T15:04:32.852Z
updated_at: 2026-08-27T15:31:35.719Z
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

Substantially improved in c00f74b (branch claude/pr-review-comment-9vmwd9) but
NOT fully closed.

## Fixed

detect_opening_fence now applies the CommonMark 0.31.2 section 4.5 rule, so a
backtick fence opener whose info string contains a backtick is no longer
mistaken for a code fence. The escape now survives and the output reaches a
fixed point in one pass:

    ```\`     ->  "```\\`\n```\n"    (was "````\n" -> "````\n````\n")
    ```\`x    ->  "```\\`x\n```\n"
    ```a\`b   ->  "```a\\`b\n```\n"

The --check contract is restored for these inputs, and the sibling case
```\$`$ now matches Python exactly (closed as fmr-sh2b).

## Residual: still diverges from Python

Python emits "```\`\n" with no closing fence; Rust appends one.

Cause: the legacy PUA escape placeholder replaces `\`` with a non-backtick
scalar, which removes the very character that made the line a non-fence. comrak
then sees a valid fence opener and closes it. This only bites when the escaped
backtick is the ONLY backtick on the line; ```\$`$ keeps a literal backtick and
is therefore correct already.

## Options

1. Have the escape placeholder for a backtick stay fence-visible on a line that
   detect_opening_fence rejected for containing a backtick, so comrak still sees
   a non-fence.
2. Protect the whole false-fence line (leading run included) and restore it
   verbatim, which avoids depending on comrak's fence rule at all.

Option 1 is narrower; option 2 is more robust and fits the direction in fmr-oe3g
(consolidating the legacy token system into the preservation registry). Either
way, add the agreed bytes as a shared case first so both ports move together.

Behavior is pinned by false_fence_with_escaped_backtick_keeps_the_escape_and_is_idempotent
in src/formatter/filling.rs, which asserts what is fixed without asserting the
parity gap.
