#!/usr/bin/env python3
"""Package a built flowmark binary into a release archive."""

from __future__ import annotations

import argparse
import os
import shutil
import tarfile
import zipfile
from pathlib import Path


def _write_output(key: str, value: str, github_output: str | None) -> None:
    line = f"{key}={value}"
    print(line)
    if github_output:
        with Path(github_output).open("a", encoding="utf-8") as handle:
            handle.write(f"{line}\n")


def _binary_name_for_runner_os(runner_os: str) -> str:
    return "flowmark.exe" if runner_os == "windows-latest" else "flowmark"


def _archive_name_for_runner_os(staging_name: str, runner_os: str) -> str:
    extension = ".zip" if runner_os == "windows-latest" else ".tar.gz"
    return f"{staging_name}{extension}"


def _create_archive(staging_dir: Path, archive_path: Path, runner_os: str) -> None:
    if runner_os == "windows-latest":
        with zipfile.ZipFile(archive_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
            for path in sorted(staging_dir.rglob("*")):
                zf.write(path, path.relative_to(staging_dir.parent))
        return

    with tarfile.open(archive_path, "w:gz") as tf:
        tf.add(staging_dir, arcname=staging_dir.name)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifact-tag", required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--runner-os", required=True)
    parser.add_argument("--workspace", default=".")
    parser.add_argument("--target-dir", default="target")
    parser.add_argument("--github-output", default=os.getenv("GITHUB_OUTPUT", ""))
    args = parser.parse_args()

    workspace = Path(args.workspace).resolve()
    binary_name = _binary_name_for_runner_os(args.runner_os)
    binary_path = workspace / args.target_dir / args.target / "release" / binary_name
    license_path = workspace / "LICENSE"
    readme_path = workspace / "README.md"

    for required_path in (binary_path, license_path, readme_path):
        if not required_path.exists():
            raise SystemExit(f"Missing required file for packaging: {required_path}")

    staging_name = f"flowmark-{args.artifact_tag}-{args.target}"
    staging_dir = workspace / staging_name
    if staging_dir.exists():
        shutil.rmtree(staging_dir)
    staging_dir.mkdir(parents=True)

    shutil.copy2(binary_path, staging_dir / binary_name)
    shutil.copy2(license_path, staging_dir / "LICENSE")
    shutil.copy2(readme_path, staging_dir / "README.md")

    archive_name = _archive_name_for_runner_os(staging_name, args.runner_os)
    archive_path = workspace / archive_name
    if archive_path.exists():
        archive_path.unlink()

    _create_archive(staging_dir, archive_path, args.runner_os)
    _write_output("archive", archive_name, args.github_output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
