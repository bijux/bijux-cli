#!/usr/bin/env python3
"""Warn CI when command-family consistency scope is still uncovered."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DRIFT = ROOT / "artifacts" / "status" / "cross_family_drift_artifact.json"


def main() -> int:
    if not DRIFT.exists():
        print("::warning title=Command Family Scope::missing artifacts/status/cross_family_drift_artifact.json")
        return 0

    payload = json.loads(DRIFT.read_text(encoding="utf-8"))
    uncovered = payload.get("uncovered_scope", [])
    if not uncovered:
        print("command-family uncovered scope warning check passed")
        return 0

    for row in uncovered:
        scope = row.get("scope", "unknown")
        reason = row.get("reason", "unreported")
        todos = row.get("impacted_todos", [])
        print(
            "::warning title=Command Family Scope::"
            f"uncovered scope `{scope}` impacts TODOs {todos}: {reason}"
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
