# Porting Log: Bugs, Fixes, and Lessons Learned

> **Doc status:** Rust port-specific (no upstream equivalent).
> Documents the Rust port lifecycle: parity verification, sync workflow, and port
> history.

All bugs, parity issues, and process failures encountered during the Python-to-Rust
flowmark port. Each entry records what went wrong, how it was fixed, and the reusable
lesson.

**Purpose:** Prevent recurring mistakes by documenting patterns.
Anyone working on this codebase should read the [Key Lessons](#key-lessons) section.

**Reference version:** Python flowmark v0.6.4

> [!NOTE]
> This is a historical defect log.
> The current port contract and workflow are recorded in
> [`docs/port-status.md`](port-status.md) and
> [`docs/port-sync-playbook.md`](port-sync-playbook.md).
> The Rust suite now consumes versioned upstream golden evidence directly instead of
> maintaining copied portable fixtures or requiring Python at normal test time.

**Foundational principles:** The
[Porting Principles and Anti-Patterns](../repos/rust-porting-playbook/guidelines/porting-principles-and-antipatterns.md)
document defines 8 non-negotiable rules for agent-driven porting.
The lessons below were learned from applying those principles to this project --- some
reinforce existing principles with concrete techniques, and some identify gaps that
should be graduated into new principles.
See [Relationship to Porting Principles](#relationship-to-porting-principles) for the
full mapping.

## Summary

| Category | Count | PRs |
| --- | --- | --- |
| Senior review bugs (H/M/L) | 20 | #4 |
| Code review findings (P0-P3) | 14 | #2 |
| Parity discrepancies (D1-D11) | 11 | #17 |
| Corpus parity bugs (P6-P9) | 4 | #17 |
| Tryscript golden test gaps (D1-D15 wrapping) | 15 | #13 |
| PR #17 false-parity bugs (D12b, D13r, D15r, D16) | 4 | (this PR) |
| **Total distinct bugs** | **~68** |  |

## Key Lessons

These are the most important recurring patterns.
Numbered for cross-reference from individual bug entries.
Organized from most general (applicable to any project) to most specific
(comrak/flowmark domain knowledge).

### General engineering and testing

These apply to any software project.

**L7. Don’t trust CI alone --- read the diff.** PR #17 passed all 430 tests and all 12
CI checks. The tests themselves were wrong (asserting incorrect behavior).
CI verifies that tests pass, not that tests are correct.

**L2. Use `assert_eq!` with exact expected output, not
`assert!(result.contains(...))`.** Weak assertions mask bugs.
PR #17’s D13 test only checked `!result.contains("\n\n>")` and missed that indentation
was wrong. PR #17’s D15 test asserted Python does NOT convert apostrophes after code ---
Python actually DOES in some cases.

**L3. Test the edge case, not just the happy path.** PR #17’s D12/P6 tests covered
standalone paragraph-to-code-fence but not mixed loose/tight lists (D12b). A single test
with a simple case is not sufficient for features that interact with parser-level
classification differences.

### Cross-language porting

These apply when porting any codebase from one language to another.

**L1. Always verify the source language’s actual byte output.** Never assume what the
source implementation does.
Run the original binary at the **pinned version** (currently `uvx flowmark@0.6.4`),
capture output, and compare byte-by-byte with `diff` or `xxd`. Never use `@latest` ---
the reference version must be fixed so results are reproducible.
PR #17 claimed “exact parity” but 4 bugs slipped through because tests asserted assumed
behavior, not verified behavior.

**L5. Post-merge corpus validation is essential.** Unit tests are necessary but
insufficient. A real-world corpus (here: 623 files in `attic/test-docs/`) catches bugs
that targeted tests miss.
Run `diff -rq` between the two implementations’ output on the full corpus before
claiming parity.

**L9. Extract corner-case inputs from corpus diffs into a checked-in regression
corpus.** The full corpus may be too large to commit, but every corpus diff reveals a
minimal reproducer. Extract those files into a small checked-in corpus (e.g.,
`tests/corpus-regressions/`) with a test that runs both implementations and asserts
byte-for-byte match.
This turns ephemeral one-off validation into a permanent regression gate.

**L10. Use red/green discipline for parity fixes.** Before writing any fix, first add
tests (or corpus entries) that **fail** against the current code, confirming the bug is
real and reproducible.
Only then write the fix and verify all tests go green.
This prevents the PR #17 failure mode where tests were written to pass from the start
--- they never actually validated anything because they asserted the wrong expected
output. Red-first ensures the test can distinguish broken from fixed.

**L11. Share reviewed, versioned golden evidence; do not hand-copy expectations.** A
static assertion is weak when its expected string was guessed or copied by the port
author. It is strong when the source implementation produced the bytes, a reviewer
accepted the exact diff, the source commit and command are recorded, and both languages
execute the same case definition.

Flowmark’s primary portable contract now consists of one upstream manifest and tryscript
suite consumed directly by both implementations.
This preserves exact provenance, works in clean Rust CI, and cannot drift into two
fixture copies. Live cross-binary and corpus comparison is still required as a
baseline-transition and discrepancy-discovery audit.
Promote every real difference it finds into a minimal shared case before fixing it.

**L8. Error parity is a first-class surface.** CLI error messages, exit codes, and
stderr output must be tested with the same rigor as formatting output.
Golden-test wildcards (`[..]`) can mask error message bugs.

### Comrak and flowmark domain-specific

These are specific to porting from Python/marko to Rust/comrak.

**L4. Comrak’s loose/tight classification is a recurring source of bugs.** Comrak marks
an entire list as “loose” when *any* sibling pair has a blank line.
Python/marko does per-item classification.
This has caused D4, D12, D12b, and the tight mode rewrite.
Always use source positions to verify original intent.

**L6. Smart quote context depends on surrounding characters.** The smart quote engine
uses character context to decide conversions.
Placeholders for non-text nodes (code spans, HTML) must preserve the right context.
See D15/P9: apostrophe after `config` (word char) vs `foo()` (non-word char).

## Bugs by PR

### PR #2: Code Review Findings

Branch: `code-review-fixes` | Merged: 2026-02-18

| ID | Title | Severity | Lesson |
| --- | --- | --- | --- |
| P0.1 | 9 clippy `inefficient_to_string` errors | P0 | Run `clippy --all-features -D warnings` locally, not just in CI |
| P0.2 | `cargo fmt` violations across nearly all files | P0 | Run `cargo fmt` before every commit |
| P0.5 | Lint config gap: Cargo.toml says `warn`, CI says `deny` | P0.5 | Set `warnings = "deny"` in Cargo.toml so local and CI match |
| P1.1 | Dead dependencies (`unicode-segmentation`, `toml`, `serde`) | P1 | Run `cargo udeps` or `cargo machete` periodically |
| P1.2 | Dead error variants (`Error::Config`, `Error::Other`) | P1 | Don’t add error variants speculatively |
| P1.3 | Fence-tracking code duplicated 3x in filling.rs (~60 lines) | P1 | Extract helper on second duplication |
| P1.4 | Unnecessary allocations (Vec\<char>, etc.) | P1 | Profile before optimizing, but avoid gratuitous allocations |
| P2.1 | Boolean parameter overload (8-11 params) | P2 | Use `FormatOptions` struct early, not after the API stabilizes |
| P2.2 | Unused `_name` field in `AtomicPattern` | P2 | Don’t port fields that aren’t used in Rust |
| P2.3 | Unnecessary string clone in code block rendering | P2 | -- |
| P2.4 | Repeated `.expect()` calls in line_wrappers.rs | P2 | Use `LazyLock` or compile-once for regexes |

### PR #4: Senior Engineering Review (Appendix D)

Branch: `fix/senior-review-bugs` | Merged: 2026-02-18 | Epic: fmr-fvw7

| ID | Bead | Title | Severity | Root Cause | Lesson |
| --- | --- | --- | --- | --- | --- |
| H1 | fmr-r5gx | Public API surface too broad (67 items vs 3 needed) | High | All items `pub` by default during porting | Port with `pub(crate)`, promote to `pub` only for the API |
| H2 | fmr-t5ep | `reformat_text()` panics on malformed escapes | High | `unwrap()` in PUA decode path | Never `unwrap()` on user input; use `unwrap_or` or `?` |
| H3 | fmr-s6r1 | Regex compiled on every `is_sentence_ending()` call | High | No caching | Use `LazyLock` for all compiled regexes |
| H4 | fmr-7f54 | Smart quotes corrupt multi-byte UTF-8 at boundaries | High | Byte-indexed string slicing | Always use char boundaries, not byte offsets, for UTF-8 |
| H5 | fmr-rjfk | Unbounded recursion in nested blockquote formatting | High | No depth limit | Add recursion depth guard |
| M1 | fmr-jc10 | Sentence wrapper off-by-one (col 81 vs 80) | Medium | Matches Python behavior | Documented as intentional parity; not a bug |
| M2 | fmr-36d3 | PUA placeholder collision at U+E05C (backslash) | Medium | `\` maps to same PUA range as content chars | Documented; collision requires specific content pattern |
| M3 | -- | No benchmark framework | Medium | -- | Deferred (no framework existed yet) |
| M4 | fmr-c1fb | `fill_text` ignores `preserve_words` for atomic patterns | Medium | Parameter not threaded through | Thread parameters through the full call chain |
| M5 | fmr-4szn | Config search stops at first `.flowmark.toml` found | Medium | Early return | Match Python’s 3-file search order |
| M6 | fmr-xz2s | Missing fallback when sentence regex fails | Medium | No error handling on regex compile | Always handle regex compilation errors |
| L1 | fmr-fj55 | `FormatOptions` uses `pub` fields, no builder | Low | Quick port | Use builder pattern for config structs |
| L2 | fmr-bvbe | Hard-coded Unicode categories | Low | Direct port from Python | Acceptable for parity; consider `unicode-segmentation` later |
| L3 | fmr-hs3b | Tag handling regex compiled per-call | Low | No caching | Use `LazyLock` |
| L4 | fmr-15ty | `fill_markdown` allocates intermediate String per line | Low | Direct port from Python | Acceptable; optimize if profiling shows need |
| L5 | -- | Refactor filling.rs (1270 LOC) into submodules | Low | Deferred (high risk) | -- |
| L6 | fmr-q5f3 | No round-trip stability test | Low | Not written | Add golden test that formats twice and asserts idempotency |
| L7 | fmr-rvdq | Error type uses String messages | Low | Quick port | Use structured enum variants |
| L8 | fmr-rn10 | Skills loaded from filesystem at runtime | Low | Direct port | Use `include_str!` for embedded resources |
| L9 | fmr-a1gx | No property-based/fuzz testing | Low | Not written | Consider `proptest` for formatter invariants |

### PR #13: Tryscript Golden Tests & 14 Parity Gaps

Branch: `claude/tryscript-golden-tests-5sQqv` | Merged: 2026-02-19

These are wrapping/sentence-level bugs found by the tryscript golden test suite.

| ID | Bead | Title | Root Cause | Fix |
| --- | --- | --- | --- | --- |
| D1 | fmr-tmf2 | Sentence break suppressed before inline code | Sentence detector didn’t see code spans | Adjust sentence boundary detection |
| D2 | fmr-4e9d | Abbreviation false positive on list-item trailing period | “St.” abbreviation matched list markers | Refine abbreviation regex |
| D3 | fmr-3txz | Over-eager sentence breaks inside parenthetical asides | Didn’t check for surrounding parens | Add parenthetical context check |
| D4 | fmr-djy2 | Sentence break inside Markdown link text | Link text treated as plain text | Skip sentence detection inside links |
| D5 | fmr-0v8s | Missing sentence break after ‘).’ ending | Close-paren not in sentence-end charset | Add ‘)’ to sentence-ending chars |
| D6 | fmr-e8v3 | Extra space before footnote reference | Footnote ref treated as word boundary | Handle footnote refs specially |
| D7 | fmr-17bm | Atomic-pattern boundary splits code info string | Atomic pattern regex too greedy | Tighten pattern boundary |
| D8 | fmr-fkfm | Sentence break suppressed before bold/italic | Inline markup not handled as boundary | Add emphasis handling |
| D9 | fmr-c30v | Sentence break before HTML comment | HTML comment not recognized | Add HTML comment pattern |
| D10 | fmr-gbya | Different wrapping for very long unbreakable tokens | Max-width handling differs | Match Python’s overflow behavior |
| D11 | fmr-4w8y | CLI `--list-files` output format differs | Formatting difference | Match Python format |
| D12 | fmr-v9kf | Missing blank line before thematic breaks | Blank line normalization skipped | Add thematic break handling |
| D13 | fmr-zrgr | GFM alert/admonition syntax not preserved | Comrak doesn’t support alerts | Preserve as raw HTML blocks |
| D14 | fmr-tprn | Tight list + trailing blank becomes loose | Comrak loose classification | Use source positions |
| D15 | fmr-g0rz | Blockquote nested list indentation diverges | Indent calculation wrong | Fix indent math |

### PR #17: “Exact Parity” (20 Parity Beads) --- FALSE CLAIM

Branch: `claude/fix-port-disparities-1mkmp` | Merged: 2026-02-19

PR #17 claimed byte-for-byte parity across all modes and closed 20 beads.
**This was false.** Running on a 623-file real-world corpus revealed 20 files with
differences across 4 bug categories.

**What PR #17 got right** (these bugs were genuinely fixed):

| ID | Bead | Title | Fix Summary |
| --- | --- | --- | --- |
| D1 | fmr-n69j | Plaintext mode code blocks collapsed | Preserve fence structure in plaintext |
| D2 | fmr-fzth | Plaintext “St.” sentence detection | Match Python’s `html_md_word_splitter` |
| D3 | fmr-bzra | Narrow width `<sup>` tag wrapping | Fix word splitter for HTML tags |
| D4 | fmr-r9k6 | Tight nested list extra blank lines | Rewrite `any_item_is_complex` detection |
| D5 | fmr-vpg4 | Loose footnote list missing blanks | Add blank after footnote list items |
| D6 | fmr-3i50 | Nested blockquote extra blank lines | Source position tracking in `render_block_children_quoted` |
| D7 | fmr-81j7 | Footnote list items collapsed | Fix FNDEF rendering for list children |
| D8 | fmr-xcr9 | Footnote blockquote collapsed | Fix FNDEF rendering for blockquote children |
| D9 | fmr-dihn | Empty input missing newline | Add `\n` for empty/whitespace input |
| D10 | fmr-gocw | HTML entities decoded | Preserve `&amp;` etc. through rendering |
| D11 | fmr-8ixa | CLI error handling gaps | Fix duplicate errors, validate --inplace stdin |
| D12 | fmr-0u55 | Paragraph before code fence extra blank (P6) | Add Rule 4 to `suppress_for_tight` |
| D14 | fmr-9kth | Escaped backtick stripped in table (P8) | PUA escape protection for table cells |
| Tight mode | fmr-afof | Complex item detection rewrite | `any_item_is_complex`, `parent_is_tight` |
| Loose Rules 3/4 | fmr-desq | Blank line suppression in loose mode | Guard with `list_spacing != Loose` |
| Loose FNDEF | fmr-8pya | Preamble-to-list separator | Double-newline separator in loose mode |
| Jinja regex | fmr-dpjh | Paired Jinja tag regex fix | Opening tag must start with `[a-zA-Z]` |
| Blockquote src | fmr-xkh3 | Nested blockquote source tracking | `originally_tight` in quoted blocks |
| Golden gating | fmr-gydk | Golden test regression | Preserve mode gating fix |

**What PR #17 got wrong** (4 bugs shipped as “fixed”):

| ID | Title | What PR #17 Did Wrong | Lesson |
| --- | --- | --- | --- |
| D12b | Mixed loose/tight list code fences (P6) | Test only covered standalone para-to-fence, not mixed lists | **L3:** Test the edge case |
| D13r | Blockquote blank line indentation (P7) | Test checked `!contains("\n\n>")` --- too weak, missed indent | **L2:** Use `assert_eq!` with exact output |
| D15r | Smart quote after inline code (P9) | Test asserted Python does NOT convert --- Python DOES for word chars | **L1:** Verify Python’s actual bytes |
| D16 | Empty code blocks get spurious blank line | Never tested at all | **L3:** Test the edge case |

### Current Work: Fix PR #17’s 4 False-Parity Bugs

Branch: (to be created) | Follows PR #17

These 4 fixes correct bugs that PR #17 claimed were resolved.
All verified byte-for-byte against `uvx flowmark@0.6.4` on 623-file corpus.

| ID | File:Line | Root Cause | Fix |
| --- | --- | --- | --- |
| D12b | filling.rs:1842-1857 | COMRAK-WORKAROUND10 only checked List children in Preserve mode, not CodeBlock in all modes | Extend workaround to CodeBlock children; use source positions to detect tight transitions |
| D13r | filling.rs:1864-1873 | `item_subsequent.trim_end()` stripped list indent from blank separator lines | Use full `item_subsequent` to preserve indent (e.g., `">    "` not `">"`) |
| D15r | filling.rs:2264-2278 | Code span placeholder was always space; Python’s smart quote is context-sensitive | Use last char of code content as placeholder: word char → smart quote matches; non-word → no match |
| D16 | filling.rs:1286-1296 | `"".split('\n')` yields one empty string → spurious blank line | Guard content loop with `if !code_content.is_empty()` |

**Verification:**

- 434 tests pass (0 failures, 0 ignored)
- `cargo clippy --all-targets --all-features -- -D warnings` clean
- 0 diffs on 623-file corpus between Rust and Python output

## Process Failures

### PR #17: False “Exact Parity” Claim

**What happened:** An agent session fixed 20 parity bugs, wrote tests, and claimed
byte-for-byte parity across all formatting modes.
The claim was false --- 4 bugs remained.

**Root causes:**

1. **Tests asserted assumed behavior, not verified behavior.** The D15 test asserted
   that Python does NOT convert apostrophes after inline code.
   Python actually DOES convert them when the code ends with a word character.
   The agent never ran Python to verify.

2. **Tests used weak assertions.** The D13 test checked `!result.contains("\n\n>")` (no
   bare blank lines) but didn’t check that the blank lines had the correct indentation.
   An `assert_eq!` with exact Python output would have caught this.

3. **Tests didn’t cover edge cases.** The D12/P6 tests only covered the simple case
   (standalone paragraph before code fence).
   Mixed loose/tight lists --- where comrak’s classification differs from Python’s ---
   were not tested.

4. **No corpus-level verification.** The claim of “exact parity” was based solely on
   unit tests. Running both formatters on a real-world corpus would have revealed the
   remaining differences immediately.

**What should have been done:**

```bash
# The verification command that would have caught all 4 bugs:
cp -a attic/test-docs attic/td-rs && cp -a attic/test-docs attic/td-py
./target/release/flowmark --auto --inplace attic/td-rs/
uvx flowmark@0.6.4 --auto --inplace attic/td-py/
diff -rq attic/td-rs/ attic/td-py/
```

### Porting Principle 8 (violated)

> **Verify against Python’s actual output, byte-by-byte.** Never assume what Python
> does. Run the Python binary, capture its output, and compare.

This principle existed in the spec before PR #17 was written.
The agent violated it for all 4 bugs.

## Relationship to Porting Principles

The
[Porting Principles](../repos/rust-porting-playbook/guidelines/porting-principles-and-antipatterns.md)
define 8 non-negotiable rules.
The lessons in this log relate to them as follows:

### Lessons that reinforce existing principles

These lessons are concrete techniques or instances of existing principles.
They don’t need to become new principles --- they’re already covered --- but they add
project-specific detail.

| Lesson | Reinforces Principle | How |
| --- | --- | --- |
| **L2** (use `assert_eq!`, not `contains`) | **P4** (tests must never hide failures) | Specific technique: weak assertions are a form of hidden failure |
| **L3** (test edge cases) | **P8** (investigate the class, not the instance) | P8 says to enumerate all instances in a category; L3 is the same insight |
| **L7** (don’t trust CI alone) | **P4** (tests must never hide failures) | Tests can be wrong even when CI is green; P4’s anti-patterns cover this |
| **L10** (red/green discipline) | **P8** (disparities must be tested before fixed) | L10 is P8 restated as a workflow: red first, then green |
| **L1** (verify source byte output) | **P8** (test before fix) + **P4** (don’t hide) | Specific technique: use pinned version, byte-by-byte diff |
| **L8** (error parity is first-class) | **P1** (parity must be defined crisply) | Error messages/exit codes are a parity surface; P1 says enumerate every dimension |
| **L11** (dynamic code-to-code assertion) | **P8** (test before fix) + **P4** (don’t hide) | Parity gate should be code-to-code, not code-to-copied-value (3 variations) |

### Lessons that should be graduated into principles

These lessons identify gaps in the current principles --- patterns that recurred despite
the existing 8 rules.

**L5 + L9 + L11: End-to-end parity validation.**

The current principles focus on individual disparity tests (P8) and test integrity (P4).
They have two gaps:

1. **No corpus-level validation.** P8 says to write disparity tests, but not to run both
   implementations on a large, real-world input set and diff the output.
   This is a fundamentally different validation method that catches bugs targeted tests
   miss (PR #17 passed 430 tests but failed on 20 of 623 corpus files).

2. **Static vs dynamic assertions.** P8 says “expected output comes from the Python
   reference” but does not distinguish between a static hand-copied string literal and a
   dynamic assertion that runs both programs.
   A static assertion tests one expected behavior; a dynamic assertion tests that two
   programs behave identically.
   The dynamic form is fundamentally stronger because the copying process itself
   introduces errors (see PR #17 D15).

A candidate Principle 9 would be:

> **Parity gates must be dynamic code-to-code assertions, not static
> code-to-copied-value assertions.** What makes a parity test “dynamic” is that it
> ensures two pieces of code produce identical results, rather than comparing one piece
> of code against a hand-copied expected value.
> Three equally strong forms: (1) run both implementations in the test harness and
> assert equivalent output, (2) run both implementations separately, save outputs, and
> auto-compare (e.g., `diff -rq`), (3) maintain a shared golden test corpus and run the
> same test script on both codebases.
> Static assertions (comparing against string literals) are useful as supplementary
> documentation, but not as the primary parity gate, because the copying process itself
> introduces errors. Before claiming parity, run both implementations on a large, diverse
> corpus and diff the output.
> When the corpus reveals a difference, extract the minimal input into a checked-in
> regression corpus (e.g., `tests/corpus-regressions/`) so the bug can never recur
> silently.

### Domain-specific lessons (not candidates for principles)

**L4** (comrak loose/tight classification) and **L6** (smart quote context) are specific
to the comrak/marko parser difference.
They belong in this project’s log, not in the general porting playbook.

## How to Add New Entries

When fixing a parity bug, follow the red/green process (**L10**):

1. **Red:** Extract a minimal reproducer from the corpus diff.
   Add it to `tests/corpus-regressions/` (**L9**) and/or as a test case.
   Confirm the test **fails** against current code.
2. **Fix:** Implement the fix.
3. **Green:** Confirm the new test passes, all existing tests still pass, and the full
   corpus diff is clean (`uvx flowmark@0.6.4`, not `@latest` --- **L1**).
4. **Log:** Add an entry to the relevant PR section (or create a new section) with:

```markdown
| ID | Bead | Title | Root Cause | Fix | Lesson |
```

If the bug reveals a new reusable lesson, add it to the [Key Lessons](#key-lessons)
section with the next available number (L11, L12, ...) and cross-reference it from the
bug entry.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
