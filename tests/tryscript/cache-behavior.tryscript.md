---
sandbox: true
env:
  NO_COLOR: "1"
  LC_ALL: C
path:
  - $TRYSCRIPT_GIT_ROOT/target/debug
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
$ flowmark --auto --no-cache --cache-dir cache-session/cache cache-session/repo && test ! -e cache-session/cache/incremental && echo "no cache artifacts"
no cache artifacts
```

## CB3: Cached run creates incremental manifest under expected structure

```console
$ flowmark --auto --cache-dir cache-session/cache cache-session/repo && find cache-session/cache -maxdepth 2 -type f | sed -E 's#^cache-session/cache/#cache/#; s#incremental/[0-9a-f]+\.toml#incremental/<project>.toml#'
cache/incremental/<project>.toml
```

## CB4: Manifest file content has expected keys and hash layout

```console
$ MANIFEST="$(find cache-session/cache/incremental -type f | head -n1)" && cat "$MANIFEST" | sed -E 's/[0-9a-f]{16}/<hex16>/g'
fingerprint = "<hex16>"
hashes = ["<hex16>", "<hex16>"]
version = 1
```

## CB5: Unchanged rerun preserves manifest bytes and reports 100% hits

```console
$ MANIFEST="$(find cache-session/cache/incremental -type f | head -n1)" && BEFORE="$(cksum "$MANIFEST" | awk '{print $1 "-" $2}')" && flowmark --auto --cache-dir cache-session/cache --perf-stats cache-session/repo 2> cache-session/perf-steady.log && AFTER="$(cksum "$MANIFEST" | awk '{print $1 "-" $2}')" && test "$BEFORE" = "$AFTER" && grep -F "incremental hits=2 misses=0 hit_rate=100.0%" cache-session/perf-steady.log && echo "manifest stable"
  incremental hits=2 misses=0 hit_rate=100.0%
manifest stable
```

## CB6: Changing one file grows stored hash set and reports partial hit-rate

```console
$ printf '# Doc One\n\nChanged sentence now.\n' > cache-session/repo/one.md && MANIFEST="$(find cache-session/cache/incremental -type f | head -n1)" && BEFORE_COUNT="$(grep -Eo '\"[0-9a-f]{16}\"' "$MANIFEST" | wc -l | tr -d '[:space:]')" && flowmark --auto --cache-dir cache-session/cache --perf-stats cache-session/repo 2> cache-session/perf-delta.log && AFTER_COUNT="$(grep -Eo '\"[0-9a-f]{16}\"' "$MANIFEST" | wc -l | tr -d '[:space:]')" && test "$AFTER_COUNT" -gt "$BEFORE_COUNT" && grep -F "incremental hits=1 misses=1 hit_rate=50.0%" cache-session/perf-delta.log && echo "hashes grew"
  incremental hits=1 misses=1 hit_rate=50.0%
hashes grew
```

## CB7: `--no-cache` bypasses cache and does not mutate manifest

```console
$ MANIFEST="$(find cache-session/cache/incremental -type f | head -n1)" && BEFORE="$(cksum "$MANIFEST" | awk '{print $1 "-" $2}')" && flowmark --auto --no-cache --cache-dir cache-session/cache --perf-stats cache-session/repo 2> cache-session/perf-nocache.log && AFTER="$(cksum "$MANIFEST" | awk '{print $1 "-" $2}')" && test "$BEFORE" = "$AFTER" && grep -F "incremental hits=0 misses=2 hit_rate=0.0%" cache-session/perf-nocache.log && echo "no-cache bypassed"
  incremental hits=0 misses=2 hit_rate=0.0%
no-cache bypassed
```
