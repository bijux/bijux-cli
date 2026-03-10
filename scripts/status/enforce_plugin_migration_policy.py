#!/usr/bin/env python3
"""Enforce plugin migration evidence completeness and clarity."""

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
    required = [
        "plugin_lifecycle_ownership_report.json",
        "plugin_scaffold_efficiency_report.json",
        "plugin_scaffold_lifecycle_proof_report.json",
        "plugin_namespace_abuse_proof_report.json",
        "plugin_doctor_clarity_report.json",
        "plugin_explain_clarity_report.json",
        "plugin_where_ownership_report.json",
        "plugin_command_set_status.json",
        "plugin_migration_report.json",
        "plugin_rollback_proof_report.json",
    ]
    for name in required:
        if not (STATUS / name).exists():
            failures.append(f"missing required plugin artifact: artifacts/status/{name}")

    lifecycle = read_json("plugin_lifecycle_ownership_report.json")
    stages = lifecycle.get("stages", [])
    if not stages:
        failures.append("plugin lifecycle ownership report has no stages")
    for row in stages:
        if not isinstance(row, dict):
            failures.append("plugin lifecycle ownership row is not an object")
            continue
        if row.get("rust_owned") is not True:
            failures.append(f"plugin lifecycle stage not rust-owned: {row.get('stage')}")
        evidence = row.get("evidence", [])
        if not evidence:
            failures.append(f"plugin lifecycle stage missing evidence: {row.get('stage')}")

    scaffold = read_json("plugin_scaffold_efficiency_report.json")
    if scaffold.get("status") != "minimal":
        failures.append("plugin scaffold efficiency report is not minimal")

    for name in (
        "plugin_doctor_clarity_report.json",
        "plugin_explain_clarity_report.json",
        "plugin_where_ownership_report.json",
    ):
        clarity = read_json(name)
        if clarity.get("status") != "clear":
            failures.append(f"{name} is not clear")

    command_set = read_json("plugin_command_set_status.json")
    if not command_set.get("frozen_law"):
        failures.append("plugin command set report missing frozen_law")
    if "reject unproven plugin complexity" not in str(
        command_set.get("dynamic_complexity_policy", "")
    ):
        failures.append("plugin command set report missing strict dynamic complexity policy")
    if command_set.get("operating_style") != "boring-and-inspectable":
        failures.append("plugin command set report must enforce boring-and-inspectable style")

    rollback = read_json("plugin_rollback_proof_report.json")
    if rollback.get("status") != "complete":
        failures.append("plugin rollback proof status is not complete")

    if failures:
        print("PLUGIN MIGRATION POLICY VIOLATION")
        for failure in failures:
            print(f" - {failure}")
        return 1
    print("Plugin migration policy passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
