#!/usr/bin/env python3
"""Tests for scripts/resolve_pypi_publish_state.py."""

from __future__ import annotations

import http.server
import subprocess
import sys
import tempfile
import textwrap
import threading
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "resolve_pypi_publish_state.py"


def _start_stub_server(status_code: int, expected_path: str):
    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802 (stdlib interface)
            code = status_code if self.path == expected_path else 400
            self.send_response(code)
            self.end_headers()
            self.wfile.write(b"{}")

        def log_message(self, _format: str, *_args) -> None:  # noqa: A003
            return

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    return server, thread


class ResolvePypiPublishStateTests(unittest.TestCase):
    maxDiff = None

    def _run_script(self, *, cargo_toml: str, status_code: int) -> tuple[subprocess.CompletedProcess[str], str]:
        expected_path = "/pypi/flowmark-rs/1.2.3/json"
        server, thread = _start_stub_server(status_code, expected_path)
        output_contents = ""

        try:
            with tempfile.TemporaryDirectory() as temp_dir:
                temp_path = Path(temp_dir)
                manifest_path = temp_path / "Cargo.toml"
                output_path = temp_path / "github_output.txt"
                manifest_path.write_text(cargo_toml, encoding="utf-8")

                base_url = f"http://127.0.0.1:{server.server_port}/pypi"
                result = subprocess.run(
                    [
                        sys.executable,
                        str(SCRIPT_PATH),
                        "--manifest-path",
                        str(manifest_path),
                        "--project",
                        "flowmark-rs",
                        "--json-url",
                        base_url,
                        "--github-output",
                        str(output_path),
                    ],
                    capture_output=True,
                    text=True,
                )

                if output_path.exists():
                    output_contents = output_path.read_text(encoding="utf-8")
        finally:
            server.shutdown()
            thread.join(timeout=5)
            server.server_close()

        return result, output_contents

    def test_detects_already_published_false_when_version_missing(self) -> None:
        cargo_toml = textwrap.dedent(
            """
            [package]
            name = "flowmark-rs"
            version = "1.2.3"
            """
        )
        result, output = self._run_script(cargo_toml=cargo_toml, status_code=404)
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        self.assertIn("project=flowmark-rs", output)
        self.assertIn("version=1.2.3", output)
        self.assertIn("already_published=false", output)

    def test_detects_already_published_true_when_version_exists(self) -> None:
        cargo_toml = textwrap.dedent(
            """
            [package]
            name = "flowmark-rs"
            version = "1.2.3"
            """
        )
        result, output = self._run_script(cargo_toml=cargo_toml, status_code=200)
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        self.assertIn("already_published=true", output)

    def test_fails_for_manifest_without_package_version(self) -> None:
        cargo_toml = textwrap.dedent(
            """
            [package]
            name = "flowmark-rs"
            """
        )
        result, _output = self._run_script(cargo_toml=cargo_toml, status_code=404)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Failed to resolve package version", result.stderr)

    def test_fails_for_registry_server_errors(self) -> None:
        cargo_toml = textwrap.dedent(
            """
            [package]
            name = "flowmark-rs"
            version = "1.2.3"
            """
        )
        result, _output = self._run_script(cargo_toml=cargo_toml, status_code=500)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Unexpected response from PyPI", result.stderr)


if __name__ == "__main__":
    unittest.main()
