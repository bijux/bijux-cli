#!/usr/bin/env python3
"""Fail CI on covered Python-bridge conversion drift."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
REQUIRED = [
    STATUS / "bridge_conversion_artifact.json",
    STATUS / "bridge_exception_mapping_artifact.json",
    STATUS / "bridge_envelope_integrity_artifact.json",
    STATUS / "bridge_conversion_drift_artifact.json",
    STATUS / "bridge_conversion_contract.json",
]


def main() -> int:
    missing = [path for path in REQUIRED if not path.exists()]
    if missing:
        raise SystemExit(
            "missing python bridge conversion artifacts: "
            + ", ".join(str(path.relative_to(ROOT)) for path in missing)
        )

    drift = json.loads((STATUS / "bridge_conversion_drift_artifact.json").read_text(encoding="utf-8"))
    if drift.get("status") != "clean" or int(drift.get("drift_count", 1)) != 0:
        raise SystemExit(f"python bridge conversion drift detected: todos={drift.get('drift_todos', [])}")

    contract = json.loads((STATUS / "bridge_conversion_contract.json").read_text(encoding="utf-8"))
    if contract.get("status") != "frozen":
        raise SystemExit("python bridge conversion contract is not frozen")

    print("python bridge conversion gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
