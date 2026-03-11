#!/usr/bin/env python3
"""Fail CI when config deep behavior artifacts regress."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--enforce", action="store_true")
    args = parser.parse_args()

    failures: list[str] = []

    required = [
        STATUS / "config_semantic_roundtrip_artifact.json",
        STATUS / "config_precedence_artifact.json",
        STATUS / "config_determinism_artifact.json",
        STATUS / "config_corruption_recovery_artifact.json",
        STATUS / "config_deep_behavior_drift_artifact.json",
    ]
    for path in required:
        if not path.exists():
            failures.append(f"missing artifact: {path.relative_to(ROOT)}")

    drift = read_json(STATUS / "config_deep_behavior_drift_artifact.json")
    if drift:
        if int(drift.get("drift_count", 1)) != 0:
            failures.append(f"config deep behavior drift detected: count={drift.get('drift_count')}")
        missing_coverage_ids = [
            row["coverage_id"]
            for row in drift.get("coverage_rows", [])
            if isinstance(row, dict) and row.get("status") != "covered"
        ]
        if missing_coverage_ids:
            failures.append(f"config deep behavior coverage_id coverage incomplete: {missing_coverage_ids}")

    for artifact_name in [
        "config_semantic_roundtrip_artifact.json",
        "config_precedence_artifact.json",
        "config_determinism_artifact.json",
        "config_corruption_recovery_artifact.json",
    ]:
        payload = read_json(STATUS / artifact_name)
        if payload and payload.get("status") != "complete":
            failures.append(f"{artifact_name} status is not complete")

    for item in failures:
        print(f"CONFIG DEEP BEHAVIOR FAILURE: {item}")

    if failures and args.enforce:
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

