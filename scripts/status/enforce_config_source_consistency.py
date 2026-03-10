#!/usr/bin/env python3
"""Fail when config source-reporting diverges from actual command resolution."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DRIFT = ROOT / "artifacts" / "status" / "config_source_drift_artifact.json"


def main() -> int:
    if not DRIFT.exists():
        raise SystemExit("missing drift artifact: artifacts/status/config_source_drift_artifact.json")

    payload = json.loads(DRIFT.read_text(encoding="utf-8"))
    if payload.get("status") != "clean" or int(payload.get("drift_count", 1)) != 0:
        reasons = payload.get("drift_reasons", [])
        raise SystemExit(
            "config source consistency drift detected: " + ("; ".join(reasons) if reasons else "unknown")
        )

    print("config source consistency gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
