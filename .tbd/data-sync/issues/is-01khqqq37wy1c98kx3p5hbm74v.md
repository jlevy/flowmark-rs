---
type: is
id: is-01khqqq37wy1c98kx3p5hbm74v
title: Implement signposts format (SP/0.1) for knowledge flow maps
kind: feature
status: open
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-02-18T06:41:33.946Z
updated_at: 2026-02-18T06:41:44.634Z
---
Implement the signposts format (SP/0.1) for knowledge flow maps with progress tracking.

## Spec
See: repos/autorust/docs/project/specs/active/plan-2026-02-17-signposts-format.md

## Key Features
- Areas/paths/routes structure (replaces flat docmap topics)
- Strict docspec prefixes (./file, ref#local, github:, http://)
- Local inline refs to avoid file proliferation
- Progress tracking via progress.yml
- Backward compatibility with `autorust guide` command

## Implementation Phases

### Phase 1: Format + Parser + Navigation (read-only)
- signposts.yml schema and validator
- YAML loader with format version check
- CLI commands: signposts, signposts node, signposts step
- Update autorust guide to read signposts.yml
- Write signposts.yml for rust-porting-playbook
- Tests for parser and validation

### Phase 2: Progress Tracking
- progress.yml schema
- CLI: start, done, skip, note, status, where, next
- State management and persistence
- Edge-based suggestions

### Phase 3: Polish + Integration
- Extra steps support
- Integration with autorust setup/report
- Documentation
