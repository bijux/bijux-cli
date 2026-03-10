#!/usr/bin/env python3
"""Fail CI when diagnostics trust schema drifts for covered commands."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
REQUIRED = [
    STATUS / "diagnostics_trust_artifact.json",
    STATUS / "actionable_diagnostics_artifact.json",
    STATUS / "diagnostics_minimalism_artifact.json",
    STATUS / "diagnostics_trust_schema_drift_artifact.json",
    STATUS / "diagnostics_trust_contract.json",
]


def main() -> int:
    missing = [path for path in REQUIRED if not path.exists()]
    if missing:
        raise SystemExit("missing diagnostics trust artifacts: " + ", ".join(str(p.relative_to(ROOT)) for p in missing))

    schema = json.loads((STATUS / "diagnostics_trust_schema_drift_artifact.json").read_text(encoding="utf-8"))
    if schema.get("status") != "clean" or int(schema.get("drift_count", 1)) != 0:
        raise SystemExit(f"diagnostics trust schema drift detected: {schema.get('drift_count')}")

    contract = json.loads((STATUS / "diagnostics_trust_contract.json").read_text(encoding="utf-8"))
    if contract.get("status") != "frozen":
        raise SystemExit("diagnostics trust contract is not frozen")

    print("diagnostics trust gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
