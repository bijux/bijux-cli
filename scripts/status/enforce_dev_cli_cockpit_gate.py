#!/usr/bin/env python3
"""Enforce top-level dev-cli cockpit coverage and integrity."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

REQUIRED = [
    "dev_cli_dashboard_report.json",
    "dev_cli_quickcheck_report.json",
    "dev_cli_truth_report.json",
    "dev_cli_blockers_report.json",
    "dev_cli_next_report.json",
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

    if failures:
        print("DEV CLI COCKPIT GATE FAILED")
        for failure in failures:
            print(f" - {failure}")
        return 1
    print("Dev CLI cockpit gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
