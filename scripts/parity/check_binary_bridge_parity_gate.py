#!/usr/bin/env python3
"""Fail when binary-vs-python-bridge parity report contains drift."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REPORT = ROOT / "artifacts" / "parity" / "binary_vs_python_bridge_parity_report.json"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--enforce", action="store_true")
    args = parser.parse_args()

    if not REPORT.exists():
        print(f"missing report: {REPORT.relative_to(ROOT)}")
        return 2 if args.enforce else 0

    payload = json.loads(REPORT.read_text(encoding="utf-8"))
    rows = payload.get("cases", []) if isinstance(payload, dict) else []

    failures: list[str] = []
    for row in rows:
        if not isinstance(row, dict):
            continue
        cmd = str(row.get("command", "<unknown>"))
        for key in ("exit_match", "stdout_match", "stderr_match"):
            if not bool(row.get(key, False)):
                failures.append(f"{cmd}: {key}=false")

    if failures:
        print("BINARY-BRIDGE PARITY DRIFT:")
        for item in failures:
            print(f" - {item}")
        return 1 if args.enforce else 0

    print("Binary-vs-python-bridge parity gate passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
