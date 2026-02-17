"""
YAML serialization for test records and mapping files.

Uses a clean, human-readable YAML style with consistent key ordering
so that diffs are meaningful and files are easy to review and hand-edit.
"""

from __future__ import annotations

import yaml
from dataclasses import asdict
from enum import StrEnum
from pathlib import Path

from strif import atomic_output_file

from flowmark_dev_tools.models import (
    MappingRecord,
    MappingStatus,
    PythonTestRecord,
    RustTestRecord,
    TestType,
)


# Ordered keys for deterministic, readable YAML output.

_PYTHON_KEY_ORDER = ["file", "function", "class_name", "test_type", "line_number", "doc_string"]
_RUST_KEY_ORDER = ["file", "function", "line_number"]
_MAPPING_KEY_ORDER = [
    "python_file",
    "python_function",
    "python_class",
    "status",
    "rust_file",
    "rust_function",
    "rust_functions",
    "notes",
]


def _ordered_dict(d: dict, key_order: list[str]) -> dict:
    """Reorder dict keys and omit None/empty values for cleaner YAML."""
    result = {}
    for key in key_order:
        if key in d:
            val = d[key]
            # Omit None values and empty lists for cleaner output.
            if val is None:
                continue
            if isinstance(val, list) and not val:
                continue
            # Convert StrEnum to plain string for clean YAML.
            if isinstance(val, StrEnum):
                val = str(val)
            result[key] = val
    return result


def write_python_tests_yaml(records: list[PythonTestRecord], output_path: Path) -> None:
    """Write Python test records to a YAML file."""
    data = [_ordered_dict(asdict(r), _PYTHON_KEY_ORDER) for r in records]
    _write_yaml(data, output_path, comment="Python flowmark test manifest")


def write_rust_tests_yaml(records: list[RustTestRecord], output_path: Path) -> None:
    """Write Rust test records to a YAML file."""
    data = [_ordered_dict(asdict(r), _RUST_KEY_ORDER) for r in records]
    _write_yaml(data, output_path, comment="Rust flowmark-rs test manifest")


def write_mapping_yaml(records: list[MappingRecord], output_path: Path) -> None:
    """Write mapping records to a YAML file."""
    data = [_ordered_dict(asdict(r), _MAPPING_KEY_ORDER) for r in records]
    _write_yaml(data, output_path, comment="Python-to-Rust test mapping")


def _write_yaml(data: list[dict], output_path: Path, comment: str) -> None:
    """Write data to YAML with a header comment, using atomic file writes."""
    content = f"# {comment}\n"
    content += f"# Auto-generated. Manual edits are preserved on re-generation.\n\n"
    content += yaml.dump(
        data,
        default_flow_style=False,
        sort_keys=False,
        allow_unicode=True,
        width=120,
    )
    with atomic_output_file(output_path, make_parents=True) as temp_path:
        Path(temp_path).write_text(content, encoding="utf-8")


def load_python_tests_yaml(path: Path) -> list[PythonTestRecord]:
    """Load Python test records from a YAML file."""
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not data:
        return []
    return [
        PythonTestRecord(
            file=d["file"],
            function=d["function"],
            class_name=d.get("class_name"),
            test_type=TestType(d["test_type"]),
            line_number=d["line_number"],
            doc_string=d.get("doc_string"),
        )
        for d in data
    ]


def load_rust_tests_yaml(path: Path) -> list[RustTestRecord]:
    """Load Rust test records from a YAML file."""
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not data:
        return []
    return [
        RustTestRecord(
            file=d["file"],
            function=d["function"],
            line_number=d["line_number"],
        )
        for d in data
    ]


def load_mapping_yaml(path: Path) -> list[MappingRecord]:
    """Load mapping records from a YAML file."""
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not data:
        return []
    return [
        MappingRecord(
            python_file=d["python_file"],
            python_function=d["python_function"],
            python_class=d.get("python_class"),
            status=MappingStatus(d.get("status", "missing")),
            rust_file=d.get("rust_file"),
            rust_function=d.get("rust_function"),
            rust_functions=d.get("rust_functions", []),
            notes=d.get("notes"),
        )
        for d in data
    ]
