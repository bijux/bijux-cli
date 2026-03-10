#!/usr/bin/env python3
"""Fail CI on REPL hostile-session drift for covered cases."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
REQUIRED = [
    STATUS / "repl_hostile_session_artifact.json",
    STATUS / "repl_recovery_artifact.json",
    STATUS / "repl_startup_resilience_artifact.json",
    STATUS / "repl_command_loop_failure_class_artifact.json",
    STATUS / "repl_hostile_session_contract.json",
    STATUS / "repl_hostile_session_drift_artifact.json",
]


def main() -> int:
    missing = [path for path in REQUIRED if not path.exists()]
    if missing:
        raise SystemExit(
            "missing repl hostile-session artifacts: "
            + ", ".join(str(path.relative_to(ROOT)) for path in missing)
        )

    drift = json.loads((STATUS / "repl_hostile_session_drift_artifact.json").read_text(encoding="utf-8"))
    if drift.get("status") != "clean" or int(drift.get("drift_count", 1)) != 0:
        raise SystemExit(f"repl hostile-session drift detected: todos={drift.get('drift_todos', [])}")

    contract = json.loads((STATUS / "repl_hostile_session_contract.json").read_text(encoding="utf-8"))
    if contract.get("status") != "frozen":
        raise SystemExit("repl hostile-session contract is not frozen")

    print("repl hostile-session gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
