# Feature: CLI Help and Program Description Cleanup

**Date:** 2026-02-25

**Author:** Codex (GPT-5)

**Status:** Implemented (Phases 1-3 complete; Phase 4 remains TODO)

## Overview

This spec defines a focused cleanup of Flowmark’s Rust CLI help so it is:

1. Closer to Python Flowmark’s readability and wording
2. More concise and scannable in terminal output
3. Explicitly helpful for agent users via `--skill`

The work is intentionally scoped to help text and help layout only.
No formatting logic, file discovery behavior, or config precedence behavior changes are
included.

## User Requirements (Locked)

1. Use Python’s program tagline in Rust help output.
2. Keep Rust help closer to Python’s readability.
3. Make default help less vertically noisy (fewer blank lines / denser layout).
4. Keep the bottom overview brief.
5. Bottom overview must include “Common usage”:
   - `--auto` and `--list-files`
   - examples for a single file, for a directory, and for all files in current directory
     (`.`), all with `--auto`
6. Bottom overview must include `--skill` guidance and emphasize that agents should run
   it for full Flowmark usage instructions.
7. Keep code and spec synchronized with explicit acceptance criteria.
8. VS Code/Cursor run-on-save setup should be documented in app docs (`--docs`) and in
   skill output (`--skill`).
9. Review Python README coverage and ensure self-documenting help (`--docs`) includes
   corresponding content areas.

## Current Baseline

### Python (`flowmark --help`)

- Strong tagline: “Flowmark: Better auto-formatting for Markdown and plaintext”
- Dense option list (easy scan)
- Very long epilog/examples section

### Rust (`flowmark --help`)

- Previously shorter tagline and terse option wording
- Long-help mode with extra vertical spacing
- No brief targeted bottom overview for agent workflow

## Scope

### In Scope

1. Program description/tagline text in Rust CLI help
2. Option help copy edits for clarity and parity of intent
3. Help layout behavior to make `--help` concise by default
4. Short bottom overview block with the exact examples/guidance requested
5. Tests that pin key help content/structure to prevent drift
6. VS Code/Cursor setup instructions in Rust README/docs and skill content

### Out of Scope

1. Any formatting algorithm behavior
2. Any file resolver behavior
3. Any config merge behavior
4. Any `--auto` behavior changes (preset remains identical)
5. Any large README rewrite

## Proposed Design

### D1: Tagline parity with Python

Set Rust CLI program description to:

`Flowmark: Better auto-formatting for Markdown and plaintext`

### D2: Concise default `--help`

Use Clap short-help rendering for `--help` so output is denser (Python-like scanability)
rather than verbose long-help spacing.

Implementation strategy:

- Disable Clap’s default help flag
- Re-add `-h, --help` mapped to short-help action
- Do not add `--help-full`; `--docs` is the full-documentation path

### D3: Option copy cleanup

Update option descriptions to be clearer and more directly useful (without changing
semantics), including explicit markdown-mode notes where relevant.

### D4: Brief bottom overview

Add a short examples/guidance footer with only the requested high-value items:

1. `flowmark --auto .` (current folder)
2. `flowmark --auto docs/` (directory)
3. `flowmark --auto README.md` (single file)
4. `flowmark --list-files .`
5. `flowmark --skill` plus explicit note that agents should run this for full guidance

### D5: Keep both Rust binaries aligned

`flowmark` and `flowmark-rs` must remain identical in help content except command name
in `Usage:`.

## File-Level Change Plan

1. `/Users/levy/wrk/github/flowmark-rs/src/main.rs`
   - Update command-level help metadata (`about`, help actions, footer text)
   - Refine argument help strings
   - Keep all runtime behavior unchanged

2. `/Users/levy/wrk/github/flowmark-rs/tests/tryscript/help.tryscript.md` (new)
   - Add focused assertions for help tagline, dense usage shape, and brief footer
     content

3. `/Users/levy/wrk/github/flowmark-rs/tests/test_tryscript_golden.rs`
   - Register the new tryscript file

4. `/Users/levy/wrk/github/flowmark-rs/repos/flowmark/src/flowmark/cli.py` (Python Phase
   3 sync target; separate repo commit)
   - Keep Python help aligned with Rust intent and wording where practical
   - Shorten Python epilog/footer overview to match the agreed brief version

5. `/Users/levy/wrk/github/flowmark-rs/README.md`
   - Add VS Code/Cursor run-on-save setup section with a concrete `settings.json`
     example

6. `/Users/levy/wrk/github/flowmark-rs/src/skills/SKILL.md`
   - Add VS Code/Cursor setup snippet for agent-facing usage guidance

7. `/Users/levy/wrk/github/flowmark-rs/src/skills/mod.rs`
   - Ensure docs are available from the binary itself via build-generated docs content
     so editor setup guidance is included in `--docs`

8. `/Users/levy/wrk/github/flowmark-rs/repos/flowmark/README.md` (reference input)
   - Use as source checklist for docs coverage audit

9. `/Users/levy/wrk/github/flowmark-rs/build.rs`
   - Generate embedded docs payload at build time (`OUT_DIR/flowmark_docs.md`)

10. `/Users/levy/wrk/github/flowmark-rs/scripts/generate_rust_readme.py`
- Generate Rust README as a superset of Python README with a Rust-specific preface

## Three-Phase Delivery Plan

### Phase 1: Python -> Rust Alignment

Copy improvements that already exist in Python help into Rust help:

1. Python tagline parity
2. Python-style clearer option phrasing
3. Python-style immediately useful bottom guidance (shortened)

### Phase 2: Rust-Only Improvements

Add Rust-side improvements that go beyond current Python behavior:

1. Denser default `--help` scanability
2. Better brief footer structure and agent callout (`--skill`)
3. Built-in docs behavior for `--docs` plus VS Code/Cursor guidance in docs and skill
4. Python README section coverage audit for Rust self-documenting docs
5. Rust README generation pipeline from Python canonical README + Rust preface

### Phase 3: Rust -> Python Sync

Backport Phase 2 improvements from Rust into Python (where they are improvements over
Python), keeping the two CLIs near-equivalent.

**Implemented 2026-02-25** in Python submodule branch `codex/cli-help-sync-2026-02-25`
(commit `9fda859`):

1. Python `--help` epilog shortened to concise footer matching Rust intent
2. Agent guidance callout (`flowmark --skill`) added in help footer
3. Help regression tests added (`tests/test_cli_help.py`) and golden assertions updated
4. VS Code/Cursor run-on-save setup added to Python skill content
5. README description updated to link both Python origin and Rust port

### Phase 4 (TODO): Shared Canonical Docs Source

Unify Python and Rust README generation from one canonical source document:

1. Maintain a single shared docs source (for example `docs/shared/flowmark-docs.md`).
2. Generate Python README with a Python-oriented top header:
   - state that Python is the original implementation
   - point to Rust implementation as a faster option
3. Generate Rust README with a Rust-oriented top header:
   - state that Rust is an auto-synced port
   - link to Python canonical project and porting playbook
4. Regenerate both READMEs during full build/release workflows.
5. Treat generated READMEs as derived artifacts (not hand-edited).

## Phase 3: Rust -> Python Sync Plan (Near-Equivalence)

The original Python implementation is available locally as a submodule:

- `/Users/levy/wrk/github/flowmark-rs/repos/flowmark`

We will use this local copy as the Python source of truth while aligning Rust and Python
help output.

### Equivalence Target

Target is **near equivalence** (same meaning and guidance), not byte-for-byte identity.
Differences caused by `argparse` vs `clap` formatting are acceptable.

### Backport Matrix

1. Tagline
   - Target text: `Flowmark: Better auto-formatting for Markdown and plaintext`
   - Rust: use this exact program description
   - Python backport: keep this exact description line

2. Help density
   - Target: concise, quickly scannable help output
   - Rust: default `--help` set to dense mode
   - Python backport: retain dense argparse layout

3. Bottom overview length
   - Target: brief footer only
   - Rust: short examples/guidance block
   - Python backport: replace long epilog with short equivalent block

4. Required footer examples
   - `flowmark --auto README.md`
   - `flowmark --auto docs/`
   - `flowmark --auto .`
   - `flowmark --list-files .`
   - Python backport: include the same set

5. Agent guidance
   - Target: include `flowmark --skill` and explicitly tell agents to run it for full
     usage guidance
   - Rust: include this in footer
   - Python backport: include the same statement in epilog/footer

6. Key option wording
   - Target: semantically aligned wording for `--auto`, `--list-files`, `--skill`, and
     related flags
   - Rust: revised copy
   - Python backport: adjust argparse help strings where needed

7. Python README coverage
   - Target: all major Python README sections have corresponding material in Rust
     self-documenting docs (`--docs`)
   - Rust: maintain checklist and fill any missing coverage areas
   - Python backport: sync any Rust improvements that should be retained upstream

### Python README Coverage Checklist (for `--docs`)

Reference source: `/Users/levy/wrk/github/flowmark-rs/repos/flowmark/README.md`

1. Installation: Covered
2. Use Cases: Covered
3. Semantic Line Breaks: Covered
4. Typographic Cleanups (smart quotes, ellipses): Covered
5. Frontmatter Support: Covered
6. Usage / Quick Start: Covered
7. Batch Formatting: Covered
8. CLI Reference: Covered
9. File Discovery: Covered
10. Configuration: Covered
11. Use in VS Code/Cursor: Covered
12. Agent Use: Covered
13. Why Another Markdown Formatter: Covered
14. Project Docs: Covered

### Python Backport Acceptance Criteria

1. Python and Rust help start with the same tagline.
2. Both include the same short footer example set.
3. Both include explicit `--skill` guidance for agents.
4. Key option descriptions are semantically equivalent even if layout differs.
5. Any intentional differences are documented here before finalization.
6. Python README coverage checklist is complete.

## Implementation Plan

1. Finalize CLI copy and footer text in `src/main.rs`.
2. Run `flowmark --help` and `flowmark-rs --help` locally, compare against this spec.
3. Add help-focused tryscript regression test.
4. Run targeted test suite.
5. Draft and review Python backport changes in local submodule
   (`repos/flowmark/src/flowmark/cli.py`).
6. Complete Python README -> Rust docs coverage audit and document any remaining gaps.
7. Verify no behavioral regressions outside help text.

## Validation Plan

### Manual checks

1. `cargo run --quiet --bin flowmark -- --help`
2. `cargo run --quiet --bin flowmark-rs -- --help`
3. `cargo run --quiet --bin flowmark -- --docs`
4. `(cd repos/flowmark && uv run flowmark --help)`
5. Compare Rust and Python help for tagline, footer examples, and `--skill` guidance
6. Verify `--docs` includes VS Code/Cursor run-on-save snippet
7. Audit Python README headings against Rust `--docs` coverage checklist

### Automated checks

1. `cargo test --test test_tryscript_golden`
2. `cargo test --test test_cli_file_discovery`

## Acceptance Criteria

1. `flowmark --help` starts with Python tagline verbatim.
2. `flowmark --help` is visually denser than previous long-help output.
3. Footer contains only the brief requested examples and `--skill` agent guidance.
4. `flowmark --help` and `flowmark-rs --help` match in content (except binary name).
5. All targeted tests pass.
6. Python backport checklist items are documented and verified from local submodule
   output.
7. `--docs` includes VS Code/Cursor instructions and covers Python README content areas.

## Risks and Mitigations

1. Risk: Clap help action customization changes expected `-h`/`--help` behavior.
   - Mitigation: keep `-h` and `--help` equivalent, with `--docs` as the full-doc path.

2. Risk: Help tests become brittle due wrapping differences by terminal width.
   - Mitigation: assert critical lines/snippets, not full help snapshots.

## Open Questions

1. Use concise `--help` only and rely on `--docs` for full documentation.
   - Decision: accepted.

2. Do we want to keep option group headings (`File Discovery Options`, `Agent Options`)
   in concise help?
   - Decision: keep headings; improves scanability for high-flag CLI.
