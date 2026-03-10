#!/usr/bin/env python3
"""Fail CI when covered cross-command consistency checks drift."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DRIFT = ROOT / "artifacts" / "status" / "command_surface_consistency_drift_artifact.json"


def main() -> int:
    if not DRIFT.exists():
        raise SystemExit("missing drift artifact: artifacts/status/command_surface_consistency_drift_artifact.json")

    payload = json.loads(DRIFT.read_text(encoding="utf-8"))
    if payload.get("status") != "clean" or int(payload.get("drift_count", 1)) != 0:
        raise SystemExit(
            f"cross-command consistency drift detected: todos={payload.get('drift_todos', [])}"
        )

    print("cross-command consistency gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
