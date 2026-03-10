#!/usr/bin/env python3
"""Fail CI when covered REPL completion behavior drifts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
REQUIRED = [
    STATUS / "repl_completion_artifact.json",
    STATUS / "repl_completion_ordering_artifact.json",
    STATUS / "repl_completion_drift_artifact.json",
    STATUS / "repl_completion_contract.json",
]


def main() -> int:
    missing = [path for path in REQUIRED if not path.exists()]
    if missing:
        raise SystemExit(
            "missing repl completion artifacts: " + ", ".join(str(path.relative_to(ROOT)) for path in missing)
        )

    drift = json.loads((STATUS / "repl_completion_drift_artifact.json").read_text(encoding="utf-8"))
    if drift.get("status") != "clean" or int(drift.get("drift_count", 1)) != 0:
        raise SystemExit(f"repl completion drift detected: todos={drift.get('drift_todos', [])}")

    ordering = json.loads((STATUS / "repl_completion_ordering_artifact.json").read_text(encoding="utf-8"))
    if ordering.get("status") != "stable":
        raise SystemExit("repl completion ordering is not stable")

    contract = json.loads((STATUS / "repl_completion_contract.json").read_text(encoding="utf-8"))
    if contract.get("status") != "frozen":
        raise SystemExit("repl completion contract is not frozen")

    print("repl completion gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
