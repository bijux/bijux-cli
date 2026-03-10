#!/usr/bin/env python3
"""Enforce Python sovereignty audit and release claim gate."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    failures: list[str] = []
    sovereignty = read_json(STATUS / "python_sovereignty_audit_report.json")
    if sovereignty.get("status") != "green":
        failures.append("python sovereignty audit is not green")

    details = sovereignty.get("python_sovereignty_audit", {})
    for key in (
        "command_law_duplication",
        "output_law_duplication",
        "exit_law_duplication",
        "route_law_duplication",
        "state_law_duplication",
    ):
        rows = details.get(key, [])
        if isinstance(rows, list) and rows:
            failures.append(f"{key} is not empty")

    release_manifest = read_json(STATUS / "release_status_manifest.json")
    release_claims = release_manifest.get("claims", {})
    if isinstance(release_claims, dict) and release_claims.get("python_surface_only") and sovereignty.get("status") != "green":
        failures.append("release claims python-surface-only before sovereignty audit is green")

    if failures:
        print("PYTHON SOVEREIGNTY GATE FAILED")
        for failure in failures:
            print(f" - {failure}")
        return 1
    print("Python sovereignty gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
