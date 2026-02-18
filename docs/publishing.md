# Publishing

How to publish a new release of flowmark to crates.io and GitHub Releases.

## Pre-Release Checklist

1. Verify all changes committed and pushed:

   ```bash
   git status && git log origin/main..HEAD
   ```

2. Run linting and tests locally:

   ```bash
   cargo fmt --check && cargo clippy --all-targets --all-features && cargo test --all-features
   ```

3. Confirm CI is passing:

   ```bash
   gh run list --limit 3
   ```

4. Determine the next version number (semver).
   Check the latest release:

   ```bash
   gh release list --limit 1
   ```

5. Update `Cargo.toml`:
   - Bump `version` field
   - Update `[package.metadata.parity]` version if Python parity has changed

6. Verify the crate packages correctly:

   ```bash
   cargo publish --dry-run
   ```

7. Commit the version bump and push to main.

## Creating a Release

1. Create a GitHub Release using the CLI:

   ```bash
   gh release create v0.X.Y --title "flowmark v0.X.Y" --generate-notes
   ```

   Or create it via the GitHub web UI with structured release notes (see below).

2. The publish workflow (`.github/workflows/publish.yml`) triggers automatically on
   release publication.
   It runs the full test suite and publishes to crates.io.

3. Verify the publish succeeded:

   ```bash
   gh run list --workflow=publish.yml --limit 1
   ```

4. Verify on crates.io: https://crates.io/crates/flowmark

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

**Publish fails with authentication error:**
Verify trusted publishing is configured on crates.io (see above).

**Publish fails with "crate already exists":**
The version in `Cargo.toml` must be higher than the latest published version.
Check: https://crates.io/crates/flowmark/versions

**Tests fail in publish workflow:**
The publish workflow runs `cargo test --locked --all-features` before publishing.
Fix the failing tests, push to main, and create a new release.
