#!/usr/bin/env -S uv run --script --python 3.14
# /// script
# requires-python = ">=3.14"
# dependencies = [
#   "jinja2>=3.1.6",
#   "strif>=3.0.1",
# ]
# ///
"""Generate the Rust README as a rendered superset of Python's canonical README."""

from __future__ import annotations

from argparse import ArgumentParser
from pathlib import Path
import re

from jinja2 import Environment, StrictUndefined
from strif import atomic_output_file

PYTHON_DOCS_BASE_URL = "https://github.com/jlevy/flowmark/blob/main/docs/"


def parse_args() -> tuple[Path, Path, Path]:
    """Parse command-line arguments and resolve default repo-relative paths."""
    repo_root = Path(__file__).resolve().parents[1]
    parser = ArgumentParser(description="Generate README.md from Python README + Rust wrapper template.")
    parser.add_argument(
        "--python-readme",
        type=Path,
        default=repo_root / "repos/flowmark/README.md",
        help="Path to canonical Python README source.",
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
    return args.python_readme, args.template, args.output


def strip_first_h1(markdown: str) -> str:
    """Drop the first top-level title from source README while preserving content order."""
    lines = markdown.splitlines()
    if lines and lines[0].startswith("# "):
        lines = lines[1:]
    body = "\n".join(lines).lstrip("\n").rstrip()
    return f"{body}\n"


def rewrite_python_local_docs_links(markdown: str) -> str:
    """Rewrite Python-local docs links to canonical upstream URLs."""
    return re.sub(
        r"\]\(docs/([^)]+)\)",
        rf"]({PYTHON_DOCS_BASE_URL}\1)",
        markdown,
    )


def render_readme(template_path: Path, python_readme_body: str) -> str:
    """Render the README wrapper template with transformed Python README content."""
    environment = Environment(
        autoescape=False,
        undefined=StrictUndefined,
        keep_trailing_newline=True,
    )
    template = environment.from_string(template_path.read_text(encoding="utf-8"))
    rendered = template.render(python_readme_body=python_readme_body)
    if not rendered.endswith("\n"):
        rendered += "\n"
    return rendered


def write_atomic(output_path: Path, content: str) -> None:
    """Write output atomically so partial output files are never created."""
    with atomic_output_file(output_path, make_parents=True) as temp_path:
        Path(temp_path).write_text(content, encoding="utf-8")


def main() -> int:
    """Generate README.md from canonical sources."""
    python_readme_path, template_path, output_path = parse_args()
    if not python_readme_path.exists():
        raise FileNotFoundError(f"missing Python README at {python_readme_path}")
    if not template_path.exists():
        raise FileNotFoundError(f"missing wrapper template at {template_path}")

    python_readme = python_readme_path.read_text(encoding="utf-8")
    python_body = strip_first_h1(python_readme)
    python_body = rewrite_python_local_docs_links(python_body)
    rendered = render_readme(template_path, python_body)
    write_atomic(output_path, rendered)

    print(f"Generated {output_path} from {python_readme_path} via {template_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
