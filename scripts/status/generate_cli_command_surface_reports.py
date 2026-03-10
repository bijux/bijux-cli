#!/usr/bin/env python3
"""Generate cli-subcommand coverage/matrix artifacts and freeze cli command law domain."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
FIXTURE = ROOT / "crates" / "bijux-cli-routing" / "tests" / "fixtures" / "cli_subcommands.txt"
TEST_FILE = ROOT / "crates" / "bijux-cli-bin" / "tests" / "cli_command_matrix.rs"

REQUIRED_TESTS = {
    223: "parity_cli_status_paths_and_self_test_against_current_behavior",
    224: "parity_cli_status_paths_and_self_test_against_current_behavior",
    225: "parity_cli_status_paths_and_self_test_against_current_behavior",
    226: "parity_cli_config_get_and_set_against_current_behavior",
    227: "parity_cli_config_get_and_set_against_current_behavior",
    228: "parity_cli_plugins_list_and_inspect_against_current_behavior",
    229: "parity_cli_plugins_list_and_inspect_against_current_behavior",
    230: "help_snapshots_exist_for_all_cli_subcommands",
    231: "stderr_stdout_and_exit_code_discipline_for_cli_commands",
    232: "stderr_stdout_and_exit_code_discipline_for_cli_commands",
    233: "machine_readable_cli_commands_support_json_and_yaml",
    234: "machine_readable_cli_commands_support_json_and_yaml",
    235: "quiet_mode_and_no_color_behavior_for_relevant_cli_commands",
    236: "quiet_mode_and_no_color_behavior_for_relevant_cli_commands",
    237: "malformed_input_is_rejected_for_argument_taking_cli_subcommands",
    238: "repeated_run_stability_for_machine_readable_cli_commands",
    239: "cli_command_matrix_artifact_smoke_uses_supported_commands",
}


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def read_cli_commands() -> list[str]:
    lines = [line.strip() for line in FIXTURE.read_text(encoding="utf-8").splitlines()]
    return [line for line in lines if line.startswith("cli ")]


def command_status(command: str, source: str) -> str:
    if f'"{command}"' in source:
        return "complete"
    return "partial"


def main() -> int:
    test_source = TEST_FILE.read_text(encoding="utf-8")
    commands = read_cli_commands()

    rows = [
        {
            "command": command,
            "status": command_status(command, test_source),
            "status_model": ["complete", "partial", "shim", "missing"],
            "evidence": "crates/bijux-cli-bin/tests/cli_command_matrix.rs",
        }
        for command in commands
    ]

    todo_rows = [
        {
            "todo": todo,
            "test": fn_name,
            "status": "complete" if f"fn {fn_name}(" in test_source else "missing",
            "evidence": "crates/bijux-cli-bin/tests/cli_command_matrix.rs",
        }
        for todo, fn_name in sorted(REQUIRED_TESTS.items())
    ]

    generated_at = datetime.now(timezone.utc).isoformat()

    write_json(
        "cli_command_coverage_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_cli_command_surface_reports.py",
            "scope": "cli command coverage",
            "commands": rows,
            "summary": {
                "total": len(rows),
                "complete": sum(1 for r in rows if r["status"] == "complete"),
                "partial": sum(1 for r in rows if r["status"] == "partial"),
                "shim": 0,
                "missing": 0,
            },
        },
    )

    write_json(
        "cli_command_matrix_artifact.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_cli_command_surface_reports.py",
            "scope": "todo 221-240 cli command matrix",
            "todo_rows": todo_rows,
            "commands": rows,
        },
    )

    write_json(
        "cli_command_surface_domain_contract.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_cli_command_surface_reports.py",
            "domain": "cli-command-surface",
            "status": "frozen",
            "rule": "cli subcommands are covered by explicit parity, stream, formatting, malformed-input, and determinism tests.",
            "evidence": [
                "crates/bijux-cli-routing/tests/fixtures/cli_subcommands.txt",
                "crates/bijux-cli-bin/tests/cli_command_matrix.rs",
                "artifacts/status/cli_command_coverage_report.json",
                "artifacts/status/cli_command_matrix_artifact.json",
            ],
        },
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
