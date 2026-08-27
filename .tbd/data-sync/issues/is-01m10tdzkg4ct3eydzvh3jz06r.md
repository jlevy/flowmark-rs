---
type: is
id: is-01m10tdzkg4ct3eydzvh3jz06r
title: "PR #81 review R10: unterminated fence needs extra passes"
kind: bug
status: closed
priority: 2
version: 7
labels:
  - pr-review
  - idempotence
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T05:17:38.287Z
updated_at: 2026-08-27T16:42:39.816Z
closed_at: 2026-08-27T16:42:39.814Z
close_reason: "Fixed on PR #81 by 6b41887, pinned by shared change FM-FENCED-CODE-001 at Python 85b6093 and Rust c50df77; all local gates and all 16 hosted Rust checks passed."
resolution: null
duplicate_of: null
---
PR #81 R10. Differential fuzzing reported a malformed unterminated code-fence case that needs extra passes to stabilize. Recover an exact reproduction, add a shared fallback case if Python and Rust should agree, and fix or explicitly defer the malformed-input policy.

## Notes

Exact reproducer recovered (this bead's blocker) and confirmed STILL PRESENT at head f833ce8.

## Minimal regression: 7 bytes

Input (no trailing newline), hex `60 60 60 5c 24 60 24`:

    ```\$`$

Pass sequence:

- v0.3.2 (release):  "```$`$\n"  -> stable in 1 pass (idempotent)
- PR 04bd444:        "```$`$\n"  -> "```$`$\n```\n" -> stable (2 passes)
- PR f833ce8 (head): "```$`$\n"  -> "```$`$\n```\n" -> stable (2 passes)

The second pass APPENDS a bare ``` fence line. This is a PR regression: the
released binary is idempotent on this input, the PR head is not.

## Original 66-byte case (differential fuzz, seed 1234, case 153)

    ```{#id}\$\(|\${#id}\]</div>\$:::+++[!NOTE]---\`$$\[: def+++{#id}`

v0.3.2 stable in 1 pass (60 B out); 04bd444 and f833ce8 both need 2 passes
(66 -> 69 -> 69 B). Reproduces with no flags; also under --semantic/--cleanups.

## User-facing impact: --check contract is violated

    $ printf '```\$`$' > a.md
    $ flowmark a.md > b.md        # format once
    $ flowmark --check b.md
    Would reformat: b.md          # exit 1 on flowmark's own output

A pre-commit hook or CI gate that formats then checks will fail on a file
flowmark itself just wrote.

## Python parity

Python at the pinned commit 9e9fd7c is idempotent here AND preserves the
backslash: "```\$`$\n". Both Rust versions drop the backslash ("```$`$\n"),
so there is a pre-existing escape-handling divergence underneath the new
idempotence regression. Worth a shared malformed-fallback golden covering both.

## Scope note

A 1,500-case differential fuzz of head f833ce8 (4 modes, PUA-hostile alphabet)
found 0 crashes and 0 content loss versus v0.3.2; this idempotence case was the
ONLY defect found. See also the separate pre-existing bug filed for "```\`".

Reopened: Side-branch fixes are validated but not yet integrated into the active PR #81 branch; reopen until push and hosted CI complete.
