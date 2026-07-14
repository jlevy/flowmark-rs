#!/usr/bin/env -S uv run --script --python 3.14
# /// script
# requires-python = ">=3.14"
# dependencies = [
#   "jinja2>=3.1.6",
#   "strif>=3.0.1",
# ]
# ///
"""Generate the Rust README from shared docs content and a Rust wrapper template."""

from __future__ import annotations

import re
import tomllib
from argparse import ArgumentParser
from pathlib import Path

from jinja2 import Environment, StrictUndefined
from strif import atomic_output_file

UPSTREAM_DOCS_BASE_URL = "https://github.com/jlevy/flowmark/blob/main/docs/"


def parse_args() -> tuple[Path, Path, Path]:
    """Parse command-line arguments and resolve default repo-relative paths."""
    repo_root = Path(__file__).resolve().parents[1]
    parser = ArgumentParser(
        description="Generate README.md from shared docs + Rust wrapper template."
    )
    parser.add_argument(
        "--shared-docs",
        "--python-readme",
        dest="shared_docs",
        type=Path,
        default=repo_root / "repos/flowmark/docs/shared/flowmark-readme-shared.md",
        help="Path to canonical shared docs source.",
    )
    parser.add_argument(
        "--template",
        type=Path,
        default=repo_root / "docs/templates/rust-readme-wrapper.md",
        help="Path to Markdown wrapper template rendered with Jinja.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=repo_root / "README.md",
        help="Output README path.",
    )
    args = parser.parse_args()
    return args.shared_docs, args.template, args.output


def rewrite_upstream_local_docs_links(markdown: str) -> str:
    """Rewrite local docs links from the shared source to canonical upstream URLs."""
    # Keep the Rust repo's development workflow link local.
    return re.sub(
        r"\]\(docs/(?!development\.md)([^)]+)\)",
        rf"]({UPSTREAM_DOCS_BASE_URL}\1)",
        markdown,
    )


# Runner-pin placeholders in the shared docs. Substitute the Python package from the
# parity baseline and the Rust package from this crate's version.
VERSION_PLACEHOLDER = "__FLOWMARK_VERSION__"
RS_VERSION_PLACEHOLDER = "__FLOWMARK_RS_VERSION__"

PERSPECTIVE_FLIP_OLD = (
    "Flowmark comes in two flavors: this Python reference implementation and an "
    "auto-synced\n[Rust port (flowmark-rs)](https://github.com/jlevy/flowmark-rs)."
)
PERSPECTIVE_FLIP_NEW = (
    "Flowmark comes in two flavors: the "
    "[Python reference implementation](https://github.com/jlevy/flowmark) and this "
    "auto-synced Rust port (flowmark-rs)."
)

# As of upstream flowmark v0.7.2 the shared README intro was rewritten to neutral,
# third-person phrasing that names both implementations directly ("written in Python
# with an auto-synced Rust port"), so there is no Python-first-person sentence left to
# flip. This anchor confirms we are looking at that (or a newer) neutral intro, so the
# no-flip path stays guarded against genuine drift rather than silently passing anything.
NEUTRAL_INTRO_ANCHOR = (
    "Flowmark is a Markdown auto-formatter, written\n"
    "[in Python](https://github.com/jlevy/flowmark) with an auto-synced\n"
    "[Rust port](https://github.com/jlevy/flowmark-rs)"
)


def rewrite_perspective_for_rust_repo(markdown: str) -> str:
    """Flip any Python-perspective phrasing in the shared source to Rust-perspective.

    Older shared docs referred to themselves as "this Python reference implementation";
    in the Rust repo's README "this" refers to the Rust port, so that sentence was
    rephrased. Newer (v0.7.2+) shared docs are already perspective-neutral and need no
    flip. Raises if neither the old flippable sentence nor the new neutral intro is
    present — better to fail loudly than silently publish the wrong perspective.
    """
    if PERSPECTIVE_FLIP_OLD in markdown:
        return markdown.replace(PERSPECTIVE_FLIP_OLD, PERSPECTIVE_FLIP_NEW)
    if NEUTRAL_INTRO_ANCHOR in markdown:
        # Already perspective-neutral; nothing to rewrite.
        return markdown
    raise ValueError(
        "shared docs match neither the legacy Python-perspective sentence nor the "
        "neutral intro anchor; update PERSPECTIVE_FLIP_OLD / NEUTRAL_INTRO_ANCHOR in "
        "scripts/generate_rust_readme.py"
    )


def read_msrv(repo_root: Path) -> str:
    """Read rust-version from Cargo.toml for the MSRV badge."""
    cargo_toml = repo_root / "Cargo.toml"
    if not cargo_toml.exists():
        raise FileNotFoundError(f"missing Cargo.toml at {cargo_toml}")
    metadata = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    package = metadata.get("package", {})
    msrv = package.get("rust-version")
    if not isinstance(msrv, str) or not msrv:
        raise ValueError(f"missing [package].rust-version in {cargo_toml}")
    return msrv


def read_parity_version(repo_root: Path) -> str:
    """Read Python parity version from Cargo.toml metadata."""
    cargo_toml = repo_root / "Cargo.toml"
    if not cargo_toml.exists():
        raise FileNotFoundError(f"missing Cargo.toml at {cargo_toml}")
    metadata = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    package = metadata.get("package", {})
    parity = package.get("metadata", {}).get("parity", {}).get("version")
    if not isinstance(parity, str) or not parity:
        raise ValueError(f"missing [package.metadata.parity].version in {cargo_toml}")
    return parity


def read_package_version(repo_root: Path) -> str:
    """Read this Rust package's version from Cargo.toml."""
    cargo_toml = repo_root / "Cargo.toml"
    if not cargo_toml.exists():
        raise FileNotFoundError(f"missing Cargo.toml at {cargo_toml}")
    metadata = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    version = metadata.get("package", {}).get("version")
    if not isinstance(version, str) or not version:
        raise ValueError(f"missing [package].version in {cargo_toml}")
    return version


def read_last_sync_date(repo_root: Path) -> str:
    """Read the port last-updated date from docs/port-status.md."""
    port_status = repo_root / "docs/port-status.md"
    if not port_status.exists():
        raise FileNotFoundError(f"missing port status file at {port_status}")
    text = port_status.read_text(encoding="utf-8")
    match = re.search(r"^\*\*Last updated:\*\*\s+([0-9]{4}-[0-9]{2}-[0-9]{2})$", text, re.MULTILINE)
    if not match:
        raise ValueError(f"missing '**Last updated:** YYYY-MM-DD' in {port_status}")
    return match.group(1)


def render_readme(
    template_path: Path,
    shared_docs_body: str,
    msrv: str,
    parity_version: str,
    rust_version: str,
    last_sync_date: str,
) -> str:
    """Render the README wrapper template with transformed shared docs content."""
    environment = Environment(
        autoescape=False,
        undefined=StrictUndefined,
        keep_trailing_newline=True,
    )
    template = environment.from_string(template_path.read_text(encoding="utf-8"))
    rendered = template.render(
        shared_docs_body=shared_docs_body,
        msrv=msrv,
        parity_version=parity_version,
        rust_version=rust_version,
        last_sync_date=last_sync_date,
    )
    if not rendered.endswith("\n"):
        rendered += "\n"
    return rendered


def write_atomic(output_path: Path, content: str) -> None:
    """Write output atomically so partial output files are never created."""
    with atomic_output_file(output_path, make_parents=True) as temp_path:
        Path(temp_path).write_text(content, encoding="utf-8")


def main() -> int:
    """Generate README.md from canonical sources."""
    shared_docs_path, template_path, output_path = parse_args()
    repo_root = Path(__file__).resolve().parents[1]
    if not shared_docs_path.exists():
        raise FileNotFoundError(f"missing shared docs source at {shared_docs_path}")
    if not template_path.exists():
        raise FileNotFoundError(f"missing wrapper template at {template_path}")

    parity_version = read_parity_version(repo_root)
    shared_docs_body = shared_docs_path.read_text(encoding="utf-8").rstrip() + "\n"
    shared_docs_body = rewrite_upstream_local_docs_links(shared_docs_body)
    shared_docs_body = rewrite_perspective_for_rust_repo(shared_docs_body)
    # Keep the shared runner examples aligned with the Python parity baseline and this
    # crate's release. Fail loudly if the shared source adds an unhandled placeholder.
    rust_version = read_package_version(repo_root)
    shared_docs_body = shared_docs_body.replace(VERSION_PLACEHOLDER, parity_version)
    shared_docs_body = shared_docs_body.replace(RS_VERSION_PLACEHOLDER, rust_version)
    for placeholder in (VERSION_PLACEHOLDER, RS_VERSION_PLACEHOLDER):
        if placeholder in shared_docs_body:
            raise ValueError(f"{placeholder} still present after substitution")
    rendered = render_readme(
        template_path,
        shared_docs_body,
        read_msrv(repo_root),
        parity_version,
        rust_version,
        read_last_sync_date(repo_root),
    )
    write_atomic(output_path, rendered)

    print(f"Generated {output_path} from {shared_docs_path} via {template_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
