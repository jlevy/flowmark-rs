#!/usr/bin/env python3
"""Sync the Rust CLI's packaged skill from the main Flowmark repository."""

from __future__ import annotations

import shutil
from argparse import ArgumentParser
from pathlib import Path

UPSTREAM_SKILL_DIR = Path("repos/flowmark/src/flowmark/skills")
RUNTIME_SKILL_DIR = Path("src/skills")
UPSTREAM_ONLY_FILES = frozenset({Path("__init__.py")})
RUNTIME_ONLY_FILES = frozenset({Path("mod.rs")})


def _payload_files(root: Path, ignored: frozenset[Path] = frozenset()) -> set[Path]:
    """Return file paths relative to `root`, excluding implementation-only files."""
    if not root.is_dir():
        raise FileNotFoundError(f"missing skill source directory at {root}")
    return {
        path.relative_to(root)
        for path in root.rglob("*")
        if path.is_file()
        and "__pycache__" not in path.parts
        and path.relative_to(root) not in ignored
    }


def _runtime_payload_files(runtime_dir: Path, upstream_files: set[Path]) -> set[Path]:
    """Return mirrored resources while leaving Rust implementation modules unmanaged."""
    return {
        relative
        for relative in _payload_files(runtime_dir, RUNTIME_ONLY_FILES)
        if relative in upstream_files or relative.suffix != ".rs"
    }


def verify_skill_runtime_mirror(repo_root: Path) -> None:
    """Raise when the packaged Rust skill differs from the upstream-owned bundle."""
    upstream_dir = repo_root / UPSTREAM_SKILL_DIR
    runtime_dir = repo_root / RUNTIME_SKILL_DIR
    upstream_files = _payload_files(upstream_dir, UPSTREAM_ONLY_FILES)
    runtime_files = _runtime_payload_files(runtime_dir, upstream_files)

    missing = sorted(upstream_files - runtime_files)
    extra = sorted(runtime_files - upstream_files)
    if missing or extra:
        raise ValueError(
            "Rust runtime skill file set drifted "
            f"(missing={missing}, extra={extra}); run "
            "`python3 scripts/sync_skill_mirror.py`"
        )

    changed = sorted(
        relative
        for relative in upstream_files
        if (upstream_dir / relative).read_bytes() != (runtime_dir / relative).read_bytes()
    )
    if changed:
        raise ValueError(
            f"Rust runtime skill content drifted ({changed}); run "
            "`python3 scripts/sync_skill_mirror.py`"
        )


def sync_skill_runtime_mirror(repo_root: Path) -> None:
    """Copy the complete upstream skill bundle while preserving Rust implementation code."""
    upstream_dir = repo_root / UPSTREAM_SKILL_DIR
    runtime_dir = repo_root / RUNTIME_SKILL_DIR
    upstream_files = _payload_files(upstream_dir, UPSTREAM_ONLY_FILES)
    runtime_files = _runtime_payload_files(runtime_dir, upstream_files)

    for relative in runtime_files - upstream_files:
        (runtime_dir / relative).unlink()
    for relative in upstream_files:
        target = runtime_dir / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(upstream_dir / relative, target)

    directories = sorted(
        (path for path in runtime_dir.rglob("*") if path.is_dir()),
        key=lambda path: len(path.parts),
        reverse=True,
    )
    for directory in directories:
        if not any(directory.iterdir()):
            directory.rmdir()

    verify_skill_runtime_mirror(repo_root)


def main() -> int:
    """Sync the mirror, or verify it without writing when `--check` is set."""
    parser = ArgumentParser(description=__doc__)
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="flowmark-rs repository root (default: inferred from this script)",
    )
    parser.add_argument("--check", action="store_true", help="verify without modifying files")
    args = parser.parse_args()
    try:
        if args.check:
            verify_skill_runtime_mirror(args.repo_root)
            print("Flowmark skill runtime mirror is in sync")
        else:
            sync_skill_runtime_mirror(args.repo_root)
            print("Synced Flowmark skill runtime mirror from repos/flowmark")
    except (OSError, ValueError) as error:
        parser.exit(1, f"error: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
