#!/usr/bin/env python3
"""Enforce frozen boundary artifacts for dev-cli extraction."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(name: str) -> dict[str, Any]:
    path = STATUS / name
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    failures: list[str] = []

    required = [
        "dev_cli_owned_behaviors_inventory.json",
        "runtime_owned_behaviors_inventory.json",
        "misplaced_dev_behaviors_report.json",
    ]
    for name in required:
        if not (STATUS / name).exists():
            failures.append(f"missing boundary artifact: artifacts/status/{name}")

    dev_inventory = read_json("dev_cli_owned_behaviors_inventory.json")
    runtime_inventory = read_json("runtime_owned_behaviors_inventory.json")
    misplaced = read_json("misplaced_dev_behaviors_report.json")

    if dev_inventory:
        if not bool(dev_inventory.get("boundary_frozen", False)):
            failures.append("dev cli boundary must be frozen before extraction")

        rules = dev_inventory.get("boundary_rules", {})
        if "bijux dev cli" not in str(rules.get("canonical_surface", "")):
            failures.append("canonical maintainer command surface rule is missing")
        if "only canonical executable" not in str(rules.get("binary_identity", "")):
            failures.append("canonical binary identity rule is missing")

        commands = dev_inventory.get("commands", [])
        if not isinstance(commands, list) or not commands:
            failures.append("dev cli command inventory is empty")
        else:
            for row in commands:
                if not isinstance(row, dict):
                    failures.append("dev cli command row must be an object")
                    continue
                if row.get("intended_owner") != "maintainer-control-plane":
                    failures.append(f"unexpected intended owner for {row.get('command', '<unknown>')}")

        missing = dev_inventory.get("missing_implementation_mappings", [])
        if isinstance(missing, list) and missing:
            failures.append(
                "dev cli implementation mappings are incomplete: " + ", ".join(str(item) for item in missing)
            )

    if runtime_inventory:
        behaviors = runtime_inventory.get("behaviors", [])
        if not isinstance(behaviors, list) or not behaviors:
            failures.append("runtime-owned behaviors inventory is empty")

    if misplaced:
        boundary_freeze = misplaced.get("boundary_freeze", {})
        if boundary_freeze.get("status") != "frozen-before-extraction":
            failures.append("misplaced behavior report must declare frozen-before-extraction status")

    if failures:
        print("DEV CLI BOUNDARY POLICY VIOLATION")
        for failure in failures:
            print(f" - {failure}")
        return 1

    print("Dev cli boundary policy passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
