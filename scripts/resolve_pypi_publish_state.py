#!/usr/bin/env python3
"""Resolve PyPI publish state for GitHub Actions.

Mirrors resolve_crate_metadata.py for the PyPI channel: determine whether the
distribution version is already published so the workflow can skip the upload
(and its non-idempotent dry-run validation) on reruns. This keeps the release
rerun-safe per the publishing invariant "PyPI publishes are skip-safe".
"""

from __future__ import annotations

import argparse
import os
import re
import sys
import tomllib
import urllib.error
import urllib.request
from pathlib import Path


def _load_version(manifest_path: Path) -> str:
    with manifest_path.open("rb") as handle:
        manifest = tomllib.load(handle)

    package = manifest.get("package")
    version = package.get("version") if isinstance(package, dict) else None

    if not isinstance(version, str) or not version:
        raise ValueError("Failed to resolve package version from Cargo.toml")

    return version


def _release_exists(json_url_base: str, project: str, version: str) -> bool:
    url = f"{json_url_base.rstrip('/')}/{project}/{version}/json"
    request = urllib.request.Request(url, method="GET")

    try:
        with urllib.request.urlopen(request, timeout=10) as response:
            status_code = response.getcode()
    except urllib.error.HTTPError as exc:
        if exc.code == 404:
            return False
        raise RuntimeError(f"Unexpected response from PyPI: HTTP {exc.code} for {url}") from exc
    except urllib.error.URLError as exc:
        raise RuntimeError(f"Failed to query PyPI {url}: {exc.reason}") from exc

    if status_code == 200:
        return True

    raise RuntimeError(f"Unexpected response from PyPI: HTTP {status_code} for {url}")


def _write_outputs(outputs: dict[str, str], github_output_path: str | None) -> None:
    lines = [f"{key}={value}" for key, value in outputs.items()]
    for line in lines:
        print(line)

    if github_output_path:
        output_file = Path(github_output_path)
        with output_file.open("a", encoding="utf-8") as handle:
            for line in lines:
                handle.write(f"{line}\n")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest-path", default="Cargo.toml")
    parser.add_argument("--project", default="flowmark-rs")
    parser.add_argument("--json-url", default="https://pypi.org/pypi")
    parser.add_argument("--github-output", default=os.getenv("GITHUB_OUTPUT", ""))
    args = parser.parse_args()

    manifest_path = Path(args.manifest_path)
    if not manifest_path.is_file():
        print(f"Manifest file not found: {manifest_path}", file=sys.stderr)
        return 1

    if not re.fullmatch(r"[A-Za-z0-9._-]+", args.project):
        print(f"Invalid project name: {args.project!r}", file=sys.stderr)
        return 1

    try:
        version = _load_version(manifest_path)
        already_published = _release_exists(args.json_url, args.project, version)
    except Exception as exc:  # noqa: BLE001 - surfacing as workflow failure is intentional
        print(str(exc), file=sys.stderr)
        return 1

    _write_outputs(
        {
            "project": args.project,
            "version": version,
            "already_published": "true" if already_published else "false",
        },
        args.github_output,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
