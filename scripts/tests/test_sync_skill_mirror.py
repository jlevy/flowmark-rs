#!/usr/bin/env python3
"""Tests for scripts/sync_skill_mirror.py."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "sync_skill_mirror.py"


class SyncSkillMirrorTests(unittest.TestCase):
    def _repo(self, root: Path) -> None:
        upstream = root / "repos/flowmark/src/flowmark/skills"
        runtime = root / "src/skills"
        (upstream / "references").mkdir(parents=True)
        runtime.mkdir(parents=True)
        (upstream / "SKILL.md").write_text("canonical skill\n", encoding="utf-8")
        (upstream / "references/project-setup.md").write_text(
            "canonical reference\n", encoding="utf-8"
        )
        (runtime / "mod.rs").write_text("// Rust implementation\n", encoding="utf-8")
        (runtime / "helper.rs").write_text("// More Rust implementation\n", encoding="utf-8")

    def _run(self, root: Path, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(SCRIPT_PATH), "--repo-root", str(root), *args],
            capture_output=True,
            text=True,
        )

    def test_sync_copies_complete_bundle_and_preserves_rust_code(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            self._repo(root)
            runtime = root / "src/skills"
            (runtime / "stale.md").write_text("stale\n", encoding="utf-8")

            result = self._run(root)

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            self.assertEqual(
                (runtime / "SKILL.md").read_text(encoding="utf-8"), "canonical skill\n"
            )
            self.assertEqual(
                (runtime / "references/project-setup.md").read_text(encoding="utf-8"),
                "canonical reference\n",
            )
            self.assertTrue((runtime / "mod.rs").is_file())
            self.assertTrue((runtime / "helper.rs").is_file())
            self.assertFalse((runtime / "stale.md").exists())

    def test_check_reports_drift_without_writing(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            self._repo(root)
            self.assertEqual(self._run(root).returncode, 0)
            runtime_skill = root / "src/skills/SKILL.md"
            runtime_skill.write_text("different\n", encoding="utf-8")

            result = self._run(root, "--check")

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("content drifted", result.stderr)
            self.assertEqual(runtime_skill.read_text(encoding="utf-8"), "different\n")

    def test_check_accepts_an_aligned_mirror(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            self._repo(root)
            self.assertEqual(self._run(root).returncode, 0)

            result = self._run(root, "--check")

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            self.assertIn("is in sync", result.stdout)


if __name__ == "__main__":
    unittest.main()
