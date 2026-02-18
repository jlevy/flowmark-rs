# Contributing to flowmark

Thank you for your interest in contributing to flowmark!

## Prerequisites

- Rust 1.85+ (see `rust-version` in Cargo.toml for MSRV)
- cargo (comes with Rust)

## Building

```bash
cargo build --all-features
```

## Testing

```bash
cargo test --all-features
```

## Linting

Before submitting a PR, run the full lint and format checks:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
```

The project uses pedantic clippy lints and denies `unsafe_code` and `unwrap_used`.

## Pull Request Guidelines

- Run the full test suite and linting locally before submitting.
- Keep PRs focused on a single change.
- Write clear commit messages following
  [Conventional Commits](https://www.conventionalcommits.org/) format.

## Releasing

See [docs/publishing.md](docs/publishing.md) for the release process.
