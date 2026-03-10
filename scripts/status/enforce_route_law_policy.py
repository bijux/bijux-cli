#!/usr/bin/env python3
"""Enforce route-law special-case reduction policy."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    failures: list[str] = []

    report = read_json(STATUS / "route_special_cases.json")
    special = report.get("report", {}) if isinstance(report, dict) else {}
    summary = special.get("summary", {}) if isinstance(special, dict) else {}

    baseline = int(summary.get("baseline_special_case_count", 0))
    current = int(summary.get("special_case_count", 0))

    if current > baseline:
        failures.append(
            f"route special cases increased above baseline: current={current} baseline={baseline}"
        )

    if "rule" not in report:
        failures.append("route special case report is missing policy rule")

    required = [
        STATUS / "route_command_owner_mapping.json",
        STATUS / "route_command_test_coverage_mapping.json",
        STATUS / "route_command_parity_status_mapping.json",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        failures.append("missing route law mapping artifacts: " + ", ".join(missing))

    for msg in failures:
        print(f"ROUTE LAW FAILURE: {msg}")

    return 2 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
