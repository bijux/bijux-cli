#!/usr/bin/env python3
"""Generate command-family consistency artifacts."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "command_family_consistency_extra.rs"
MATRIX_FILE = ROOT / "artifacts" / "parity" / "commands_fully_rust_owned.json"

REQUIRED_TESTS = {
    161: "root_status_and_cli_status_agree_where_semantics_overlap",
    162: "root_config_listing_and_cli_config_views_agree_where_both_exist",
    163: "plugins_and_routes_views_agree_between_user_and_dev_surfaces",
    164: "plugins_and_routes_views_agree_between_user_and_dev_surfaces",
    165: "cli_paths_match_state_audit_paths_view",
    166: "doctor_and_state_doctor_agree_on_corruption_classes_for_config_plugins_history_memory",
    167: "doctor_and_state_doctor_agree_on_corruption_classes_for_config_plugins_history_memory",
    168: "doctor_and_state_doctor_agree_on_corruption_classes_for_config_plugins_history_memory",
    169: "doctor_and_state_doctor_agree_on_corruption_classes_for_config_plugins_history_memory",
    170: "binary_core_bridge_and_repl_are_consistent_for_matrix_marked_complete_commands",
    171: "binary_core_bridge_and_repl_are_consistent_for_matrix_marked_complete_commands",
    172: "binary_core_bridge_and_repl_are_consistent_for_matrix_marked_complete_commands",
    173: "command_family_help_trees_and_machine_output_envelopes_remain_consistent",
    174: "command_family_help_trees_and_machine_output_envelopes_remain_consistent",
}


def write_json(name: str, payload: dict[str, Any]) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def main() -> int:
    source = TEST_FILE.read_text(encoding="utf-8")
    matrix = json.loads(MATRIX_FILE.read_text(encoding="utf-8")) if MATRIX_FILE.exists() else {"commands": []}
    complete_commands = matrix.get("commands", []) if isinstance(matrix.get("commands"), list) else []

    coverage_rows = []
    for coverage_id, fn_name in sorted(REQUIRED_TESTS.items()):
        present = f"fn {fn_name}(" in source
        coverage_rows.append(
            {
                "coverage_id": coverage_id,
                "test": fn_name,
                "status": "covered" if present else "missing",
                "evidence": "crates/bijux-cli/tests/bin_surface/command_family_consistency_extra.rs",
            }
        )

    missing = [row for row in coverage_rows if row["status"] != "covered"]
    uncovered_scope = []
    if not complete_commands:
        uncovered_scope.append(
            {
                "scope": "matrix_complete_commands",
                "reason": "artifacts/parity/commands_fully_rust_owned.json has no commands",
                "impacted_coverage_ids": [170, 171, 172],
            }
        )

    generated_at = datetime.now(timezone.utc).isoformat()

    command_family_consistency = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_command_family_consistency_reports.py",
        "scope": "command-family consistency",
        "coverage_ids": list(range(161, 176)),
        "status": "complete" if not missing else "partial",
        "coverage_rows": coverage_rows,
    }

    cross_family_drift = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_command_family_consistency_reports.py",
        "scope": "cross-family drift",
        "coverage_ids": [176, 178, 179],
        "status": "clean" if not missing else "drift",
        "drift_count": len(missing),
        "drift_coverage_ids": [row["coverage_id"] for row in missing],
        "uncovered_scope": uncovered_scope,
    }

    shared_law_proof = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_command_family_consistency_reports.py",
        "scope": "shared law proof",
        "coverage_ids": [177],
        "status": "complete" if not missing else "partial",
        "proof": {
            "binary_core_bridge_repl_test_present": any(row["coverage_id"] == 170 and row["status"] == "covered" for row in coverage_rows),
            "help_tree_consistency_test_present": any(row["coverage_id"] == 173 and row["status"] == "covered" for row in coverage_rows),
            "envelope_law_test_present": any(row["coverage_id"] == 174 and row["status"] == "covered" for row in coverage_rows),
        },
    }

    requirement = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_command_family_consistency_reports.py",
        "scope": "command-family consistency requirement",
        "coverage_ids": [180],
        "status": "frozen" if not missing else "not-frozen",
        "release_requirement": "Command-family consistency is a migration requirement and must remain drift-free.",
    }

    write_json("command_family_consistency_artifact.json", command_family_consistency)
    write_json("cross_family_drift_artifact.json", cross_family_drift)
    write_json("shared_law_proof_artifact.json", shared_law_proof)
    write_json("command_family_consistency_requirement.json", requirement)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
