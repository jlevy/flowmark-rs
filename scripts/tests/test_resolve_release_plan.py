#!/usr/bin/env python3
"""Tests for scripts/resolve-release-plan.py."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "resolve-release-plan.py"


class ResolveReleasePlanTests(unittest.TestCase):
    maxDiff = None

    def _run_script(
        self,
        *,
        event_name: str,
        ref_name: str = "",
        tag: str = "dry-run",
        publish: str = "false",
        publish_prerelease: str = "false",
        run_id: str = "12345",
    ) -> tuple[subprocess.CompletedProcess[str], str]:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "github_output.txt"
            result = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT_PATH),
                    "--event-name",
                    event_name,
                    "--ref-name",
                    ref_name,
                    "--tag",
                    tag,
                    "--publish",
                    publish,
                    "--publish-prerelease",
                    publish_prerelease,
                    "--run-id",
                    run_id,
                    "--github-output",
                    str(output_path),
                ],
                capture_output=True,
                text=True,
            )
            output = output_path.read_text(encoding="utf-8") if output_path.exists() else ""
            return result, output

    @staticmethod
    def _to_map(output: str) -> dict[str, str]:
        mapped: dict[str, str] = {}
        for line in output.splitlines():
            if "=" not in line:
                continue
            key, value = line.split("=", 1)
            mapped[key] = value
        return mapped

    def test_push_event_for_stable_tag_enables_publish_channels(self) -> None:
        result, output = self._run_script(
            event_name="push",
            ref_name="v1.2.3",
            tag="dry-run",
            publish="false",
            publish_prerelease="false",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        data = self._to_map(output)
        self.assertEqual(data["release_tag"], "v1.2.3")
        self.assertEqual(data["artifact_tag"], "v1.2.3")
        self.assertEqual(data["prerelease"], "false")
        self.assertEqual(data["publish"], "true")
        self.assertEqual(data["publish_channels"], "true")

    def test_dispatch_dry_run_disables_publish(self) -> None:
        result, output = self._run_script(
            event_name="workflow_dispatch",
            tag="dry-run",
            publish="false",
            publish_prerelease="false",
            run_id="98765",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        data = self._to_map(output)
        self.assertEqual(data["release_tag"], "")
        self.assertEqual(data["artifact_tag"], "v0.0.0-dry-run-98765")
        self.assertEqual(data["prerelease"], "true")
        self.assertEqual(data["publish"], "false")
        self.assertEqual(data["publish_channels"], "false")

    def test_dispatch_stable_tag_normalizes_v_prefix(self) -> None:
        result, output = self._run_script(
            event_name="workflow_dispatch",
            tag="1.2.3",
            publish="true",
            publish_prerelease="false",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        data = self._to_map(output)
        self.assertEqual(data["release_tag"], "v1.2.3")
        self.assertEqual(data["artifact_tag"], "v1.2.3")
        self.assertEqual(data["prerelease"], "false")
        self.assertEqual(data["publish"], "true")
        self.assertEqual(data["publish_channels"], "true")

    def test_dispatch_prerelease_requires_explicit_allow(self) -> None:
        result, output = self._run_script(
            event_name="workflow_dispatch",
            tag="v1.2.3-rc.1",
            publish="true",
            publish_prerelease="false",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        data = self._to_map(output)
        self.assertEqual(data["prerelease"], "true")
        self.assertEqual(data["publish_channels"], "false")

    def test_dispatch_prerelease_publishes_when_allowed(self) -> None:
        result, output = self._run_script(
            event_name="workflow_dispatch",
            tag="v1.2.3-rc.1",
            publish="true",
            publish_prerelease="true",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        data = self._to_map(output)
        self.assertEqual(data["prerelease"], "true")
        self.assertEqual(data["publish_channels"], "true")

    def test_publish_mode_rejects_dry_run_tag(self) -> None:
        result, _output = self._run_script(
            event_name="workflow_dispatch",
            tag="dry-run",
            publish="true",
            publish_prerelease="false",
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Publish mode requires a real tag", result.stderr)


if __name__ == "__main__":
    unittest.main()
