# Development Workflow (Rust Port)

This is a short operational index for development and release work in `flowmark-rs`.
Reusable synchronization methodology lives in the `rust-porting-playbook` submodule.

## Start Here

- Reusable sync-release workflow:
  [`repos/rust-porting-playbook/playbooks/python-to-rust-sync-release-workflow.md`](../repos/rust-porting-playbook/playbooks/python-to-rust-sync-release-workflow.md)
- Upstream sync checklist:
  [`repos/rust-porting-playbook/playbooks/port-checklist-update-template.md`](../repos/rust-porting-playbook/playbooks/port-checklist-update-template.md)
- Agent sync prompt template:
  [`repos/rust-porting-playbook/playbooks/auto-sync-agent-prompt-template.md`](../repos/rust-porting-playbook/playbooks/auto-sync-agent-prompt-template.md)

## Two Release Modes

1. **Mode A: Rust-only stabilization release** Keep Python baseline unchanged.
   Use this for cleanups, docs, and build/release hardening before upstream sync.
2. **Mode B: Upstream sync release** Update to a new Python release/tag/commit and port
   baseline->target changes.

## flowmark-rs Local Docs

- Publishing steps: [`docs/publishing.md`](publishing.md)
- Rust-only CLI features: [`docs/rust-only-features.md`](rust-only-features.md)
- Incremental cache (developer reference): [`docs/cache.md`](cache.md)
- flowmark-specific sync details (fixtures, mapping files, local scripts):
  [`docs/port-sync-playbook.md`](port-sync-playbook.md)
- Coverage mapping system:
  [`docs/port-sync-playbook.md`](port-sync-playbook.md#test-coverage-mapping)

Cache docs are kept concise and include:

- default cache path resolution order
- fallback and warning behavior
- manifest layout and CLI/config knobs

## Project-Specific Baseline Field

`flowmark-rs` tracks parity baseline in:

```toml
[package.metadata.parity]
version = "..."
```

Mode A keeps this value unchanged.
Mode B updates it after successful sync validation.
