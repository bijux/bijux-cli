#!/usr/bin/env python3
"""Fail when cross-surface drift report shows uncovered required equivalence coverage."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DRIFT_REPORT = ROOT / "artifacts" / "status" / "cross_surface_drift_report.json"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--enforce", action="store_true")
    args = parser.parse_args()

    if not DRIFT_REPORT.exists():
        print(f"missing report: {DRIFT_REPORT.relative_to(ROOT)}")
        return 2 if args.enforce else 0

    payload = json.loads(DRIFT_REPORT.read_text(encoding="utf-8"))
    drift_count = int(payload.get("drift_count", 0)) if isinstance(payload, dict) else 0
    drift_items = payload.get("drift_items", []) if isinstance(payload, dict) else []

    if drift_count > 0:
        print("CROSS-SURFACE DRIFT DETECTED:")
        for item in drift_items:
            if isinstance(item, dict):
                coverage_id = item.get("coverage_id", "?")
                law = item.get("law", "<unknown>")
                test = item.get("test", "<unknown>")
                print(f" - Coverage {coverage_id}: {law} missing ({test})")
        return 1 if args.enforce else 0

    print("Cross-surface drift gate passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
