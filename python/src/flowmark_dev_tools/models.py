"""Data models for test discovery and mapping."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import StrEnum


class TestType(StrEnum):
    """Classification of a test by its purpose."""

    unit = "unit"
    integration = "integration"
    golden = "golden"
    infrastructure = "infrastructure"


class MappingStatus(StrEnum):
    """Status of a Python-to-Rust test mapping."""

    mapped = "mapped"
    excluded = "excluded"
    partial = "partial"
    missing = "missing"


@dataclass(frozen=True)
class PythonTestRecord:
    """A single test function discovered in the Python flowmark repo."""

    file: str
    function: str
    class_name: str | None
    test_type: TestType
    line_number: int
    doc_string: str | None


@dataclass(frozen=True)
class RustTestRecord:
    """A single test function discovered in the Rust flowmark-rs repo."""

    file: str
    function: str
    line_number: int


@dataclass
class MappingRecord:
    """
    A mapping entry linking a Python test to its Rust counterpart(s).
    The `status` field tracks whether the mapping has been verified.
    """

    python_file: str
    python_function: str
    python_class: str | None = None
    status: MappingStatus = MappingStatus.missing
    rust_file: str | None = None
    rust_function: str | None = None
    rust_functions: list[str] = field(default_factory=list)
    notes: str | None = None
