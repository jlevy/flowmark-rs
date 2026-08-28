---
type: is
id: is-01m12r7kpzpf0bc7km9bbbftt9
title: "Vacuous exclusion assertions: file-discovery fixtures are deleted before the test runs"
kind: bug
status: closed
priority: 2
version: 4
labels:
  - testing
  - parity
dependencies: []
parent_id: is-01m12r94hb1se4rttk097t6zea
created_at: 2026-08-27T23:17:41.215Z
updated_at: 2026-08-28T00:06:31.456Z
closed_at: 2026-08-28T00:06:31.456Z
close_reason: "Implemented and verified: fixed list-specific pipe-continuation idempotence, restored strict shared discovery coverage, restored Rust-only cache help assertions, hardened the ledger gate, and passed the complete Linux, macOS, and Windows matrix."
resolution: null
duplicate_of: null
---
Found by auditing whether PR #81's test changes are strict improvements.

## Three assertions can no longer fail

The shared tryscript file-discovery.tryscript.md deletes four fixture
directories in its `before:` block:

    rm -rf fixtures/project/.venv fixtures/project/build \
           fixtures/project/skip fixtures/project/nested/generated

and the fixture tree no longer contains them. But the document still asserts
they are excluded from discovery:

    line  41: flowmark --list-files fixtures/project/ | grep -c '\.venv'   -> 0
    line  47: flowmark --list-files fixtures/project/ | grep -c build/      -> 0
    line 110: flowmark --list-files fixtures/project/ | grep -c skip/       -> 0

With the directories absent, `--list-files` cannot emit those paths, so each
assertion passes trivially. If default-directory exclusion or .flowmarkignore
handling broke tomorrow, none of the three would catch it.

## This is a reduction from the previous behavior

flowmark-rs main committed all four fixtures
(tests/tryscript/fixtures/project/{.venv/lib/README.md, build/output.md,
skip/ignored.md, nested/generated/output.md}) and its copy of the tryscript had
no `rm -rf` line, so the same three assertions were meaningful there. The
nested/generated fixture, used for nested-gitignore coverage, is also gone.

The `rm -rf` comment says it exists to ignore untracked local artifacts, so
deleting the committed fixtures alongside them looks unintended rather than a
deliberate policy change.

## Ask

Restore the four fixtures in the shared corpus and narrow the `before:` cleanup
so it removes only genuinely untracked artifacts, keeping the exclusion
assertions meaningful. The doc is shared, so this is an upstream change that
flowmark-rs then picks up with the next submodule pin.

Note the assertions still PASS today; this is a loss of test power, not a
failure, so it does not block PR #81.

## Notes

Fixed upstream in flowmark ba9de9e3: restored all four committed exclusion fixtures, removed the vacuous cleanup, and restored nested gitignore/no-respect-gitignore scenarios. Rust pins that commit and the shared tryscript passes.
