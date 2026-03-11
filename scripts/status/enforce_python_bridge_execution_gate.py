#!/usr/bin/env python3
"""Fail CI on covered Python-bridge execution drift."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
REQUIRED = [
    STATUS / "python_bridge_execution_artifact.json",
    STATUS / "python_bridge_drift_artifact.json",
    STATUS / "python_bridge_execution_contract.json",
]


def main() -> int:
    missing = [path for path in REQUIRED if not path.exists()]
    if missing:
        raise SystemExit(
            "missing python bridge execution artifacts: "
            + ", ".join(str(path.relative_to(ROOT)) for path in missing)
        )

    drift = json.loads((STATUS / "python_bridge_drift_artifact.json").read_text(encoding="utf-8"))
    if drift.get("status") != "clean" or int(drift.get("drift_count", 1)) != 0:
        raise SystemExit(f"python bridge execution drift detected: coverage_ids={drift.get('drift_coverage_ids', [])}")

    contract = json.loads((STATUS / "python_bridge_execution_contract.json").read_text(encoding="utf-8"))
    if contract.get("status") != "frozen":
        raise SystemExit("python bridge execution contract is not frozen")

    print("python bridge execution gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
