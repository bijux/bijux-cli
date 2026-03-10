#!/usr/bin/env python3
"""Fail CI on dev-cli determinism or side-effect regressions."""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(name: str) -> dict:
    path = STATUS / name
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    failures: list[str] = []
    resilience = read_json("dev_cli_control_plane_resilience_artifact.json")
    determinism = read_json("dev_cli_determinism_artifact.json")
    side_effect = read_json("dev_cli_side_effect_audit_artifact.json")
    drift = read_json("dev_cli_resilience_drift_artifact.json")

    if resilience.get("status") != "complete":
        failures.append("dev cli control-plane resilience artifact is not complete")
    if determinism.get("status") != "clean":
        failures.append("dev cli determinism artifact is not clean")
    if side_effect.get("status") != "clean":
        failures.append("dev cli side-effect audit artifact is not clean")
    if drift.get("status") != "clean" or int(drift.get("drift_count", 1)) != 0:
        failures.append("dev cli resilience drift detected")

    checks = resilience.get("checks", {})
    if not isinstance(checks, dict):
        failures.append("dev cli resilience checks payload missing")
    else:
        for key, ok in sorted(checks.items()):
            if not ok:
                failures.append(f"resilience check failed: {key}")

    if failures:
        print("DEV CLI RESILIENCE GATE FAILED")
        for failure in failures:
            print(f" - {failure}")
        return 1
    print("Dev CLI resilience gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
