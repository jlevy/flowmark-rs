---
type: is
id: is-01m10tdzkg4ct3eydzvh3jz06r
title: "PR #81 review R10: unterminated fence needs extra passes"
kind: bug
status: open
priority: 3
version: 2
labels:
  - pr-review
  - idempotence
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T05:17:38.287Z
updated_at: 2026-08-27T05:36:21.709Z
---
PR #81 R10. Differential fuzzing reported a malformed unterminated code-fence case that needs extra passes to stabilize. Recover an exact reproduction, add a shared fallback case if Python and Rust should agree, and fix or explicitly defer the malformed-input policy.

## Notes

Deferred pending an exact reproducer. The review supplied no input bytes. A deterministic 1,200-case randomized sweep across unterminated backtick/tilde fences, escapes, suffix blocks, and four modes found no non-idempotent case at the current head. Add a shared malformed-fallback golden as soon as exact source/flags are recovered.
