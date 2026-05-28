# Contributing to flowmark

Thank you for your interest in contributing to flowmark!

## Prerequisites

- Rust 1.85+ (see `rust-version` in Cargo.toml for MSRV)
- cargo (comes with Rust)
- Python flowmark v0.7.0, for cross-binary parity tests
  (`uv tool install flowmark==0.7.0`)
- Node.js 22+ and tryscript, for golden CLI tests (`npm install -g tryscript@latest`)

All test dependencies are required.
Tests fail loudly when a dependency is missing, there is no skip logic.
See
[Principle 6](repos/rust-porting-playbook/guidelines/porting-principles-and-antipatterns.md)
of the porting playbook.

## Building

```bash
cargo build --all-features
```

## Testing

```bash
cargo test --all-features
```

The full test suite (501 tests) includes unit tests, integration tests, D11 cross-binary
parity tests (requires Python flowmark), and tryscript golden tests (requires
tryscript).

For full project structure, CI pipeline details, and architecture, see
[`docs/development.md`](docs/development.md).

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
