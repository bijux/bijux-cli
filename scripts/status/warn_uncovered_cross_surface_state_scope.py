#!/usr/bin/env python3
"""Warn CI if cross-surface state scope remains uncovered."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DRIFT = ROOT / "artifacts" / "status" / "cross_surface_state_drift_artifact.json"


def main() -> int:
    if not DRIFT.exists():
        print("::warning title=Cross-Surface State::missing artifacts/status/cross_surface_state_drift_artifact.json")
        return 0

    payload = json.loads(DRIFT.read_text(encoding="utf-8"))
    status = payload.get("status")
    if status != "clean":
        coverage_ids = payload.get("drift_coverage_ids", [])
        print(
            "::warning title=Cross-Surface State::"
            f"cross-surface state coverage is partial for coverage_ids: {coverage_ids}"
        )
    else:
        print("cross-surface state warning check passed")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
