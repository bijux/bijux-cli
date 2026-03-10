#!/usr/bin/env python3
"""Generate root-command coverage/status matrix and explicit domain contract artifacts."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli-bin" / "tests" / "root_command_matrix.rs"

ROOT_COMMANDS = [
    "atlas",
    "audit",
    "completion",
    "config",
    "doctor",
    "docs",
    "history",
    "inspect",
    "memory",
    "plugins",
    "repl",
    "sleep",
    "status",
    "version",
]

REQUIRED_TESTS = {
    203: "parity_version_against_current_expected_behavior",
    204: "parity_status_against_current_expected_behavior",
    205: "parity_doctor_against_current_expected_behavior",
    206: "parity_inspect_against_current_expected_behavior",
    207: "parity_docs_against_current_expected_behavior",
    208: "parity_audit_against_current_expected_behavior",
    209: "parity_sleep_against_current_expected_behavior",
    210: "help_snapshot_exists_for_every_root_command",
    211: "exit_code_and_stream_discipline_for_root_commands",
    212: "exit_code_and_stream_discipline_for_root_commands",
    213: "machine_readable_root_commands_support_json_and_yaml",
    214: "machine_readable_root_commands_support_json_and_yaml",
    215: "quiet_mode_is_supported_for_relevant_root_commands",
    216: "no_color_is_supported_for_text_root_commands",
    217: "malformed_input_is_rejected_for_argument_taking_root_commands",
    218: "repeated_run_determinism_for_machine_readable_root_commands",
    219: "root_command_matrix_artifact_smoke_uses_supported_commands",
}


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def status_for(command: str, tested_roots: set[str]) -> str:
    if command in tested_roots:
        return "complete"
    return "partial"


def main() -> int:
    source = TEST_FILE.read_text(encoding="utf-8")

    tested_roots = {cmd for cmd in ROOT_COMMANDS if f'"{cmd}"' in source}

    rows = []
    for command in ROOT_COMMANDS:
        rows.append(
            {
                "command": command,
                "status": status_for(command, tested_roots),
                "evidence": "crates/bijux-cli-bin/tests/root_command_matrix.rs",
                "status_model": ["complete", "partial", "shim", "missing"],
            }
        )

    todo_rows = []
    for todo, fn_name in sorted(REQUIRED_TESTS.items()):
        todo_rows.append(
            {
                "todo": todo,
                "test": fn_name,
                "status": "complete" if f"fn {fn_name}(" in source else "missing",
                "evidence": "crates/bijux-cli-bin/tests/root_command_matrix.rs",
            }
        )

    generated_at = datetime.now(timezone.utc).isoformat()

    write_json(
        "root_command_coverage_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_root_command_surface_reports.py",
            "scope": "root command coverage",
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
        "root_command_matrix_artifact.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_root_command_surface_reports.py",
            "scope": "todo 201-220 root command matrix",
            "todo_rows": todo_rows,
            "commands": rows,
        },
    )

    write_json(
        "root_command_surface_domain_contract.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_root_command_surface_reports.py",
            "domain": "root-command-surface",
            "status": "frozen",
            "rule": "Root commands are covered by explicit parity, stream, formatting, malformed-input, and determinism tests.",
            "evidence": [
                "crates/bijux-cli-bin/tests/root_command_matrix.rs",
                "artifacts/status/root_command_coverage_report.json",
                "artifacts/status/root_command_matrix_artifact.json",
            ],
        },
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
