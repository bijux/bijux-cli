#!/usr/bin/env python3
"""Fail CI when concurrent state-race hardening evidence drifts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

REQUIRED = [
    STATUS / "state_race_campaign_artifact.json",
    STATUS / "state_race_outcome_classification_artifact.json",
    STATUS / "state_race_reproducer_retention_artifact.json",
    STATUS / "state_race_regression_artifact.json",
    STATUS / "state_race_contract.json",
]


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    missing = [p for p in REQUIRED if not p.exists()]
    if missing:
        raise SystemExit(
            "missing state race artifacts: "
            + ", ".join(str(p.relative_to(ROOT)) for p in missing)
        )

    campaign = read_json(STATUS / "state_race_campaign_artifact.json")
    retention = read_json(STATUS / "state_race_reproducer_retention_artifact.json")
    regressions = read_json(STATUS / "state_race_regression_artifact.json")
    contract = read_json(STATUS / "state_race_contract.json")

    if campaign.get("status") != "complete":
        raise SystemExit("state race campaign coverage is incomplete")
    if retention.get("status") != "complete":
        raise SystemExit("state race reproducer retention is incomplete")
    if regressions.get("status") != "clean":
        raise SystemExit("state race regression replay drift detected")
    if contract.get("status") != "frozen":
        raise SystemExit(f"state race contract is not frozen: {contract.get('missing_todos', [])}")

    print("state race campaign gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
