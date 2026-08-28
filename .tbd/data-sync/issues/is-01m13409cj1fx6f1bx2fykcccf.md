---
type: is
id: is-01m13409cj1fx6f1bx2fykcccf
title: Link reference definitions are silently dropped when the single-line regex misses them
kind: bug
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-27-idempotence-verification.md
labels:
  - parity
  - commonmark
dependencies: []
parent_id: is-01m12n1d12xj4jjmaczmn9etzm
created_at: 2026-08-28T02:43:24.178Z
updated_at: 2026-08-28T02:43:24.178Z
---
`extract_link_ref_defs` recognizes a link reference definition with `LINK_REF_DEF`, a
single-line regex, and wraps each match in a marker comment so comrak carries it through.
Anything the regex misses is left as ordinary text, comrak consumes it as a reference
definition, and comrak does not emit reference definitions — so the authored definition
disappears from the output. This is content loss, not a spelling difference.

Two families are missed:

- **Definitions spanning lines.** CommonMark 0.31.2 section 4.7 allows the label,
  destination and title to sit on separate lines. Examples 193, 195 and 196 all take this
  form.

        [foo]:
        /url
        'the title'

        [foo]

  Rust emits `[foo](/url "the title")` and the definition is gone. Python emits
  `[foo]: /url 'the title'` and `[foo][]`.

- **Definitions carrying escapes or parentheses.** Examples 194 (`my_(url)`) and 202
  (`/url\bar\*baz`) are single-line but still fall outside the regex.

Reproduces identically on the v0.3.2 release, so this predates PR #81. It is the reason
`commonmark.default.0193` and `0194` sit in the divergence ledger under fmr-rz9f, and it
is why the shared case `link-reference-definition.title-delimiters` (`FM-LINK-REF-DEF-001`)
covers only the shapes the regex does match.

## Shape of the fix

Recognition has to follow the CommonMark definition rather than a line pattern: consume
the label, then the destination (angle-bracketed or not, escapes resolved), then an
optional title in `"`, `'` or `(...)`, across line breaks, and stop where the spec says a
definition ends. The sibling Python bead fm-c9gr covers the escape half of the same
problem there; land the two together with a shared case for the escaped and multi-line
shapes so both ports are pinned at once.
