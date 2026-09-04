#!/usr/bin/env python3
"""Resolve release workflow mode and derived outputs for GitHub Actions."""

from __future__ import annotations

import argparse
import os
import re
import sys
import tomllib
from pathlib import Path


def _parse_bool(raw: str, field_name: str) -> bool:
    normalized = raw.strip().lower()
    if normalized in {"1", "true", "yes", "on"}:
        return True
    if normalized in {"", "0", "false", "no", "off"}:
        return False
    raise ValueError(f"Invalid boolean for {field_name}: {raw!r}")


def _write_outputs(outputs: dict[str, str], github_output_path: str | None) -> None:
    lines = [f"{key}={value}" for key, value in outputs.items()]
    for line in lines:
        print(line)

    if github_output_path:
        output_file = Path(github_output_path)
        with output_file.open("a", encoding="utf-8") as handle:
            for line in lines:
                handle.write(f"{line}\n")


def _read_package_version(manifest_path: Path) -> str:
    try:
        manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
        version = manifest["package"]["version"]
    except (OSError, KeyError, TypeError, tomllib.TOMLDecodeError) as error:
        raise ValueError(f"Cannot read package.version from {manifest_path}: {error}") from error
    if not isinstance(version, str) or not version.strip():
        raise ValueError(f"Cannot read package.version from {manifest_path}: expected a string")
    return version.strip()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--event-name", required=True)
    parser.add_argument("--ref-name", default="")
    parser.add_argument("--tag", default="dry-run")
    parser.add_argument("--publish", default="false")
    parser.add_argument("--publish-prerelease", default="false")
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--manifest-path", type=Path, default=Path("Cargo.toml"))
    parser.add_argument("--github-output", default=os.getenv("GITHUB_OUTPUT", ""))
    args = parser.parse_args()

    try:
        if args.event_name == "push":
            requested_tag = args.ref_name
            requested_publish = True
            publish_prerelease = False
        else:
            requested_tag = args.tag
            requested_publish = _parse_bool(args.publish, "publish")
            publish_prerelease = _parse_bool(args.publish_prerelease, "publish_prerelease")

        if not requested_tag or requested_tag == "dry-run":
            if requested_publish:
                raise ValueError("Publish mode requires a real tag (not dry-run).")
            release_tag = ""
            artifact_tag = f"v0.0.0-dry-run-{args.run_id}"
            prerelease = True
            publish = False
        else:
            release_tag = requested_tag if requested_tag.startswith("v") else f"v{requested_tag}"
            package_version = _read_package_version(args.manifest_path)
            expected_tag = f"v{package_version}"
            if release_tag != expected_tag:
                raise ValueError(
                    f"Release tag {release_tag} does not match Cargo package version "
                    f"{package_version} (expected {expected_tag})."
                )
            artifact_tag = release_tag
            prerelease = re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", release_tag) is None
            publish = requested_publish

        publish_channels = False
        if publish:
            publish_channels = not prerelease or publish_prerelease

    except Exception as exc:  # noqa: BLE001 - surfacing as workflow failure is intentional
        print(str(exc), file=sys.stderr)
        return 1

    _write_outputs(
        {
            "release_tag": release_tag,
            "artifact_tag": artifact_tag,
            "prerelease": "true" if prerelease else "false",
            "publish": "true" if publish else "false",
            "publish_channels": "true" if publish_channels else "false",
        },
        args.github_output,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
