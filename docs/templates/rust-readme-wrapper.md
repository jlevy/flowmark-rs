<!-- Generated from repos/flowmark/README.md via scripts/generate-rust-readme.py. -->

[![Follow @ojoshe on X](https://img.shields.io/badge/follow_%40ojoshe-black?logo=x&logoColor=white)](https://x.com/ojoshe)
[![CI](https://github.com/jlevy/flowmark-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/jlevy/flowmark-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/flowmark.svg)](https://crates.io/crates/flowmark)
[![docs.rs](https://docs.rs/flowmark/badge.svg)](https://docs.rs/flowmark)
![MSRV](https://img.shields.io/badge/MSRV-{{ msrv }}-blue)

# flowmark

## Rust Port

> [!INFO]
>
> This repository (`flowmark-rs`) is an auto-synced Rust port of the original
> [Python flowmark](https://github.com/jlevy/flowmark). It is
> feature-equivalent and fastest for CLI use.

## Installing Flowmark (Rust Binary)

- Install Rust CLI (source): `cargo install flowmark`
- Install Rust CLI (binary): `cargo binstall flowmark`
- Primary command: `flowmark` (`flowmark-rs` is also available in this repo)
- Port sync process: [`docs/port-sync-playbook.md`](docs/port-sync-playbook.md)
- Porting methodology: [rust-porting-playbook](https://github.com/jlevy/rust-porting-playbook)

---

## Python Project Overview

{{ python_readme_body }}
