---
type: is
id: is-01kss3qmd08pep1hrspzhggbth
title: "PARITY GAP: code-block->list + nested indented-code spacing (last_content_line over-count)"
kind: bug
status: open
priority: 2
version: 1
labels:
  - parity
dependencies: []
parent_id: is-01kss3qk7et1jycwp3snxgs7zd
created_at: 2026-05-29T05:36:23.455Z
updated_at: 2026-05-29T05:36:23.455Z
---
render_block_children originally_tight over-counts an indented code block's end line (includes trailing blanks), so code->list tightness can't be suppressed safely. ~5 residual corpus lines (research-modern-typescript, rust-cli-best-practices, etc.). Needs a last_content_line clamp for CodeBlock (regression-prone — touches widely-used helper).
