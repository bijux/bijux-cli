#!/usr/bin/env python3
"""Fail CI when evidence integrity is broken."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    audit = read_json(STATUS / "dev_cli_evidence_audit_report.json")
    if audit.get("status") != "pass":
        raise SystemExit("evidence audit failed")

    release_bundle = read_json(STATUS / "release_evidence_bundle.json")
    valid_ids = {
        str(row.get("id"))
        for row in audit.get("records", [])
        if isinstance(row, dict) and row.get("id") is not None
    }

    referenced = set()
    for row in release_bundle.get("items", []):
        if not isinstance(row, dict):
            continue
        evidence_ids = row.get("evidence_ids", [])
        if isinstance(evidence_ids, list):
            referenced.update(str(item) for item in evidence_ids)

    missing = sorted(item for item in referenced if item and item not in valid_ids)
    if missing:
        raise SystemExit(f"release evidence bundle references unknown evidence ids: {missing}")

    print("evidence integrity gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
