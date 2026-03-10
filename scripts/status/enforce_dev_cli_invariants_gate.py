#!/usr/bin/env python3
"""Fail CI when dev-cli invariants regress."""

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
    report = read_json("dev_cli_invariants_artifact.json")
    drift = read_json("dev_cli_invariants_drift_artifact.json")

    if report.get("status") != "complete":
        failures.append("dev cli invariants report is not complete")
    if drift.get("status") != "clean":
        failures.append("dev cli invariants drift artifact is not clean")

    checks = report.get("checks", {})
    if isinstance(checks, dict):
        for name, ok in sorted(checks.items()):
            if not ok:
                failures.append(f"invariant failed: {name}")
    else:
        failures.append("dev cli invariants checks payload missing")

    if failures:
        print("DEV CLI INVARIANTS GATE FAILED")
        for failure in failures:
            print(f" - {failure}")
        return 1
    print("Dev CLI invariants gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
