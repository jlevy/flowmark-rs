---
title: Rust-Python Parity E2E QA Playbook
description: Manual end-to-end QA workflow for validating Rust flowmark against Python source and documentation alignment
author: Codex (adapted from tbd QA playbook template)
---
# QA Playbook: Rust-Python Parity End-to-End Validation

Manual QA playbook for validating that the Rust port remains behaviorally aligned with
Python flowmark and that docs/versioning stay consistent.

**Purpose**: Prove that automated tests pass, tryscript golden outputs still make sense
on manual inspection, and documentation/version references are synchronized across Rust
and Python sources.

**Estimated Time**: ~35-50 minutes

> This is a manual validation playbook on top of automated tests.
> Goals:
> 
> - Run all automated checks that are expected for parity confidence.
> - Manually sanity-check tryscript golden sessions for output quality.
> - Verify documentation and parity-version metadata are aligned.

* * *

## Current Status (last update 2026-02-25)

| Phase | Status | Notes |
| --- | --- | --- |
| Phase 1: Setup & Environment | ⏳ Pending | Validate tools, submodules, and version pointers |
| Phase 2: Automated Test Suite | ⏳ Pending | Run full Rust tests + parity/mapping checks |
| Phase 3: Tryscript Sanity Review | ⏳ Pending | Human-read outputs for reasonableness |
| Phase 4: Docs & Version Sync | ⏳ Pending | Verify README/--docs/version alignment |
| Phase 5: Cleanup & Report | ⏳ Pending | Summarize findings and archive evidence |

**Status Legend**: ✅ Passed | ❌ Failed | ⏳ Pending | ⏸️ Blocked

**Test Results (last update 2026-02-25):** *(fill in during execution)*

- `cargo test` → [✅/❌] [summary]
- `cargo test --test test_tryscript_golden` → [✅/❌] [summary]
- Tryscript manual sanity review → [✅/❌] [summary]
- Docs/version alignment checks → [✅/❌] [summary]

**Next Steps:**

1. Run Phase 1 prerequisites and environment checks.
2. Execute all Phase 2 automated checks.
3. Complete manual sanity and docs alignment checks in Phases 3-4.

* * *

**Prerequisites**:

- Rust toolchain installed (`cargo`, matching project MSRV policy)
- Node.js + `tryscript` available (`npx tryscript@latest` or global `tryscript`)
- `uvx` available for Python parity checks
- Git submodules initialized (`repos/flowmark`, `repos/rust-porting-playbook`)
- Working tree in a known state (clean or intentionally dirty, but understood)

* * *

## Related Documentation — Read for Context

- [port-sync-playbook.md](/Users/levy/wrk/github/flowmark-rs/docs/port-sync-playbook.md)
  \- Operational sync process and mapping workflow
- [port-status.md](/Users/levy/wrk/github/flowmark-rs/docs/port-status.md) - Current
  project status and parity scope
- [plan-2026-02-25-cli-help-cleanup.md](/Users/levy/wrk/github/flowmark-rs/docs/project/specs/active/plan-2026-02-25-cli-help-cleanup.md)
  \- Current CLI/help/doc sync plan
- [admin/README.md](/Users/levy/wrk/github/flowmark-rs/admin/README.md) - Mapping/admin
  tooling overview

## Phase 1: Setup & Environment

### 1.1 Verify repository and submodules

```bash
pwd
git submodule status
```

**Expected output**:

- Current directory is repository root.
- `repos/flowmark` and `repos/rust-porting-playbook` are present.

**Verify**:

- [ ] `repos/flowmark` exists and has a checked-out commit
- [ ] `repos/rust-porting-playbook` exists and has a checked-out commit

**Troubleshooting**:

- **Issue**: Submodule missing.
  **Fix**: `git submodule update --init --recursive`

### 1.2 Verify required tools

```bash
cargo --version
uvx --version
node --version
npx tryscript@latest --version
```

**Expected output**:

- All commands return successfully.

**Verify**:

- [ ] Rust tooling available
- [ ] `uvx` available for Python parity checks
- [ ] Node/tryscript available for golden tests

**Troubleshooting**:

- **Issue**: `tryscript` unavailable.
  **Fix**: install Node.js and rerun `npx tryscript@latest --version`

* * *

## Phase 2: Automated Test Suite

### 2.1 Run full Rust test suite

```bash
cargo test
```

**Expected behavior**:

- Entire suite passes.
- No failing unit, integration, parity, or tryscript-wrapper tests.

**Verify**:

- [ ] `test result: ok` across all crates/targets
- [ ] No failed tests in parity/tryscript groups

**Check for ERROR conditions** (any of these = FAIL):

- [ ] No panics or failed tests
- [ ] No silently skipped critical groups

### 2.2 Run explicit golden/tryscript check

```bash
cargo test --test test_tryscript_golden
```

**Expected behavior**:

- All tryscript golden sessions pass.

**Verify**:

- [ ] 12/12 tryscript tests pass (or current expected count)
- [ ] `tryscript_cli_golden`, `tryscript_help`, and `tryscript_verbose_docs` pass

### 2.3 Run file-discovery and skill/docs focused suites

```bash
cargo test --test test_cli_file_discovery
cargo test --test test_skill
```

**Expected behavior**:

- CLI file-discovery behavior and `--skill`/`--docs` tests pass.

**Verify**:

- [ ] No regressions in `--auto`, `--list-files`, stdin, and error-path checks
- [ ] Skill/docs checks pass (including VSCode/Cursor snippet checks)

### 2.4 Run optional cross-binary parity suite (recommended)

```bash
FLOWMARK_PARITY_PYTHON=1 cargo test --test test_parity_cross_binary
```

**Expected behavior**:

- Python and Rust outputs match for the cross-binary parity corpus modes.

**Verify**:

- [ ] `test_cross_binary_corner_cases` passes
- [ ] `test_golden_files_match_python` passes

**Troubleshooting**:

- **Issue**: Python flowmark missing via `uvx`. **Fix**: ensure network access and
  `uvx flowmark@<parity-version> --version` works.

### 2.5 Run mapping integrity checks

```bash
cd python
uv run flowmark-dev discover-python --local-path ../repos/flowmark
uv run flowmark-dev discover-rust
uv run flowmark-dev check-mapping
cd ..
```

**Expected behavior**:

- Mapping check passes with no missing Python test mappings.

**Verify**:

- [ ] Discovery commands complete without errors
- [ ] `check-mapping` passes

* * *

## Phase 3: Tryscript Sanity Review

### 3.1 Manually inspect key tryscript session outputs

```bash
npx tryscript@latest run tests/tryscript/cli-golden.tryscript.md
npx tryscript@latest run tests/tryscript/help.tryscript.md
npx tryscript@latest run tests/tryscript/verbose-docs.tryscript.md
```

**Expected behavior**:

- Sessions pass.
- Output text is coherent, readable, and behaviorally sensible.

**Verify**:

- [ ] Help output is concise and includes expected common usage + `--skill` guidance
- [ ] Error messages read clearly and match intended UX
- [ ] Skill/docs outputs look intentional (not placeholders/truncated nonsense)

**Check for ERROR conditions** (any of these = FAIL):

- [ ] No suspicious output masking that hides real differences
- [ ] No contradictory command behavior within a session

### 3.2 Sanity spot-check representative formatting output

```bash
printf '# Title\n\nFirst sentence. Second sentence that is long enough to wrap.\n' | target/debug/flowmark --semantic -
printf 'He said "hello"...\n' | target/debug/flowmark --smartquotes --ellipses -
```

**Expected behavior**:

- Semantic line break behavior appears reasonable.
- Typography conversions match expectations.

**Verify**:

- [ ] Semantic wrapping is sentence-aware
- [ ] Smart quotes + ellipses conversion is sensible in prose

* * *

## Phase 4: Docs & Version Sync

### 4.1 Verify parity-version alignment

```bash
PARITY_VERSION=$(grep -A1 '\[package.metadata.parity\]' Cargo.toml | grep version | sed 's/.*"\(.*\)"/\1/')
echo "Cargo parity version: $PARITY_VERSION"
uvx "flowmark@${PARITY_VERSION}" --version
```

**Expected output**:

- `uvx` resolves the same version declared in Cargo metadata.

**Verify**:

- [ ] Cargo parity metadata points to a real Python release
- [ ] `uvx` version check succeeds for that exact version

### 4.2 Verify README/docs generation sync

```bash
scripts/generate-rust-readme.sh
git diff -- README.md
```

**Expected behavior**:

- Script runs successfully.
- No diff after generation when repo is already in sync.

**Verify**:

- [ ] README generation is reproducible
- [ ] Rust README reflects Python canonical content + Rust preface

### 4.3 Verify runtime docs output (`--docs`) and help metadata

```bash
target/debug/flowmark --help | head -40
target/debug/flowmark --docs | head -30
target/debug/flowmark --docs | rg -n "Use in VSCode/Cursor|Use in VS Code/Cursor|Agent Use|Configuration|CLI Reference"
```

**Expected behavior**:

- `--help` includes concise guidance and key flags.
- `--docs` shows comprehensive orientation and specific docs sections.

**Verify**:

- [ ] `--help` tagline and bottom guidance are present and readable
- [ ] `--docs` contains major sections expected from canonical docs
- [ ] VSCode/Cursor setup is present in docs output

### 4.4 Verify version references in status/docs are not stale

```bash
cargo pkgid
rg -n "v0\.|parity target|Python parity target|Last updated" docs/port-status.md docs/publishing.md README.md -S
```

**Expected behavior**:

- Version references are consistent or have documented intentional differences.

**Verify**:

- [ ] No obvious version contradictions across core docs
- [ ] Any stale values are captured as follow-up action items

### 4.5 Manual cross-language docs/help equivalence check (Python vs Rust)

```bash
PARITY_VERSION=$(grep -A1 '\\[package.metadata.parity\\]' Cargo.toml | grep version | sed 's/.*\"\\(.*\\)\"/\\1/')

# Capture outputs
target/debug/flowmark --help > /tmp/flowmark-rust-help.txt
target/debug/flowmark --docs > /tmp/flowmark-rust-docs.txt
target/debug/flowmark --skill > /tmp/flowmark-rust-skill.txt

uvx "flowmark@${PARITY_VERSION}" --help > /tmp/flowmark-py-help.txt
uvx "flowmark@${PARITY_VERSION}" --docs > /tmp/flowmark-py-docs.txt
uvx "flowmark@${PARITY_VERSION}" --skill > /tmp/flowmark-py-skill.txt

# Quick headline and guidance checks
head -5 /tmp/flowmark-rust-help.txt
head -5 /tmp/flowmark-py-help.txt
rg -n "Flowmark: Better auto-formatting|Common usage|--skill|--docs" /tmp/flowmark-rust-help.txt /tmp/flowmark-py-help.txt -S
rg -n "^# |Use in VSCode/Cursor|Use in VS Code/Cursor|Agent Use|Configuration|CLI Reference" /tmp/flowmark-rust-docs.txt /tmp/flowmark-py-docs.txt -S
```

**Expected behavior**:

- Rust and Python descriptions and help/docs/skill outputs are substantially equivalent
  in meaning and user guidance.
- Layout differences (argparse vs clap) may exist, but the overall orientation and
  recommended usage should match.

**Verify**:

- [ ] Program description/tagline intent is equivalent in both CLIs
- [ ] `--help` surfaces equivalent key flags and usage guidance (`--auto`,
  `--list-files`, `--skill`, `--docs`)
- [ ] `--docs` and `--skill` are both comprehensive and appropriate for each runtime
  context
- [ ] Differences are intentional and documented (not accidental drift)

* * *

## Phase 5: Cleanup & Report

### 5.1 Cleanup temporary artifacts and summarize

```bash
# Remove any temporary parity debug outputs if generated
git status --short
```

**Verify**:

- [ ] No unintended artifacts left untracked (unless intentionally kept)
- [ ] QA result summary recorded in this file

* * *

## Troubleshooting

### Cross-binary parity tests skip or fail to launch Python

```bash
PARITY_VERSION=$(grep -A1 '\[package.metadata.parity\]' Cargo.toml | grep version | sed 's/.*"\(.*\)"/\1/')
uvx "flowmark@${PARITY_VERSION}" --version
```

**Solution**: Ensure `uvx` can fetch/run the pinned Python package version and network
access is available.

### Tryscript runs but output seems suspiciously weak

```bash
npx tryscript@latest run tests/tryscript/cli-golden.tryscript.md
sed -n '1,220p' tests/tryscript/cli-golden.tryscript.md
```

**Solution**: Inspect for over-broad match patterns and tighten assertions if needed.

### README generation drift

```bash
scripts/generate-rust-readme.sh
git diff -- README.md repos/flowmark/README.md
```

**Solution**: Regenerate README, then commit generated output or investigate canonical
source changes.

* * *

## Success Criteria

Before marking this test as **PASSED**, verify:

- [ ] Full automated Rust suite passes (`cargo test`)
- [ ] Golden/tryscript suite passes and manual sanity review confirms outputs make sense
- [ ] Docs are in sync (`README` generation stable, `--docs` comprehensive)
- [ ] Parity version metadata aligns with Python source version used for checks
- [ ] Python and Rust project description + `--help`/`--docs`/`--skill` outputs are
  substantially equivalent and appropriate
- [ ] Any drift or TODOs are documented with explicit follow-up actions

* * *
