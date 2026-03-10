#!/usr/bin/env python3
"""Generate a docs duplication report focused on overlap groups, not counts."""

from __future__ import annotations

import json
import re
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"
OUT = ROOT / "artifacts" / "status" / "docs_duplication_report.json"


def norm_key(value: str) -> str:
    cleaned = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    cleaned = re.sub(r"-(report|audit|baseline|guide|rules|law|milestone)$", "", cleaned)
    return cleaned


def first_heading(path: Path) -> str:
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("# "):
            return line.removeprefix("# ").strip()
    return path.stem


def main() -> int:
    by_name: dict[str, list[str]] = defaultdict(list)
    by_heading: dict[str, list[str]] = defaultdict(list)

    for md in sorted(DOCS.rglob("*.md")):
        rel = str(md.relative_to(ROOT))
        by_name[norm_key(md.stem)].append(rel)
        by_heading[norm_key(first_heading(md))].append(rel)

    duplicate_name_groups = [paths for paths in by_name.values() if len(paths) > 1]
    duplicate_heading_groups = [paths for paths in by_heading.values() if len(paths) > 1]

    payload = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "generator": "scripts/status/generate_docs_duplication_report.py",
        "duplicate_stem_groups": sorted(duplicate_name_groups),
        "duplicate_heading_groups": sorted(duplicate_heading_groups),
        "action_rule": "docs exist to explain law or change; overlapping prose should be merged or replaced by artifacts",
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
