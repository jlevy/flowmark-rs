# Next sync: Python flowmark v0.6.5 → v0.7.0

**Status:** Ready to start.
**Mode:** B (upstream baseline change).

**Baseline:** Python flowmark `v0.6.5` (current Rust parity target).
**Target:** Python flowmark `v0.7.0` (tagged upstream; `repos/flowmark` main).

This spec is the self-contained kickoff for the next sync agent.
It already contains the upstream triage so the agent can start from a simple prompt (see
bottom). Follow the updated playbook — in particular the **mandatory differential parity
sweep** and the **bidirectional library-divergence check**
([`auto-sync-agent-prompt-template.md`](../../../../repos/rust-porting-playbook/playbooks/auto-sync-agent-prompt-template.md),
[`python-to-rust-sync-release-workflow.md`](../../../../repos/rust-porting-playbook/playbooks/python-to-rust-sync-release-workflow.md)).

## Already done (do NOT redo)

Two v0.7.0 formatter changes were already ported during the v0.6.5 stabilization (PR
#55):

- **`0af9e24` reference-link normalization (issue #45)** → ported as **D18** in
  `tests/test_parity_discrepancies.rs` and `encode_ref_links`/render in
  `src/formatter/filling.rs`. When you run `discover-python` at v0.7.0, the new
  `tests/test_reference_links.py` will appear — **map it to the existing D18 tests**.
- **Thematic-break spacing** → ported as **D17** (this was a Rust-only comrak bug, not
  an upstream change; already correct in both Python versions).

## Upstream changes to triage (v0.6.5 → v0.7.0)

Formatter/behavior-relevant commits (run
`git -C repos/flowmark log --oneline v0.6.5..v0.7.0`):

| Upstream commit | Type | Expected Rust work |
| --- | --- | --- |
| `c9bc36f` feat(wrap): atomic-aware semantic wrapping by default | feature | **Port.** Semantic line wrapping must treat links, inline code, and other atomic spans as unbreakable units (don’t split `[St. John's …](url)` mid-link). Likely changes in `src/formatter/` wrapping. Verify against `testdoc.expected.semantic.md` / `.auto.md` (upstream golden changed). |
| `6c71c82` fix: table rows not broken by line wrapping (#36) | fix | **Verify + port if needed.** Run the new “Wide Table Adjacent to Paragraph” fixture through Rust; comrak may already handle it (bidirectional check). |
| `b4d0f04` / `8cc404c` table separator dash width preservation/normalization | fix | **Verify + port if needed.** Check separator-dash behavior for paragraph-adjacent tables. |
| `e102339`,`9c8e4e1`,`b9f0e24`,`c84e242`,`451db7c`,`de11dbf` public inline API (`flowmark.atomic`, `flowmark.ast`, atomic spans, `split_sentences_atomic`) | feature (library API) | **Decision required.** This is a Python *library* API for consumers, not CLI/formatter output. Decide: expose an equivalent Rust API, or mark the new `tests/test_public_inline_api.py` as `excluded` in the mapping with rationale (Rust lib API differs). The CLI output impact is only via `c9bc36f` (atomic-aware wrapping). |
| SKILL.md content changes | docs | Mirror into `src/skills/SKILL.md` if content differs; port the skill tests. |

New/changed Python test files to map: `tests/test_reference_links.py` (→ D18, done),
`tests/test_public_inline_api.py` (decision above), additions in `tests/test_filling.py`
and `tests/test_wrapping.py` (atomic wrapping — port to Rust wrapping tests).

Upstream golden output changed:
`tests/testdocs/testdoc.expected.{auto,semantic,cleaned,plain}.md` and `testdoc.orig.md`
(new wide-table section).
Refresh these fixtures and re-verify.

## Required process (per updated playbook)

1. Bump `repos/flowmark` submodule to `v0.7.0`;
   `git submodule update --init --recursive`.
2. Install Python flowmark at the target: `uv tool install flowmark==0.7.0` (for
   cross-binary parity tests).
   Keep a v0.7.0 binary available for the differential sweep.
3. Triage each upstream change with the **bidirectional** empirical check: the new
   behavior may already be correct under comrak, OR comrak may diverge — confirm by
   running representative inputs through the existing Rust binary.
4. **Mandatory differential parity sweep** (do not skip even though two items are
   already done): run both binaries over a diverse corpus, diff every file with FULL
   output, and build class-level truth tables for any discrepancy (e.g.
   semantic-wrapping of atomic spans).
   Resolve every diff or record it as an approved tolerated variation.
5. For each new feature: port behavior, port ALL its tests, and refresh the test-mapping
   manifest (`flowmark-dev discover-python/discover-rust/init-mapping/check-mapping`) so
   completeness stays green.
   Update smoke-test count constants in `python/tests/test_smoke.py`.
6. Bump all parity-version references (Cargo.toml `[package.metadata.parity]`,
   `.github/workflows/{ci,publish}.yml` `FLOWMARK_PY_VERSION`,
   `python/src/flowmark_dev_tools/cli.py` `DEFAULT_REF`, `docs/port-status.md`,
   `docs/port-sync-playbook.md`, README via `scripts/generate_rust_readme.py`).
7. Regenerate parity golden (`scripts/generate-parity-golden.sh`) and run the full
   validation gates (fmt, clippy, `cargo test --all-features`, cross-binary parity,
   smoke, check-mapping, corpus parity).
8. Write the sync artifact `docs/sync-artifacts/<date>-sync-v0.6.5-to-v0.7.0.md` and
   move this spec to `done/`.

## Copy/paste kickoff prompt

```text
Sync flowmark-rs from Python flowmark v0.6.5 to v0.7.0 (Mode B).

Read docs/project/specs/active/plan-2026-05-19-next-sync-v0.7.0.md first — it has the
upstream triage and notes that D17 (thematic breaks) and D18 (reference links / issue
#45) are already done; map the new tests/test_reference_links.py to the existing D18
tests, do not re-port.

Follow the rust-porting-playbook Mode B workflow, including the MANDATORY differential
parity sweep (both binaries over a corpus, full untruncated diff, class-level truth
tables) and the bidirectional library-divergence check — comrak may already implement a
fix OR carry its own divergence.

New features to port: atomic-aware semantic line wrapping (c9bc36f) and table-row
wrapping/separator handling (6c71c82, b4d0f04, 8cc404c). For the new public inline API
(flowmark.atomic/ast), decide port-vs-exclude and record the rationale in the mapping.

Port all new/changed upstream tests, refresh the test-mapping manifest and smoke counts,
bump all parity-version references, regenerate goldens and README, and run the full
validation gates. Produce a sync artifact and open a draft PR.
```
