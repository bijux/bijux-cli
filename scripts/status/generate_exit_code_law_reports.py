#!/usr/bin/env python3
"""Generate exit-code contract and drift artifacts for TODOs 21-40."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "exit_code_law_matrix.rs"

MATRIX: dict[str, list[dict[str, Any]]] = {
    "root": [
        {"command": ["version"], "expected_exit_code": 0},
        {"command": ["status"], "expected_exit_code": 0},
        {"command": ["doctor"], "expected_exit_code": 0},
        {"command": ["inspect"], "expected_exit_code": 0},
        {"command": ["docs"], "expected_exit_code": 0},
        {"command": ["audit"], "expected_exit_code": 0},
        {"command": ["sleep", "0"], "expected_exit_code": 0},
        {"command": ["sleep", "--bad-flag"], "expected_exit_code": 2},
    ],
    "cli": [
        {"command": ["cli", "status"], "expected_exit_code": 0},
        {"command": ["cli", "paths"], "expected_exit_code": 0},
        {"command": ["cli", "self-test"], "expected_exit_code": 0},
        {"command": ["cli", "config", "get"], "expected_exit_code": 2},
        {"command": ["cli", "config", "set", "INVALID"], "expected_exit_code": 2},
        {"command": ["cli", "plugins", "list"], "expected_exit_code": 0},
        {"command": ["cli", "plugins", "inspect"], "expected_exit_code": 0},
    ],
    "dev_cli": [
        {"command": ["dev", "cli", "routes"], "expected_exit_code": 0},
        {"command": ["dev", "cli", "registry"], "expected_exit_code": 0},
        {"command": ["dev", "cli", "env"], "expected_exit_code": 0},
        {"command": ["dev", "cli", "doctor"], "expected_exit_code": 0},
        {"command": ["dev", "cli", "contracts"], "expected_exit_code": 0},
        {"command": ["dev", "cli", "status"], "expected_exit_code": 0},
        {"command": ["dev", "cli", "parity"], "expected_exit_code": 0},
        {"command": ["dev", "cli", "does-not-exist"], "expected_exit_code": 2},
    ],
    "plugin_lifecycle": [
        {"command": ["plugins", "list"], "expected_exit_code": 0},
        {"command": ["plugins", "inspect"], "expected_exit_code": 0},
        {"command": ["plugins", "doctor"], "expected_exit_code": 0},
        {"command": ["plugins", "uninstall"], "expected_exit_code": 1},
        {"command": ["plugins", "enable"], "expected_exit_code": 1},
        {"command": ["plugins", "disable"], "expected_exit_code": 1},
    ],
    "config": [
        {"command": ["cli", "config", "list"], "expected_exit_code": 0},
        {"command": ["cli", "config", "get"], "expected_exit_code": 2},
        {"command": ["cli", "config", "set", "INVALID"], "expected_exit_code": 2},
    ],
    "history": [
        {"command": ["history"], "expected_exit_code": 0},
        {"command": ["history", "--format", "json", "--no-pretty"], "expected_exit_code": 0},
        {"command": ["history", "--bad-flag"], "expected_exit_code": 2},
    ],
    "memory": [
        {"command": ["memory"], "expected_exit_code": 0},
        {"command": ["memory", "list", "--format", "json", "--no-pretty"], "expected_exit_code": 0},
        {"command": ["memory", "set"], "expected_exit_code": 2},
    ],
    "diagnostics": [
        {"command": ["inspect"], "expected_exit_code": 0},
        {"command": ["doctor"], "expected_exit_code": 0},
        {"command": ["dev", "cli", "state-doctor", "invalid"], "expected_exit_code": 2},
    ],
}

REQUIRED_TESTS = {
    21: "root_command_exit_code_matrix_is_complete_and_stable",
    22: "cli_command_exit_code_matrix_is_complete_and_stable",
    23: "dev_cli_command_exit_code_matrix_is_complete_and_stable",
    24: "plugin_lifecycle_command_exit_code_matrix_is_complete_and_stable",
    25: "config_history_memory_and_diagnostics_exit_code_matrices_are_complete_and_stable",
    26: "config_history_memory_and_diagnostics_exit_code_matrices_are_complete_and_stable",
    27: "config_history_memory_and_diagnostics_exit_code_matrices_are_complete_and_stable",
    28: "config_history_memory_and_diagnostics_exit_code_matrices_are_complete_and_stable",
    29: "identical_usage_and_validation_failures_map_to_same_code_across_surfaces",
    30: "identical_usage_and_validation_failures_map_to_same_code_across_surfaces",
    31: "identical_plugin_and_internal_failure_classes_map_to_same_code_across_surfaces",
    32: "identical_plugin_and_internal_failure_classes_map_to_same_code_across_surfaces",
    33: "binary_python_bridge_and_repl_agree_on_exit_code_classes_for_covered_commands",
    34: "binary_python_bridge_and_repl_agree_on_exit_code_classes_for_covered_commands",
    35: "machine_readable_and_text_failures_keep_same_exit_codes",
    36: "corrupted_state_and_missing_file_failures_do_not_drift_in_exit_class",
    37: "corrupted_state_and_missing_file_failures_do_not_drift_in_exit_class",
}


def run_exit_code(args: list[str]) -> int:
    result = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli", "--", *args],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    return result.returncode


def has_test(source: str, test_name: str) -> bool:
    return f"fn {test_name}(" in source


def main() -> None:
    source = TEST_FILE.read_text(encoding="utf-8")

    rows: list[dict[str, Any]] = []
    drift_items: list[dict[str, Any]] = []
    for domain, entries in MATRIX.items():
        for entry in entries:
            observed = run_exit_code(entry["command"])
            row = {
                "domain": domain,
                "command": " ".join(entry["command"]),
                "expected_exit_code": entry["expected_exit_code"],
                "observed_exit_code": observed,
                "status": "covered" if observed == entry["expected_exit_code"] else "drift",
            }
            rows.append(row)
            if row["status"] != "covered":
                drift_items.append(row)

    todo_rows = []
    for todo, test_name in sorted(REQUIRED_TESTS.items()):
        todo_rows.append(
            {
                "todo": todo,
                "test_name": test_name,
                "status": "covered" if has_test(source, test_name) else "missing",
                "evidence": "crates/bijux-cli/tests/bin_surface/exit_code_law_matrix.rs",
            }
        )

    missing_todos = [row for row in todo_rows if row["status"] != "covered"]
    contract = {
        "generator": "scripts/status/generate_exit_code_law_reports.py",
        "scope": "exit-code law",
        "status": "complete" if not drift_items and not missing_todos else "partial",
        "tasks": list(range(21, 39)),
        "release_blocking": True,
        "rows": rows,
        "todo_coverage": todo_rows,
        "summary": {
            "domains": sorted(MATRIX.keys()),
            "covered_rows": len(rows) - len(drift_items),
            "drift_rows": len(drift_items),
            "covered_todos": len(todo_rows) - len(missing_todos),
            "missing_todos": len(missing_todos),
        },
    }

    drift = {
        "generator": "scripts/status/generate_exit_code_law_reports.py",
        "scope": "exit-code law drift",
        "status": "clean" if not drift_items else "drift-detected",
        "tasks": [39, 40],
        "drift_count": len(drift_items),
        "drift_items": drift_items,
        "missing_todos": [row["todo"] for row in missing_todos],
    }

    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / "exit_code_contract_artifact.json").write_text(
        json.dumps(contract, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "exit_code_drift_artifact.json").write_text(
        json.dumps(drift, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print("wrote artifacts/status/exit_code_contract_artifact.json")
    print("wrote artifacts/status/exit_code_drift_artifact.json")


if __name__ == "__main__":
    main()

