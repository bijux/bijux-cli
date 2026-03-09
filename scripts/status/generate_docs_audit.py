#!/usr/bin/env python3
"""Generate documentation inventory and overlap audit."""

from __future__ import annotations

import json
import re
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "artifacts" / "status" / "docs_audit.json"


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT)).replace("\\", "/")


def list_markdown() -> list[Path]:
    return sorted(p for p in ROOT.rglob("*.md") if ".git" not in p.parts)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="ignore")


def heading_set(text: str) -> set[str]:
    out = set()
    for line in text.splitlines():
        m = re.match(r"^#{1,6}\s+(.+?)\s*$", line.strip())
        if m:
            out.add(m.group(1).strip().lower())
    return out


def overlap_candidates(files: list[Path]) -> list[dict[str, object]]:
    headings = {p: heading_set(read(p)) for p in files}
    out: list[dict[str, object]] = []
    for i, left in enumerate(files):
        for right in files[i + 1 :]:
            if left == right:
                continue
            shared = sorted(headings[left] & headings[right])
            if len(shared) >= 3:
                out.append(
                    {
                        "left": rel(left),
                        "right": rel(right),
                        "shared_heading_count": len(shared),
                        "shared_headings": shared[:12],
                    }
                )
    out.sort(key=lambda x: (-(x["shared_heading_count"]), x["left"], x["right"]))
    return out[:80]


def readme_docs_crate_dupes(files: list[Path]) -> list[dict[str, object]]:
    buckets: dict[str, list[str]] = defaultdict(list)
    for p in files:
        name = p.name.lower()
        if name in {"readme.md", "index.md", "installation.md", "quickstart.md", "repl.md", "plugins.md", "commands.md"}:
            buckets[name].append(rel(p))
    out = [{"name": k, "files": sorted(v), "count": len(v)} for k, v in buckets.items() if len(v) > 1]
    out.sort(key=lambda x: (-x["count"], x["name"]))
    return out


def docs_vs_tests_overlap(files: list[Path]) -> list[dict[str, object]]:
    docs = [p for p in files if "docs" in p.parts]
    tests = [p for p in files if "tests" in p.parts and p.suffix == ".md"]
    if not tests:
        return []

    docs_heads = {p: heading_set(read(p)) for p in docs}
    tests_heads = {p: heading_set(read(p)) for p in tests}
    out: list[dict[str, object]] = []
    for d in docs:
        for t in tests:
            shared = sorted(docs_heads[d] & tests_heads[t])
            if len(shared) >= 2:
                out.append(
                    {
                        "doc": rel(d),
                        "test_doc": rel(t),
                        "shared_heading_count": len(shared),
                        "shared_headings": shared[:10],
                    }
                )
    out.sort(key=lambda x: (-(x["shared_heading_count"]), x["doc"], x["test_doc"]))
    return out[:40]


def generated_artifact_count() -> int:
    artifacts = ROOT / "artifacts"
    if not artifacts.exists():
        return 0
    return sum(1 for p in artifacts.rglob("*") if p.is_file())


def main() -> int:
    files = list_markdown()
    overlap = overlap_candidates(files)
    readme_dupes = readme_docs_crate_dupes(files)
    tests_overlap = docs_vs_tests_overlap(files)

    report = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "generator": "scripts/status/generate_docs_audit.py",
        "markdown_count": len(files),
        "markdown_files": [rel(p) for p in files],
        "generated_artifact_file_count": generated_artifact_count(),
        "readme_docs_crate_duplicates": readme_dupes,
        "docs_heading_overlap_candidates": overlap,
        "docs_tests_overlap_candidates": tests_overlap,
        "target_long_form_docs_cap": 60,
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
