---
type: is
id: is-01m10rx6n7j14yddk5vh0c0hcp
title: Port remaining shared GLFM preservation cases
kind: bug
status: closed
priority: 1
version: 5
labels:
  - glfm
  - parity
  - preservation
dependencies: []
created_at: 2026-08-27T04:50:59.878Z
updated_at: 2026-08-27T05:14:04.012Z
closed_at: 2026-08-27T05:08:52.665Z
close_reason: Ported FM-EXT-GLFM-001 from the shared upstream contract at 19c840e. Rust uses the same bracketed-reference allowlist, source-exact reference/table-pipe handling, and compatible paired >>> container scan. All four unchanged shared cases pass twice; focused Rust scanner tests, cargo fmt, clippy with warnings denied, all-feature tests, and no-default-feature tests pass. No divergence entry was added.
resolution: null
duplicate_of: null
---
Port the upstream language-neutral cases and behavior for the unresolved GitLab Markdown forms from flowmark issue #67, including paired >>> multiline blockquote fences and bracketed directive/reference text such as [issue:_123_]. Begin from the shared desired-output cases on Python PR #71, advance the repos/flowmark submodule to the reviewed upstream commit, and make the unchanged cases pass through the native Rust conformance runner at a fixed point. Reconcile fmr-qmd8 and fmr-rz9f only from shared evidence.

## Notes

Pinned repos/flowmark and the machine-checked mapping through 644be24. Direct Rust behavior remains identical to the Python contract for the GLFM allowlist, source-exact reference and table-pipe handling, paired >>> container scanning, and fail-closed fallback. Native shared conformance now passes all five FM-EXT-GLFM-001 cases twice: 483 exact passes overall with the unchanged 34-entry CommonMark ledger. Focused scanner tests, cargo fmt, clippy with warnings denied, all-feature tests, no-default-feature tests, and traceability passed; no divergence entry was added.
