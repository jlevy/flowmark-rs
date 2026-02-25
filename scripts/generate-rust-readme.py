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

from argparse import ArgumentParser
from pathlib import Path
import re
import tomllib

from jinja2 import Environment, StrictUndefined
from strif import atomic_output_file

UPSTREAM_DOCS_BASE_URL = "https://github.com/jlevy/flowmark/blob/main/docs/"


def parse_args() -> tuple[Path, Path, Path]:
    """Parse command-line arguments and resolve default repo-relative paths."""
    repo_root = Path(__file__).resolve().parents[1]
    parser = ArgumentParser(description="Generate README.md from shared docs + Rust wrapper template.")
    parser.add_argument(
        "--shared-docs",
        "--python-readme",
        dest="shared_docs",
        type=Path,
        default=repo_root / "repos/flowmark/README.md",
        help="Path to canonical shared docs source (currently Python README).",
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


def strip_first_h1(markdown: str) -> str:
    """Drop the first top-level title from source README while preserving content order."""
    lines = markdown.splitlines()
    if lines and lines[0].startswith("# "):
        lines = lines[1:]
    body = "\n".join(lines).lstrip("\n").rstrip()
    return f"{body}\n"


def rewrite_upstream_local_docs_links(markdown: str) -> str:
    """Rewrite local docs links from the shared source to canonical upstream URLs."""
    return re.sub(
        r"\]\(docs/([^)]+)\)",
        rf"]({UPSTREAM_DOCS_BASE_URL}\1)",
        markdown,
    )


def strip_leading_badges(markdown: str) -> str:
    """Drop a leading badge block from the shared docs body for Rust wrappers."""
    badge_line = re.compile(r"^\[!\[[^]]+\]\([^)]+\)\]\([^)]+\)$")
    lines = markdown.splitlines()

    index = 0
    while index < len(lines) and not lines[index].strip():
        index += 1

    start = index
    while index < len(lines) and badge_line.match(lines[index].strip()):
        index += 1

    if index == start:
        return f"{markdown.rstrip()}\n"

    while index < len(lines) and not lines[index].strip():
        index += 1
    return f"{'\n'.join(lines[index:]).rstrip()}\n"


def drop_section(markdown: str, heading: str) -> str:
    """Drop the first level-2 heading section with the provided title."""
    pattern = re.compile(rf"^## {re.escape(heading)}\n.*?(?=^## |\Z)", re.MULTILINE | re.DOTALL)
    return re.sub(pattern, "", markdown, count=1)


def normalize_shared_docs_for_rust(markdown: str) -> str:
    """Apply Rust-specific cleanup to shared docs content."""
    # Rust README has its own installation section above the shared docs content.
    normalized = drop_section(markdown, "Installation")
    # Replace Python runner-specific command examples with neutral CLI commands.
    normalized = re.sub(r"\buvx\s+flowmark@latest\b", "flowmark", normalized)
    normalized = re.sub(r"\buvx\s+flowmark\b", "flowmark", normalized)
    # Remove Python runtime installation guidance from shared docs in Rust.
    normalized = re.sub(
        r"^For how to install uv and Python, see \[installation\.md\]\([^)]+\)\.\n\n",
        "",
        normalized,
        flags=re.MULTILINE,
    )
    return f"{normalized.rstrip()}\n"


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


def render_readme(template_path: Path, shared_docs_body: str, msrv: str) -> str:
    """Render the README wrapper template with transformed shared docs content."""
    environment = Environment(
        autoescape=False,
        undefined=StrictUndefined,
        keep_trailing_newline=True,
    )
    template = environment.from_string(template_path.read_text(encoding="utf-8"))
    rendered = template.render(shared_docs_body=shared_docs_body, msrv=msrv)
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

    shared_docs = shared_docs_path.read_text(encoding="utf-8")
    shared_docs_body = strip_first_h1(shared_docs)
    shared_docs_body = strip_leading_badges(shared_docs_body)
    shared_docs_body = rewrite_upstream_local_docs_links(shared_docs_body)
    shared_docs_body = normalize_shared_docs_for_rust(shared_docs_body)
    rendered = render_readme(template_path, shared_docs_body, read_msrv(repo_root))
    write_atomic(output_path, rendered)

    print(f"Generated {output_path} from {shared_docs_path} via {template_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
