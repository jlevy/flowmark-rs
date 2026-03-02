#!/usr/bin/env python3
"""Resolve crate metadata and publish state for GitHub Actions."""

from __future__ import annotations

import argparse
import os
import sys
import tomllib
import urllib.error
import urllib.request
from pathlib import Path


def _load_name_and_version(manifest_path: Path) -> tuple[str, str]:
    with manifest_path.open("rb") as handle:
        manifest = tomllib.load(handle)

    package = manifest.get("package")
    name = package.get("name") if isinstance(package, dict) else None
    version = package.get("version") if isinstance(package, dict) else None

    if not isinstance(name, str) or not name or not isinstance(version, str) or not version:
        raise ValueError("Failed to resolve crate name/version from Cargo.toml")

    return name, version


def _crate_version_exists(registry_url: str, crate_name: str, crate_version: str) -> bool:
    url = f"{registry_url.rstrip('/')}/{crate_name}/{crate_version}"
    request = urllib.request.Request(url, method="GET")

    try:
        with urllib.request.urlopen(request, timeout=10) as response:
            status_code = response.getcode()
    except urllib.error.HTTPError as exc:
        if exc.code == 404:
            return False
        raise RuntimeError(f"Unexpected response from registry: HTTP {exc.code} for {url}") from exc
    except urllib.error.URLError as exc:
        raise RuntimeError(f"Failed to query registry {url}: {exc.reason}") from exc

    if status_code == 200:
        return True

    raise RuntimeError(f"Unexpected response from registry: HTTP {status_code} for {url}")


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
    parser.add_argument("--registry-url", default="https://crates.io/api/v1/crates")
    parser.add_argument("--github-output", default=os.getenv("GITHUB_OUTPUT", ""))
    args = parser.parse_args()

    manifest_path = Path(args.manifest_path)
    if not manifest_path.is_file():
        print(f"Manifest file not found: {manifest_path}", file=sys.stderr)
        return 1

    try:
        crate_name, crate_version = _load_name_and_version(manifest_path)
        already_published = _crate_version_exists(args.registry_url, crate_name, crate_version)
    except Exception as exc:  # noqa: BLE001 - surfacing as workflow failure is intentional
        print(str(exc), file=sys.stderr)
        return 1

    _write_outputs(
        {
            "crate_name": crate_name,
            "crate_version": crate_version,
            "already_published": "true" if already_published else "false",
        },
        args.github_output,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
