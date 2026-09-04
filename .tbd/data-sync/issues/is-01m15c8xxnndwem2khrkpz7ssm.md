---
type: is
id: is-01m15c8xxnndwem2khrkpz7ssm
title: Batch aborts on the first invalid-UTF-8 file, and the error does not name it
kind: bug
status: open
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-08-28T23:46:24.820Z
updated_at: 2026-09-04T08:22:28.612Z
---
Running the formatter over a directory stops at the first non-UTF-8 file and leaves every later file unprocessed. The error is 'Error: input is not valid UTF-8' with no path, so over a large tree you cannot tell which file failed.

Found while comparing v0.3.2 against main over 1611 documents: the release processed 453 files before aborting, current main 70, both leaving the rest untouched. It also silently corrupted that comparison — unprocessed files looked like 'output preserved the input'.

The v0.3.2 release had strictly better diagnostics here: 'error: failed to format <path>: stream did not contain valid UTF-8'. Current Rust matches Python exactly (exit 2, identical generic message), so this is parity-correct but a real usability regression against the last release.

Two separable improvements:
1. Name the offending path in the error (restores what v0.3.2 reported).
2. Skip the unreadable file, keep formatting the rest, and report a non-zero exit at the end — a batch formatter reporting partial completion as a hard stop is the worse default.

Shared behavior, so it needs the upstream-first flow: agree the intended behavior in Python, replicate in flowmark-rs.

## Notes

Release review update (2026-09-04): the v0.3.2 diagnostic regression (missing offending path) is fixed in synchronized prep work fm-vc1q/fmr-unyc with shared and native tests. This bead remains open only for the separate behavior improvement of continuing past invalid files and reporting aggregate failure at batch end; that broader change is not required to restore last-release behavior.
