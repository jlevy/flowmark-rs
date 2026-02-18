# flowmark

[![CI](https://github.com/jlevy/flowmark-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/jlevy/flowmark-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/flowmark.svg)](https://crates.io/crates/flowmark)
[![docs.rs](https://docs.rs/flowmark/badge.svg)](https://docs.rs/flowmark)
![MSRV](https://img.shields.io/badge/MSRV-1.85-blue)
[![codecov](https://codecov.io/gh/jlevy/flowmark-rs/graph/badge.svg)](https://codecov.io/gh/jlevy/flowmark-rs)

A Markdown auto-formatter for clean diffs and semantic line breaks.

This is a Rust port of [flowmark](https://github.com/jlevy/flowmark) (Python).
Identical CLI, identical output.
The port was created and fully tested using the
[rust-porting-playbook](https://github.com/jlevy/rust-porting-playbook).

See the [Python project](https://github.com/jlevy/flowmark) for full documentation,
including features, CLI reference, configuration, IDE setup, and agent use.

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

## Library Usage

```rust
use flowmark::FormatOptions;

let opts = FormatOptions::default();
let formatted = opts.reformat_text("# Hello\n\nSome text.");
assert_eq!(formatted, "# Hello\n\nSome text.\n");
```

See [docs.rs/flowmark](https://docs.rs/flowmark) for full API documentation.

## License

[MIT](LICENSE)
