# Publishing

How to publish a new release of flowmark to crates.io and GitHub Releases.

This is the end-to-end process, from version bump through crates.io publication.

## Prerequisites

### GitHub CLI (`gh`)

The `gh` CLI is required for creating releases and monitoring CI.
Verify it is installed and authenticated:

```bash
gh auth status
```

Expected: "Logged in to github.com" with your account.
Required token scopes: `repo`, `workflow`.

If `gh` is not set up, install it from https://cli.github.com/ and authenticate with
`gh auth login` or by setting the `GH_TOKEN` environment variable.

**Agent note:** In Claude Code Cloud sessions, `gh` is auto-installed by the
SessionStart hook.
If it isn't working, run `tbd shortcut setup-github-cli`.

### Repo variable

Many `gh` commands need `--repo` when the git remote URL doesn't point directly at
GitHub (e.g., in Claude Code Cloud sessions using a local proxy).
Set it once:

```bash
REPO=$(git remote get-url origin | sed -E 's#.*/git/##; s#.*github.com[:/]##; s#\.git$##')
# Should resolve to: jlevy/flowmark-rs
```

Use `--repo $REPO` on all `gh` commands below.

## Step 1: Pre-Release Checks

1. Determine the next version number (semver).
   Check the latest release:

   ```bash
   gh release list --repo $REPO --limit 1
   ```

2. Run linting and tests locally:

   ```bash
   cargo build --all-features && cargo fmt --check && cargo clippy --all-targets --all-features && cargo test --all-features
   ```

   Note: `cargo build` must run before `cargo test` because some integration tests
   invoke the compiled binary.
   Cross-binary parity tests (D11) require Python flowmark installed and will be skipped
   locally if unavailable — they run in CI.

## Step 2: Version Bump

1. Update `Cargo.toml`:
   - Bump the `version` field
   - Update `[package.metadata.parity]` version if Python parity has changed

2. Update `CHANGELOG.md`:
   - Move items from `[Unreleased]` into a new version section
   - Add the new version's comparison link at the bottom
   - Follow the release notes guidelines (`tbd guidelines release-notes-guidelines`)

3. Verify the crate packages correctly:

   ```bash
   cargo publish --dry-run
   ```

## Step 3: PR and CI

1. Commit the version bump on a release branch:

   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: bump version to X.Y.Z for release"
   ```

2. Push and create a PR:

   ```bash
   git push -u origin <branch-name>
   gh pr create --repo $REPO --head <branch-name> --base main \
     --title "chore: release vX.Y.Z" --body "Version bump and changelog for vX.Y.Z."
   ```

3. Wait for CI to pass (all 12 checks):

   ```bash
   gh pr checks <branch-name> --repo $REPO --watch 2>&1
   ```

   **Important:** The `--watch` flag blocks until all checks complete.
   Do not proceed until you see the final summary showing all checks passed.

4. Merge the PR:

   ```bash
   gh pr merge <branch-name> --repo $REPO --squash --delete-branch
   ```

5. Pull the merged main:

   ```bash
   git checkout main && git pull origin main
   ```

## Step 4: Create the GitHub Release

Create a GitHub Release, which automatically triggers the publish workflow:

```bash
gh release create vX.Y.Z --repo $REPO \
  --title "flowmark vX.Y.Z" \
  --notes "$(cat <<'EOF'
<release notes here — see format below>
EOF
)"
```

The publish workflow (`.github/workflows/publish.yml`) triggers on release publication.
It runs the full test suite and publishes to crates.io via OIDC trusted publishing.

## Step 5: Verify Publication

1. Watch the publish workflow:

   ```bash
   gh run list --repo $REPO --workflow=publish.yml --limit 1
   gh run watch --repo $REPO <run-id>
   ```

2. Verify on crates.io: https://crates.io/crates/flowmark

3. Test installation:

   ```bash
   cargo install flowmark
   flowmark --version
   ```

## Release Notes Format

```markdown
## flowmark vX.Y.Z (parity: flowmark-py vA.B.C)

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

## Trusted Publishing (OIDC)

The publish workflow uses OpenID Connect (OIDC) trusted publishing to authenticate with
crates.io.
This means no `CARGO_REGISTRY_TOKEN` secret is needed in the repository.

To set this up (one-time):

1. Go to https://crates.io/settings/tokens
2. Add a trusted publisher with:
   - GitHub repository: `jlevy/flowmark-rs`
   - Workflow: `publish.yml`
   - Environment: (leave blank)

## Troubleshooting

**`gh` commands fail with "none of the git remotes configured...":**
Use `--repo jlevy/flowmark-rs` on all `gh` commands.
This is required when the git remote uses a proxy URL (e.g., Claude Code Cloud).

**Publish fails with authentication error:**
Verify trusted publishing is configured on crates.io (see above).

**Publish fails with "crate already exists":**
The version in `Cargo.toml` must be higher than the latest published version.
Check: https://crates.io/crates/flowmark/versions

**Tests fail in publish workflow:**
The publish workflow runs `cargo test --locked --all-features` before publishing.
Fix the failing tests, push to main, and create a new release.
