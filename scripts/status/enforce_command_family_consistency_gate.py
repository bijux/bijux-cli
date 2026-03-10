#!/usr/bin/env python3
"""Fail CI when covered command-family consistency checks drift."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
DRIFT = STATUS / "cross_family_drift_artifact.json"
REQUIRED = [
    STATUS / "command_family_consistency_artifact.json",
    STATUS / "cross_family_drift_artifact.json",
    STATUS / "shared_law_proof_artifact.json",
    STATUS / "command_family_consistency_requirement.json",
]


def main() -> int:
    missing = [path for path in REQUIRED if not path.exists()]
    if missing:
        raise SystemExit(
            "missing command-family artifacts: "
            + ", ".join(str(path.relative_to(ROOT)) for path in missing)
        )

    payload = json.loads(DRIFT.read_text(encoding="utf-8"))
    if payload.get("status") != "clean" or int(payload.get("drift_count", 1)) != 0:
        raise SystemExit(
            f"command-family consistency drift detected: todos={payload.get('drift_todos', [])}"
        )

    print("command-family consistency gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
