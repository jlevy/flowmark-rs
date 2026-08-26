---
name: port-sync
description: >-
  Check whether the upstream Python flowmark has a newer release than the Rust
  port's parity baseline, summarize what changed, and produce a proposed
  port/sync plan for review. Use when opening flowmark-rs, when the user mentions
  syncing or porting a new flowmark version, an upstream release, the parity
  baseline, or asks "what changed upstream" / "are we up to date with Python".
allowed-tools: Bash, Read, Edit, Write
---
# Port Sync — upstream check and plan

Keep flowmark-rs at parity with the Python `flowmark` source. This skill is the
**front door** to a sync: it detects a new upstream release, computes the
discrepancy, and proposes a plan. It **routes to** the canonical runbooks rather
than restating them — read those for the exact, current command sequences:

- `docs/port-sync-playbook.md` — the flowmark-rs Mode B runbook (authoritative)
- `repos/rust-porting-playbook/playbooks/auto-sync-agent-prompt-template.md` — the
  reusable sync agent prompt (differential-sweep discipline, per-change table)
- `repos/rust-porting-playbook/playbooks/python-to-rust-sync-release-workflow.md` —
  Mode A (Rust-only) vs Mode B (upstream baseline change)

## Modes

- **Interactive (default):** run steps 1–4, present the plan, then **stop and ask
  for confirmation or adjustments**. Do not start porting until the user approves.
- **Autonomous (`port-sync --yes`, or when run by a Routine / GitHub Action):**
  run the full Mode B playbook through validation, open a **draft PR**, and write
  the sync artifact. **Never publish the Rust release** — that boundary stays
  human. Escalate on genuine ambiguity instead of guessing (playbook Principle 1).

## Step 0 — Ensure submodules

```bash
git submodule update --init repos/flowmark repos/rust-porting-playbook
```

Most sync work fails confusingly without these.

## Step 1 — Detect versions (cheap; this is the on-open check)

The SessionStart hook `.claude/scripts/port-sync-check.py` already runs this on
open and nudges when upstream is ahead. To check on demand:

```bash
python3 .claude/scripts/port-sync-check.py   # prints a nudge only if upstream is newer
```

Or compute both versions explicitly:

```bash
BASELINE=$(grep -A1 '\[package.metadata.parity\]' Cargo.toml | grep version | sed 's/.*"\(.*\)"/\1/')
git -C repos/flowmark fetch --tags >/dev/null 2>&1
LATEST=$(git -C repos/flowmark tag -l 'v[0-9]*' | sort -V | tail -1)
echo "Current parity baseline: v${BASELINE}"
echo "Latest upstream tag:     ${LATEST}"
```

## Step 2 — If already in sync

If `LATEST` equals `v${BASELINE}`: report "up to date with Python v${BASELINE}"
and stop. (If the user wants a Rust-only stabilization release, that is **Mode A**
in the sync-release-workflow — different path.)

## Step 3 — If upstream is newer: compute the discrepancy

Summarize the gap so the plan is grounded in the real diff:

```bash
git -C repos/flowmark log  --oneline "v${BASELINE}..${LATEST}"
git -C repos/flowmark diff  --name-status "v${BASELINE}..${LATEST}"
```

Then triage test/mapping gaps early (does not modify checked-in files):

```bash
cd python
TMP=$(mktemp)
uv run flowmark-dev discover-python --local-path ../repos/flowmark --output "$TMP"
uv run flowmark-dev check-mapping --python-yaml "$TMP" \
  --rust-yaml ../admin/port-coverage-mapping/rust-tests.yaml \
  --mapping-yaml ../admin/port-coverage-mapping/test-mapping.yaml || true
cd ..
```

Classify each upstream change: **bug fix / new feature / test change /
refactor-noop**. Remember the divergence is bidirectional — comrak may already
implement an upstream fix (port the tests only), or carry its own divergence the
upstream diff never reveals (a real parity bug). The differential sweep, not the
diff, is what proves parity. See the auto-sync template, steps 4–6.

## Step 4 — Produce the proposed plan

Present, for confirmation:

1. **Version move:** `v${BASELINE}` → `${LATEST}`.
2. **Upstream change table** (commit → type → expected Rust impact) — the format
   in the auto-sync template.
3. **Mapping gaps:** new/changed Python tests to port or mark `excluded`.
4. **Required Rust edits** (playbook §"Update all parity version references"):
   `Cargo.toml` parity version, `.github/workflows/ci.yml` `FLOWMARK_PY_VERSION`,
   `python/src/flowmark_dev_tools/cli.py` `DEFAULT_REF`, `README.md` last-sync line,
   `docs/port-status.md`, fixtures in `tests/testdocs/`.
5. **Validation gates** to be run before the PR is considered done:
   - `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -D warnings`
   - `cargo test --all-features`
   - `FLOWMARK_PARITY_PYTHON=1 cargo test --test test_parity_cross_binary`
   - `cd python && uv run flowmark-dev check-mapping`
   - `scripts/generate-parity-golden.sh` + `cargo build --release` + `scripts/corpus-parity-check.sh`
6. **Sync artifact** to be written at
   `docs/sync-artifacts/$(date +%F)-sync-v${BASELINE}-to-${LATEST}.md`.

## Step 5 — Confirm

In interactive mode: ask the user to approve, adjust, or defer the plan. On
approval, execute `docs/port-sync-playbook.md` end-to-end (tests-first), then open
a **draft PR** and write the sync artifact. Stop before publishing the release.
