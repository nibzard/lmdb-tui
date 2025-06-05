#!/usr/bin/env python3
"""Generate docs/llms.txt from Markdown sources."""
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs"

pages = [p for p in DOCS.glob("*.md") if p.name != "llms.txt"]

lines = [
    "# lmdb-tui",
    "",
    "> Quick links for the lmdb-tui project.",
    "",
    "## Documentation",
]

for p in sorted(pages):
    lines.append(f"- [{p.stem}]({p.name})")

lines += [
    "",
    "## Repository",
    f"- [README](../README.md)",
    f"- [SPECS](../SPECS.md)",
    f"- [CONTRIBUTING](../CONTRIBUTING.md)",
    "",
    "## Optional",
    f"- [Todo](../Todo.md)",
]

(DOCS / "llms.txt").write_text("\n".join(lines) + "\n")
print("llms.txt generated")
