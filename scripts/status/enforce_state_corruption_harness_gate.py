#!/usr/bin/env python3
"""Fail CI when randomized state-corruption harness evidence drifts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

REQUIRED = [
    STATUS / "state_corruption_campaign_artifact.json",
    STATUS / "state_corruption_reproducer_retention_artifact.json",
    STATUS / "state_corruption_harness_contract.json",
]


def main() -> int:
    missing = [path for path in REQUIRED if not path.exists()]
    if missing:
        raise SystemExit(
            "missing state corruption harness artifacts: "
            + ", ".join(str(p.relative_to(ROOT)) for p in missing)
        )

    campaign = json.loads((STATUS / "state_corruption_campaign_artifact.json").read_text(encoding="utf-8"))
    retention = json.loads((STATUS / "state_corruption_reproducer_retention_artifact.json").read_text(encoding="utf-8"))
    contract = json.loads((STATUS / "state_corruption_harness_contract.json").read_text(encoding="utf-8"))

    if campaign.get("status") != "clean":
        raise SystemExit("randomized state corruption campaign is not clean")
    if retention.get("status") != "clean":
        raise SystemExit("minimized corrupted-state reproducer retention is not clean")
    if contract.get("status") != "frozen":
        raise SystemExit(f"state corruption harness contract is not frozen: {contract.get('missing_todos', [])}")

    print("state corruption harness gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
