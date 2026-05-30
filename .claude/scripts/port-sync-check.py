#!/usr/bin/env python3
"""Lightweight SessionStart nudge for the flowmark-rs port sync.

Is the upstream Python flowmark ahead of our Rust parity baseline? This is a
read-only, fast, network-tolerant check. It prints a one-line nudge to run
/port-sync ONLY when upstream has a newer release; it is silent when in sync and
degrades silently when offline or unparseable. It never mutates the repo and uses
only the standard library (no third-party deps, so it is safe to run on every
session start).
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

UPSTREAM_URL = "https://github.com/jlevy/flowmark.git"
PARITY_RE = re.compile(r"\[package\.metadata\.parity\][^\[]*?version\s*=\s*\"([^\"]+)\"", re.DOTALL)
TAG_RE = re.compile(r"refs/tags/(v[0-9]+\.[0-9][^\^\s]*)$")


def version_key(tag: str) -> tuple[int, ...]:
    """Sort key for a `vX.Y.Z` tag; non-numeric parts sort as 0."""
    return tuple(int(p) if p.isdigit() else 0 for p in tag.lstrip("v").split("."))


def baseline_version(project_dir: Path) -> str | None:
    cargo = project_dir / "Cargo.toml"
    if not cargo.is_file():
        return None
    match = PARITY_RE.search(cargo.read_text(encoding="utf-8"))
    return match.group(1) if match else None


def latest_upstream_tag() -> str | None:
    try:
        result = subprocess.run(
            ["git", "ls-remote", "--tags", "--refs", UPSTREAM_URL],
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )
    except (subprocess.TimeoutExpired, OSError):
        return None
    if result.returncode != 0:
        return None
    tags = [m.group(1) for line in result.stdout.splitlines() if (m := TAG_RE.search(line))]
    return max(tags, key=version_key) if tags else None


def main() -> int:
    project_dir = Path(os.environ.get("CLAUDE_PROJECT_DIR", "."))
    baseline = baseline_version(project_dir)
    if not baseline:
        return 0
    latest = latest_upstream_tag()
    if not latest:
        return 0
    if version_key(latest) > version_key(f"v{baseline}"):
        print(
            f"⚠️  Upstream Python flowmark {latest} is newer than the Rust parity "
            f"baseline v{baseline}.\n"
            f"    Run /port-sync to review the upstream diff and propose a port plan."
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
