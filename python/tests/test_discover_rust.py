# pyright: reportPrivateUsage=false

from pathlib import Path

from flowmark_dev_tools.discover_rust import _find_integration_test_file, _find_line_number


def test_macro_generated_integration_test_has_a_source_location(tmp_path: Path) -> None:
    test_file = tmp_path / "tests" / "test_generated.rs"
    test_file.parent.mkdir()
    test_file.write_text('shared_test!(generated_case, "case.toml");\n')

    assert _find_integration_test_file(tmp_path, "generated_case") == ("tests/test_generated.rs")
    assert _find_line_number(tmp_path, "tests/test_generated.rs", "generated_case") == 1
