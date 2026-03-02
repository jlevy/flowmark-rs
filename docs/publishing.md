# Publishing

This is the canonical release process for `flowmark-rs` going forward, including
Homebrew.

## Variables

```bash
REPO=jlevy/flowmark-rs
TAP_REPO=jlevy/homebrew-flowmark
VERSION=X.Y.Z
TAG=v${VERSION}
```

## 0. Preconditions

1. Release bump PR is merged to `main` (`Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`,
   README sync, etc.).
1. `gh` is authenticated (`gh auth status`).
1. Local checkout is clean.

## 1. Required Dry-Run

```bash
gh workflow run release.yml --repo "$REPO" -r main \
  -f tag=dry-run \
  -f publish=false \
  -f publish_prerelease=false

gh run list --repo "$REPO" --workflow release.yml --limit 1
gh run watch --repo "$REPO" <run-id>
```

Do not publish until this run is green.

## 2. Publish Crate First (Current Required Order)

Current crates.io trusted publishing is bound to `publish.yml`; running crates publish
from `release.yml` can fail auth. So publish crate first:

```bash
gh workflow run publish.yml --repo "$REPO" -r main -f publish=true
gh run list --repo "$REPO" --workflow publish.yml --limit 1
gh run watch --repo "$REPO" <run-id>
```

Verify:

```bash
curl -fsSL "https://crates.io/api/v1/crates/flowmark/${VERSION}" >/dev/null
```

## 3. Run Real Orchestrated Release

```bash
gh workflow run release.yml --repo "$REPO" -r main \
  -f tag="$TAG" \
  -f publish=true \
  -f publish_prerelease=false

gh run list --repo "$REPO" --workflow release.yml --limit 1
gh run watch --repo "$REPO" <run-id>
```

Expected:
- crates job skips as already published
- PyPI publishes
- GitHub release/tag is created or updated

## 4. Verify Release Outputs

```bash
gh release view "$TAG" --repo "$REPO" --json url,publishedAt,isDraft,isPrerelease
gh release view "$TAG" --repo "$REPO" --json assets --jq '.assets[].name'
```

Verify channels:

```bash
curl -fsSL "https://crates.io/api/v1/crates/flowmark/${VERSION}" | jq -r '.version.num'
python3 - <<'PY'
import json, urllib.request
data = json.load(urllib.request.urlopen("https://pypi.org/pypi/flowmark-rs/json", timeout=10))
print(data["info"]["version"])
PY
```

## 5. Update Homebrew Tap (Required Manual Step)

Download checksums from the release:

```bash
workdir="$(mktemp -d)"
gh release download "$TAG" --repo "$REPO" --pattern SHA256SUMS --dir "$workdir"
gh repo clone "$TAP_REPO" "$workdir/homebrew-flowmark"
```

Update `Formula/flowmark.rb` in the tap repo:
- set `version` to `${VERSION}`
- update 4 `sha256` entries from `SHA256SUMS` for:
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

## 6. Final Verification

```bash
brew update
brew tap jlevy/flowmark
brew upgrade flowmark || true
brew install jlevy/flowmark/flowmark
flowmark --version
```

## Recovery and Reruns

- If `release.yml` fails after crate publish, fix and rerun `release.yml` with same tag.
- `crates` is idempotent (already-published versions are skipped).
- PyPI publish is rerun-safe (`uv publish --check-url`).
- If Homebrew update is wrong, push a follow-up commit in `homebrew-flowmark`.

## One-Time Settings Checklist

- crates.io trusted publisher must allow releases from this repo. Today the stable path
  is `publish.yml`; if you want one-step-only release from `release.yml`, add that
  workflow in crates trusted publisher settings as well.
- PyPI trusted publisher should be configured for project `flowmark-rs` and workflow
  `pypi.yml` (environment blank).
