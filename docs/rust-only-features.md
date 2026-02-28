# Rust-Only Features

This page tracks CLI behavior that is currently specific to `flowmark-rs`
(and not available in the Python `flowmark` CLI).

## Performance-Oriented Features

| Feature | Rust CLI | Python CLI |
| --- | --- | --- |
| Incremental cache | Yes | No |
| Cache controls (`--no-cache`, `--cache-dir`, `--incremental`, `--show-cache`, `--clear-cache`) | Yes | No |
| Stage-level perf stats (`--perf-stats`) | Yes | No |
| Parallel file processing (`--threads`) | Yes | No |

## Why these are Rust-only (for now)

- They are implementation/performance features of the Rust engine.
- The Python version remains the formatting-behavior reference.
- Rust is the preferred CLI for high-throughput formatting workflows.

For cache-path details and fallback behavior, see [`docs/cache.md`](cache.md).
