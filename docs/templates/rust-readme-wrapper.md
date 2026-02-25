<!-- Generated from shared docs source (currently repos/flowmark/README.md) via scripts/generate-rust-readme.py. -->

[![Follow @ojoshe on X](https://img.shields.io/badge/follow_%40ojoshe-black?logo=x&logoColor=white)](https://x.com/ojoshe)
[![CI](https://github.com/jlevy/flowmark-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/jlevy/flowmark-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/flowmark.svg)](https://crates.io/crates/flowmark)
[![docs.rs](https://docs.rs/flowmark/badge.svg)](https://docs.rs/flowmark)
![MSRV](https://img.shields.io/badge/MSRV-{{ msrv }}-blue)

# flowmark

## Rust Port of Python Flowmark

This is an auto-synced Rust port of the
[Python version](https://github.com/jlevy/flowmark). The original Python version is
well tested and this port aims for identical CLI usage and formatting behavior. It is a
fast binary and best for CLI usage.

Last sync: **{{ last_sync_date }}** against **Python v{{ parity_version }}**

- Port sync process: [`docs/port-sync-playbook.md`](docs/port-sync-playbook.md)
- Porting methodology: [rust-porting-playbook](https://github.com/jlevy/rust-porting-playbook)

## Why the Rust version?

- **Single binary**: install via Cargo, Cargo binstall, or Homebrew.
- **Fast CLI**: good for large repos and CI pipelines.
- **Library crate**: embed in Rust tooling via [docs.rs/flowmark](https://docs.rs/flowmark).

## Installing Flowmark (Rust CLI)

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
brew install flowmark
```

Primary command: `flowmark` (`flowmark-rs` is also available in this repo).

---
{{ shared_docs_body }}
