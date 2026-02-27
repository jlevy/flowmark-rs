---
type: is
id: is-01kjgke6rv2bcxb1shkfcpk6fe
title: "Hotspot follow-up: optimize dominant fill_markdown stages"
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-02-27-incremental-cache-and-performance-roadmap.md
labels: []
dependencies:
  - type: blocks
    target: is-01kjgke312artqxqyeb075d8r0
parent_id: is-01kjf287ax0e8s1bqrcamkb2m0
created_at: 2026-02-27T22:27:49.146Z
updated_at: 2026-02-27T22:27:59.102Z
---
Use new perf stats to target expensive functions in src/formatter/filling.rs (for example preprocess_tag_block_spacing, extract_footnote_defs, comrak parse/render and postprocess passes) and validate improvements without formatting drift.
