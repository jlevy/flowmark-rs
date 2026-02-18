"""
Check the completeness and consistency of the Python-to-Rust test mapping.

Loads the three YAML files (python tests, rust tests, mapping) and validates:
1. Every Python test has a mapping entry.
2. Every mapped Rust function/file actually exists.
3. No stale mapping entries reference removed Python tests.
4. Extra Rust tests (not referenced in mapping) are logged for visibility.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from flowmark_dev_tools.models import MappingRecord, MappingStatus
from flowmark_dev_tools.yaml_io import (
    load_mapping_yaml,
    load_python_tests_yaml,
    load_rust_tests_yaml,
)


@dataclass
class CheckResult:
    """Result of a mapping completeness check."""

    total_python: int = 0
    mapped: int = 0
    excluded: int = 0
    partial: int = 0
    missing: int = 0
    stale_entries: list[str] | None = None
    unmapped_python: list[str] | None = None
    broken_rust_refs: list[str] | None = None
    extra_rust: list[str] | None = None

    @property
    def ok(self) -> bool:
        return self.missing == 0 and not self.stale_entries and not self.broken_rust_refs


def check_mapping(
    python_yaml: Path,
    rust_yaml: Path,
    mapping_yaml: Path,
) -> CheckResult:
    """
    Validate the mapping against both test manifests.
    Returns a `CheckResult` with detailed findings.
    """
    python_tests = load_python_tests_yaml(python_yaml)
    rust_tests = load_rust_tests_yaml(rust_yaml)
    mapping = load_mapping_yaml(mapping_yaml)

    # Build lookup sets.
    python_keys: set[tuple[str, str | None, str]] = {
        (t.file, t.class_name, t.function) for t in python_tests
    }
    rust_keys: set[tuple[str, str]] = {(t.file, t.function) for t in rust_tests}
    mapping_python_keys: set[tuple[str, str | None, str]] = {
        (m.python_file, m.python_class, m.python_function) for m in mapping
    }

    result = CheckResult(total_python=len(python_tests))

    # 1. Check that every Python test has a mapping entry.
    unmapped = python_keys - mapping_python_keys
    if unmapped:
        result.unmapped_python = sorted(
            f"{f}::{c + '::' if c else ''}{fn}" for f, c, fn in unmapped
        )

    # 2. Count by status.
    for m in mapping:
        match m.status:
            case MappingStatus.mapped:
                result.mapped += 1
            case MappingStatus.excluded:
                result.excluded += 1
            case MappingStatus.partial:
                result.partial += 1
            case MappingStatus.missing:
                result.missing += 1

    # 3. Check that mapped Rust refs actually exist.
    broken: list[str] = []
    for m in mapping:
        if m.status in (MappingStatus.mapped, MappingStatus.partial):
            _check_rust_refs(m, rust_keys, broken)
    if broken:
        result.broken_rust_refs = broken

    # 4. Check for stale mapping entries (Python test no longer exists).
    stale = mapping_python_keys - python_keys
    if stale:
        result.stale_entries = sorted(f"{f}::{c + '::' if c else ''}{fn}" for f, c, fn in stale)

    # 5. Find extra Rust tests not referenced by any mapping.
    mapped_rust_keys: set[tuple[str, str]] = set()
    for m in mapping:
        if m.rust_file and m.rust_function:
            mapped_rust_keys.add((m.rust_file, m.rust_function))
        if m.rust_file and m.rust_functions:
            for fn in m.rust_functions:
                mapped_rust_keys.add((m.rust_file, fn))
    extra = rust_keys - mapped_rust_keys
    if extra:
        result.extra_rust = sorted(f"{f}::{fn}" for f, fn in extra)

    # Also count unmapped Python tests as missing.
    result.missing += len(unmapped) if unmapped else 0

    return result


def _check_rust_refs(
    m: MappingRecord,
    rust_keys: set[tuple[str, str]],
    broken: list[str],
) -> None:
    """Check that the Rust function references in a mapping actually exist."""
    if m.rust_file and m.rust_function:
        if (m.rust_file, m.rust_function) not in rust_keys:
            broken.append(f"{m.rust_file}::{m.rust_function}")
    if m.rust_file and m.rust_functions:
        for fn in m.rust_functions:
            if (m.rust_file, fn) not in rust_keys:
                broken.append(f"{m.rust_file}::{fn}")


def format_report(result: CheckResult) -> str:
    """Format a human-readable report of the check result."""
    lines: list[str] = []
    lines.append("=== Test Mapping Coverage Report ===")
    lines.append("")
    lines.append(f"Python tests:  {result.total_python}")
    lines.append(f"  Mapped:      {result.mapped}")
    lines.append(f"  Excluded:    {result.excluded}")
    lines.append(f"  Partial:     {result.partial}")
    lines.append(f"  Missing:     {result.missing}")
    lines.append("")

    if result.unmapped_python:
        lines.append(f"FAIL: {len(result.unmapped_python)} Python test(s) have no mapping entry:")
        for entry in result.unmapped_python:
            lines.append(f"  - {entry}")
        lines.append("")

    if result.broken_rust_refs:
        lines.append(
            f"FAIL: {len(result.broken_rust_refs)} mapping(s) reference non-existent Rust test(s):"
        )
        for entry in result.broken_rust_refs:
            lines.append(f"  - {entry}")
        lines.append("")

    if result.stale_entries:
        lines.append(
            f"WARN: {len(result.stale_entries)} mapping(s) reference "
            f"Python tests that no longer exist:"
        )
        for entry in result.stale_entries:
            lines.append(f"  - {entry}")
        lines.append("")

    if result.extra_rust:
        lines.append(
            f"INFO: {len(result.extra_rust)} Rust test(s) not referenced in mapping "
            f"(consider upstreaming or documenting):"
        )
        for entry in result.extra_rust:
            lines.append(f"  - {entry}")
        lines.append("")

    if result.ok:
        lines.append("PASS: All Python tests are covered by the mapping.")
    else:
        lines.append("FAIL: Mapping is incomplete or inconsistent.")

    return "\n".join(lines)


def run_check(
    python_yaml: Path,
    rust_yaml: Path,
    mapping_yaml: Path,
) -> int:
    """Run the check and print the report. Returns 0 on pass, 1 on fail."""
    result = check_mapping(python_yaml, rust_yaml, mapping_yaml)
    report = format_report(result)
    print(report)
    return 0 if result.ok else 1
