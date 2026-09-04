---
type: is
id: is-01m1nt8z9sjm73t1pvex4qmekq
title: Stop code-span protection at setext block boundaries
kind: bug
status: closed
priority: 1
version: 3
labels:
  - release-blocker
  - parity
  - preservation
dependencies: []
parent_id: is-01m1nn49g1bds1ekgvg22r297p
created_at: 2026-09-04T08:58:57.204Z
updated_at: 2026-09-04T09:15:09.750Z
closed_at: 2026-09-04T09:15:09.749Z
close_reason: Restored the v0.3.2/Python setext boundary behavior and pinned it in the language-neutral two-pass corpus.
resolution: null
duplicate_of: null
---
The four-way 1,677-document release sweep found a Rust v0.3.2 regression on CommonMark 0.31.2 example 91: Python 0.7.3, Rust 0.3.2, and current Python all end an inline scope at a setext underline, while current Rust protects a backtick run across that block boundary and changes the output. Port Python's setext-aware inline-scope flush, pin the exact current-Python bytes as a shared code-span case, and prove no newly divergent corpus documents remain.

## Notes

Implemented a setext-aware flush in inline_scopes so code-span protection cannot cross a Markdown block boundary. Added shared FM-CODE-SPAN-002 case preservation.code-span.setext-boundary. Rust failed the two-pass case before the fix and passes after it; Python already passed. The direct 258-case deferred CommonMark audit dropped from 15 to 14 first- and second-pass differences, removing example 0091. The 1,677-document four-version sweep across six modes now reports zero newly divergent current Python/Rust documents.
