# Cache Settings and Behavior

Flowmark uses a persistent incremental cache for unchanged-file fast paths when
formatting in place (`--auto` or `--inplace`).

## Default Cache Location

Cache paths are defined centrally in [`src/settings.rs`](../src/settings.rs):

- `FALLBACK_CACHE_DIR = ".flowmark-cache"`
- `APP_CACHE_DIR = "flowmark"`
- `INCREMENTAL_CACHE_SUBDIR = "incremental"`

Effective default cache root:

- OS cache dir + `/flowmark` when available
  - macOS: `~/Library/Caches/flowmark`
  - Linux: `~/.cache/flowmark`
  - Windows: `%LOCALAPPDATA%\\flowmark`
- Fallback when OS cache root is unavailable:
  - `./.flowmark-cache/flowmark`

Incremental manifests are stored under:

- `<cache-root>/incremental/<project-hash>.toml`

## CLI Settings

- `--no-cache`
  - disables cache for the current run
  - alias: `--no-incremental`
- `--cache-dir <DIR>`
  - overrides cache root directory
  - alias: `--incremental-cache-dir`
- `--incremental[=true|false]`
  - explicit enable/disable form
  - visible alias: `--cache`

## Config File Settings

Cache settings can be configured in `flowmark.toml`, `.flowmark.toml`, or
`pyproject.toml` (`[tool.flowmark]`).

Recommended keys:

```toml
[performance]
cache = true
cache-dir = "/absolute/path/to/cache-root"
```

Backward-compatible keys are also supported:

- `incremental = true|false`
- `incremental-cache-dir = "..."`

CLI explicit flags still take precedence over config values.

## What the Cache Stores

- A project-scoped manifest keyed by project root hash.
- Hashes of formatted file content for the current formatter fingerprint.
- Formatter fingerprint includes:
  - binary version
  - formatting options
  - config file path and contents (when present)

## When Cache Helps

Cache hits do not depend on duplicate files. They help when the same files are
seen again unchanged (for example a second run in a repo with no Markdown edits).
On cache hit, flowmark skips parse/render/write for that file.

## Disabling and Troubleshooting

- Use `--no-cache` for a forced full run.
- Use `--cache-dir` to isolate cache state per benchmark or CI job.
- Corrupt manifests are ignored and rebuilt automatically on successful runs.
