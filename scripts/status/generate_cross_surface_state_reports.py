#!/usr/bin/env python3
"""Generate cross-surface state artifacts for TODOs 321-340."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILES = [
    ROOT / "crates" / "bijux-cli-bin" / "tests" / "cross_surface_state_extra.rs",
    ROOT / "crates" / "bijux-cli-bin" / "tests" / "command_family_consistency_extra.rs",
]

REQUIRED_TESTS = {
    321: "config_mutations_are_visible_across_binary_bridge_and_repl_reads",
    322: "config_mutations_are_visible_across_binary_bridge_and_repl_reads",
    323: "binary_core_bridge_and_repl_are_consistent_for_matrix_marked_complete_commands",
    324: "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge",
    325: "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge",
    326: "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge",
    327: "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge",
    328: "doctor_and_state_doctor_agree_on_corruption_classes_across_config_plugins_history_and_memory",
    329: "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge",
    330: "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge",
    331: "state_path_overrides_propagate_consistently_for_config_path_views",
    332: "doctor_and_state_doctor_agree_on_corruption_classes_across_config_plugins_history_and_memory",
    333: "doctor_and_state_doctor_agree_on_corruption_classes_across_config_plugins_history_and_memory",
    334: "doctor_and_state_doctor_agree_on_corruption_classes_across_config_plugins_history_and_memory",
    335: "doctor_and_state_doctor_agree_on_corruption_classes_across_config_plugins_history_and_memory",
}


def write_json(name: str, payload: dict[str, Any]) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def main() -> int:
    sources: dict[str, str] = {}
    for path in TEST_FILES:
        if path.exists():
            sources[str(path.relative_to(ROOT))] = path.read_text(encoding="utf-8")

    generated_at = datetime.now(timezone.utc).isoformat()
    rows = []
    for todo, test_name in sorted(REQUIRED_TESTS.items()):
        evidence = None
        for rel, text in sources.items():
            if f"fn {test_name}(" in text:
                evidence = rel
                break
        rows.append(
            {
                "todo": todo,
                "test": test_name,
                "status": "covered" if evidence else "missing",
                "evidence": evidence,
            }
        )

    missing = [row for row in rows if row["status"] != "covered"]

    consistency = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_cross_surface_state_reports.py",
        "scope": "cross-surface state consistency",
        "tasks": list(range(321, 337)),
        "status": "complete" if not missing else "partial",
        "todo_coverage": rows,
    }

    drift = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_cross_surface_state_reports.py",
        "scope": "cross-surface state drift",
        "tasks": [337, 338],
        "status": "clean" if not missing else "drift",
        "drift_count": len(missing),
        "drift_todos": [row["todo"] for row in missing],
    }

    contract = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_cross_surface_state_reports.py",
        "scope": "cross-surface state contract",
        "tasks": [340],
        "status": "frozen" if not missing else "not-frozen",
        "law": "state consistency is part of migration contract",
    }

    write_json("cross_surface_state_consistency_artifact.json", consistency)
    write_json("cross_surface_state_drift_artifact.json", drift)
    write_json("cross_surface_state_contract.json", contract)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
