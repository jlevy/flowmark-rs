"""
Discover all test functions in Rust test files using regex parsing.

Walks `test_*.rs` files and finds `#[test]` annotated `fn` declarations.
"""

from __future__ import annotations

import re
from pathlib import Path

from flowmark_dev_tools.models import RustTestRecord

# Match `#[test]` or `#[tokio::test]` followed eventually by `fn name(`.
# Allows intervening attributes, comments, and whitespace.
_TEST_ATTR_PATTERN = re.compile(r"#\[(?:tokio::)?test\]")
_FN_PATTERN = re.compile(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\(", re.MULTILINE)


def discover_rust_tests(tests_dir: Path) -> list[RustTestRecord]:
    """
    Walk all `test_*.rs` files under `tests_dir` and extract test records.
    Returns a sorted list of `RustTestRecord`.
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
                    records.append(RustTestRecord(
                        file=relative_path,
                        function=fn_match.group(1),
                        line_number=i + 1,  # 1-indexed
                    ))
                    saw_test_attr = False
                elif line.strip() and not line.strip().startswith(("//", "#[")):
                    # Non-empty, non-comment, non-attribute line without fn — reset.
                    saw_test_attr = False

    records.sort(key=lambda r: (r.file, r.function))
    return records
