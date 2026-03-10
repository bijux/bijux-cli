#!/usr/bin/env python3
"""Generate command metadata consistency artifacts for TODOs 61-80."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli-core" / "tests" / "bin_surface" / "metadata_inspection_matrix.rs"

REQUIRED_TESTS = {
    61: "every_routable_command_has_inspectable_metadata_and_stable_route_identity",
    62: "every_routable_command_has_inspectable_metadata_and_stable_route_identity",
    63: "inspect_exposes_builtin_and_plugin_metadata_consistently",
    64: "inspect_exposes_builtin_and_plugin_metadata_consistently",
    65: "inspect_routes_and_registry_agree_on_namespace_ownership_and_plugin_source_metadata",
    66: "inspect_routes_and_registry_agree_on_namespace_ownership_and_plugin_source_metadata",
    67: "route_metadata_is_stable_and_json_serializable_for_covered_commands",
    68: "route_metadata_is_stable_and_json_serializable_for_covered_commands",
    69: "command_metadata_fields_do_not_disappear_or_rename_silently",
    70: "command_metadata_fields_do_not_disappear_or_rename_silently",
    71: "reserved_namespaces_and_alias_metadata_are_consistent_and_non_canonical",
    72: "reserved_namespaces_and_alias_metadata_are_consistent_and_non_canonical",
    73: "reserved_namespaces_and_alias_metadata_are_consistent_and_non_canonical",
    74: "help_output_and_inspect_metadata_agree_on_command_names_and_grouping",
    75: "help_output_and_inspect_metadata_agree_on_command_names_and_grouping",
}


def run_json(args: list[str]) -> dict[str, Any]:
    out = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-core", "--", *args, "--format", "json", "--no-pretty"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(out.stdout or "{}")


def has_test(source: str, test_name: str) -> bool:
    return f"fn {test_name}(" in source


def route_key(row: dict[str, Any]) -> str:
    segments = row.get("segments", [])
    if not isinstance(segments, list):
        return ""
    return " ".join(str(item) for item in segments)


def main() -> None:
    source = TEST_FILE.read_text(encoding="utf-8")
    inspect = run_json(["inspect"])
    routes = run_json(["dev", "cli", "routes"])
    registry = run_json(["dev", "cli", "registry"])

    inspect_routes = inspect.get("route_sources", [])
    dev_routes = routes.get("routes", [])
    inspect_route_set = {route_key(row) for row in inspect_routes if isinstance(row, dict)}
    dev_route_set = {route_key(row) for row in dev_routes if isinstance(row, dict)}

    required_keys = [
        "status",
        "builtins",
        "route_sources",
        "reserved_namespaces",
        "plugin_origins",
        "alias_rewrites",
        "contracts",
    ]
    missing_keys = [key for key in required_keys if key not in inspect]

    reserved_inspect = {
        str(row.get("name"))
        for row in inspect.get("reserved_namespaces", [])
        if isinstance(row, dict) and row.get("reserved") is True
    }
    reserved_registry = {
        str(row.get("name"))
        for row in registry.get("registry", [])
        if isinstance(row, dict) and row.get("reserved") is True
    }

    command_metadata_artifact = {
        "generator": "scripts/status/generate_metadata_consistency_reports.py",
        "scope": "command metadata consistency",
        "tasks": [61, 63, 64, 68, 69, 70, 71, 72, 73, 74, 75, 76, 80],
        "release_blocking": True,
        "required_keys": required_keys,
        "missing_keys": missing_keys,
        "status": "complete" if not missing_keys else "partial",
    }

    route_metadata_artifact = {
        "generator": "scripts/status/generate_metadata_consistency_reports.py",
        "scope": "route metadata consistency",
        "tasks": [62, 65, 67, 77, 79],
        "inspect_route_count": len(inspect_route_set),
        "dev_route_count": len(dev_route_set),
        "route_identity_match": inspect_route_set == dev_route_set,
        "status": "complete" if inspect_route_set == dev_route_set else "partial",
    }

    ownership_artifact = {
        "generator": "scripts/status/generate_metadata_consistency_reports.py",
        "scope": "command ownership",
        "tasks": [66, 79],
        "registry_owners": sorted(
            {
                str(row.get("owner"))
                for row in registry.get("registry", [])
                if isinstance(row, dict) and row.get("owner") is not None
            }
        ),
        "plugin_origin_owners": sorted(
            {
                str(row.get("owner"))
                for row in inspect.get("plugin_origins", [])
                if isinstance(row, dict) and row.get("owner") is not None
            }
        ),
        "reserved_namespace_match": reserved_inspect == reserved_registry,
        "status": "complete" if reserved_inspect == reserved_registry else "partial",
    }

    todo_rows = []
    for todo, test_name in sorted(REQUIRED_TESTS.items()):
        todo_rows.append(
            {
                "todo": todo,
                "test_name": test_name,
                "status": "covered" if has_test(source, test_name) else "missing",
                "evidence": "crates/bijux-cli-core/tests/bin_surface/metadata_inspection_matrix.rs",
            }
        )
    missing_todos = [row for row in todo_rows if row["status"] != "covered"]

    drift_items: list[dict[str, Any]] = []
    if missing_keys:
        drift_items.append({"kind": "missing-inspect-keys", "keys": missing_keys})
    if inspect_route_set != dev_route_set:
        drift_items.append({"kind": "route-identity-mismatch"})
    if reserved_inspect != reserved_registry:
        drift_items.append({"kind": "reserved-namespace-mismatch"})
    if missing_todos:
        drift_items.append({"kind": "missing-todo-coverage", "todos": [row["todo"] for row in missing_todos]})

    drift_artifact = {
        "generator": "scripts/status/generate_metadata_consistency_reports.py",
        "scope": "metadata drift",
        "tasks": [78, 80],
        "status": "clean" if not drift_items else "drift-detected",
        "drift_count": len(drift_items),
        "drift_items": drift_items,
        "todo_coverage": todo_rows,
    }

    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / "command_metadata_artifact.json").write_text(
        json.dumps(command_metadata_artifact, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "route_metadata_artifact.json").write_text(
        json.dumps(route_metadata_artifact, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "metadata_drift_artifact.json").write_text(
        json.dumps(drift_artifact, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "command_ownership_artifact.json").write_text(
        json.dumps(ownership_artifact, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print("wrote artifacts/status/command_metadata_artifact.json")
    print("wrote artifacts/status/route_metadata_artifact.json")
    print("wrote artifacts/status/metadata_drift_artifact.json")
    print("wrote artifacts/status/command_ownership_artifact.json")


if __name__ == "__main__":
    main()

