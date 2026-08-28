# Contributing to flowmark

> **Doc status:** Rust port-specific (no upstream equivalent).
> Upstream Python contributing guidance is folded into
> [`docs/development.md`](https://github.com/jlevy/flowmark/blob/main/docs/development.md);
> the Rust port keeps this file separate for the Rust contributor onboarding flow.

Thank you for your interest in contributing to flowmark!

## Prerequisites

- Rust 1.85+ (see `rust-version` in Cargo.toml for MSRV)
- cargo (comes with Rust)
- Git submodules initialized with `git submodule update --init --recursive`
- Node.js 22+ and tryscript 0.1.7 for golden CLI tests
  (`npm install -g tryscript@0.1.7`)

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

The full suite includes Rust unit and integration tests, a native runner for the pinned
upstream conformance manifest, and the upstream tryscript suite.
Portable tests read their reviewed assets directly from `repos/flowmark`; they do not
invoke Python or keep synchronized fixture copies in this repository.

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

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
