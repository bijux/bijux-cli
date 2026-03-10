#!/usr/bin/env python3
"""Fail CI when parity/migration consistency artifacts regress."""

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
        STATUS / "migration_truth_artifact.json",
        STATUS / "parity_evidence_consistency_artifact.json",
        STATUS / "parity_drift_artifact.json",
        STATUS / "command_family_closure_report.json",
    ]
    for path in required:
        if not path.exists():
            failures.append(f"missing artifact: {path.relative_to(ROOT)}")

    if not failures:
        migration_truth = read_json(STATUS / "migration_truth_artifact.json")
        parity_consistency = read_json(STATUS / "parity_evidence_consistency_artifact.json")
        drift = read_json(STATUS / "parity_drift_artifact.json")
        closure = read_json(STATUS / "command_family_closure_report.json")

        if migration_truth.get("status") != "complete":
            failures.append("migration truth artifact is not complete")
        if parity_consistency.get("status") != "complete":
            failures.append("parity evidence consistency artifact is not complete")
        if drift.get("status") != "clean" or int(drift.get("drift_count", 1)) != 0:
            failures.append("parity drift artifact is not clean")
        if closure.get("status") not in {"complete", "partial", "evolving", "attention-required"}:
            failures.append("command family closure report has invalid status")
        if not closure.get("reports"):
            failures.append("command family closure report is empty")

    if failures:
        print("DEV CLI PARITY CONSISTENCY GATE FAILED")
        for failure in failures:
            print(f" - {failure}")
        return 1
    print("Dev CLI parity consistency gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
