"""
Discover all test functions in the Python flowmark repo using AST parsing.

Walks `test_*.py` files, extracts `test_*` functions (including those inside classes),
and classifies each test by type.
"""

from __future__ import annotations

import ast
from pathlib import Path

from flowmark_dev_tools.models import PythonTestRecord, TestType

# Files that test CLI, config, file resolution, and skill infrastructure.
INFRASTRUCTURE_FILES = frozenset(
    {
        "test_cli_file_discovery.py",
        "test_config.py",
        "test_file_resolver.py",
        "test_skill.py",
    }
)

# Files that test golden/reference document output.
GOLDEN_FILES = frozenset(
    {
        "test_ref_docs.py",
    }
)

# Functions that indicate an integration test when called.
INTEGRATION_INDICATORS = frozenset(
    {
        "fill_markdown",
        "fill_lines",
        "fill_string",
    }
)


def _classify_test_type(file_name: str, func_node: ast.FunctionDef) -> TestType:
    """
    Classify a test based on its file name and the functions it calls.
    Infrastructure and golden files are classified by filename; otherwise
    we look for calls to pipeline-level functions like `fill_markdown`.
    """
    if file_name in INFRASTRUCTURE_FILES:
        return TestType.infrastructure
    if file_name in GOLDEN_FILES:
        return TestType.golden

    # Walk the function body looking for calls to integration-level functions.
    for node in ast.walk(func_node):
        if isinstance(node, ast.Call):
            callee = node.func
            name: str | None = None
            if isinstance(callee, ast.Name):
                name = callee.id
            elif isinstance(callee, ast.Attribute):
                name = callee.attr
            if name and name in INTEGRATION_INDICATORS:
                return TestType.integration

    return TestType.unit


def _extract_docstring(node: ast.FunctionDef) -> str | None:
    """Extract the first line of a function's docstring, if present."""
    docstring = ast.get_docstring(node)
    if docstring:
        return docstring.strip().split("\n")[0]
    return None


def discover_python_tests(repo_path: Path) -> list[PythonTestRecord]:
    """
    Walk all `test_*.py` files under `repo_path/tests/` and extract test records.
    Returns a sorted list of `PythonTestRecord`.
    """
    tests_dir = repo_path / "tests"
    if not tests_dir.is_dir():
        raise FileNotFoundError(f"Tests directory not found: {tests_dir}")

    records: list[PythonTestRecord] = []

    for test_file in sorted(tests_dir.glob("test_*.py")):
        source = test_file.read_text(encoding="utf-8")
        tree = ast.parse(source, filename=str(test_file))
        file_name = test_file.name
        relative_path = f"tests/{file_name}"

        for node in ast.iter_child_nodes(tree):
            # Top-level test functions.
            if isinstance(node, ast.FunctionDef) and node.name.startswith("test_"):
                records.append(
                    PythonTestRecord(
                        file=relative_path,
                        function=node.name,
                        class_name=None,
                        test_type=_classify_test_type(file_name, node),
                        line_number=node.lineno,
                        doc_string=_extract_docstring(node),
                    )
                )

            # Test methods inside classes.
            if isinstance(node, ast.ClassDef):
                for method in ast.iter_child_nodes(node):
                    if isinstance(method, ast.FunctionDef) and method.name.startswith("test_"):
                        records.append(
                            PythonTestRecord(
                                file=relative_path,
                                function=method.name,
                                class_name=node.name,
                                test_type=_classify_test_type(file_name, method),
                                line_number=method.lineno,
                                doc_string=_extract_docstring(method),
                            )
                        )

    # Sort by file, class, function for deterministic output.
    records.sort(key=lambda r: (r.file, r.class_name or "", r.function))
    return records
