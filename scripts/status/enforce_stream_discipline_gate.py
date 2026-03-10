#!/usr/bin/env python3
"""Fail CI when stdout/stderr discipline regresses."""

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
    contract = read_json(STATUS / "stream_discipline_artifact.json")
    drift = read_json(STATUS / "stream_drift_artifact.json")

    if not contract:
        failures.append("missing artifacts/status/stream_discipline_artifact.json")
    if not drift:
        failures.append("missing artifacts/status/stream_drift_artifact.json")

    if contract:
        if not bool(contract.get("release_blocking", False)):
            failures.append("stream discipline contract must be release_blocking=true")
        missing_todos = int(contract.get("summary", {}).get("missing_todos", 1))
        if missing_todos != 0:
            failures.append(f"stream discipline todo coverage incomplete: missing_todos={missing_todos}")

    if drift:
        if int(drift.get("drift_count", 1)) != 0:
            failures.append(f"stream discipline drift detected: count={drift.get('drift_count')}")
        if drift.get("missing_todos"):
            failures.append(f"stream drift artifact has missing todo coverage: {drift.get('missing_todos')}")

    for item in failures:
        print(f"STREAM DISCIPLINE FAILURE: {item}")

    if failures and args.enforce:
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

