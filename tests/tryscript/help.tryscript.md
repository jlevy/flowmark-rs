---
sandbox: true
env:
  NO_COLOR: "1"
  LC_ALL: C
path:
  - $TRYSCRIPT_GIT_ROOT/target/debug
---

# Help Output

## H1: Help tagline

```console
$ flowmark --help | head -1
Flowmark: Better auto-formatting for Markdown and plaintext
```

## H2: Common usage examples are present

```console
$ flowmark --help | grep -F "flowmark --auto README.md"
  flowmark --auto README.md
```

```console
$ flowmark --help | grep -F "flowmark --auto docs/"
  flowmark --auto docs/
```

```console
$ flowmark --help | grep -F "flowmark --auto ."
  flowmark --auto .
```

```console
$ flowmark --help | grep -F "flowmark --list-files ."
  flowmark --list-files .
```

## H3: Agent guidance is explicit

```console
$ flowmark --help | grep -Fx "  flowmark --skill"
  flowmark --skill
```

```console
$ flowmark --help | grep -F "Agents should run"
  Agents should run `flowmark --skill` for full Flowmark usage guidance.
```

## H4: Full docs are via --docs

```console
$ flowmark --help | grep -F "flowmark --docs"
Use `flowmark --docs` for full documentation.
```

## H5: Performance flags are present

```console
$ flowmark --help | grep -F -- "--no-cache"
      --no-cache              Disable incremental cache for this run
```

```console
$ flowmark --help | grep -F -- "--cache-dir"
      --cache-dir <DIR>       Override incremental cache directory
```

```console
$ flowmark --help | grep -F -- "--show-cache"
      --show-cache            Show cache directory, file count, and total size
```

```console
$ flowmark --help | grep -F -- "--clear-cache"
      --clear-cache           Delete the entire cache directory (non-interactive)
```

```console
$ flowmark --help | grep -F -- "--perf-stats"
      --perf-stats            Print performance statistics summary
```
