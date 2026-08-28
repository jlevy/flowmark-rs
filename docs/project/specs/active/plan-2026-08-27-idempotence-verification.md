# Feature: General idempotence verification

**Date:** 2026-08-27

**Author:** Senior review (Claude) with @jlevy direction

**Status:** Phase 1 and 2 complete — PR #81 is clean; the rest is a follow-on PR

**Tracker:** `fmr-1xlk` epic.

## Overview

Establish idempotence — `format(format(x)) == format(x)` — as a verified, corpus-wide
property of flowmark, and document the full scope of where it does not hold today.

Flowmark's value proposition is that it is safe to run repeatedly: in a pre-commit hook,
in CI, on save. That rests on the formatter reaching a fixed point in one pass. Where it
does not, `flowmark --check` reports files that flowmark itself just wrote, and repeated
runs silently rewrite authored content.

This spec deliberately **documents scope before proposing any fix**. Every defect below
is measured in both implementations and left unfixed. Fixes follow the project's standard
sequence: agree the intended bytes, land them in Python, then replicate exactly in Rust.

## Background

### What is already verified

The language-neutral conformance corpus is in good shape on this axis. Of 776 cases,
**768 declare `idempotent = true`** and the shared runner verifies exact second-pass
output for each.

The limitation is that each case pins **one** CLI invocation. A case authored with
`--width 0` proves nothing about `--width 40`; one authored with default flags proves
nothing about `--semantic --cleanups`. Idempotence is a property of the formatter across
its option space, and the corpus samples that space one point per document. Width is the
axis sampled most thinly, and it is where most of the defects below hide.

### Method

Every Markdown document the project ships, formatted twice and compared byte for byte,
across a six-mode matrix (`default`, `--semantic`, `--cleanups`, full typography,
`--width 0`, `--width 40`). Run against both implementations in process: flowmark-rs at
`c00f74b` and Python flowmark at the pinned `9e9fd7c`.

### Measured scope

| | Documents | Checks | Failing checks | Distinct files |
| --- | ---: | ---: | ---: | ---: |
| **Python** (reference) | 1,528 | 9,168 | **138 (1.51%)** | **28** |
| **Rust** (port) | 1,519 | 9,114 | **68 (0.75%)** | **18** |

Overlap: 54 failing checks and 14 files fail in both. 84 checks / 14 files are
Python-only; 14 checks / 4 files are Rust-only.

**The reference implementation is the less stable of the two.** That is the headline
result and the reason this work has to start upstream: roughly two thirds of Python's
instability has no Rust counterpart, and fixing it will change bytes that Rust must then
match. Fixing Rust first would mean fixing it twice.

Twelve further non-zero results in each run were the deliberate invalid-UTF-8 fixture
behaving correctly, and are excluded throughout.

### Regressions versus pre-existing defects

The audit was re-run against three Rust builds — v0.3.2 (the pre-PR release), PR #81's
head `f833ce8`, and this branch — to separate what the preservation work broke from what
it inherited. This is the axis that decides what blocks the PR.

| Build | Failing checks | Files |
| --- | ---: | ---: |
| v0.3.2 (pre-PR release) | **225** | 44 |
| PR #81 head `f833ce8` | **80** | 20 |
| This branch `c00f74b` | **68** | 18 |

| Split | Checks | Files |
| --- | ---: | ---: |
| **Regressions introduced by PR #81** | **1** (now fixed) | **1** |
| Pre-existing, still failing | 79 | 19 |
| **Fixed by PR #81** | **146** | **25** |

That one regression is fixed (`fmr-0pxh`), so **PR #81 now introduces no idempotence
regression at all**. The ledger holds 67 entries across 17 files, every one of which also
fails on the v0.3.2 release.

A second sweep confirmed there is no regression on the output axis either. Comparing
every shipped document in five modes against the Python reference, before and after:

| Rust-vs-Python parity, v0.3.2 to this branch | Checks | Files |
| --- | ---: | ---: |
| **Regressions** (v0.3.2 matched Python, branch does not) | **0** | **0** |
| Fixed (branch matches Python, v0.3.2 did not) | **1,458** | **297** |
| Both diverge, output changed | 40 | 8 |

Of the 40 where both versions diverge from Python, 30 moved **closer** to the reference
and 10 stayed the same distance; **none moved farther**. So on both axes the preservation
work is strictly better or equal, with no incorrect behavior change to block on.

**PR #81 is a large net improvement to idempotence**: it fixes 146 failing checks across
25 files and introduces exactly one regression, on one file. The preservation work has
made the formatter substantially more stable, not less. A further 12 checks were fixed by
this branch's fence-escape work on top.

That one regression is the only item that blocks landing PR #81 on this axis, and it is
tracked as `fmr-0pxh`:

```console
$ printf -- '- Bead: fmr-hr43 | Scope: Phase 8.5/8.6 | Repo: playbook\n- Depends on: WI-1, WI-4\n- Findings: F1-F12 (12 lessons + anti-patterns)\n' > a.md
$ flowmark --width 40 a.md > b.md && flowmark --width 40 b.md > c.md && diff b.md c.md
4a5
>
```

Pass 1 wraps the first item so a continuation line begins with `|`; pass 2 inserts a
blank line before the third item, partially converting the tight list to loose.
v0.3.2 and Python are both stable on it, so Rust needs only to match the reference — no
upstream decision required. Substitution confirms the trigger: replacing the pipes with
commas is stable, two items are stable, and an unrelated list containing a pipe is
stable. The shape is a continuation line *beginning* with `|` plus a third item, which is
what the new pre-parse scanner's structural-pipe and table detection reacts to.

Everything else in the inventory below predates the PR and belongs in separate changes.

### Defect inventory

| ID | Shape | Effect | Python | Rust |
| --- | --- | --- | :-: | :-: |
| A | Wrapped line begins with `=` | Reparsed as a setext H1 underline: list text is promoted to a heading and the list structure is destroyed | yes | yes |
| B | Setext heading with multi-line content (CommonMark 0081, 0082, 0095) | Second pass splits the heading; the trailing line escapes into a paragraph | yes | yes |
| C | Ordered list with an empty item (CommonMark 0283) | First pass drops the empty item, second pass renumbers | yes | yes |
| D | Link label containing a newline (CommonMark 0552) | Second pass rewrites a shortcut reference as a collapsed reference | no | yes |
| E | Blockquote with a heading then prose (CommonMark 0228–0232) | Passes oscillate between splitting the quote in two and rejoining it | yes | no |
| F | Nested list indentation (CommonMark 0307, 0315) | Second pass inserts a blank line after the outer item, changing tight to loose | yes | no |
| G | Escape sequences in a fence info string | Loses one escape level per pass (`fmr-c6xs` / `fm-ww33`) | yes | yes |
| H | Interior `U+FEFF` with leading whitespace | Leading space dropped on the second pass (`fmr-uao3` / `fm-jtwj`) | yes | yes |

G and H were found by the generated-input harness in
`tests/test_preservation_properties.rs` rather than the corpus walk, and are already
tracked; they are listed here so the inventory is complete.

**Defect A is the most serious.** It is content corruption, not merely instability:

```console
$ printf -- '- alpha beta gamma delta epsilon word =\n' | flowmark --width 20 -
- alpha beta gamma
  delta epsilon word
  =

$ printf -- '- alpha beta gamma delta epsilon word =\n' | flowmark --width 20 - | flowmark --width 20 -
- # alpha beta gamma
delta epsilon word
```

The list item's text becomes an H1, the `=` is consumed, and the remaining text escapes
the list. The likely cause is a gap in the escape set used when wrapping: both ports
guard a wrapped line starting with `-`, `*`, `+`, `>` or `#` (Rust's `MD_SPECIALS_PAT` is
`^([-*+>]|#+)$`) but not `=`, and neither guards multi-character runs such as `---` or
`===`. Any prose line ending in an equals sign — a config snippet, `x =`, an assignment
quoted in prose — can trigger it at a narrow width. It also fires on the project's own
reference document, `testdoc.orig.md`, and on one of this repository's specs.

### A correction worth recording

An earlier draft of this audit asserted that formatting a corpus **golden** should be a
no-op, and reported roughly 190 failures on that basis. That premise was wrong: **258 of
the 673 CommonMark cases are tagged `deferred`**, so their `expected.default.md` files
are aspirational spec expectations rather than output flowmark currently produces.
Requiring `format(golden) == golden` therefore measures the deferred backlog, not
stability. The check was removed; the fixed-point property over inputs already covers
what it was reaching for, and the numbers in this spec exclude it.

## Goals

- **Document the scope completely before changing behavior.** The inventory above, the
  ledger, and the gate are the deliverable; no defect is fixed under this spec until its
  intended bytes are agreed.
- **Idempotence becomes a gate, not an assumption.** Every shipped document is formatted
  twice across the option space on every CI run, and any *new* instability fails the
  build.
- **The known set is exact and shrinking.** A known gap is a named entry with a tracking
  bead, asserted in both directions so it cannot rot or quietly grow.
- **Python leads; Rust replicates exactly.** Every shared defect is fixed upstream first,
  pinned as a shared case, and only then ported. Rust never changes shared behavior
  unilaterally.

## Non-Goals

- **Fixing anything under this spec.** Phase 1 is measurement and gating only. Fixes are
  Phase 2 and later, gated on agreed intended bytes.
- **Convergence beyond two passes.** The contract is a fixed point after one format. A
  document that stabilises only after three passes is a defect, not a weaker guarantee to
  codify.
- **Idempotence across option changes.** `format(format(x, A), B) == format(x, B)` is not
  claimed and is not desirable; only same-options repetition is in scope.
- **Clearing the CommonMark deferred backlog.** Defects B–F touch constructs already
  represented in the deferred set; this spec addresses their *instability*, not every
  inherited semantic difference.

## Design

### The property

For every corpus document `x` and every mode `m` in the matrix,
`format(format(x, m), m) == format(x, m)`.

### Mode matrix

Six modes covering the option space's independent axes rather than its combinations:
`default`, `--semantic`, `--cleanups`, full typography
(`--semantic --cleanups --smartquotes --ellipses`), `--width 0`, and `--width 40`.

The narrow width earns its place: defect A is invisible at the default width and appears
only when wrapping pushes a hazardous token to the start of a line.

### Corpus sources

All of them: shared corpus inputs and goldens, CommonMark spec examples, reference
testdocs, tryscript documents and fixtures, and the repository's own docs. Goldens are
included as documents in their own right — not as an assertion that they are stable
output, per the correction above. The repository's own docs matter because they are the
only genuinely human-authored prose in the set, and defect A was found there.

### Known-divergence ledger

`tests/idempotence_known_divergences.toml` names each failing document, mode and bead,
following the precedent of `tests/parity_corpus_known_divergences.toml`. The gate asserts
it **exactly**: an unlisted failure fails the build, and a listed entry that now passes
also fails it, so the ledger shrinks and cannot rot.

### Components

- `tests/test_idempotence_corpus.rs` — the gate.
- `tests/idempotence_known_divergences.toml` — the ledger, seeded with the 68 Rust entries.
- Upstream `tests/parity_corpus/` — where agreed bytes get pinned before any fix.

### API Changes

None. This spec adds test infrastructure only.

## Implementation Plan

### Phase 1: Measure and gate (this change)

- [x] Audit both implementations across the corpus and mode matrix; record the numbers
      and the defect inventory above.
- [x] Add `tests/test_idempotence_corpus.rs` asserting the fixed-point property, using
      the library directly rather than spawning the CLI.
- [x] Add `tests/idempotence_known_divergences.toml` seeded with the 68 current Rust
      entries, each naming its bead.
- [x] Remove the incorrect golden-stability assertion and record why.
- [x] Confirm the full suite, the 776-case conformance corpus and the tryscript documents
      stay green.

### Phase 2: Clear the one regression, so PR #81 lands clean (done)

- [x] Fix `fmr-0pxh` in the bridge, at `repair_synthetic_list_looseness`. Output matches
      Python byte for byte, and upstream now pins the exact bytes as the shared case
      `preservation.extension.line-block.wrapped-pipe-continuation`.
- [x] Remove its ledger entry.
- [x] Confirm no output-parity regression on any shipped document (table above).

Three earlier attempts were rejected, which is worth recording because each one located
the layer the fix does *not* belong in.

**Not the bridge's block boundaries.** Dropping the synthetic blank lines to match
Python's bridge exactly, and gating them on list depth, both failed. Python teaches its
parser about block tokens — `ProtectedBlock` is a real block element and
`Paragraph.break_paragraph` yields to it — so the parser breaks the paragraph and no
blank line is needed. comrak cannot be extended that way, so the blank line stands in for
that break. Without it comrak merges the token into the surrounding paragraph, and at
`--width 0` renders `Before … TOKEN … After` on one line, which destroys the line
structure restoration depends on. Here "replicate Python exactly" means replicate the
*behavior*, and the boundaries are how this parser reaches it.

**Not the scanner.** Excluding a line-block opener whose previous line is non-blank in the
same container did fix the shape and passed every test then in the tree, but it dropped
protection for a line block that follows a paragraph — which upstream requires. It was
reverted once the pin caught up (`2fc45fa`); the corpus case
`preservation.extension.line-block.adjacency` now carries typography-shaped text so the
same mistake fails loudly instead of silently.

**The bridge's *observable side effects*.** A blank line is not inert in CommonMark: one
inside a list makes the whole list loose, and the renderer then spends it on separation
between items nowhere near the token, which restoration cannot take back. The fix
re-tightens a list only when every blank line inside it is one the bridge wrote, so an
authored loose list is untouched. That keeps the parser scaffolding invisible in the
output, which is the property Python gets for free.

Everything below is pre-existing and belongs in its own change.

### Phase 3: Agree intended bytes, upstream first

For each defect A–H, in severity order starting with A:

- [ ] Decide the intended output. For several this is not obvious — defect C's *first*
      pass already drops an empty list item, so idempotence alone is the wrong target.
- [ ] Add a shared case to the language-neutral manifest pinning those bytes.
- [ ] Fix in Python, the reference implementation.
- [ ] Replicate exactly in Rust; confirm byte-identical output.
- [ ] Remove the ledger entry, which now fails the gate until it is gone.

Defect D is Rust-only and Python is already correct, so it needs no upstream change —
only a shared case pinning Python's existing bytes and a Rust fix to match.

### Phase 4: Exact parity

Idempotence is the weaker of the two properties this corpus can prove. The stronger one
is **exactness**: for every shipped document and every mode, the Rust output equals the
Python output byte for byte. Today that is measured but not gated.

Current state, from the same audit: across 7,640 comparisons this branch matches Python
everywhere v0.3.2 did, plus 1,458 checks across 297 files where v0.3.2 did not. What
remains is 40 checks across 8 files where both ports diverge, and the 34 entries in
`tests/parity_corpus_known_divergences.toml`, plus the 258 CommonMark cases upstream
tags `deferred`.

- [ ] Add an exactness gate alongside the idempotence gate: same corpus, same mode
      matrix, asserting Rust output equals recorded Python output, with its own exact
      ledger. It cannot call Python at test time, so the reference bytes come from the
      shared corpus, generated upstream and pinned.
- [ ] Seed the ledger from the current divergence set and drive it to zero, defect by
      defect, Python first.
- [ ] Fold the 34-entry CommonMark ledger into the same mechanism so there is one place
      that answers "where do the ports still disagree".
- [ ] When both ledgers are empty, exactness subsumes idempotence: a formatter that
      matches a fixed-point reference is itself a fixed point.

### Phase 5: Close the loop

- [ ] When the ledger is empty, restore the excluded generator fragments and promote
      `generated_documents_reach_a_fixed_point` in `tests/test_preservation_properties.rs`
      from an on-demand harness to a gate.

## Producing per-item prompts

Phases 3 and 4 are a queue of independent defects, each needing its own agent session.
Rather than hand-writing prompts, each is derived mechanically from what the gates
already record, so the prompt cannot drift from the evidence:

1. **Group ledger entries by defect.** Both ledgers key every entry as
   `document::mode` with a `bead`, so grouping by bead yields the exact document and mode
   set for one item.
2. **Minimize before prompting.** Run the delta reducer over each group's largest
   document to get a byte-minimal reproducer. Every defect in this spec was reduced this
   way, most to under ten bytes, and a minimal case is what makes a prompt actionable.
3. **Record the three-way status.** For each reproducer capture v0.3.2, the current
   branch, and Python, so the prompt states plainly whether it is a regression, inherited,
   or a port divergence, and whether Python already has the intended bytes.
4. **State the target, or say it is undecided.** Where Python is already correct, the
   target is "match this byte sequence". Where both ports are wrong (defects B, C, E, F)
   the prompt must say the intended output is an open question and name it as the first
   deliverable, so nobody pins the wrong golden.
5. **Name the exit condition.** A shared case is added, both ports agree, and the ledger
   entry is removed, which the gate then requires.

The bead descriptions in this epic already follow that shape; `fmr-0pxh` is the worked
example, and it is what let the fix land in one pass.

## Testing Strategy

The gate is the test: every shipped document, formatted twice, in every mode, with the
ledger asserted exactly in both directions. It runs in about 25 seconds in the debug test
profile over roughly 9,100 format pairs, which is comfortably inline for the default
`cargo test`.

Each defect additionally gets a focused regression test at the layer it lives in when it
is fixed, so the behavior is pinned independently of the corpus walk.

The Python audit script used for the numbers above is not committed; the equivalent gate
belongs upstream and is proposed as part of Phase 2 so both ports carry the same
guarantee.

## Open Questions

- **What are the intended bytes for defects B, C, E and F?** All four are CommonMark
  shapes where the first pass is already arguably wrong. Fixing idempotence without
  deciding the target risks pinning the wrong output.
- **Should `=` be escaped at every line start, or only where a setext underline can
  follow?** Unconditional is simpler and matches how `-` is handled today, at the cost of
  emitting `\=` in a few more places.
- **Should the repository's own docs stay in the gate?** They are the best source of human
  prose but they change often, so a failure may reflect a new doc rather than a new
  defect. Current proposal: keep them, since a doc flowmark cannot format stably is itself
  a bug.
- **Should the ledger be shared rather than Rust-only?** Python has 14 files of
  instability Rust does not. A shared ledger would make the asymmetry visible to both
  ports, at the cost of coupling their test infrastructure.

## References

- PR [jlevy/flowmark-rs#81](https://github.com/jlevy/flowmark-rs/pull/81) — review R10 and
  the fence-escape work that prompted this.
- `tests/test_preservation_properties.rs` — generated-input harness; found defects G and H.
- `docs/project/specs/active/plan-2026-05-28-shared-parity-corpus.md` — the shared corpus
  this builds on.
- `tests/parity_corpus_known_divergences.toml` — the ledger pattern reused here.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
