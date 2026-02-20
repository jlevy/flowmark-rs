# flowmark

[![Follow @ojoshe on X](https://img.shields.io/badge/follow_%40ojoshe-black?logo=x&logoColor=white)](https://x.com/ojoshe)
[![CI](https://github.com/jlevy/flowmark-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/jlevy/flowmark-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/flowmark.svg)](https://crates.io/crates/flowmark)
[![docs.rs](https://docs.rs/flowmark/badge.svg)](https://docs.rs/flowmark)
![MSRV](https://img.shields.io/badge/MSRV-1.85-blue)

Flowmark is a Markdown auto-formatter designed for **better LLM workflows**, **clean git
diffs**, and **flexible use from CLI, from IDEs, or as a library**.

## Rust Port of Python Flowmark

This is an auto-synced Rust port of the
[Python version](https://github.com/jlevy/flowmark).
The original Python version is well tested and this port aims for identical CLI usage
and formatting behavior.

Last sync: **2026-02-19** against **Python v0.6.4**

## More Info

[**See the Python version**](https://github.com/jlevy/flowmark) for full documentation,
including features, CLI reference, configuration, IDE setup, and agent use.

## Why the Rust version?

- **Single binary, no runtime** — `cargo install` or download a binary.
  No Python environment needed.
- **Fast** — useful for large repos or CI pipelines.
- **Library crate** — embed formatting in Rust toolchains via
  [docs.rs/flowmark](https://docs.rs/flowmark).

## Installation

Install from [crates.io](https://crates.io/crates/flowmark):

```bash
cargo install flowmark
```

Or download a pre-built binary from
[GitHub Releases](https://github.com/jlevy/flowmark-rs/releases).

## Usage

```bash
# Format a file in place
flowmark --auto myfile.md

# Format all markdown in current directory
flowmark --auto .

# Read from stdin, write to stdout
cat myfile.md | flowmark -
```

## Library usage

```rust
use flowmark::FormatOptions;

let opts = FormatOptions::default();
let formatted = opts.reformat_text("# Hello\n\nSome text.");
assert_eq!(formatted, "# Hello\n\nSome text.\n");
```

See [docs.rs/flowmark](https://docs.rs/flowmark) for full API documentation.

## Port Status

This crate tracks [flowmark](https://github.com/jlevy/flowmark) with full feature
parity:

- Identical formatting output for all supported Markdown constructs (CommonMark + GFM:
  tables, footnotes, alerts, strikethrough, task lists, math)
- Identical CLI flags, configuration files, and file discovery
- 430 tests (0 ignored, 0 failures), including 31 cross-language parity tests
- All 292 Python tests have verified Rust counterparts — see the
  [port coverage mapping](admin/port-coverage-mapping/) for the full test-by-test
  mapping, which is CI-enforced

The port was created using the
[rust-porting-playbook](https://github.com/jlevy/rust-porting-playbook).

## Documentation

| Document | Description |
| --- | --- |
| [docs/port-status.md](docs/port-status.md) | **Full project status**, release readiness, architecture, and doc index |
| [docs/publishing.md](docs/publishing.md) | Release process and crates.io publishing |
| [docs/port-sync-playbook.md](docs/port-sync-playbook.md) | How the Rust codebase stays in sync with Python upstream |
| [admin/](admin/) | Port administration: test mapping, dev tools, maintenance procedures |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Build, test, and lint instructions |
| [CHANGELOG.md](CHANGELOG.md) | Release notes |

## License

[MIT](LICENSE)
