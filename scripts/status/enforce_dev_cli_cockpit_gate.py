#!/usr/bin/env python3
"""Enforce top-level dev-cli cockpit coverage and integrity."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

REQUIRED = [
    "dev_cli_status_report.json",
    "dev_cli_dashboard_report.json",
    "dev_cli_quickcheck_report.json",
    "dev_cli_truth_report.json",
    "dev_cli_blockers_report.json",
    "dev_cli_next_report.json",
    "dev_cli_summary_surface_artifact.json",
    "dev_cli_summary_surface_drift_artifact.json",
]


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    failures: list[str] = []
    for name in REQUIRED:
        path = STATUS / name
        if not path.exists():
            failures.append(f"missing cockpit artifact: artifacts/status/{name}")
            continue
        payload = read_json(path)
        if not isinstance(payload, dict) or not payload:
            failures.append(f"invalid cockpit payload: artifacts/status/{name}")

    if not (STATUS / "dev_cli_cockpit_text_heads.json").exists():
        failures.append("missing cockpit text heads snapshot artifact")

    summary = STATUS / "dev_cli_summary_surface_artifact.json"
    if summary.exists():
        payload = read_json(summary)
        if payload.get("status") != "complete":
            failures.append("dev cli summary surface artifact is not complete")
    drift = STATUS / "dev_cli_summary_surface_drift_artifact.json"
    if drift.exists():
        payload = read_json(drift)
        if payload.get("status") != "clean" or int(payload.get("drift_count", 1)) != 0:
            failures.append("dev cli summary surface drift detected")

    if failures:
        print("DEV CLI COCKPIT GATE FAILED")
        for failure in failures:
            print(f" - {failure}")
        return 1
    print("Dev CLI cockpit gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
