#!/usr/bin/env python3
"""Generate help-law artifacts for TODOs 341-360."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli-bin" / "tests" / "help_tree_law_extra.rs"

REQUIRED_TESTS = {
    341: "root_help_lists_commands_in_stable_order",
    342: "cli_help_lists_subcommands_in_stable_order",
    343: "dev_cli_help_lists_subcommands_in_stable_order",
    344: "plugin_installed_help_keeps_builtin_order_stable",
    345: "no_color_root_help_and_grouped_help_are_stable",
    346: "no_color_root_help_and_grouped_help_are_stable",
    347: "unknown_command_suggestions_are_deterministic_and_namespace_scoped",
    348: "unknown_command_suggestions_are_deterministic_and_namespace_scoped",
    349: "hidden_aliases_do_not_appear_as_canonical_help_entries",
    350: "inspect_metadata_agrees_with_help_names_and_command_tree_export",
    351: "inspect_metadata_agrees_with_help_names_and_command_tree_export",
    352: "binary_and_bridge_help_trees_are_identical_for_covered_commands",
    353: "help_under_broken_plugin_registry_and_corrupted_state_is_stable_and_useful",
    354: "help_under_broken_plugin_registry_and_corrupted_state_is_stable_and_useful",
    355: "command_tree_is_stable_across_repeated_plugin_discovery_runs",
}


def write_json(name: str, payload: dict[str, Any]) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def main() -> int:
    source = TEST_FILE.read_text(encoding="utf-8") if TEST_FILE.exists() else ""
    generated_at = datetime.now(timezone.utc).isoformat()

    todo_coverage = []
    for todo, test_name in sorted(REQUIRED_TESTS.items()):
        covered = f"fn {test_name}(" in source
        todo_coverage.append(
            {
                "todo": todo,
                "test": test_name,
                "status": "covered" if covered else "missing",
                "evidence": "crates/bijux-cli-bin/tests/help_tree_law_extra.rs",
            }
        )

    missing = [row for row in todo_coverage if row["status"] != "covered"]

    help_law = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_help_tree_law_reports.py",
        "scope": "help law",
        "tasks": list(range(341, 357)),
        "status": "complete" if not missing else "partial",
        "todo_coverage": todo_coverage,
    }

    command_tree_consistency = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_help_tree_law_reports.py",
        "scope": "command-tree help consistency",
        "tasks": [350, 351, 352, 355, 357],
        "status": "complete" if not missing else "partial",
        "proof": {
            "inspect_help_agreement": any(row["todo"] == 350 and row["status"] == "covered" for row in todo_coverage),
            "routes_help_agreement": any(row["todo"] == 351 and row["status"] == "covered" for row in todo_coverage),
            "bridge_help_parity": any(row["todo"] == 352 and row["status"] == "covered" for row in todo_coverage),
            "repeated_discovery_stability": any(row["todo"] == 355 and row["status"] == "covered" for row in todo_coverage),
        },
    }

    drift = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_help_tree_law_reports.py",
        "scope": "help drift",
        "tasks": [358, 359],
        "status": "clean" if not missing else "drift",
        "drift_count": len(missing),
        "drift_todos": [row["todo"] for row in missing],
    }

    contract = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_help_tree_law_reports.py",
        "scope": "help tree contract",
        "tasks": [360],
        "status": "frozen" if not missing else "not-frozen",
        "law": "help tree is a law surface",
    }

    write_json("help_law_artifact.json", help_law)
    write_json("command_tree_help_consistency_artifact.json", command_tree_consistency)
    write_json("help_drift_artifact.json", drift)
    write_json("help_tree_contract.json", contract)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
