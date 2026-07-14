---
sandbox: true
env:
  NO_COLOR: "1"
  LC_ALL: C
path:
  - $TRYSCRIPT_GIT_ROOT/target/debug
before: |
  cp -r $TRYSCRIPT_TEST_DIR/fixtures/. fixtures/
---
# Verbose, Docs, and Skill Tests

Tests for --skill, --install-skill (including cross-agent --surfaces), and --docs
output. Note: --verbose is Rust-only (not in Python).
The --docs body is the Rust port’s own README, so its headings differ from the Python
build; the skill (--skill) and the installed skill artifacts are byte-identical across
both implementations.

## V1: Skill prints SKILL.md content

```console
$ flowmark --skill | sed -n '1,6p'
---
name: flowmark
description: Fast, consistent Markdown auto-formatter for typographic cleanup, normalization, and clean semantic line breaks. Use when creating, editing, or cleaning Markdown; formatting LLM-generated docs; adopting Flowmark in a repository; adding Markdown format scripts or commit hooks; or replacing Prettier or another Markdown formatter.
allowed-tools: Bash(flowmark:*), Bash(uvx:*), Read, Write
---
# Flowmark - Markdown Auto-Formatter
```

## V2: Docs prints documentation

The Rust port’s --docs renders its own README (top-level `# flowmark-rs`) but shares the
upstream body, so the `## What Is Flowmark?` section is present in both.

```console
$ flowmark --docs | grep -Fx "# flowmark-rs"
# flowmark-rs
```

```console
$ flowmark --docs | grep -Fx "## What Is Flowmark?"
## What Is Flowmark?
```

## V3: Install skill creates skill file

```console
$ flowmark --install-skill --agent-base tmpagent >/dev/null && test -f tmpagent/skills/flowmark/SKILL.md && echo "skill installed"
skill installed
```

## V4: Install skill creates nested directories

```console
$ flowmark --install-skill --agent-base deep/nested/path >/dev/null && test -f deep/nested/path/skills/flowmark/SKILL.md && echo "nested dirs created"
nested dirs created
```

## V5: Skill output routes to the self-documenting CLI

```console
$ flowmark --skill | grep -F -- "the full guide:" | sed 's/^- //'
`flowmark --docs` — the full guide: configuration, file discovery, editor setup, the
```

## V6: Install skill project-local default writes all three surfaces

```console
$ mkdir v6 && cd v6 && flowmark --install-skill >/dev/null && test -f .agents/skills/flowmark/SKILL.md && test -f .claude/skills/flowmark/SKILL.md && test -f AGENTS.md && echo "all surfaces installed"
all surfaces installed
```

## V7: --surfaces=claude writes only the Claude mirror

```console
$ mkdir v7 && cd v7 && flowmark --install-skill --surfaces=claude >/dev/null && test -f .claude/skills/flowmark/SKILL.md && test ! -e .agents && test ! -e AGENTS.md && echo "claude-only"
claude-only
```

## V8: --surfaces=portable writes only the portable surface

```console
$ mkdir v8 && cd v8 && flowmark --install-skill --surfaces=portable >/dev/null && test -f .agents/skills/flowmark/SKILL.md && test ! -e .claude && test ! -e AGENTS.md && echo "portable-only"
portable-only
```

## V9: --surfaces=agents-md writes only the AGENTS.md block

```console
$ mkdir v9 && cd v9 && flowmark --install-skill --surfaces=agents-md >/dev/null && test -f AGENTS.md && test ! -e .agents && test ! -e .claude && echo "agents-md-only"
agents-md-only
```

## V10: --surfaces=all is an alias for the default

```console
$ mkdir v10 && cd v10 && flowmark --install-skill --surfaces=all >/dev/null && test -f .agents/skills/flowmark/SKILL.md && test -f .claude/skills/flowmark/SKILL.md && test -f AGENTS.md && echo "all surfaces installed"
all surfaces installed
```

## V11: --surfaces=portable,agents-md writes a subset

```console
$ mkdir v11 && cd v11 && flowmark --install-skill --surfaces=portable,agents-md >/dev/null && test -f .agents/skills/flowmark/SKILL.md && test -f AGENTS.md && test ! -e .claude && echo "subset"
subset
```

## V12: --surfaces with an unknown value exits non-zero

```console (exit-code=2)
$ mkdir v12 && cd v12 && flowmark --install-skill --surfaces=cursor 2>&1 1>/dev/null | grep -o "unknown surface"
unknown surface
```

## V13: Installed Claude skill has the right content at the right path

Beyond existence: read the installed file and assert the frontmatter and the
`DO NOT EDIT` format stamp, confirming the skill landed at
`.claude/skills/flowmark/SKILL.md` with the expected content.

```console
$ mkdir v13 && cd v13 && flowmark --install-skill --surfaces=claude >/dev/null && grep -Fx "name: flowmark" .claude/skills/flowmark/SKILL.md && grep -F "DO NOT EDIT" .claude/skills/flowmark/SKILL.md
name: flowmark
<!-- DO NOT EDIT: `flowmark --install-skill` (format=f03 surface=skill-md) -->
```

## V14: Installed AGENTS.md block has marker-bounded content

```console
$ mkdir v14 && cd v14 && flowmark --install-skill --surfaces=agents-md >/dev/null && grep -Fx "<!-- BEGIN FLOWMARK INTEGRATION format=f03 surface=agents-md -->" AGENTS.md && grep -Fx "<!-- END FLOWMARK INTEGRATION -->" AGENTS.md
<!-- BEGIN FLOWMARK INTEGRATION format=f03 surface=agents-md -->
<!-- END FLOWMARK INTEGRATION -->
```
