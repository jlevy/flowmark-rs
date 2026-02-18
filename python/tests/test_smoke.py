"""
Smoke tests for flowmark-dev-tools.

These validate the tooling works end-to-end:
- YAML serialization round-trips correctly
- Discovery scripts find expected test counts
- check-mapping reports consistent results
"""

from __future__ import annotations

from pathlib import Path

from flowmark_dev_tools.check_mapping import run_check
from flowmark_dev_tools.models import MappingStatus
from flowmark_dev_tools.yaml_io import (
    load_mapping_yaml,
    load_python_tests_yaml,
    load_rust_tests_yaml,
    write_mapping_yaml,
    write_python_tests_yaml,
    write_rust_tests_yaml,
)

# All paths relative to the repo root (flowmark-rs/).
REPO_ROOT = Path(__file__).parent.parent.parent
MAPPING_DIR = REPO_ROOT / "port-coverage-mapping"
PYTHON_YAML = MAPPING_DIR / "python-tests.yaml"
RUST_YAML = MAPPING_DIR / "rust-tests.yaml"
MAPPING_YAML = MAPPING_DIR / "test-mapping.yaml"


class TestYamlRoundTrip:
    """Verify YAML files load and re-serialize without data loss."""

    def test_python_tests_round_trip(self, tmp_path: Path) -> None:
        records = load_python_tests_yaml(PYTHON_YAML)
        assert len(records) > 0, "Expected Python test records"
        out = tmp_path / "python-tests.yaml"
        write_python_tests_yaml(records, out)
        reloaded = load_python_tests_yaml(out)
        assert len(reloaded) == len(records)
        for orig, new in zip(records, reloaded, strict=True):
            assert orig.file == new.file
            assert orig.function == new.function
            assert orig.class_name == new.class_name

    def test_rust_tests_round_trip(self, tmp_path: Path) -> None:
        records = load_rust_tests_yaml(RUST_YAML)
        assert len(records) > 0, "Expected Rust test records"
        out = tmp_path / "rust-tests.yaml"
        write_rust_tests_yaml(records, out)
        reloaded = load_rust_tests_yaml(out)
        assert len(reloaded) == len(records)
        for orig, new in zip(records, reloaded, strict=True):
            assert orig.file == new.file
            assert orig.function == new.function

    def test_mapping_round_trip(self, tmp_path: Path) -> None:
        records = load_mapping_yaml(MAPPING_YAML)
        assert len(records) > 0, "Expected mapping records"
        out = tmp_path / "test-mapping.yaml"
        write_mapping_yaml(records, out)
        reloaded = load_mapping_yaml(out)
        assert len(reloaded) == len(records)
        for orig, new in zip(records, reloaded, strict=True):
            assert orig.python_file == new.python_file
            assert orig.python_function == new.python_function
            assert orig.status == new.status


class TestYamlDeterminism:
    """Verify YAML output is byte-for-byte stable across writes."""

    def test_python_tests_stable_output(self, tmp_path: Path) -> None:
        records = load_python_tests_yaml(PYTHON_YAML)
        out1 = tmp_path / "a.yaml"
        out2 = tmp_path / "b.yaml"
        write_python_tests_yaml(records, out1)
        write_python_tests_yaml(records, out2)
        assert out1.read_text() == out2.read_text(), "Python YAML output is not stable"

    def test_rust_tests_stable_output(self, tmp_path: Path) -> None:
        records = load_rust_tests_yaml(RUST_YAML)
        out1 = tmp_path / "a.yaml"
        out2 = tmp_path / "b.yaml"
        write_rust_tests_yaml(records, out1)
        write_rust_tests_yaml(records, out2)
        assert out1.read_text() == out2.read_text(), "Rust YAML output is not stable"

    def test_mapping_stable_output(self, tmp_path: Path) -> None:
        records = load_mapping_yaml(MAPPING_YAML)
        out1 = tmp_path / "a.yaml"
        out2 = tmp_path / "b.yaml"
        write_mapping_yaml(records, out1)
        write_mapping_yaml(records, out2)
        assert out1.read_text() == out2.read_text(), "Mapping YAML output is not stable"

    def test_checked_in_files_match_canonical_output(self, tmp_path: Path) -> None:
        """Re-serializing checked-in YAML must reproduce the same bytes.

        If this fails, run the discover/init commands to regenerate.
        """
        for load_fn, write_fn, src in [
            (load_python_tests_yaml, write_python_tests_yaml, PYTHON_YAML),
            (load_rust_tests_yaml, write_rust_tests_yaml, RUST_YAML),
            (load_mapping_yaml, write_mapping_yaml, MAPPING_YAML),
        ]:
            records = load_fn(src)
            out = tmp_path / src.name
            write_fn(records, out)
            expected = src.read_text()
            actual = out.read_text()
            assert actual == expected, (
                f"{src.name} is not in canonical form. "
                f"Re-run the appropriate flowmark-dev command to regenerate."
            )


class TestDiscoveryCounts:
    """Verify discovery produces expected counts from checked-in YAML."""

    def test_python_test_count(self) -> None:
        records = load_python_tests_yaml(PYTHON_YAML)
        assert len(records) == 281, f"Expected 281 Python tests, got {len(records)}"

    def test_rust_test_count(self) -> None:
        records = load_rust_tests_yaml(RUST_YAML)
        assert len(records) == 250, f"Expected 250 Rust tests, got {len(records)}"

    def test_mapping_count(self) -> None:
        records = load_mapping_yaml(MAPPING_YAML)
        assert len(records) == 281, f"Expected 281 mapping entries, got {len(records)}"


class TestMappingCompleteness:
    """Track progress toward complete mapping — the TDD driving test."""

    def test_no_unmapped_entries(self) -> None:
        """Every Python test must have a mapping entry (mapped, excluded, or partial).

        This is the primary TDD target: it fails until all 64 missing tests are ported.
        """
        records = load_mapping_yaml(MAPPING_YAML)
        missing = [r for r in records if r.status == MappingStatus.missing]
        assert len(missing) == 0, (
            f"{len(missing)} Python tests still have status 'missing' in test-mapping.yaml. "
            f"Port these tests or mark them excluded:\n"
            + "\n".join(f"  - {r.python_file}::{r.python_function}" for r in missing[:10])
            + (f"\n  ... and {len(missing) - 10} more" if len(missing) > 10 else "")
        )

    def test_all_mapped_rust_refs_exist(self) -> None:
        """Every mapped entry must reference a Rust test that actually exists."""
        mapping = load_mapping_yaml(MAPPING_YAML)
        rust_tests = load_rust_tests_yaml(RUST_YAML)
        rust_funcs = {(r.file, r.function) for r in rust_tests}

        broken_refs: list[str] = []
        for m in mapping:
            if m.status != MappingStatus.mapped:
                continue
            if m.rust_function:
                if (m.rust_file or "", m.rust_function) not in rust_funcs:
                    broken_refs.append(
                        f"{m.python_file}::{m.python_function} -> "
                        f"{m.rust_file}::{m.rust_function}"
                    )
            for fn in m.rust_functions:
                if (m.rust_file or "", fn) not in rust_funcs:
                    broken_refs.append(
                        f"{m.python_file}::{m.python_function} -> "
                        f"{m.rust_file}::{fn}"
                    )

        assert len(broken_refs) == 0, (
            f"{len(broken_refs)} mapped entries reference non-existent Rust tests:\n"
            + "\n".join(f"  - {r}" for r in broken_refs[:10])
        )

    def test_check_mapping_passes(self) -> None:
        """The check-mapping command exits 0 when mapping is complete."""
        exit_code = run_check(PYTHON_YAML, RUST_YAML, MAPPING_YAML)
        assert exit_code == 0, "check-mapping reports incomplete mapping"
