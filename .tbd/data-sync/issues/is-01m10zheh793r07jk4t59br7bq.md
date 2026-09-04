---
type: is
id: is-01m10zheh793r07jk4t59br7bq
title: "PR #81 suggestion: consolidate legacy and preservation token systems"
kind: task
status: open
priority: 3
version: 1
labels:
  - architecture
  - preservation
dependencies: []
parent_id: is-01m10zgx85zvg3r07e73xj2733
created_at: 2026-08-27T06:46:54.749Z
updated_at: 2026-08-27T06:46:54.749Z
---
The follow-up review observes that filling.rs legacy COMRAK-WORKAROUND markers and the new preservation bridge are parallel token systems with separate collision and restoration rules. Defer consolidation beyond PR #81, then design a staged migration into the preservation registry with exact shared regression coverage.
