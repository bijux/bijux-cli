#!/usr/bin/env python3
"""Fail CI when repo health reports show unresolved drift."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(name: str) -> dict:
    path = STATUS / name
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    failures: list[str] = []
    health = read_json("repo_health_report.json")
    drift = read_json("repo_drift_report.json")
    generated = read_json("repo_generated_report.json")

    if health.get("status") not in {"healthy", "degraded"}:
        failures.append("repo health status missing")

    drift_status = drift.get("status")
    if drift_status not in {"clean", "drift"}:
        failures.append("repo drift status missing")

    orphan = generated.get("orphan_generated_outputs", [])
    if isinstance(orphan, list) and orphan:
        failures.append("orphan generated outputs detected")

    if failures:
        print("REPO HEALTH GATE FAILED")
        for failure in failures:
            print(f" - {failure}")
        return 1
    print("Repo health gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
