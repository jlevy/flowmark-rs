---
type: is
id: is-01m10tdxbw30eyj8xvqf7asmtk
title: "PR #81 review R3: invalid UTF-8 exit code uses message matching"
kind: bug
status: closed
priority: 1
version: 3
labels:
  - pr-review
  - cli
dependencies: []
parent_id: is-01m10tcw81ta6bfvxa5xkj7707
created_at: 2026-08-27T05:17:35.995Z
updated_at: 2026-08-27T05:36:17.942Z
closed_at: 2026-08-27T05:36:17.940Z
close_reason: Fixed R3 without string matching or a breaking public enum variant.
resolution: null
duplicate_of: null
---
PR #81 R3. src/main.rs and src/lib.rs infer the invalid UTF-8 exit contract from duplicated error strings. Introduce a typed error/discriminator without breaking the public API unexpectedly, and test library plus CLI behavior.

## Notes

Kept the public Error enum semver-compatible. Invalid UTF-8 is stored as the typed Utf8Error cause inside Error::Io; Error::is_invalid_utf8 provides the discriminator used by main. Library tests retain the byte offset and the shared CLI golden remains exact.
