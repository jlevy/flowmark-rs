# Incremental Cache (Developer Notes)

Flowmark uses a persistent incremental cache for unchanged-file fast paths when
formatting in place (`--auto` or `--inplace`).
This cache behavior is currently specific to the Rust CLI.

## Cache Root Resolution

Default cache root is a **shared user cache**. Resolution order is:

1. `dirs::cache_dir()/flowmark` (OS-native user cache dir)
2. `dirs::home_dir()/.flowmark-cache/flowmark` (fallback, with warning)
3. `std::env::temp_dir()/flowmark-cache/flowmark` (last resort, with warning)

Flowmark no longer falls back to a cache directory under the current working
directory by default.

### How the OS cache path is determined

`dirs::cache_dir()` is used for the primary location:

- macOS: typically `~/Library/Caches`
- Linux: typically `$XDG_CACHE_HOME` or `~/.cache`
- Windows: typically `%LOCALAPPDATA%`

If this lookup returns `None` (for example, unusual runtime environments with
missing user/home metadata), Flowmark falls back to the next resolution step
and prints a warning.

## Cache Layout

Cache manifest files are stored under:

- `<cache-root>/incremental/<project-hash>.toml`

The manifest stores:

- cache format version
- formatter fingerprint
- formatted-content hashes

## CLI Options

- `--no-cache`
  - disable cache reads/writes for this run
- `--cache-dir <DIR>`
  - override cache root directory
- `--incremental[=true|false]`
  - explicit enable/disable form

## Config File Keys

Supported in `flowmark.toml`, `.flowmark.toml`, or `pyproject.toml`
(`tool.flowmark`):

```toml
[performance]
cache = true
cache-dir = "/absolute/path/to/cache-root"
```

Also accepted:

- `incremental = true|false`
- `incremental-cache-dir = "..."`

Explicit CLI flags take precedence over config values.
