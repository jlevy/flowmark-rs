---
type: is
id: is-01m10tdz0v6w74nxzm80brdq9v
title: "PR #81 review R8: zero matched files reports multiple files"
kind: bug
status: closed
priority: 3
version: 3
labels:
  - pr-review
  - cli
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T05:17:37.689Z
updated_at: 2026-08-27T05:36:19.690Z
closed_at: 2026-08-27T05:36:19.690Z
close_reason: Fixed R8 by removing the false multiple-files error and matching the Python no-op contract.
resolution: null
duplicate_of: null
---
PR #81 R8. src/main.rs treats zero and multiple resolved inputs as the same output-file error. Add an exact zero-match diagnostic and preserve the many-file diagnostic.

## Notes

The reviewer correctly identified the misleading message, but Python defines zero resolved files as a successful no-op even with output. Rust now matches that behavior exactly; an integration test requires exit 0, empty stderr, and no output file. Multiple inputs retain the existing error.
