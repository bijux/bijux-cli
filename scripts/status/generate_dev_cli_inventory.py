#!/usr/bin/env python3
"""Generate scripts and makefile inventory for dev-cli migration planning."""

from __future__ import annotations

import json
import re
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "artifacts" / "status" / "dev_cli_inventory.json"


def classify_script(path: str) -> str:
    if path.startswith("scripts/status/") or path.startswith("scripts/parity/"):
        return "replace"
    if path.startswith("scripts/git-hooks/") or path.startswith("scripts/docs_builder/"):
        return "keep"
    if path == "scripts/__init__.py":
        return "delete"
    return "replace"


def classify_make_target(target: str) -> str:
    if target.startswith(("docs", "api", "test")):
        return "replace"
    if target.startswith(("publish", "sbom", "security")):
        return "keep"
    return "replace"


def iter_files(base: Path) -> list[Path]:
    if not base.exists():
        return []
    return sorted(p for p in base.rglob("*") if p.is_file())


def parse_make_targets(path: Path) -> list[str]:
    out: list[str] = []
    text = path.read_text(encoding="utf-8", errors="ignore")
    for raw in text.splitlines():
        if not raw or raw.startswith(("\t", "#")):
            continue
        m = re.match(r"^([A-Za-z0-9_.-]+)\s*:\s*", raw)
        if not m:
            continue
        target = m.group(1)
        if target.startswith("."):
            continue
        out.append(target)
    return sorted(set(out))


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT)).replace("\\", "/")


def main() -> int:
    scripts = [
        {"path": rel(path), "classification": classify_script(rel(path))}
        for path in iter_files(ROOT / "scripts")
    ]

    makefiles = []
    for mk in iter_files(ROOT / "makefiles"):
        targets = [
            {"target": t, "classification": classify_make_target(t)} for t in parse_make_targets(mk)
        ]
        makefiles.append({"file": rel(mk), "targets": targets})

    report = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "generator": "scripts/status/generate_dev_cli_inventory.py",
        "scripts": scripts,
        "makefiles": makefiles,
        "maintainer_script_replacements": [
            {"from": "scripts/status/generate_current_rust_state.py", "to": "bijux dev cli status"},
            {"from": "scripts/status/generate_crate_boundary_metrics.py", "to": "bijux dev cli crate-health"},
            {"from": "scripts/parity/run_rust_python_parity.py", "to": "bijux dev cli parity"},
        ],
        "policy": "new maintainer automation defaults to bijux dev cli commands",
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
