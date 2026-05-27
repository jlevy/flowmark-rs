# Stabilization: parity discrepancies D17 (thematic breaks) and D18 (reference links)

**Date:** 2026-05-19

**Context:** Follow-up hardening on top of the v0.6.5 sync (PR #55). Deep differential
testing (full-corpus Python-vs-Rust diff plus a reference-link truth-table sweep against
Python v0.6.5 and `main`) surfaced two genuine, previously-untested formatter parity
gaps. Both are fixed here with discriminating tests on the Rust side and verification
against the Python reference.
Per the porting playbook Principle 8 (test-before-fix, investigate the class) and
Principle 1 (bug fixes in the original require explicit approval — granted by the
maintainer for this cycle).

## D17 — Thematic-break spacing (Rust-only bug)

**Symptom.** comrak’s renderer forces blank lines on both sides of every thematic break
(`* * *`, `---`). Python flowmark (both v0.6.5 and `main`) preserves the source’s
spacing: a thematic break written tight against a neighbouring block stays tight.

**Class.** Block-separator spacing — the same family as the existing tight-transition
rules in `render_block_children` (HTML comment, paragraph→list, paragraph→code).

**Fix.** Added a symmetric tight-suppression rule: when the source was originally tight
and either neighbour is a `ThematicBreak`, suppress the inserted blank line.
`src/formatter/filling.rs`.

**Which side is correct.** Python.
Verified against Python v0.6.5 and `main` — both preserve tightness.
No Python change required.
A regression test is provided for the reference implementation in
[`2026-05-19-upstream-thematic-break-test.patch`](2026-05-19-upstream-thematic-break-test.patch)
(passes against both v0.6.5 and `main`); the flowmark-rs repo cannot push to
`jlevy/flowmark`, so it is delivered as a patch.

**Tests.** `tests/test_parity_discrepancies.rs::test_d17_*` (5).

## D18 — Reference-link normalization (upstream flowmark issue #45)

**Symptom.** A reference link whose text equals its normalized label was emitted by Rust
in inconsistent forms (shortcut `[foo]`, or `[foo][foo]`), diverging from Python.

**Canonical behaviour** (derived from a truth-table sweep against Python `main`, which
carries the issue #45 fix `0af9e24`, and the upstream `tests/test_reference_links.py`
spec):

| Input | Canonical output | Reason |
| --- | --- | --- |
| `[foo]` (def `[foo]:`) | `[foo][]` | text == normalized label → collapsed |
| `[foo][]` | `[foo][]` | collapsed preserved |
| `[foo][foo]` | `[foo][]` | text == label → collapsed |
| `[bar][foo]` | `[bar][foo]` | text != label → full |
| `[Unreleased]` (def `[unreleased]:`) | `[Unreleased][unreleased]` | text != normalized label → full |
| `[Foo]` (def `[Foo]:`) | `[Foo][foo]` | emitted label is normalized lowercase |

The shortcut form is fragile: it merges with a following `(...)` (reparses as an inline
link, changing the destination) or `[...]` (reparses as a full/collapsed reference,
dropping a link). The collapsed form `[foo][]` is unambiguous.

**Fix.** Extended the `COMRAK-WORKAROUND1` reference-link encoder (`encode_ref_links`)
to (a) normalize the encoded label to lowercase for full and collapsed forms and (b)
handle shortcut references `[text]` (regex with trailing-context capture, since the
`regex` crate has no lookahead).
The render path emits `[text][]` when the rendered text equals the normalized label,
else `[text][label]`. `src/formatter/filling.rs`.

**Which side is correct.** Python `main` (issue #45 fix).
Released v0.6.5 still emits the buggy shortcut form, so this is an **intentional,
documented divergence from released v0.6.5** — recorded in the tolerated-variations list
in `docs/port-status.md`. The Python fix already exists upstream (commit `0af9e24`) with
its own test suite (`tests/test_reference_links.py`); no Python change is required.

**Tests.** `tests/test_parity_discrepancies.rs::test_d18_*` (10) plus two
`encode_ref_links` unit tests in `src/formatter/filling.rs`. All assert the canonical
output and were verified byte-for-byte against Python `main` (v0.6.6.dev).

## Edge case noted (not fixed)

`--auto --inplace` on a file whose **entire** content is a single reference link plus
its definition skips reformatting via the incremental block path (pre-existing; the
library `fill_markdown` path is correct, as is any document with other content).
Zero real-world impact; tracked for a separate look.

## Deferred (feature-level, not stabilization)

Upstream `main` (post-v0.6.5) also adds atomic-aware semantic line wrapping
(links/inline code kept atomic) and table-row-adjacent-to-paragraph handling.
These are feature changes beyond stabilizing v0.6.5 and are deferred to a dedicated
v0.6.6 sync.

## Validation

| Gate | Result |
| --- | --- |
| `cargo test --locked --all-features` | 501 passed, 0 failed, 0 ignored |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | zero warnings |
| `flowmark-dev check-mapping` | 309 mapped, 0 missing |
| `pytest python/tests/test_smoke.py` | 13 passed (Rust manifest count 499→516) |
| Reference-link truth table (Rust vs Python `main`) | all forms match |
| Upstream thematic-break test (Python v0.6.5 and `main`) | 4 passed |
