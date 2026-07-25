#!/usr/bin/env python3
"""Reject Mermaid node identifiers that conflict with diagram syntax."""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import sys

MERMAID_FENCE_RE = re.compile(
    r"^```mermaid[ \t]*\n(?P<body>.*?)^```[ \t]*$",
    re.MULTILINE | re.DOTALL,
)
NODE_IDENTIFIER_RE = re.compile(
    r"^\s*(?P<name>[A-Za-z_][A-Za-z0-9_-]*)\s*(?=[\[\(\{])",
    re.MULTILINE,
)
RESERVED_IDENTIFIERS = frozenset(
    {
        "class",
        "classdef",
        "click",
        "direction",
        "end",
        "flowchart",
        "graph",
        "linkstyle",
        "style",
        "subgraph",
    }
)


def validate_markdown(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8")
    issues: list[str] = []
    for diagram_number, fence in enumerate(MERMAID_FENCE_RE.finditer(text), start=1):
        body = fence.group("body")
        body_offset = fence.start("body")
        for node in NODE_IDENTIFIER_RE.finditer(body):
            identifier = node.group("name")
            if identifier.lower() not in RESERVED_IDENTIFIERS:
                continue
            line_number = text.count("\n", 0, body_offset + node.start()) + 1
            issues.append(
                f"{path}:{line_number}: Mermaid diagram {diagram_number} uses "
                f"reserved node identifier {identifier!r}"
            )
    return issues


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("docs_dir", nargs="?", type=Path, default=Path("docs"))
    args = parser.parse_args()

    issues: list[str] = []
    for path in sorted(args.docs_dir.rglob("*.md")):
        issues.extend(validate_markdown(path))

    if issues:
        for issue in issues:
            print(issue, file=sys.stderr)
        return 1

    print(f"Mermaid source sanity OK for {args.docs_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
