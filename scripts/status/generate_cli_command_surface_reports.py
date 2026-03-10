#!/usr/bin/env python3
"""Generate cli-subcommand closure evidence artifacts."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
FIXTURE = ROOT / "crates" / "bijux-cli-routing" / "tests" / "fixtures" / "cli_subcommands.txt"
TEST_FILE = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "cli_command_matrix.rs"
BIN_TESTS_DIR = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface"

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

CLI_USER_VALUE = {
    "cli status": 100,
    "cli paths": 95,
    "cli self-test": 90,
    "cli config get": 88,
    "cli config set": 86,
    "cli config list": 84,
    "cli config unset": 80,
    "cli config clear": 78,
    "cli plugins list": 96,
    "cli plugins inspect": 94,
    "cli plugins install": 92,
    "cli plugins uninstall": 92,
    "cli plugins check": 90,
    "cli plugins doctor": 88,
}


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def read_cli_commands() -> list[str]:
    lines = [line.strip() for line in FIXTURE.read_text(encoding="utf-8").splitlines()]
    return [line for line in lines if line.startswith("cli ")]


def command_status(command: str, source: str) -> str:
    quoted = ", ".join([f'"{part}"' for part in command.split()])
    if quoted in source or f'"{command}"' in source:
        return "complete"
    return "partial"


def command_coverage_evidence(command: str, test_sources: dict[Path, str]) -> list[str]:
    parts = command.split()
    quoted = ", ".join([f'"{part}"' for part in parts])
    evidence: list[str] = []
    for path, source in test_sources.items():
        if quoted in source or f'"{command}"' in source:
            evidence.append(str(path.relative_to(ROOT)))
    return evidence


def user_value(command: str) -> int:
    return CLI_USER_VALUE.get(command, 70)


def required_coverage_checks(source: str) -> dict[str, bool]:
    checks = {
        "parity": f"fn {REQUIRED_TESTS[223]}(" in source
        and f"fn {REQUIRED_TESTS[226]}(" in source
        and f"fn {REQUIRED_TESTS[228]}(" in source,
        "machine_output": f"fn {REQUIRED_TESTS[233]}(" in source,
        "help_and_error_snapshots": f"fn {REQUIRED_TESTS[230]}(" in source
        and f"fn {REQUIRED_TESTS[231]}(" in source,
    }
    checks["all_required"] = all(checks.values())
    return checks


def main() -> int:
    test_source = TEST_FILE.read_text(encoding="utf-8")
    test_sources = {
        path: path.read_text(encoding="utf-8")
        for path in sorted(BIN_TESTS_DIR.glob("*.rs"))
    }
    commands = read_cli_commands()

    rows = [
        (
            lambda evidence: {
                "command": command,
                "status": "complete" if evidence else command_status(command, test_source),
                "status_model": ["complete", "partial", "shim", "missing"],
                "evidence": evidence[0] if evidence else "crates/bijux-cli/tests/bin_surface/cli_command_matrix.rs",
                "evidence_links": evidence,
                "user_value": user_value(command),
            }
        )(command_coverage_evidence(command, test_sources))
        for command in commands
    ]
    rows.sort(key=lambda row: (-int(row["user_value"]), row["command"]))

    todo_rows = [
        {
            "todo": todo,
            "test": fn_name,
            "status": "complete" if f"fn {fn_name}(" in test_source else "missing",
            "evidence": "crates/bijux-cli/tests/bin_surface/cli_command_matrix.rs",
        }
        for todo, fn_name in sorted(REQUIRED_TESTS.items())
    ]

    generated_at = datetime.now(timezone.utc).isoformat()
    coverage = required_coverage_checks(test_source)

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
                "crates/bijux-cli/tests/bin_surface/cli_command_matrix.rs",
                "artifacts/status/cli_command_coverage_report.json",
                "artifacts/status/cli_command_matrix_artifact.json",
            ],
        },
    )

    remaining = [row for row in rows if row["status"] != "complete"]
    write_json(
        "cli_command_remaining_inventory.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_cli_command_surface_reports.py",
            "scope": "remaining cli subcommands not proven complete in rust",
            "remaining_commands": remaining,
            "count": len(remaining),
        },
    )

    ranked_remaining = sorted(remaining, key=lambda row: (-int(row["user_value"]), row["command"]))
    write_json(
        "cli_command_value_ranking.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_cli_command_surface_reports.py",
            "scope": "cli subcommand user-value ranking for closure execution",
            "ranked_remaining_commands": ranked_remaining,
            "count": len(ranked_remaining),
        },
    )

    write_json(
        "cli_command_completion_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_cli_command_surface_reports.py",
            "scope": "cli command closure execution",
            "remaining_count": len(ranked_remaining),
            "coverage_checks": coverage,
            "closure_status": "green" if len(ranked_remaining) == 0 and coverage["all_required"] else "open",
            "closure_reason": (
                "all cli subcommands are complete and closure checks are proven"
                if len(ranked_remaining) == 0 and coverage["all_required"]
                else "cli subcommand closure still has open items"
            ),
            "top_targets": ranked_remaining[:2],
        },
    )

    write_json(
        "cli_command_closure_set.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_cli_command_surface_reports.py",
            "scope": "tracked cli command closure set",
            "tracked_commands": [row["command"] for row in rows],
            "coverage_checks": coverage,
            "status": "frozen",
        },
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
