#!/usr/bin/env python3
"""Fail CI when command metadata consistency artifacts drift."""

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
    metadata = read_json(STATUS / "command_metadata_artifact.json")
    routes = read_json(STATUS / "route_metadata_artifact.json")
    drift = read_json(STATUS / "metadata_drift_artifact.json")
    ownership = read_json(STATUS / "command_ownership_artifact.json")

    if not metadata:
        failures.append("missing artifacts/status/command_metadata_artifact.json")
    if not routes:
        failures.append("missing artifacts/status/route_metadata_artifact.json")
    if not drift:
        failures.append("missing artifacts/status/metadata_drift_artifact.json")
    if not ownership:
        failures.append("missing artifacts/status/command_ownership_artifact.json")

    if metadata and not bool(metadata.get("release_blocking", False)):
        failures.append("metadata artifact must be marked release_blocking=true")

    if routes and not bool(routes.get("route_identity_match", False)):
        failures.append("inspect and dev cli route identity mismatch")

    if ownership and not bool(ownership.get("reserved_namespace_match", False)):
        failures.append("reserved namespace ownership mismatch")

    if drift:
        if int(drift.get("drift_count", 1)) != 0:
            failures.append(f"metadata drift detected: count={drift.get('drift_count')}")
        coverage_rows = drift.get("coverage_rows", [])
        missing_coverage_ids = [row["coverage_id"] for row in coverage_rows if isinstance(row, dict) and row.get("status") != "covered"]
        if missing_coverage_ids:
            failures.append(f"metadata coverage_id coverage incomplete: {missing_coverage_ids}")

    for item in failures:
        print(f"METADATA CONSISTENCY FAILURE: {item}")

    if failures and args.enforce:
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

