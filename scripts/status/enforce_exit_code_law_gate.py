#!/usr/bin/env python3
"""Fail CI when exit-code law coverage or drift regresses."""

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
    contract = read_json(STATUS / "exit_code_contract_artifact.json")
    drift = read_json(STATUS / "exit_code_drift_artifact.json")

    if not contract:
        failures.append("missing artifacts/status/exit_code_contract_artifact.json")
    if not drift:
        failures.append("missing artifacts/status/exit_code_drift_artifact.json")

    if contract:
        if not bool(contract.get("release_blocking", False)):
            failures.append("exit-code law must be marked release_blocking=true")
        domains = contract.get("summary", {}).get("domains", [])
        required_domains = {
            "root",
            "cli",
            "dev_cli",
            "plugin_lifecycle",
            "config",
            "history",
            "memory",
            "diagnostics",
        }
        if set(domains) != required_domains:
            failures.append("exit-code contract domains are incomplete")
        missing_coverage_ids = int(contract.get("summary", {}).get("missing_coverage_ids", 1))
        if missing_coverage_ids != 0:
            failures.append(f"exit-code law coverage_id coverage incomplete: missing_coverage_ids={missing_coverage_ids}")

    if drift:
        if int(drift.get("drift_count", 1)) != 0:
            failures.append(f"exit-code drift detected: count={drift.get('drift_count')}")
        if drift.get("missing_coverage_ids"):
            failures.append(f"exit-code drift artifact has missing coverage_id coverage: {drift.get('missing_coverage_ids')}")

    for item in failures:
        print(f"EXIT CODE LAW FAILURE: {item}")

    if failures and args.enforce:
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

