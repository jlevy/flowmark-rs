# Sync Artifacts

Store baseline->target upstream diff summaries for Mode B releases here.

File naming convention:

```text
YYYY-MM-DD-sync-v<baseline>-to-v<target>.md
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

These artifacts are the audit trail proving that every upstream change was reviewed and
categorized before porting work started.
