#!/usr/bin/env python3
"""Fail CI when kernel invariants regress."""

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
    report = read_json(STATUS / "kernel_invariants_report.json")
    diff = read_json(STATUS / "kernel_invariants_diff.json")

    if not report:
        failures.append("missing artifacts/status/kernel_invariants_report.json")
    if not diff:
        failures.append("missing artifacts/status/kernel_invariants_diff.json")

    rows = report.get("rows", []) if isinstance(report, dict) else []
    missing_rows = [row for row in rows if isinstance(row, dict) and row.get("status") != "covered"]
    if missing_rows:
        failures.append(
            "kernel invariants missing coverage for TODOs: "
            + ", ".join(str(row.get("todo", "?")) for row in missing_rows)
        )

    drift_items = diff.get("drift_items", []) if isinstance(diff, dict) else []
    if drift_items:
        failures.append(
            "kernel invariants drift detected: "
            + ", ".join(str(item.get("todo", "?")) for item in drift_items if isinstance(item, dict))
        )

    for item in failures:
        print(f"KERNEL INVARIANT FAILURE: {item}")

    if failures and args.enforce:
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

