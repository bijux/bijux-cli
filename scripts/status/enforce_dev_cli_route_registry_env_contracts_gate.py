#!/usr/bin/env python3
"""Fail CI when routes/registry/env/contracts truth drifts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    failures: list[str] = []
    truth = STATUS / "route_registry_env_contracts_artifact.json"
    drift = STATUS / "route_registry_env_contracts_drift_artifact.json"
    if not truth.exists():
        failures.append("missing artifact: artifacts/status/route_registry_env_contracts_artifact.json")
    if not drift.exists():
        failures.append("missing artifact: artifacts/status/route_registry_env_contracts_drift_artifact.json")

    if not failures:
        truth_payload = read_json(truth)
        drift_payload = read_json(drift)
        if truth_payload.get("status") != "complete":
            failures.append("route/registry/env/contracts truth artifact is not complete")
        if drift_payload.get("status") != "clean" or int(drift_payload.get("drift_count", 1)) != 0:
            failures.append("route/registry/env/contracts drift detected")

    if failures:
        print("DEV CLI ROUTE REGISTRY ENV CONTRACTS GATE FAILED")
        for failure in failures:
            print(f" - {failure}")
        return 1
    print("Dev CLI route/registry/env/contracts gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
