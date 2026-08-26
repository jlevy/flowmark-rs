# Baseline Audit: Python Flowmark v0.7.2 to v0.7.3

This artifact closes the released-baseline classification required before the Markdown
preservation port. It covers exactly `v0.7.2..v0.7.3`; later shared preservation work is
tracked separately by `FM-*` change IDs.

## Boundary

| Reference | Commit |
| --- | --- |
| Python v0.7.2 | `32d367fc189f5aac3d6d473c2d8ec2ecf5e2676f` |
| Python v0.7.3 | `7912c322417ae49c5c45ab099997c142cf392db8` |
| Rust v0.3.2 merge | `c6449a5cba5b069c5bd29be0e08e83d036279b59` |

The Python range contains two commits:

1. `593f70d`: consolidate Flowmark skill guidance.
2. `7912c32`: prepare the Python v0.7.3 release.

## Classification and Disposition

| Surface | Python change | Rust disposition | Evidence |
| --- | --- | --- | --- |
| Formatter algorithms | No change to parsing, wrapping, cleanups, typography, or rendered Markdown | No Rust formatter change | The source range touches no formatter module |
| Skill bundle | Move to format `f03`; bundle `references/project-setup.md`; validate both artifacts; publish the reference before `SKILL.md`; update generated guidance | Already implemented idiomatically in Rust | Rust `7a7c332`; `src/skills/mod.rs`; `tests/test_skill.rs`; `tests/test_skill_artifacts.rs` |
| Skill pinning | Publish Python `0.7.3` and Rust `0.3.2` discovery pins | Advance the sibling Python pin and declared parity baseline to `0.7.3`; retain Rust `0.3.2` | `Cargo.toml`; `src/skills/mod.rs`; regenerated `README.md` |
| Skill public helpers | Add project-setup content, composition, rendering, and discovery helpers | Equivalent Rust helpers already exist; no new API work | Supplemental mapping links every new Python test to a concrete Rust test |
| File resolver | Add generic `PathSpec[Pattern]` type aliases and annotations | No port; type-only Python change with identical runtime operations | Direct source diff of both resolver modules |
| CLI behavior | No new formatter or file-operation flag; skill output changes with the bundle | Existing Rust skill CLI and direct upstream tryscript cover the observable delta | `tests/test_skill_cli.rs`; upstream `verbose-docs` tryscript |
| Runtime dependencies | No Python runtime dependency change relevant to formatter behavior | No Rust dependency change | `pyproject.toml` runtime dependency list is unchanged |
| Build and development dependencies | Pin build backends; update uv, test, lint, type-check, and audit tooling | Do not copy Python packages or workflow mechanics into Cargo | Rust v0.3.2 already has its independently reviewed locked toolchain and supply-chain gates |
| CI, publishing, template, and license | Adopt current Python template/release workflow and generated maintenance content | Python-only, except for the version declaration and generated shared documentation above | Rust workflows remain Cargo-native and already passed release/package checks |
| tbd and agent guidance | Refresh repository-local workflow instructions | No runtime port; Rust uses its own current tbd and playbook integration | Repository instructions and playbook submodule are independently current |

There is no unclassified formatter, Markdown syntax, file-selection, exit-status, or
runtime-dependency delta in this released range.

## Supplemental Test Inventory

The inventory tool now discovers both Python `test_*.py` functions and the
language-neutral `*.tryscript.md` suites.
Discovery is authoritative: renamed or deleted tests disappear instead of surviving as
phantom hand-preserved rows.

At exact tag `v0.7.3` the inventory contains:

- 430 Python test functions and methods;
- 12 language-neutral tryscript suites;
- 442 total source records;
- 395 mapped Rust counterparts;
- 47 explicit language- or implementation-specific exclusions;
- zero missing entries and zero broken Rust references.

Eight cases became visible during this refresh: seven skill-bundle cases and the shared
help transcript. All eight already have executable Rust counterparts.
The YAML map remains supplemental; direct shared conformance and tryscript execution are
the portable acceptance authority.

## Validation

The following checks pass after the baseline update:

```bash
cd python
uv run flowmark-dev discover-python --ref v0.7.3
uv run flowmark-dev discover-rust
uv run flowmark-dev init-mapping
uv run flowmark-dev check-mapping
uv run ruff check .
uv run basedpyright
uv run pytest
```

The administration suite reports 16 passing tests.
The only warnings are the three pre-existing pytest collection warnings caused by the
`TestType` enum name in package modules.

The affected Rust gates also pass: formatting, all-target/all-feature Clippy, the full
all-feature suite, the no-default-feature suite, rustdoc tests, and API documentation.
A release-mode differential run of pinned Python v0.7.3 and the Cargo-built Rust binary
reported zero differences across all five upstream reference documents.
The focused syntactic-surface sweep passed all 35 classes.

The Rust branch may now declare whole-program parity with released Python v0.7.3. That
declaration does not include the later preservation-contract commits through `093c924`;
those remain an in-progress implementation target.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
