#!/usr/bin/env python3
"""Structural regression tests for the release workflow's version contract."""

from __future__ import annotations

import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
RELEASE_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "release.yml"


class ReleaseWorkflowTests(unittest.TestCase):
    def test_archive_build_receives_the_resolved_release_tag(self) -> None:
        workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")
        expected_build_step = """      - name: Build
        env:
          FLOWMARK_RELEASE_TAG: ${{ needs.plan.outputs.release_tag }}
        run: RUSTFLAGS="--deny warnings --codegen target-feature=+crt-static ${{ matrix.target_rustflags }}" cargo build --bin flowmark --release --locked --target ${{ matrix.target }}
"""
        self.assertIn(expected_build_step, workflow)


if __name__ == "__main__":
    unittest.main()
