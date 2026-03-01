# Publishing

How to publish a new flowmark release across GitHub Releases, crates.io, PyPI, and
Homebrew with a single orchestrated workflow.

## Prerequisites

### GitHub CLI (`gh`)

Verify `gh` is installed and authenticated:

```bash
gh auth status
```

Expected: logged in to `github.com` with `repo` and `workflow` access.

### Repo variable

```bash
REPO=$(git remote get-url origin | sed -E 's#.*/git/##; s#.*github.com[:/]##; s#\.git$##')
# Expected: jlevy/flowmark-rs
```

Use `--repo $REPO` on `gh` commands below.

## Step 1: Pre-Release Checks

1. Determine the next semver version.

   ```bash
   gh release list --repo "$REPO" --limit 1
   ```

1. Run local quality checks.

   ```bash
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --locked --all-features
   cargo publish --dry-run --locked
   ```

## Step 2: Version Bump

1. Update `Cargo.toml` version.
1. Update `CHANGELOG.md`.
1. Commit the release bump.

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: bump version to X.Y.Z"
```

## Step 3: PR and CI

1. Push branch and open PR.

   ```bash
   git push -u origin <branch>
   gh pr create --repo "$REPO" --head <branch> --base main \
     --title "chore: release vX.Y.Z" \
     --body "Version bump and changelog for vX.Y.Z."
   ```

1. Wait for checks and merge.

   ```bash
   gh pr checks <branch> --repo "$REPO" --watch
   gh pr merge <branch> --repo "$REPO" --squash --delete-branch
   git checkout main && git pull origin main
   ```

## Step 4: Run Release Dry-Run (Required)

Run the release orchestrator in validation mode before publishing:

```bash
gh workflow run release.yml --repo "$REPO" \
  -f tag=dry-run \
  -f publish=false \
  -f publish_prerelease=false
```

Watch the run:

```bash
gh run list --repo "$REPO" --workflow=release.yml --limit 1
gh run watch --repo "$REPO" <run-id>
```

Dry-run validates:
- cross-platform release archive builds
- checksum generation
- crates channel tests + `cargo publish --dry-run`
- PyPI wheel/sdist builds + smoke tests + wheel-content checks + `uv publish --dry-run`

## Step 5: Publish Release

Trigger the real publish by pushing a release tag:

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

The single `release.yml` workflow orchestrates the full pipeline:
1. build release archives and `SHA256SUMS`
2. run crates and PyPI channel workflows in publish mode
3. create/update the GitHub Release
4. update Homebrew tap only after successful crates+PyPI publish (if
   `HOMEBREW_TAP_TOKEN` is configured)

## Step 6: Verify Release Run

```bash
gh run list --repo "$REPO" --workflow=release.yml --limit 1
gh run watch --repo "$REPO" <run-id>
```

Verify release artifacts:

```bash
gh release view vX.Y.Z --repo "$REPO" --json assets --jq '.assets[].name'
```

Expected: 6 platform archives + `SHA256SUMS`.

## Step 7: Verify Channels

1. crates.io: https://crates.io/crates/flowmark
1. PyPI: https://pypi.org/project/flowmark-rs/
1. Install checks:

```bash
cargo install flowmark --force
flowmark --version

uvx flowmark-rs --version
uv tool install flowmark-rs --force
flowmark-rs --version

pip install --upgrade flowmark-rs
flowmark-rs --version
```

1. Homebrew install check:

```bash
brew update
brew upgrade flowmark || true
brew tap jlevy/flowmark
brew install jlevy/flowmark/flowmark
"$(brew --prefix)/bin/flowmark" --version
```

## Step 8: Homebrew Fallback (If Tap Token Not Configured)

If `HOMEBREW_TAP_TOKEN` is not configured in the repo secrets, the release workflow skips
Homebrew update. In that case update
[`jlevy/homebrew-flowmark`](https://github.com/jlevy/homebrew-flowmark) manually using
`SHA256SUMS` from the GitHub Release.

## Release Workflows

- **`release.yml`**: orchestrator (dry-run + publish). Triggered by tag push or
  `workflow_dispatch`.
- **`publish.yml`**: reusable crates channel workflow (`workflow_call`), with rerun-safe
  skip when the crate version already exists.
- **`pypi.yml`**: reusable PyPI channel workflow (`workflow_call`), with wheel/sdist
  validation and rerun-safe duplicate handling via `uv publish --check-url`.

## Trusted Publishing Setup (One-Time)

### crates.io

Configure trusted publisher for:
- repository: `jlevy/flowmark-rs`
- workflow: `publish.yml`
- environment: blank

### PyPI

Configure trusted publisher for:
- project: `flowmark-rs`
- repository: `jlevy/flowmark-rs`
- workflow: `pypi.yml`
- environment: blank

## Rerun and Recovery

Partial success can still happen across external registries, so recovery must be
idempotent. The workflows are designed for safe reruns.

Rerun failed jobs:

```bash
gh run rerun <run-id> --failed
```

Rerun a specific job:

```bash
gh run view <run-id> --json jobs --jq '.jobs[] | {name, databaseId}'
gh run rerun <run-id> --job <databaseId>
```

Behavior on rerun:
- crates: skips publish if crate version already exists
- PyPI: `uv publish --check-url` skips already-uploaded files
- release: creates release if missing, otherwise updates assets with `--clobber`
- Homebrew: no-op if formula already matches target version/checksums

If a bad version is already published to crates.io or PyPI, cut a new patch version.

## Release Notes Format

```markdown
## flowmark X.Y.Z-dev.N+g<hash> (Rust port of flowmark-py A.B.C; base vX.Y.Z)

### What's Changed

#### Features

**Short title of feature**

Description.

#### Bug Fixes

**Short title of fix**

Description.

### Full Changelog

https://github.com/jlevy/flowmark-rs/compare/vPREV...vX.Y.Z
```
