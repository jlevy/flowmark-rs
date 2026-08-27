---
type: is
id: is-01m10tdzb5cq8bedqvm3qqe90w
title: "PR #81 review R9: path stat failures hardcode ENOENT"
kind: bug
status: closed
priority: 3
version: 3
labels:
  - pr-review
  - cli
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T05:17:38.020Z
updated_at: 2026-08-27T05:36:20.296Z
closed_at: 2026-08-27T05:36:20.295Z
close_reason: Fixed R9 with actual operating-system diagnostics and exact parity coverage.
resolution: null
duplicate_of: null
---
PR #81 R9. src/main.rs uses Path::exists and hardcodes Errno 2 for all stat failures, misreporting ENOTDIR or EACCES and missing Python parity. Preserve the actual io::Error diagnostic and add exact tests.

## Notes

Direct explicit paths now use symlink_metadata and format the actual raw OS error rather than Path::exists plus hardcoded ENOENT. Unix integration coverage pins ENOTDIR as Error: [Errno 20] Not a directory with exit 2; existing missing-path parity remains.
