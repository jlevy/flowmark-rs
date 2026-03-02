#!/usr/bin/env python3
"""Validate expected flowmark CLI entrypoints are present in built wheels."""

from __future__ import annotations

import argparse
import zipfile
from pathlib import Path


def validate_wheels(wheels_dir: Path) -> None:
    wheels = sorted(wheels_dir.glob("*.whl"))
    if not wheels:
        raise ValueError("No wheel files found to validate")

    missing: list[str] = []
    for wheel in wheels:
        with zipfile.ZipFile(wheel) as zf:
            names = zf.namelist()
        has_flowmark = any(name.endswith("/flowmark") or name.endswith("/flowmark.exe") for name in names)
        has_flowmark_rs = any(name.endswith("/flowmark-rs") or name.endswith("/flowmark-rs.exe") for name in names)
        if not (has_flowmark and has_flowmark_rs):
            missing.append(wheel.name)

    if missing:
        joined = ", ".join(missing)
        raise ValueError(f"Missing expected entrypoints in wheels: {joined}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--wheels-dir", default="wheels")
    args = parser.parse_args()

    wheels_dir = Path(args.wheels_dir)
    try:
        validate_wheels(wheels_dir)
    except Exception as exc:  # noqa: BLE001 - fail workflow with clear message
        raise SystemExit(str(exc))

    print(f"Validated wheel entrypoints for {len(list(wheels_dir.glob('*.whl')))} wheel(s).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
