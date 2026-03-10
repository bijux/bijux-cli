#!/usr/bin/env python3
"""Fail CI when diagnostics deep behavior artifacts regress."""

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
        STATUS / "diagnostics_consistency_artifact.json",
        STATUS / "doctor_determinism_artifact.json",
        STATUS / "diagnostics_schema_drift_artifact.json",
        STATUS / "diagnostics_source_of_truth_artifact.json",
        STATUS / "findings_order_artifact.json",
        STATUS / "diagnostics_contract_artifact.json",
        STATUS / "diagnostics_deep_behavior_drift_artifact.json",
    ]
    for path in required:
        if not path.exists():
            failures.append(f"missing artifact: {path.relative_to(ROOT)}")

    drift = read_json(STATUS / "diagnostics_deep_behavior_drift_artifact.json")
    if drift:
        if int(drift.get("drift_count", 1)) != 0:
            failures.append(f"diagnostics deep behavior drift detected: count={drift.get('drift_count')}")
        missing_todos = [
            row["todo"]
            for row in drift.get("todo_coverage", [])
            if isinstance(row, dict) and row.get("status") != "covered"
        ]
        if missing_todos:
            failures.append(f"diagnostics deep behavior todo coverage incomplete: {missing_todos}")

    for artifact_name in [
        "diagnostics_consistency_artifact.json",
        "doctor_determinism_artifact.json",
        "diagnostics_schema_drift_artifact.json",
        "diagnostics_source_of_truth_artifact.json",
        "findings_order_artifact.json",
        "diagnostics_contract_artifact.json",
    ]:
        payload = read_json(STATUS / artifact_name)
        if payload and payload.get("status") != "complete":
            failures.append(f"{artifact_name} status is not complete")

    for item in failures:
        print(f"DIAGNOSTICS DEEP BEHAVIOR FAILURE: {item}")

    if failures and args.enforce:
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
