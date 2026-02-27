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

Latest local cross-formatter validation on a 928-file corpus (23 MB, macOS arm64):

| Formatter | Workload | Mean |
| --- | --- | --- |
| dprint (`--incremental=false`) | fresh format | 0.48 s |
| **flowmark-rs** (`--auto`) | fresh format | **0.76 s** |
| dprint (`--incremental=false`) | re-format | 0.36 s |
| **flowmark-rs** (`--auto`) | re-format | **0.67 s** |
| dprint (incremental default) | re-format | 0.03 s |

The remaining fresh/re-format gap is now mostly in per-file formatting cost plus
incremental caching behavior. dprint's warm incremental cache remains significantly
faster on unchanged re-runs.

The Rust port is **10–17x faster** than the Python reference implementation:

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
