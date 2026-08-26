from pathlib import Path

from flowmark_dev_tools.discover_python import discover_python_tests
from flowmark_dev_tools.models import TestType as FlowmarkTestType


def test_discovers_python_functions_and_language_neutral_tryscript(tmp_path: Path) -> None:
    tests_dir = tmp_path / "tests"
    tests_dir.mkdir()
    (tests_dir / "test_sample.py").write_text(
        "def test_top_level():\n    pass\n\n"
        "class TestGroup:\n    def test_method(self):\n        pass\n",
        encoding="utf-8",
    )
    tryscript_dir = tests_dir / "tryscript"
    tryscript_dir.mkdir()
    (tryscript_dir / "cli-golden.tryscript.md").write_text(
        "# CLI golden suite\n",
        encoding="utf-8",
    )

    records = discover_python_tests(tmp_path)

    assert [(record.file, record.class_name, record.function) for record in records] == [
        ("tests/test_sample.py", None, "test_top_level"),
        ("tests/test_sample.py", "TestGroup", "test_method"),
        ("tests/tryscript/cli-golden.tryscript.md", None, "cli-golden"),
    ]
    assert records[-1].test_type == FlowmarkTestType.golden
    assert records[-1].line_number == 0
