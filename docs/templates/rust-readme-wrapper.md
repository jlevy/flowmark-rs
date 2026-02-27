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
> This is an auto-synced Rust port of the
> [Python version](https://github.com/jlevy/flowmark).
> The original Python version is the reference implementation.
> But this port aims for identical CLI usage and formatting behavior.
> It is a fast binary and best for CLI and IDE usage.

Last sync: **{{ last_sync_date }}** against **Python v{{ parity_version }}**

- Port sync process: [`docs/port-sync-playbook.md`](docs/port-sync-playbook.md)
- Porting methodology:
  [rust-porting-playbook](https://github.com/jlevy/rust-porting-playbook)

## Why the Rust version?

- **Single binary**: install via Cargo, Cargo binstall, or Homebrew.
- **Fast CLI**: good for large repos and CI pipelines.
- **Library crate**: embed in Rust tooling via
  [docs.rs/flowmark](https://docs.rs/flowmark).

### Performance

Fresh-run cross-formatter ranking (profiled benchmark suite, 928 files / 8.8 MB):

| Rank | Formatter | Mean (fresh) | Relative |
| --- | --- | --- | --- |
| 1 | dprint | 0.37 s | 1.0x |
| 2 | **flowmark-rs** | **0.73 s** | **2.0x** |
| 3 | markdownfmt | 0.95 s | 2.6x |
| 4 | prettier | 38.0 s | 103x |
| 5 | flowmark-py | ~48 s | ~130x |
| 6 | mdformat | 72.9 s | 197x |

flowmark-rs is currently the #2 fastest formatter in this comparison set.

These are two different benchmark scopes:

- Cross-formatter ranking above: 928-file corpus (8.8 MB), six tools.
- Python vs Rust table below: separate 1,080-file flowmark-focused corpus.

Because corpus sizes and workloads differ, compare numbers within each table.

On the separate Python-vs-Rust corpus, the Rust port is **10–17x faster** than
the Python reference implementation:

| Benchmark | Python | Rust | Speedup |
| --- | --- | --- | --- |
| Single file (1,734 lines) | 471.7 ms | 27.3 ms | **17.3x** |
| Batch 1,080 files (`--auto`) | 32.1 s | 2.69 s | **11.9x** |
| Batch 1,080 files (`--semantic`) | 27.2 s | 2.5 s | **10.9x** |
| File discovery (`--list-files`) | 1.31 s | 169 ms | **7.8x** |

See [`benchmarks/REPORT.md`](benchmarks/REPORT.md) for full profiling details
and methodology.

## Installing Rust Flowmark CLI

### Cargo (source build)

```bash
cargo install flowmark
```

### Cargo binstall (prebuilt binary)

```bash
cargo binstall flowmark
```

### Homebrew (macOS)

```bash
brew tap jlevy/flowmark
brew install jlevy/flowmark/flowmark
"$(brew --prefix)/bin/flowmark" --version
```

If `flowmark --version` shows Python `v0.6.4`, your PATH is resolving Python first.
Use `type -a flowmark` to inspect precedence.

Primary command: `flowmark` (`flowmark-rs` is also available in this repo).

* * *

{{ shared_docs_body }}
