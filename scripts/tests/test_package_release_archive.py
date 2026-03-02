#!/usr/bin/env python3
"""Tests for scripts/package_release_archive.py."""

from __future__ import annotations

import subprocess
import sys
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "package_release_archive.py"


class PackageReleaseArchiveTests(unittest.TestCase):
    def _write_workspace(self, root: Path, *, target: str, runner_os: str) -> None:
        binary_name = "flowmark.exe" if runner_os == "windows-latest" else "flowmark"
        bin_path = root / "target" / target / "release"
        bin_path.mkdir(parents=True)
        (bin_path / binary_name).write_bytes(b"dummy-binary")
        (root / "LICENSE").write_text("license", encoding="utf-8")
        (root / "README.md").write_text("readme", encoding="utf-8")

    def _run_script(self, root: Path, *, target: str, runner_os: str, artifact_tag: str = "v1.2.3") -> tuple[subprocess.CompletedProcess[str], Path]:
        output_path = root / "github_output.txt"
        result = subprocess.run(
            [
                sys.executable,
                str(SCRIPT_PATH),
                "--artifact-tag",
                artifact_tag,
                "--target",
                target,
                "--runner-os",
                runner_os,
                "--workspace",
                str(root),
                "--github-output",
                str(output_path),
            ],
            capture_output=True,
            text=True,
        )
        return result, output_path

    def test_creates_tar_gz_archive_on_linux(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            target = "x86_64-unknown-linux-musl"
            self._write_workspace(root, target=target, runner_os="ubuntu-latest")

            result, output_path = self._run_script(root, target=target, runner_os="ubuntu-latest")
            self.assertEqual(result.returncode, 0, msg=result.stderr)

            archive = root / f"flowmark-v1.2.3-{target}.tar.gz"
            self.assertTrue(archive.exists())
            with tarfile.open(archive, "r:gz") as tf:
                names = set(tf.getnames())
            prefix = f"flowmark-v1.2.3-{target}"
            self.assertIn(f"{prefix}/flowmark", names)
            self.assertIn(f"{prefix}/LICENSE", names)
            self.assertIn(f"{prefix}/README.md", names)
            self.assertIn(f"archive=flowmark-v1.2.3-{target}.tar.gz", output_path.read_text(encoding="utf-8"))

    def test_creates_zip_archive_on_windows(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            target = "x86_64-pc-windows-msvc"
            self._write_workspace(root, target=target, runner_os="windows-latest")

            result, output_path = self._run_script(root, target=target, runner_os="windows-latest")
            self.assertEqual(result.returncode, 0, msg=result.stderr)

            archive = root / f"flowmark-v1.2.3-{target}.zip"
            self.assertTrue(archive.exists())
            with zipfile.ZipFile(archive) as zf:
                names = set(zf.namelist())
            prefix = f"flowmark-v1.2.3-{target}"
            self.assertIn(f"{prefix}/flowmark.exe", names)
            self.assertIn(f"{prefix}/LICENSE", names)
            self.assertIn(f"{prefix}/README.md", names)
            self.assertIn(f"archive=flowmark-v1.2.3-{target}.zip", output_path.read_text(encoding="utf-8"))

    def test_fails_when_binary_is_missing(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            target = "x86_64-unknown-linux-musl"
            (root / "LICENSE").write_text("license", encoding="utf-8")
            (root / "README.md").write_text("readme", encoding="utf-8")

            result, _output_path = self._run_script(root, target=target, runner_os="ubuntu-latest")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Missing required file for packaging", result.stderr)


if __name__ == "__main__":
    unittest.main()
