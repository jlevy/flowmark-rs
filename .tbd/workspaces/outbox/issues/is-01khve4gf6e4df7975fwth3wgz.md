---
type: is
id: is-01khve4gf6e4df7975fwth3wgz
title: "Plaintext mode: paired Jinja tag regex incorrectly matches two closing tags as atomic pair"
kind: bug
status: closed
priority: 2
version: 2
labels:
  - parity
dependencies: []
created_at: 2026-02-19T17:11:05.445Z
updated_at: 2026-02-19T17:11:34.900Z
closed_at: 2026-02-19T17:11:34.899Z
close_reason: "Fixed: changed paired tag regex patterns in atomic_patterns.rs to require opening tag starts with [a-zA-Z], preventing two closing tags (e.g. {% /field %}{% /group %}) from matching as an atomic pair."
---
