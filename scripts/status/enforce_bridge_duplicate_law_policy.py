#!/usr/bin/env python3
"""Enforce python-bridge duplicate-law report is clean."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REPORT = ROOT / "artifacts" / "status" / "bridge_duplicate_law_report.json"


def main() -> int:
    if not REPORT.exists():
        print(f"missing report: {REPORT.relative_to(ROOT)}")
        return 2

    payload = json.loads(REPORT.read_text(encoding="utf-8"))
    summary = payload.get("summary", {}) if isinstance(payload, dict) else {}
    count = int(summary.get("duplicate_rule_count", 0))
    if count > 0:
        print("BRIDGE DUPLICATE LAW POLICY FAILED")
        for check in payload.get("checks", []):
            if int(check.get("count", 0)) > 0:
                print(
                    f" - {check.get('area', 'unknown')}: {', '.join(check.get('duplicate_rules', []))}"
                )
        return 1

    print("Bridge duplicate-law policy passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
