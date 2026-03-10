#!/usr/bin/env python3
"""Generate dev-cli command coverage/matrix artifacts and freeze maintainer control surface."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
FIXTURE = ROOT / "crates" / "bijux-cli-routing" / "tests" / "fixtures" / "dev_cli_subcommands.txt"
TEST_FILE = ROOT / "crates" / "bijux-cli-bin" / "tests" / "dev_cli_command_matrix.rs"

REQUIRED_TESTS = {
    243: "parity_for_key_dev_cli_commands_against_current_behavior",
    244: "parity_for_key_dev_cli_commands_against_current_behavior",
    245: "parity_for_key_dev_cli_commands_against_current_behavior",
    246: "parity_for_key_dev_cli_commands_against_current_behavior",
    247: "parity_for_key_dev_cli_commands_against_current_behavior",
    248: "parity_for_key_dev_cli_commands_against_current_behavior",
    249: "parity_for_key_dev_cli_commands_against_current_behavior",
    250: "help_snapshots_exist_for_all_dev_cli_subcommands",
    251: "json_and_text_outputs_are_available_for_machine_and_text_heavy_dev_cli_commands",
    252: "json_and_text_outputs_are_available_for_machine_and_text_heavy_dev_cli_commands",
    253: "stderr_stdout_and_exit_code_discipline_for_dev_cli_commands",
    254: "stderr_stdout_and_exit_code_discipline_for_dev_cli_commands",
    255: "malformed_input_is_rejected_for_dev_cli_subcommands",
    256: "repeated_run_determinism_for_machine_readable_dev_cli_commands",
    257: "consistency_across_dev_cli_routes_inspect_and_registry_state",
    258: "consistency_across_dev_cli_env_and_config_resolution_paths",
    259: "dev_cli_command_matrix_artifact_smoke_uses_supported_commands",
}


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def read_dev_cli_commands() -> list[str]:
    return [
        line.strip()
        for line in FIXTURE.read_text(encoding="utf-8").splitlines()
        if line.strip().startswith("dev cli ")
    ]


def command_status(command: str, source: str) -> str:
    quoted = ', '.join([f'"{part}"' for part in command.split()])
    if quoted in source:
        return "complete"
    return "partial"


def main() -> int:
    source = TEST_FILE.read_text(encoding="utf-8")
    commands = read_dev_cli_commands()

    rows = [
        {
            "command": command,
            "status": command_status(command, source),
            "status_model": ["complete", "partial", "shim", "missing"],
            "evidence": "crates/bijux-cli-bin/tests/dev_cli_command_matrix.rs",
        }
        for command in commands
    ]

    todo_rows = [
        {
            "todo": todo,
            "test": fn_name,
            "status": "complete" if f"fn {fn_name}(" in source else "missing",
            "evidence": "crates/bijux-cli-bin/tests/dev_cli_command_matrix.rs",
        }
        for todo, fn_name in sorted(REQUIRED_TESTS.items())
    ]

    generated_at = datetime.now(timezone.utc).isoformat()

    write_json(
        "dev_cli_command_coverage_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_dev_cli_command_surface_reports.py",
            "scope": "dev cli command coverage",
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
        "dev_cli_command_matrix_artifact.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_dev_cli_command_surface_reports.py",
            "scope": "todo 241-260 dev cli command matrix",
            "todo_rows": todo_rows,
            "commands": rows,
        },
    )

    write_json(
        "dev_cli_command_surface_domain_contract.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_dev_cli_command_surface_reports.py",
            "domain": "dev-cli-command-surface",
            "status": "frozen",
            "rule": "dev cli commands are the maintainer control surface and must keep parity, diagnostics, and deterministic output law.",
            "evidence": [
                "crates/bijux-cli-routing/tests/fixtures/dev_cli_subcommands.txt",
                "crates/bijux-cli-bin/tests/dev_cli_command_matrix.rs",
                "artifacts/status/dev_cli_command_coverage_report.json",
                "artifacts/status/dev_cli_command_matrix_artifact.json",
            ],
        },
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
