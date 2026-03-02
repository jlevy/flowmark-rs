#!/usr/bin/env python3
"""Tests for scripts/validate_wheel_entrypoints.py."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "validate_wheel_entrypoints.py"


def _create_wheel(path: Path, files: list[str]) -> None:
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        for name in files:
            zf.writestr(name, b"")


class ValidateWheelEntrypointsTests(unittest.TestCase):
    def _run_script(self, wheels_dir: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(SCRIPT_PATH), "--wheels-dir", str(wheels_dir)],
            capture_output=True,
            text=True,
        )

    def test_passes_for_wheels_with_expected_entrypoints(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            wheels_dir = Path(temp_dir)
            _create_wheel(
                wheels_dir / "flowmark_rs-0.2.5-py3-none-any.whl",
                [
                    "flowmark_rs-0.2.5.data/scripts/flowmark",
                    "flowmark_rs-0.2.5.data/scripts/flowmark-rs",
                ],
            )
            result = self._run_script(wheels_dir)
            self.assertEqual(result.returncode, 0, msg=result.stderr)
            self.assertIn("Validated wheel entrypoints for 1 wheel(s).", result.stdout)

    def test_fails_when_entrypoint_is_missing(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            wheels_dir = Path(temp_dir)
            _create_wheel(
                wheels_dir / "flowmark_rs-0.2.5-py3-none-any.whl",
                ["flowmark_rs-0.2.5.data/scripts/flowmark"],
            )
            result = self._run_script(wheels_dir)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Missing expected entrypoints in wheels", result.stderr)

    def test_fails_when_no_wheels_exist(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            result = self._run_script(Path(temp_dir))
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("No wheel files found to validate", result.stderr)


if __name__ == "__main__":
    unittest.main()
