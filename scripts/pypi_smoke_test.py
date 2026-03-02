#!/usr/bin/env python3
"""Run PyPI wheel smoke tests when the built wheel is runnable on this runner."""

from __future__ import annotations

import argparse
import platform
import subprocess
import sys


def _normalize_runner_os(raw: str) -> str:
    value = raw.lower().strip()
    if value in {"linux"}:
        return "linux"
    if value in {"darwin", "macos", "mac"}:
        return "darwin"
    if value in {"windows", "win32", "win"}:
        return "windows"
    raise ValueError(f"Unsupported runner OS: {raw!r}")


def _normalize_runner_arch(raw: str) -> str:
    value = raw.lower().strip()
    if value in {"x86_64", "amd64"}:
        return "x86_64"
    if value in {"aarch64", "arm64"}:
        return "aarch64"
    raise ValueError(f"Unsupported runner architecture: {raw!r}")


def _auto_runner_os() -> str:
    if sys.platform.startswith("linux"):
        return "linux"
    if sys.platform == "darwin":
        return "darwin"
    if sys.platform.startswith("win"):
        return "windows"
    raise ValueError(f"Unsupported platform: {sys.platform!r}")


def _auto_runner_arch() -> str:
    return _normalize_runner_arch(platform.machine())


def _target_os_and_arch(target: str) -> tuple[str, str]:
    parts = target.split("-")
    if len(parts) < 3:
        raise ValueError(f"Invalid target triple: {target!r}")

    arch = _normalize_runner_arch(parts[0])
    if "windows" in parts:
        os_name = "windows"
    elif "apple" in parts and "darwin" in parts:
        os_name = "darwin"
    elif "linux" in parts:
        os_name = "linux"
    else:
        raise ValueError(f"Unsupported target OS in target triple: {target!r}")

    return os_name, arch


def _skip_reason(target: str, runner_os: str, runner_arch: str) -> str | None:
    target_os, target_arch = _target_os_and_arch(target)
    if target_os != runner_os:
        return f"target OS {target_os} does not match runner OS {runner_os}"
    if target_arch != runner_arch:
        return f"target arch {target_arch} does not match runner arch {runner_arch}"
    return None


def _run_command(command: list[str], dry_run: bool) -> int:
    printable = " ".join(command)
    print(f"+ {printable}")
    if dry_run:
        return 0
    result = subprocess.run(command, check=False)
    return result.returncode


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True, help="Rust target triple used for the built wheel")
    parser.add_argument("--dist-dir", default="dist", help="Directory containing built wheel artifacts")
    parser.add_argument("--package", default="flowmark-rs", help="Package name used for pip install")
    parser.add_argument("--entrypoint", default="flowmark-rs", help="CLI entrypoint used for version check")
    parser.add_argument("--runner-os", default="", help="Override auto-detected runner OS for testing")
    parser.add_argument("--runner-arch", default="", help="Override auto-detected runner arch for testing")
    parser.add_argument("--dry-run", action="store_true", help="Print commands without executing them")
    args = parser.parse_args()

    try:
        runner_os = _normalize_runner_os(args.runner_os) if args.runner_os else _auto_runner_os()
        runner_arch = _normalize_runner_arch(args.runner_arch) if args.runner_arch else _auto_runner_arch()
        reason = _skip_reason(args.target, runner_os, runner_arch)
    except ValueError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    if reason:
        print(f"Skipping smoke test for {args.target}: {reason}")
        return 0

    install_command = [
        sys.executable,
        "-m",
        "pip",
        "install",
        "--no-index",
        "--find-links",
        args.dist_dir,
        args.package,
        "--force-reinstall",
    ]
    install_code = _run_command(install_command, args.dry_run)
    if install_code != 0:
        return install_code

    entrypoint = args.entrypoint
    if runner_os == "windows" and not entrypoint.lower().endswith(".exe"):
        entrypoint = f"{entrypoint}.exe"
    return _run_command([entrypoint, "--version"], args.dry_run)


if __name__ == "__main__":
    raise SystemExit(main())
