#!/usr/bin/env python3
"""Fail CI when covered help-law behavior drifts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
REQUIRED = [
    STATUS / "help_law_artifact.json",
    STATUS / "command_tree_help_consistency_artifact.json",
    STATUS / "help_drift_artifact.json",
    STATUS / "help_tree_contract.json",
]


def main() -> int:
    missing = [path for path in REQUIRED if not path.exists()]
    if missing:
        raise SystemExit("missing help-law artifacts: " + ", ".join(str(path.relative_to(ROOT)) for path in missing))

    drift = json.loads((STATUS / "help_drift_artifact.json").read_text(encoding="utf-8"))
    if drift.get("status") != "clean" or int(drift.get("drift_count", 1)) != 0:
        raise SystemExit(f"help-law drift detected: todos={drift.get('drift_todos', [])}")

    consistency = json.loads((STATUS / "command_tree_help_consistency_artifact.json").read_text(encoding="utf-8"))
    if consistency.get("status") != "complete":
        raise SystemExit("command-tree/help consistency is not complete")

    contract = json.loads((STATUS / "help_tree_contract.json").read_text(encoding="utf-8"))
    if contract.get("status") != "frozen":
        raise SystemExit("help-tree law contract is not frozen")

    print("help-tree law gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
