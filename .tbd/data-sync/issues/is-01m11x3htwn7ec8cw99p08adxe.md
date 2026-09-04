---
type: is
id: is-01m11x3htwn7ec8cw99p08adxe
title: Blank-line normalization strips whitespace-only lines inside code fences
kind: bug
status: closed
priority: 2
version: 5
labels:
  - pr-review
  - idempotence
  - parity
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T15:23:36.668Z
updated_at: 2026-08-27T16:42:40.328Z
closed_at: 2026-08-27T16:42:40.325Z
close_reason: "Fixed on PR #81 by preserving whitespace-only fenced content; focused property coverage, full local gates, and all 16 hosted checks passed."
resolution: null
duplicate_of: null
---
Found by the new generated-input property test (fmr-htol) and fixed in the same change.

## Reproducer: 5 bytes

Input `` ```\n `` followed by a single space (hex `60 60 60 0a 20`):

- v0.3.2:        "```\n```\n"    idempotent, but drops the authored space line
- PR f833ce8:    "```\n\n```\n"  NOT idempotent; second pass collapses to "```\n```\n"
- Python 9e9fd7c:"```\n \n```\n" idempotent, space line preserved (correct)

## Cause

normalize_blank_lines applied the `(?m)^[ \t]+$` blanking to the whole rendered
document, including inside fenced code blocks. Fenced content is literal, so a
whitespace-only line there is authored data, not stray trailing whitespace.

Before this PR the `text.trim()` in fill_markdown removed the trailing space
before it ever reached comrak, which masked the bug. The PR removed that trim
(correctly, for source-exact preservation), exposing it as a new
non-idempotence: the blanked line survives as an empty line and the next format
run collapses it.

## Fix

normalize_blank_lines now runs through transform_outside_code_fences, so fence
bodies are left alone. Rust output now matches Python byte-for-byte on this
input and reaches a fixed point in one pass.

## Notes

Reopened: Side-branch fixes are validated but not yet integrated into the active PR #81 branch; reopen until push and hosted CI complete.
