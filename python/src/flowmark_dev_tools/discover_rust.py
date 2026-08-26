"""
Discover all test functions in the Rust flowmark-rs project.

Two strategies:
1. **cargo-based** (preferred): Runs `cargo test -- --list --format terse` which is
   compiler-authoritative and finds both integration tests (`tests/`) and unit tests
   (`src/` modules).
2. **regex-based** (fallback): Walks `test_*.rs` files and finds `#[test]` annotated
   `fn` declarations. Only finds integration tests in the `tests/` directory.
"""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

from flowmark_dev_tools.models import RustTestRecord

# Match `#[test]` or `#[tokio::test]` followed eventually by `fn name(`.
_TEST_ATTR_PATTERN = re.compile(r"#\[(?:tokio::)?test\]")
_FN_PATTERN = re.compile(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\(", re.MULTILINE)


def discover_rust_tests_cargo(project_dir: Path) -> list[RustTestRecord]:
    """
    Use `cargo test -- --list` to discover all tests. This is the most authoritative
    approach since the Rust compiler determines what's a test. Finds both integration
    tests in `tests/` and unit tests in `src/` modules.
    """
    result = subprocess.run(
        ["cargo", "test", "--", "--list", "--format", "terse"],
        capture_output=True,
        text=True,
        cwd=project_dir,
    )
    if result.returncode != 0:
        raise RuntimeError(f"cargo test --list failed (exit {result.returncode}):\n{result.stderr}")

    records: list[RustTestRecord] = []
    for line in result.stdout.splitlines():
        line = line.strip()
        if not line.endswith(": test"):
            continue
        test_path = line[: -len(": test")].strip()

        # cargo outputs two forms:
        #   Integration tests: "test_function_name: test"
        #   Unit tests: "module::submodule::tests::test_name: test"
        if "::" in test_path:
            # Unit test in src/ — convert module path to a file reference.
            parts = test_path.split("::")
            function = parts[-1]
            # Strip the "tests" module suffix if present (convention for #[cfg(test)] mod tests).
            module_parts = [p for p in parts[:-1] if p != "tests"]
            file = "src/" + "/".join(module_parts) + ".rs"
        else:
            # Integration test — need to find which file it's in.
            function = test_path
            file = _find_integration_test_file(project_dir, function)

        records.append(
            RustTestRecord(
                file=file,
                function=function,
                line_number=_find_line_number(project_dir, file, function),
            )
        )

    records.sort(key=lambda r: (r.file, r.function))
    return records


def _find_integration_test_file(project_dir: Path, function_name: str) -> str:
    """
    Find which `tests/test_*.rs` file contains a given integration test function.
    Falls back to "tests/unknown.rs" if not found.
    """
    tests_dir = project_dir / "tests"
    if not tests_dir.is_dir():
        return "tests/unknown.rs"

    for test_file in tests_dir.glob("test_*.rs"):
        content = test_file.read_text(encoding="utf-8")
        # Look for `fn function_name(` in the file.
        if re.search(rf"\bfn\s+{re.escape(function_name)}\s*\(", content):
            return f"tests/{test_file.name}"

    # Declarative macros commonly generate repetitive integration tests. Cargo lists
    # the expanded function, while the source contains it as the macro's first argument.
    macro_invocation = re.compile(rf"!\s*\(\s*{re.escape(function_name)}\s*,")
    for test_file in tests_dir.glob("test_*.rs"):
        if macro_invocation.search(test_file.read_text(encoding="utf-8")):
            return f"tests/{test_file.name}"

    return "tests/unknown.rs"


def _find_line_number(project_dir: Path, file_path: str, function_name: str) -> int:
    """Find the line number of a test function in a file. Returns 0 if not found."""
    full_path = project_dir / file_path
    if not full_path.exists():
        return 0

    for i, line in enumerate(full_path.read_text(encoding="utf-8").splitlines()):
        if re.search(rf"\bfn\s+{re.escape(function_name)}\s*\(", line):
            return i + 1
        if re.search(rf"!\s*\(\s*{re.escape(function_name)}\s*,", line):
            return i + 1

    return 0


def discover_rust_tests_regex(tests_dir: Path) -> list[RustTestRecord]:
    """
    Fallback: walk `test_*.rs` files and find `#[test]` annotated functions using
    regex. Only finds integration tests in the `tests/` directory; misses unit tests
    in `src/`.
    """
    if not tests_dir.is_dir():
        raise FileNotFoundError(f"Tests directory not found: {tests_dir}")

    records: list[RustTestRecord] = []

    for test_file in sorted(tests_dir.glob("test_*.rs")):
        lines = test_file.read_text(encoding="utf-8").splitlines()
        relative_path = f"tests/{test_file.name}"

        # Track when we see a #[test] attribute and look for the next fn.
        saw_test_attr = False

        for i, line in enumerate(lines):
            if _TEST_ATTR_PATTERN.search(line):
                saw_test_attr = True
                continue

            if saw_test_attr:
                fn_match = _FN_PATTERN.match(line)
                if fn_match:
                    records.append(
                        RustTestRecord(
                            file=relative_path,
                            function=fn_match.group(1),
                            line_number=i + 1,
                        )
                    )
                    saw_test_attr = False
                elif line.strip() and not line.strip().startswith(("//", "#[")):
                    # Non-empty, non-comment, non-attribute line without fn — reset.
                    saw_test_attr = False

    records.sort(key=lambda r: (r.file, r.function))
    return records
