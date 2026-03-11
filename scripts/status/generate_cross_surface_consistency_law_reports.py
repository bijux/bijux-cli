#!/usr/bin/env python3
"""Generate cross-surface consistency artifacts."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "cross_command_consistency_matrix.rs"
MIGRATION = ROOT / "artifacts" / "status" / "command_migration_matrix.json"

REQUIRED_TESTS: list[tuple[int, str, str, list[str]]] = [
    (141, "inspect_and_dev_routes_agree_on_route_ownership", "inspect/dev routes ownership agreement", ["inspect", "dev cli routes"]),
    (142, "plugins_list_and_dev_registry_agree_on_installed_plugin_namespace_rules", "plugins list/dev registry installed set agreement", ["plugins list", "dev cli registry"]),
    (143, "config_get_and_dev_env_agree_on_source_precedence", "config get/dev env precedence agreement", ["config get", "dev cli env"]),
    (144, "doctor_and_state_audit_agree_on_corruption_detection_when_applicable", "doctor/state-audit corruption agreement", ["doctor", "dev cli state-audit"]),
    (145, "binary_and_direct_core_agree_on_same_command_results", "binary/direct-core agreement for covered roots", ["status"]),
    (146, "binary_and_python_bridge_agree_on_config_history_memory_and_diagnostics_outputs", "binary/python-bridge agreement for covered roots", ["config", "history", "memory list", "doctor"]),
    (147, "repl_execution_matches_non_interactive_for_config_get_plugins_list_and_status", "binary/repl agreement for shared commands", ["config get", "plugins list", "status"]),
    (148, "plugin_command_help_integrates_into_root_help_tree_deterministically", "plugin help integration is deterministic", ["plugins"]),
    (149, "command_tree_export_is_identical_across_binary_and_bridge", "command-tree export identical across binary and bridge", ["dev cli routes"]),
    (150, "route_ownership_is_stable_across_repeated_runs", "route ownership stable across repeated runs", ["dev cli routes"]),
    (151, "command_metadata_is_stable_across_repeated_runs", "command metadata stable across repeated runs", ["inspect"]),
    (152, "diagnostics_payloads_do_not_drift_across_surfaces", "diagnostics payloads stable across surfaces", ["doctor"]),
    (153, "output_envelopes_do_not_drift_across_surfaces", "output envelopes stable across surfaces", ["unknown-command"]),
    (154, "exit_code_classes_do_not_drift_across_surfaces", "exit-code classes stable across surfaces", ["status", "unknown-command"]),
]


def read_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def migration_status(command: str, matrix: dict[str, Any]) -> str:
    for row in matrix.get("rows", []):
        if isinstance(row, dict) and str(row.get("command", "")).strip() == command:
            return str(row.get("status", "rust-partial"))
    return "rust-partial"


def main() -> int:
    source = TEST_FILE.read_text(encoding="utf-8")
    matrix = read_json(MIGRATION)

    rows: list[dict[str, Any]] = []
    drift_items: list[dict[str, Any]] = []
    warnings: list[dict[str, Any]] = []

    for coverage_id, fn_name, law, related in REQUIRED_TESTS:
        present = f"fn {fn_name}(" in source
        related_statuses = [migration_status(cmd, matrix) for cmd in related]
        coverage_class = (
            "covered"
            if related_statuses and all(status == "rust-complete" for status in related_statuses)
            else "partial"
        )
        row = {
            "coverage_id": coverage_id,
            "law": law,
            "test": f"crates/bijux-cli/tests/bin_surface/cross_command_consistency_matrix.rs::{fn_name}",
            "present": present,
            "coverage_class": coverage_class,
            "related_commands": related,
            "related_command_statuses": related_statuses,
        }
        rows.append(row)
        if not present:
            drift_items.append(row)
            if coverage_class == "partial":
                warnings.append(row)

    consistency = {
        "generator": "scripts/status/generate_cross_surface_consistency_law_reports.py",
        "scope": "cross-surface consistency",
        "status": "clean" if not drift_items else "drift",
        "rows": rows,
        "summary": {
            "required": len(REQUIRED_TESTS),
            "covered": sum(1 for row in rows if row["present"]),
            "missing": len(drift_items),
        },
    }

    drift = {
        "generator": "scripts/status/generate_cross_surface_consistency_law_reports.py",
        "scope": "cross-surface drift",
        "status": "clean" if not drift_items else "drift",
        "drift_count": len(drift_items),
        "drift_items": drift_items,
        "warnings_for_partial": warnings,
    }

    contract = {
        "generator": "scripts/status/generate_cross_surface_consistency_law_reports.py",
        "scope": "cross-surface consistency contract",
        "release_review_rule": "cross-surface consistency artifacts are mandatory release evidence",
        "freeze_rule": "one command law is frozen only when covered drift remains zero",
        "gate": "scripts/status/enforce_cross_surface_consistency_law.py --enforce",
        "evidence": [
            "artifacts/status/cross_surface_consistency_artifact.json",
            "artifacts/status/cross_surface_drift_artifact.json",
            "artifacts/status/cross_surface_consistency_contract.json",
        ],
    }

    write_json(STATUS / "cross_surface_consistency_artifact.json", consistency)
    write_json(STATUS / "cross_surface_drift_artifact.json", drift)
    write_json(STATUS / "cross_surface_consistency_contract.json", contract)
    print("wrote artifacts/status/cross_surface_consistency_artifact.json")
    print("wrote artifacts/status/cross_surface_drift_artifact.json")
    print("wrote artifacts/status/cross_surface_consistency_contract.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
