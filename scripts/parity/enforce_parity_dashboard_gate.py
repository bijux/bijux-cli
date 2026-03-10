#!/usr/bin/env python3
"""Enforce command-law parity reports and dashboard completeness."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PARITY = ROOT / "artifacts" / "parity"

REQUIRED = [
    "command_precedence_report.json",
    "command_flag_normalization_report.json",
    "command_stream_report.json",
    "command_exit_code_report.json",
    "command_help_diff_report.json",
    "command_machine_output_diff_report.json",
    "parity_dashboard.json",
    "parity_dashboard.txt",
]


def read_json(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    failures: list[str] = []

    for name in REQUIRED:
        p = PARITY / name
        if not p.exists():
            failures.append(f"missing {p.relative_to(ROOT)}")

    dashboard = read_json(PARITY / "parity_dashboard.json")
    summary = dashboard.get("summary", {}) if isinstance(dashboard, dict) else {}
    coverage = summary.get("coverage", {}) if isinstance(summary, dict) else {}

    if not summary:
        failures.append("parity_dashboard.json missing summary")
    if not coverage:
        failures.append("parity_dashboard.json missing coverage")

    if isinstance(coverage, dict) and int(coverage.get("parity_tests", 0)) <= 0:
        failures.append("parity dashboard shows zero parity tests")

    if failures:
        print("PARITY DASHBOARD GATE FAILED")
        for failure in failures:
            print(f" - {failure}")
        return 1

    print("Parity dashboard gate passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
