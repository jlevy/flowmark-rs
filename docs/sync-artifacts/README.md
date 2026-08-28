# Sync Artifacts

Store baseline->target upstream diff summaries for Mode B releases here.

For release-to-release syncs, use:

```text
YYYY-MM-DD-sync-v<baseline>-to-v<target>.md
```

For an in-progress branch contract, use the immutable short target commit instead of a
pretend release version:

```text
YYYY-MM-DD-sync-v<baseline>-to-<target-commit>.md
```

Recommended sections:

1. Baseline and target release/tag/commit
2. Upstream commits in range
3. Changed modules/functions
4. Changed tests
5. CLI/help/interface changes
6. Dependency/build/docs changes
7. Rust porting impact assessment
8. Checklist of required Rust updates
9. Shared change IDs and exact Rust dispositions
10. Validation commands and results
11. Publication or clean-clone blockers

These artifacts are the audit trail proving that every upstream change was reviewed and
categorized before porting work started.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
