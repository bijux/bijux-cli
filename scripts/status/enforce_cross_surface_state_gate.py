#!/usr/bin/env python3
"""Fail CI if covered cross-surface state behavior drifts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
REQUIRED = [
    STATUS / "cross_surface_state_consistency_artifact.json",
    STATUS / "cross_surface_state_drift_artifact.json",
    STATUS / "cross_surface_state_contract.json",
]


def main() -> int:
    missing = [path for path in REQUIRED if not path.exists()]
    if missing:
        raise SystemExit(
            "missing cross-surface state artifacts: "
            + ", ".join(str(path.relative_to(ROOT)) for path in missing)
        )

    drift = json.loads((STATUS / "cross_surface_state_drift_artifact.json").read_text(encoding="utf-8"))
    if drift.get("status") != "clean" or int(drift.get("drift_count", 1)) != 0:
        raise SystemExit(f"cross-surface state drift detected: coverage_ids={drift.get('drift_coverage_ids', [])}")

    contract = json.loads((STATUS / "cross_surface_state_contract.json").read_text(encoding="utf-8"))
    if contract.get("status") != "frozen":
        raise SystemExit("cross-surface state contract is not frozen")

    print("cross-surface state gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
