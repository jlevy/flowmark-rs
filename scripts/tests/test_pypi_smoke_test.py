#!/usr/bin/env python3
"""Tests for scripts/pypi_smoke_test.py."""

from __future__ import annotations

import subprocess
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "pypi_smoke_test.py"


class PypiSmokeTestScriptTests(unittest.TestCase):
    def _run_script(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(SCRIPT_PATH), *args],
            capture_output=True,
            text=True,
        )

    def test_skips_cross_arch_macos_smoke_test(self) -> None:
        result = self._run_script(
            "--target",
            "x86_64-apple-darwin",
            "--runner-os",
            "darwin",
            "--runner-arch",
            "aarch64",
            "--dry-run",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        self.assertIn("Skipping smoke test for x86_64-apple-darwin", result.stdout)

    def test_skips_cross_os_smoke_test(self) -> None:
        result = self._run_script(
            "--target",
            "x86_64-pc-windows-msvc",
            "--runner-os",
            "linux",
            "--runner-arch",
            "x86_64",
            "--dry-run",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        self.assertIn("target OS windows does not match runner OS linux", result.stdout)

    def test_runs_install_and_version_check_for_native_target(self) -> None:
        result = self._run_script(
            "--target",
            "x86_64-unknown-linux-gnu",
            "--runner-os",
            "linux",
            "--runner-arch",
            "x86_64",
            "--dry-run",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        self.assertIn("pip install --no-index --find-links dist flowmark-rs --force-reinstall", result.stdout)
        self.assertIn("flowmark-rs --version", result.stdout)

    def test_rejects_invalid_target(self) -> None:
        result = self._run_script(
            "--target",
            "not-a-target",
            "--runner-os",
            "linux",
            "--runner-arch",
            "x86_64",
            "--dry-run",
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Unsupported", result.stderr)


if __name__ == "__main__":
    unittest.main()
