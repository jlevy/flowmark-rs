# flowmark

[![Follow @ojoshe on X](https://img.shields.io/badge/follow_%40ojoshe-black?logo=x&logoColor=white)](https://x.com/ojoshe)
[![CI](https://github.com/jlevy/flowmark-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/jlevy/flowmark-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/flowmark.svg)](https://crates.io/crates/flowmark)
[![docs.rs](https://docs.rs/flowmark/badge.svg)](https://docs.rs/flowmark)
![MSRV](https://img.shields.io/badge/MSRV-1.85-blue)

A Markdown auto-formatter for clean diffs and semantic line breaks.

This is a Rust port of [flowmark](https://github.com/jlevy/flowmark) (Python),
with identical CLI and identical output.
See the [Python project](https://github.com/jlevy/flowmark) for full
documentation, including features, CLI reference, configuration, IDE setup, and
agent use.

## Why the Rust version?

- **Single binary, no runtime** — `cargo install` or download a binary. No Python
  environment needed.
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

## Port status

This crate tracks [flowmark](https://github.com/jlevy/flowmark) **v0.6.4**
with full feature parity:

- Identical formatting output for all supported Markdown constructs
  (CommonMark + GFM: tables, footnotes, alerts, strikethrough, task lists, math)
- Identical CLI flags, configuration files, and file discovery
- All 292 Python tests have verified Rust counterparts — see the
  [port coverage mapping](port-coverage-mapping/) for the full test-by-test
  mapping, which is CI-enforced

The port was created using the
[rust-porting-playbook](https://github.com/jlevy/rust-porting-playbook).
See [docs/port-sync-playbook.md](docs/port-sync-playbook.md) for how the Rust
codebase stays in sync when the Python upstream is updated.

## License

[MIT](LICENSE)
