# Publishing (Canonical Hybrid Rust/Python)

> **Doc status:** Parallel to upstream
> [`docs/publishing.md`](https://github.com/jlevy/flowmark/blob/main/docs/publishing.md).
> Both cover the same role; content differs because the Rust toolchain (cargo,
> multi-channel release) and the Python toolchain (uv, PyPI-only) require different
> commands and steps.

This file is the canonical release runbook for `flowmark-rs` and the template baseline
for future hybrid Rust/Python projects.

## Release Invariants

1. Complex workflow logic must live in testable scripts (`scripts/*.py`), not long
   inline bash in Actions YAML.
2. A release tag (`vX.Y.Z`) must flow into build env (`FLOWMARK_RELEASE_TAG`) so built
   binaries/wheels embed the stable version, not `dev.unknown`.
3. Release steps must be rerun-safe:
   - crates publishes are skip-safe when version already exists
   - PyPI publishes are skip-safe via `uv publish --check-url`
   - GitHub release updates are idempotent
4. Homebrew tap updates remain explicit and auditable (manual commit in tap repo).

## Execution Defaults (for agents — no clarification needed)

When asked to “publish the release” / “do the release” / “follow the release process”,
treat that as full authorization to run this runbook end-to-end with these defaults.
Do **not** stop to ask about any of the following — they are pre-decided:

1. **Drive the whole flow.** The agent prepares the release-prep change (version +
   `CHANGELOG.md`), opens the PR, waits for CI green, **merges it to `main`**, then runs
   the publish workflows.
   Merging the agent’s own release-prep PR is part of the process.
2. **Version = current `Cargo.toml` version.** If `CHANGELOG.md` lacks that version’s
   section, add it (that is the release-prep change).
   Never bump the version number without explicit instruction.
3. **Publishing is authorized and runs without pausing.** crates.io + PyPI are
   immutable, but “publish the release” is the go-ahead.
   Run the **dry-run gate first**; if it is green, proceed straight through
   `publish.yml` (crates) and `release.yml` (PyPI + GitHub release) without asking
   again. If the dry-run **fails**, stop and report.
4. **Homebrew (Phase 3): the agent does it** via `gh` credentials
   (`git commit --no-gpg-sign`,
   `git -c credential.helper='!gh auth git-credential' push`), after the GitHub release
   publishes `SHA256SUMS`. Only escalate if tap auth genuinely fails.
5. **CI status checks:** a `DeepSource` check reporting `failure` with body “Analysis in
   progress…” is a transient placeholder, not a real finding — wait for it to flip
   rather than treating it as a blocker.
   All first-party gates (fmt, clippy, tests, parity, semver, README/markdown) must be
   genuinely green.
6. **Stop only for:** a failed dry-run, a real (non-placeholder) CI failure, a genuine
   secret/security finding, or a registry/tag collision (version already published).

## Variables

```bash
REPO=jlevy/flowmark-rs
TAP_REPO=jlevy/homebrew-flowmark
VERSION=X.Y.Z
TAG=v${VERSION}
```

## One-Time Setup

1. `gh auth status` succeeds for the maintainer account.
2. crates.io trusted publisher is configured for this repo.
   - Current working path can be `publish.yml` (crate-first flow).
   - If you want fully one-step release from `release.yml`, add that workflow in crates
     trusted publisher settings too.
3. PyPI trusted publisher is configured for project `flowmark-rs`, workflow `pypi.yml`,
   environment blank.
4. Homebrew tap access is configured for `jlevy/homebrew-flowmark` (maintainer account
   or token with push access).

## Phase 0: Preflight

1. Ensure release bump is merged to `main`:
   - `Cargo.toml`
   - `Cargo.lock`
   - `CHANGELOG.md`
   - README/docs sync updates
2. Confirm local checkout is clean and on current `main`.
3. Run required dry-run orchestration:

```bash
gh workflow run release.yml --repo "$REPO" -r main \
  -f tag=dry-run \
  -f publish=false \
  -f publish_prerelease=false

gh run list --repo "$REPO" --workflow release.yml --limit 1
gh run watch --repo "$REPO" <run-id>
```

Do not proceed until dry-run is green.

## Phase 1: Publish Registry Channels

### 1A. Publish crate first (current trusted-publisher-safe path)

```bash
gh workflow run publish.yml --repo "$REPO" -r main -f publish=true
gh run list --repo "$REPO" --workflow publish.yml --limit 1
gh run watch --repo "$REPO" <run-id>
```

Verify crate availability (crates.io requires a `User-Agent` or it returns HTTP 403):

```bash
curl -fsSL -H "User-Agent: flowmark-release-check (maintainer@example.com)" \
  "https://crates.io/api/v1/crates/flowmark/${VERSION}" >/dev/null
```

### 1B. Run orchestrated tagged release

This publishes PyPI and creates/updates the GitHub release.
The workflow passes `release_tag` through to reusable publish workflows so compiled
artifacts embed the stable version string.

```bash
gh workflow run release.yml --repo "$REPO" -r main \
  -f tag="$TAG" \
  -f publish=true \
  -f publish_prerelease=false

gh run list --repo "$REPO" --workflow release.yml --limit 1
gh run watch --repo "$REPO" <run-id>
```

Expected:
1. crates job skips as already published
2. PyPI publishes
3. GitHub release/tag is created or updated

## Phase 2: Verify Outputs

GitHub release:

```bash
gh release view "$TAG" --repo "$REPO" --json url,publishedAt,isDraft,isPrerelease
gh release view "$TAG" --repo "$REPO" --json assets --jq '.assets[].name'
```

Registry versions:

```bash
# crates.io rejects requests without a User-Agent (HTTP 403), so always send one.
curl -fsSL -H "User-Agent: flowmark-release-check (maintainer@example.com)" \
  "https://crates.io/api/v1/crates/flowmark/${VERSION}" | jq -r '.version.num'

# On PyPI, info.version is the aggregate "latest" field and can lag for a minute or
# two right after upload — check the version-specific release instead so a fresh
# publish is not reported as missing.
python3 - "$VERSION" <<'PY'
import json, sys, urllib.request
version = sys.argv[1]
data = json.load(urllib.request.urlopen("https://pypi.org/pypi/flowmark-rs/json", timeout=10))
print("published" if version in data["releases"] else f"MISSING (info.version={data['info']['version']})")
PY
```

CLI version smoke check from PyPI package:

```bash
uvx "flowmark-rs==${VERSION}" --version
```

Expected output starts with `flowmark ${VERSION}` and does not include
`-dev.unknown+gunknown`.

## Phase 3: Homebrew Tap Update (Manual, Required)

Download checksums and clone tap:

```bash
workdir="$(mktemp -d)"
gh release download "$TAG" --repo "$REPO" --pattern SHA256SUMS --dir "$workdir"
gh repo clone "$TAP_REPO" "$workdir/homebrew-flowmark"
```

Update `Formula/flowmark.rb`:
1. Set `version` to `${VERSION}`.
2. Update the four `sha256` values from `SHA256SUMS`:
   - `aarch64-apple-darwin`
   - `x86_64-apple-darwin`
   - `aarch64-unknown-linux-musl`
   - `x86_64-unknown-linux-musl`

Commit and push:

```bash
cd "$workdir/homebrew-flowmark"
git add Formula/flowmark.rb
git commit -m "Update flowmark to ${TAG}"
git push origin main
```

> **Releasing from an automated/ephemeral environment** (e.g. Claude Code on the web):
> the tap is a *separate* repo cloned over HTTPS, so the local checkout’s commit-signing
> and credential setup do not apply to it.
> If `git commit` fails with a signing error, add `--no-gpg-sign`; if `git push` fails
> with `could not read Username for 'https://github.com'`, supply credentials via the
> `gh` helper:
>
> ```bash
> git commit --no-gpg-sign -m "Update flowmark to ${TAG}"
> git -c credential.helper='!gh auth git-credential' push origin main
> ```

Homebrew verification:

```bash
brew update
brew tap jlevy/flowmark
brew upgrade flowmark || true
brew install jlevy/flowmark/flowmark
flowmark --version
```

## Failure Handling and Partial Success

Cross-channel publishing is not atomic.
Treat release as a controlled two-phase process:
1. Publish immutable registries first (crates, PyPI).
2. Update Homebrew only after registries and GitHub release are confirmed good.

If a channel fails:
1. Fix the issue.
2. Rerun the same workflow/tag.
3. Verify channel state before proceeding.

Current channel behavior:
1. crates: already-published versions are detected and skipped.
2. PyPI: `uv publish --check-url` avoids duplicate uploads.
3. Homebrew: push a corrective follow-up formula commit if needed.

## Template Guidance (For Upstream Playbook)

Use this as the baseline pattern for hybrid Rust/Python projects:
1. `release.yml` orchestrates packaging + checksums + announce.
2. Reusable channel workflows (`publish.yml`, `pypi.yml`) do channel-specific publish.
3. Workflow decision logic is delegated to tested `scripts/*.py`.
4. Script tests run in CI
   (`python3 -m unittest discover -s scripts/tests -p 'test_*.py'`).
5. Naming conventions stay consistent (`snake_case.py` for Python scripts).
