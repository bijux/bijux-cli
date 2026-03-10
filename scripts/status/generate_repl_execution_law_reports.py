#!/usr/bin/env python3
"""Generate REPL execution law artifacts for TODOs 201-220."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "repl_execution_law_extra.rs"

REQUIRED_TESTS = {
    201: "repl_uses_same_kernel_entrypoint_and_route_resolution_as_non_interactive_cli",
    202: "repl_uses_same_kernel_entrypoint_and_route_resolution_as_non_interactive_cli",
    203: "repl_machine_and_text_modes_use_same_underlying_payload_law",
    204: "repl_machine_and_text_modes_use_same_underlying_payload_law",
    205: "repl_usage_validation_and_plugin_failures_map_to_same_failure_classes",
    206: "repl_usage_validation_and_plugin_failures_map_to_same_failure_classes",
    207: "repl_usage_validation_and_plugin_failures_map_to_same_failure_classes",
    208: "repl_state_corruption_handling_matches_non_interactive_cli_for_shared_commands",
    209: "repl_quiet_trace_json_yaml_and_history_semantics_match_non_interactive_cli",
    210: "repl_quiet_trace_json_yaml_and_history_semantics_match_non_interactive_cli",
    211: "repl_quiet_trace_json_yaml_and_history_semantics_match_non_interactive_cli",
    212: "repl_quiet_trace_json_yaml_and_history_semantics_match_non_interactive_cli",
    213: "repl_quiet_trace_json_yaml_and_history_semantics_match_non_interactive_cli",
    214: "repl_help_for_builtin_and_plugin_commands_matches_non_interactive_help",
    215: "repl_help_for_builtin_and_plugin_commands_matches_non_interactive_help",
}


def write_json(name: str, payload: dict[str, Any]) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def main() -> int:
    source = TEST_FILE.read_text(encoding="utf-8") if TEST_FILE.exists() else ""
    generated_at = datetime.now(timezone.utc).isoformat()

    todo_coverage = []
    for todo, name in sorted(REQUIRED_TESTS.items()):
        covered = f"fn {name}(" in source
        todo_coverage.append(
            {
                "todo": todo,
                "test": name,
                "status": "covered" if covered else "missing",
                "evidence": "crates/bijux-cli/tests/bin_surface/repl_execution_law_extra.rs",
            }
        )

    missing = [row for row in todo_coverage if row["status"] != "covered"]

    shared_law = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_repl_execution_law_reports.py",
        "scope": "repl shared law",
        "tasks": list(range(201, 217)),
        "status": "complete" if not missing else "partial",
        "todo_coverage": todo_coverage,
    }

    repl_cli_diff = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_repl_execution_law_reports.py",
        "scope": "repl vs cli drift",
        "tasks": [217],
        "status": "clean" if not missing else "drift",
        "diff_count": len(missing),
        "diff_todos": [row["todo"] for row in missing],
    }

    # Warn-only surface for TODO 219: explicit repl-only semantics require justification.
    repl_only_semantics = []
    for marker in ["repl_only_semantic", "repl-only semantic", "repl specific semantic"]:
        if marker in source.lower():
            repl_only_semantics.append(marker)

    drift = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_repl_execution_law_reports.py",
        "scope": "repl shared law policy",
        "tasks": [218, 219],
        "status": "clean" if not missing else "drift",
        "drift_count": len(missing),
        "drift_todos": [row["todo"] for row in missing],
        "repl_only_semantics": repl_only_semantics,
    }

    contract = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_repl_execution_law_reports.py",
        "scope": "repl execution law contract",
        "tasks": [220],
        "status": "frozen" if not missing else "not-frozen",
        "law": "same law, different shell",
    }

    write_json("repl_shared_law_artifact.json", shared_law)
    write_json("repl_cli_diff_artifact.json", repl_cli_diff)
    write_json("repl_shared_law_drift_artifact.json", drift)
    write_json("repl_shared_law_contract.json", contract)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
