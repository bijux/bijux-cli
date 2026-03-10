#!/usr/bin/env python3
"""Fail CI when state diagnostics hardening artifacts regress."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    failures: list[str] = []
    required = [
        STATUS / "state_audit_truth_artifact.json",
        STATUS / "state_doctor_truth_artifact.json",
        STATUS / "corrupted_state_truth_artifact.json",
        STATUS / "state_diagnostics_drift_artifact.json",
    ]
    for path in required:
        if not path.exists():
            failures.append(f"missing artifact: {path.relative_to(ROOT)}")

    if not failures:
        state_audit = read_json(STATUS / "state_audit_truth_artifact.json")
        state_doctor = read_json(STATUS / "state_doctor_truth_artifact.json")
        corrupted_state = read_json(STATUS / "corrupted_state_truth_artifact.json")
        drift = read_json(STATUS / "state_diagnostics_drift_artifact.json")

        if state_audit.get("status") != "complete":
            failures.append("state audit truth artifact is not complete")
        if state_doctor.get("status") != "complete":
            failures.append("state doctor truth artifact is not complete")
        if corrupted_state.get("status") != "complete":
            failures.append("corrupted state truth artifact is not complete")
        if drift.get("status") != "clean" or int(drift.get("drift_count", 1)) != 0:
            failures.append("state diagnostics drift detected")

    if failures:
        print("DEV CLI STATE DIAGNOSTICS GATE FAILED")
        for failure in failures:
            print(f" - {failure}")
        return 1
    print("Dev CLI state diagnostics gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
