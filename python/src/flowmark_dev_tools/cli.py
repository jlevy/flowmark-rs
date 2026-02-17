"""
CLI entry point for flowmark dev tools.

Commands:
  discover-python  Discover all tests in the Python flowmark repo and write YAML.
  discover-rust    Discover all tests in the Rust flowmark-rs repo and write YAML.
  init-mapping     Generate a skeleton mapping file from existing Python test manifest.
  check-mapping    Validate the mapping against both test manifests.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
from pathlib import Path

from flowmark_dev_tools.check_mapping import run_check
from flowmark_dev_tools.discover_python import discover_python_tests
from flowmark_dev_tools.discover_rust import discover_rust_tests
from flowmark_dev_tools.models import MappingRecord, MappingStatus
from flowmark_dev_tools.yaml_io import (
    load_mapping_yaml,
    load_python_tests_yaml,
    write_mapping_yaml,
    write_python_tests_yaml,
    write_rust_tests_yaml,
)

# Default paths relative to the flowmark-rs repo root.
DEFAULT_MAPPING_DIR = "port-coverage-mapping"
DEFAULT_PYTHON_YAML = f"{DEFAULT_MAPPING_DIR}/python-tests.yaml"
DEFAULT_RUST_YAML = f"{DEFAULT_MAPPING_DIR}/rust-tests.yaml"
DEFAULT_MAPPING_YAML = f"{DEFAULT_MAPPING_DIR}/test-mapping.yaml"
DEFAULT_REPO_URL = "https://github.com/jlevy/flowmark"
DEFAULT_REF = "v0.6.4"


def _resolve_root() -> Path:
    """Find the repo root (directory containing Cargo.toml)."""
    cwd = Path.cwd()
    for parent in [cwd, *cwd.parents]:
        if (parent / "Cargo.toml").exists():
            return parent
    return cwd


def cmd_discover_python(args: argparse.Namespace) -> int:
    """Discover Python tests and write YAML manifest."""
    root = _resolve_root()
    output = Path(args.output) if args.output else root / DEFAULT_PYTHON_YAML

    if args.local_path:
        repo_path = Path(args.local_path)
        print(f"Using local path: {repo_path}")
        records = discover_python_tests(repo_path)
    else:
        repo_url = args.repo_url
        ref = args.ref
        print(f"Cloning {repo_url} at {ref}...")
        with tempfile.TemporaryDirectory(prefix="flowmark-python-") as tmpdir:
            subprocess.run(
                ["git", "clone", "--depth=1", "--branch", ref, repo_url, tmpdir],
                check=True,
                capture_output=True,
                text=True,
            )
            records = discover_python_tests(Path(tmpdir))

    write_python_tests_yaml(records, output)
    print(f"Discovered {len(records)} Python tests -> {output}")
    return 0


def cmd_discover_rust(args: argparse.Namespace) -> int:
    """Discover Rust tests and write YAML manifest."""
    root = _resolve_root()
    tests_dir = Path(args.tests_dir) if args.tests_dir else root / "tests"
    output = Path(args.output) if args.output else root / DEFAULT_RUST_YAML

    records = discover_rust_tests(tests_dir)
    write_rust_tests_yaml(records, output)
    print(f"Discovered {len(records)} Rust tests -> {output}")
    return 0


def cmd_init_mapping(args: argparse.Namespace) -> int:
    """
    Generate a skeleton mapping file from the Python test manifest.
    If a mapping file already exists, preserves existing entries and adds
    new ones as `missing`. Does not remove entries for deleted tests
    (use `check-mapping` to detect those).
    """
    root = _resolve_root()
    python_yaml = Path(args.python_yaml) if args.python_yaml else root / DEFAULT_PYTHON_YAML
    output = Path(args.output) if args.output else root / DEFAULT_MAPPING_YAML

    python_tests = load_python_tests_yaml(python_yaml)

    # Load existing mapping if it exists, to preserve manual edits.
    existing: dict[tuple[str, str | None, str], MappingRecord] = {}
    if output.exists():
        for m in load_mapping_yaml(output):
            existing[(m.python_file, m.python_class, m.python_function)] = m

    records: list[MappingRecord] = []
    new_count = 0
    for pt in python_tests:
        key = (pt.file, pt.class_name, pt.function)
        if key in existing:
            records.append(existing[key])
        else:
            records.append(MappingRecord(
                python_file=pt.file,
                python_function=pt.function,
                python_class=pt.class_name,
                status=MappingStatus.missing,
            ))
            new_count += 1

    records.sort(key=lambda r: (r.python_file, r.python_class or "", r.python_function))
    write_mapping_yaml(records, output)
    total = len(records)
    preserved = total - new_count
    print(f"Mapping file: {output}")
    print(f"  Total entries: {total} ({new_count} new, {preserved} preserved)")
    return 0


def cmd_check_mapping(args: argparse.Namespace) -> int:
    """Check the completeness and consistency of the mapping."""
    root = _resolve_root()
    python_yaml = Path(args.python_yaml) if args.python_yaml else root / DEFAULT_PYTHON_YAML
    rust_yaml = Path(args.rust_yaml) if args.rust_yaml else root / DEFAULT_RUST_YAML
    mapping_yaml = Path(args.mapping_yaml) if args.mapping_yaml else root / DEFAULT_MAPPING_YAML

    return run_check(python_yaml, rust_yaml, mapping_yaml)


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(
        prog="flowmark-dev",
        description="Dev tools for flowmark-rs porting: test discovery and mapping.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    # discover-python
    dp = subparsers.add_parser(
        "discover-python",
        help="Discover all tests in the Python flowmark repo.",
    )
    dp.add_argument("--repo-url", default=DEFAULT_REPO_URL, help="Git repo URL.")
    dp.add_argument("--ref", default=DEFAULT_REF, help="Branch, tag, or commit.")
    dp.add_argument("--local-path", help="Use a local checkout instead of cloning.")
    dp.add_argument("--output", "-o", help="Output YAML path.")

    # discover-rust
    dr = subparsers.add_parser(
        "discover-rust",
        help="Discover all tests in the Rust flowmark-rs repo.",
    )
    dr.add_argument("--tests-dir", help="Path to Rust tests/ directory.")
    dr.add_argument("--output", "-o", help="Output YAML path.")

    # init-mapping
    im = subparsers.add_parser(
        "init-mapping",
        help="Generate or update a skeleton mapping file.",
    )
    im.add_argument("--python-yaml", help="Path to Python tests YAML.")
    im.add_argument("--output", "-o", help="Output mapping YAML path.")

    # check-mapping
    cm = subparsers.add_parser(
        "check-mapping",
        help="Validate mapping completeness and consistency.",
    )
    cm.add_argument("--python-yaml", help="Path to Python tests YAML.")
    cm.add_argument("--rust-yaml", help="Path to Rust tests YAML.")
    cm.add_argument("--mapping-yaml", help="Path to mapping YAML.")

    args = parser.parse_args(argv)

    handlers: dict[str, object] = {
        "discover-python": cmd_discover_python,
        "discover-rust": cmd_discover_rust,
        "init-mapping": cmd_init_mapping,
        "check-mapping": cmd_check_mapping,
    }
    handler = handlers[args.command]
    sys.exit(handler(args))  # pyright: ignore[reportCallIssue]
