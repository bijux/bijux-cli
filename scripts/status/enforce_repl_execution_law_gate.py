#!/usr/bin/env python3
"""Fail CI when covered REPL execution law drifts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
REQUIRED = [
    STATUS / "repl_shared_law_artifact.json",
    STATUS / "repl_cli_diff_artifact.json",
    STATUS / "repl_shared_law_drift_artifact.json",
    STATUS / "repl_shared_law_contract.json",
]


def main() -> int:
    missing = [path for path in REQUIRED if not path.exists()]
    if missing:
        raise SystemExit(
            "missing repl execution artifacts: " + ", ".join(str(path.relative_to(ROOT)) for path in missing)
        )

    drift = json.loads((STATUS / "repl_shared_law_drift_artifact.json").read_text(encoding="utf-8"))
    if drift.get("status") != "clean" or int(drift.get("drift_count", 1)) != 0:
        raise SystemExit(f"repl shared-law drift detected: coverage_ids={drift.get('drift_coverage_ids', [])}")

    contract = json.loads((STATUS / "repl_shared_law_contract.json").read_text(encoding="utf-8"))
    if contract.get("status") != "frozen":
        raise SystemExit("repl shared-law contract is not frozen")

    print("repl execution law gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
