---
sandbox: true
env:
  NO_COLOR: "1"
  LC_ALL: C
path:
  - $FLOWMARK_BIN_DIR
patterns:
  PROJECT_MANIFEST: '[0-9a-f]{16}\.toml'
  MS: '\d+\.\d{3}'
  CACHE_FILES: '\d+'
  CACHE_SIZE: '\d+(?:\.\d)? (?:B|KiB|MiB|GiB|TiB)'
  CACHE_MANIFESTS: '\d+'
  CACHE_ENTRIES: '\d+'
---

# Incremental Cache Behavior

Session-style golden test for incremental cache structure and lifecycle.

## CB1: Create a clean test repo with two markdown files

```console
$ mkdir -p cache-session/repo && printf '# Doc One\n\nA  test sentence.\n' > cache-session/repo/one.md && printf '# Doc Two\n\nSecond file here.\n' > cache-session/repo/two.md && echo "ready"
ready
```

## CB2: `--no-cache` does not create cache artifacts

```console
$ flowmark --auto --no-cache --cache-dir cache-session/cache cache-session/repo && find cache-session -maxdepth 3 -print | sort
cache-session
cache-session/repo
cache-session/repo/one.md
cache-session/repo/two.md
```

## CB3: Cached run creates incremental manifest under expected structure

```console
$ flowmark --auto --cache-dir cache-session/cache cache-session/repo && find cache-session/cache -maxdepth 3 -print | sort
cache-session/cache
cache-session/cache/incremental
cache-session/cache/incremental/[PROJECT_MANIFEST]
```

## CB4: Manifest file content has expected keys and hash layout

```console
$ MANIFEST="$(find cache-session/cache/incremental -maxdepth 1 -type f | head -n1)" && printf '%s\n' "$MANIFEST" && cat "$MANIFEST"
cache-session/cache/incremental/[PROJECT_MANIFEST]
fingerprint = "[..]"
hashes = ["[..]", "[..]"]
version = 1
```

## CB5: Unchanged rerun preserves manifest bytes and reports 100% hits

```console
$ MANIFEST="$(find cache-session/cache/incremental -maxdepth 1 -type f | head -n1)" && cp "$MANIFEST" cache-session/manifest-before.toml && flowmark --auto --cache-dir cache-session/cache --perf-stats cache-session/repo 2> cache-session/perf-steady.log && cmp -s cache-session/manifest-before.toml "$MANIFEST" && echo "manifest stable" && cat cache-session/perf-steady.log
manifest stable
perf-stats:
  fill_markdown files=0 total=[MS]ms preprocess=[MS]ms parse=[MS]ms transforms=[MS]ms render=[MS]ms postprocess=[MS]ms
  incremental hits=2 misses=0 hit_rate=100.0%
```

## CB6: Changing one file grows stored hash set and reports partial hit-rate

```console
$ printf '# Doc One\n\nChanged sentence now.\n' > cache-session/repo/one.md && MANIFEST="$(find cache-session/cache/incremental -maxdepth 1 -type f | head -n1)" && cp "$MANIFEST" cache-session/manifest-before.toml && flowmark --auto --cache-dir cache-session/cache --perf-stats cache-session/repo 2> cache-session/perf-delta.log && echo "before" && cat cache-session/manifest-before.toml && echo "after" && cat "$MANIFEST" && cat cache-session/perf-delta.log
before
fingerprint = "[..]"
hashes = ["[..]", "[..]"]
version = 1
after
fingerprint = "[..]"
hashes = ["[..]", "[..]", "[..]"]
version = 1
perf-stats:
  fill_markdown files=1 total=[MS]ms preprocess=[MS]ms parse=[MS]ms transforms=[MS]ms render=[MS]ms postprocess=[MS]ms
  incremental hits=1 misses=1 hit_rate=50.0%
```

## CB7: `--no-cache` bypasses cache and does not mutate manifest

```console
$ MANIFEST="$(find cache-session/cache/incremental -maxdepth 1 -type f | head -n1)" && cp "$MANIFEST" cache-session/manifest-before.toml && flowmark --auto --no-cache --cache-dir cache-session/cache --perf-stats cache-session/repo 2> cache-session/perf-nocache.log && cmp -s cache-session/manifest-before.toml "$MANIFEST" && echo "no-cache manifest unchanged" && cat cache-session/perf-nocache.log
no-cache manifest unchanged
perf-stats:
  fill_markdown files=2 total=[MS]ms preprocess=[MS]ms parse=[MS]ms transforms=[MS]ms render=[MS]ms postprocess=[MS]ms
  incremental hits=0 misses=2 hit_rate=0.0%
```

## CB8: `--show-cache` reports directory, file count, and total size

```console
$ flowmark --show-cache --cache-dir cache-session/cache
Cache directory: cache-session/cache
Cache files: [CACHE_FILES]
Cache size: [CACHE_SIZE]
Cache manifests: [CACHE_MANIFESTS]
Current project manifest: cache-session/cache/incremental/[PROJECT_MANIFEST]
Current project entries: [CACHE_ENTRIES]
```

## CB9: `--clear-cache` removes the cache directory immediately

```console
$ flowmark --clear-cache --cache-dir cache-session/cache && find cache-session -maxdepth 3 -print | sort
Cache directory: cache-session/cache
Cache cleared.
cache-session
cache-session/manifest-before.toml
cache-session/perf-delta.log
cache-session/perf-nocache.log
cache-session/perf-steady.log
cache-session/repo
cache-session/repo/one.md
cache-session/repo/two.md
```

## CB10: `--clear-cache` is idempotent when cache is already empty

```console
$ flowmark --clear-cache --cache-dir cache-session/cache
Cache directory: cache-session/cache
Cache already empty.
```
