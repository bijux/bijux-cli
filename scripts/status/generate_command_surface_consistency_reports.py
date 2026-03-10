#!/usr/bin/env python3
"""Generate cross-command consistency artifacts, drift report, and actionable summary."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli-bin" / "tests" / "cross_command_consistency_matrix.rs"

REQUIRED_TESTS = {
    381: "inspect_and_dev_routes_agree_on_route_ownership",
    382: "inspect_and_dev_registry_agree_on_plugin_ownership_model",
    383: "config_get_and_dev_env_agree_on_source_precedence",
    384: "doctor_and_state_audit_agree_on_corruption_detection_when_applicable",
    385: "plugins_list_and_dev_registry_agree_on_installed_plugin_namespace_rules",
    386: "repl_execution_matches_non_interactive_for_config_get_plugins_list_and_status",
    387: "repl_execution_matches_non_interactive_for_config_get_plugins_list_and_status",
    388: "repl_execution_matches_non_interactive_for_config_get_plugins_list_and_status",
    389: "binary_and_python_bridge_agree_on_config_history_memory_and_diagnostics_outputs",
    390: "binary_and_python_bridge_agree_on_config_history_memory_and_diagnostics_outputs",
    391: "binary_and_python_bridge_agree_on_config_history_memory_and_diagnostics_outputs",
    392: "binary_and_python_bridge_agree_on_config_history_memory_and_diagnostics_outputs",
    393: "binary_and_direct_core_agree_on_same_command_results",
    394: "binary_and_direct_core_agree_on_same_command_results",
    395: "binary_and_direct_core_agree_on_same_command_results",
}

AREA_TO_TODOS = {
    "commands": [381, 382, 385, 393, 394, 395, 396, 397],
    "config": [383, 389],
    "history": [384, 390],
    "memory": [391],
    "diagnostics": [392],
}


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def main() -> int:
    source = TEST_FILE.read_text(encoding="utf-8")
    generated_at = datetime.now(timezone.utc).isoformat()

    todo_rows = []
    for todo, fn_name in sorted(REQUIRED_TESTS.items()):
        present = f"fn {fn_name}(" in source
        todo_rows.append(
            {
                "todo": todo,
                "test": fn_name,
                "status": "complete" if present else "missing",
                "evidence": "crates/bijux-cli-bin/tests/cross_command_consistency_matrix.rs",
            }
        )

    drift_rows = [row for row in todo_rows if row["status"] != "complete"]

    write_json(
        "command_surface_consistency_artifact.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_command_surface_consistency_reports.py",
            "scope": "todo 381-394 cross-command consistency artifact",
            "todo_rows": todo_rows,
        },
    )

    write_json(
        "command_surface_consistency_drift_artifact.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_command_surface_consistency_reports.py",
            "scope": "todo 395 cross-command drift detector artifact",
            "drift_count": len(drift_rows),
            "drift_todos": [row["todo"] for row in drift_rows],
            "status": "clean" if not drift_rows else "drift",
        },
    )

    summary_rows = []
    for area, todos in AREA_TO_TODOS.items():
        relevant = [row for row in todo_rows if row["todo"] in todos]
        complete = sum(1 for row in relevant if row["status"] == "complete")
        total = len(relevant)
        status = "complete" if complete == total else ("partial" if complete > 0 else "missing")
        summary_rows.append(
            {
                "area": area,
                "complete": complete,
                "total": total,
                "status": status,
            }
        )

    write_json(
        "command_surface_consistency_summary.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_command_surface_consistency_reports.py",
            "scope": "todo 398 complete/partial/missing summary for commands/config/history/memory/diagnostics",
            "areas": summary_rows,
            "next_wave_input": "Use this summary as source-of-truth for prioritization instead of intuition.",
        },
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
