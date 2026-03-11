#!/usr/bin/env python3
"""Enforce cross-surface consistency drift policy."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DRIFT = ROOT / "artifacts" / "status" / "cross_surface_drift_artifact.json"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--enforce", action="store_true")
    args = parser.parse_args()

    if not DRIFT.exists():
        print("missing artifacts/status/cross_surface_drift_artifact.json")
        return 2 if args.enforce else 0

    payload = json.loads(DRIFT.read_text(encoding="utf-8"))
    drift_items = payload.get("drift_items", []) if isinstance(payload, dict) else []

    failures = []
    warnings = []
    for item in drift_items:
        if not isinstance(item, dict):
            continue
        coverage = str(item.get("coverage_class", "partial"))
        coverage_id = item.get("coverage_id", "?")
        law = item.get("law", "<unknown>")
        if coverage == "covered":
            failures.append(f"Coverage {coverage_id}: {law}")
        else:
            warnings.append(f"Coverage {coverage_id}: {law}")

    for msg in warnings:
        print(f"CROSS-SURFACE WARNING (partial): {msg}")
    for msg in failures:
        print(f"CROSS-SURFACE FAILURE (covered): {msg}")

    if failures and args.enforce:
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
