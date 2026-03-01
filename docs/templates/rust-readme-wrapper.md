<!-- Generated from shared docs source
(repos/flowmark/docs/shared/flowmark-readme-shared.md) via
scripts/generate-rust-readme.py.
-->

# flowmark

[![Follow @ojoshe on X](https://img.shields.io/badge/follow_%40ojoshe-black?logo=x&logoColor=white)](https://x.com/ojoshe)
[![CI](https://github.com/jlevy/flowmark-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/jlevy/flowmark-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/flowmark.svg)](https://crates.io/crates/flowmark)
[![docs.rs](https://docs.rs/flowmark/badge.svg)](https://docs.rs/flowmark)
![MSRV](https://img.shields.io/badge/MSRV-{{ msrv }}-blue)

## Rust Port of Python Flowmark

> [!TIP]
> This is a 100% agent-written, auto-synced Rust port of
> [**Python Flowmark**](https://github.com/jlevy/flowmark), the original reference
> implementation.
> 
> This Rust port has carefully tested identical CLI usage and formatting behavior, while
> giving 50x+ faster performance processing large numbers of files.
> So it is now the recommended version for CLI and IDE usage.

Last sync: **{{ last_sync_date }}** against **Python v{{ parity_version }}**

- Port sync process: [`docs/port-sync-playbook.md`](docs/port-sync-playbook.md)
- Porting methodology:
  [rust-porting-playbook](https://github.com/jlevy/rust-porting-playbook)

## Why the Rust Version?

- **Single binary**: install via Cargo, Cargo binstall, or Homebrew.
- **Fast CLI**: good for large repos and CI pipelines.
- **Library crate**: embed in Rust tooling via
  [docs.rs/flowmark](https://docs.rs/flowmark).

### Performance

TLDR:
- If you use it for auto-save in your IDE, it feels instant.
- If you run it on 1000 documents over and over in your build system, it only takes
  milliseconds.

Flowmark is now arguably the most sophisticated Markdown autoformatter, given its
advanced wrapping and typographic rules.
But because it was pure Python, it was never highly performant.

Now flowmark-rs has identical functionality and in a rough benchmark is the #1 fastest
Markdown formatter for repeated runs of large numbers of documents, the #2 fastest on
new documents, and 50X or more faster than other TypeScript or Python formatters.

Fresh-run cross-formatter ranking (profiled benchmark suite, 928 files / 8.8 MB):

| Rank | Formatter | Mean (fresh) | Relative speed |
| --- | --- | --- | --- |
| 1 | dprint | 0.37 s | 1.0x |
| 2 | **flowmark-rs** | **0.73 s** | **2.0x** |
| 3 | markdownfmt | 0.95 s | 2.6x |
| 4 | prettier | 38.0 s | 103x |
| 5 | flowmark-py | ~48 s | ~130x |
| 6 | mdformat | 72.9 s | 197x |

Cached second run (unchanged files, warm cache):

| Formatter | Mean (cached) | Relative speed |
| --- | --- | --- |
| **flowmark-rs** (`--auto`) | **0.023 s** | **1.0x** |
| **dprint** (`fmt`) | **0.031 s** | **1.3x** |

So on the same corpus flowmark-rs is roughly **60-70x faster than flowmark-py**.

### Rust-Only Features

The only exception to the exact parity of the port of Python Flowmark are these
Rust-only performance features:

- incremental cache (`--no-cache`, `--cache-dir`, `--incremental`, `--show-cache`,
  `--clear-cache`)
- stage-level performance stats (`--perf-stats`)

See [`docs/rust-only-features.md`](docs/rust-only-features.md) for a concise feature
matrix and [`docs/cache.md`](docs/cache.md) for cache behavior details.

See [`benchmarks/REPORT.md`](benchmarks/REPORT.md) for full profiling details and
methodology.

## Installing Rust Flowmark

### Cargo (source build)

```bash
cargo install flowmark
```

### Cargo binstall (prebuilt binary)

```bash
cargo binstall flowmark
```

### PyPI (via uv or pip)

```bash
uvx flowmark-rs          # run on demand (no install needed)
uv tool install flowmark-rs  # persistent install
pip install flowmark-rs      # classic pip
```

### Homebrew (macOS)

```bash
brew tap jlevy/flowmark
brew install jlevy/flowmark/flowmark
"$(brew --prefix)/bin/flowmark" --version
```

**Note on the `flowmark` command name:** The PyPI package `flowmark-rs` provides both
`flowmark` and `flowmark-rs` commands.
If you only want the CLI tool, just install `flowmark-rs` — you don’t need the Python
`flowmark` package. If you have both packages installed as uv tools,
`uv tool install flowmark-rs --force` will let the Rust version take precedence for the
`flowmark` command.

* * *

{{ shared_docs_body }}

Rust-specific docs:

- [`docs/rust-only-features.md`](docs/rust-only-features.md)
- [`docs/cache.md`](docs/cache.md)
- [`docs/development.md`](docs/development.md)
