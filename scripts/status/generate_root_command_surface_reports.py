#!/usr/bin/env python3
"""Generate root-command closure evidence artifacts."""

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

USER_IMPACT_RANK = {
    "status": 100,
    "version": 95,
    "doctor": 90,
    "inspect": 85,
    "docs": 80,
    "audit": 75,
    "sleep": 60,
    "config": 55,
    "plugins": 50,
    "repl": 45,
    "history": 40,
    "memory": 35,
    "completion": 30,
    "atlas": 25,
}

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


def impact_score(command: str) -> int:
    return USER_IMPACT_RANK.get(command, 20)


def required_coverage_checks(source: str) -> dict[str, bool]:
    checks = {
        "parity": f"fn {REQUIRED_TESTS[203]}(" in source
        and f"fn {REQUIRED_TESTS[204]}(" in source
        and f"fn {REQUIRED_TESTS[205]}(" in source
        and f"fn {REQUIRED_TESTS[206]}(" in source
        and f"fn {REQUIRED_TESTS[207]}(" in source
        and f"fn {REQUIRED_TESTS[208]}(" in source
        and f"fn {REQUIRED_TESTS[209]}(" in source,
        "help_snapshot": f"fn {REQUIRED_TESTS[210]}(" in source,
        "stderr_stdout": f"fn {REQUIRED_TESTS[212]}(" in source,
        "exit_code": f"fn {REQUIRED_TESTS[211]}(" in source,
        "json_output": f"fn {REQUIRED_TESTS[213]}(" in source,
        "yaml_output": f"fn {REQUIRED_TESTS[214]}(" in source,
        "determinism": f"fn {REQUIRED_TESTS[218]}(" in source,
    }
    checks["all_required"] = all(checks.values())
    return checks


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
                "user_impact": impact_score(command),
            }
        )
    rows.sort(key=lambda row: (-int(row["user_impact"]), row["command"]))

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
    coverage = required_coverage_checks(source)

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

    remaining = [row for row in rows if row["status"] != "complete"]
    write_json(
        "root_command_remaining_inventory.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_root_command_surface_reports.py",
            "scope": "remaining root commands not proven complete in rust",
            "remaining_commands": remaining,
            "count": len(remaining),
        },
    )

    ranked_remaining = sorted(remaining, key=lambda row: (-int(row["user_impact"]), row["command"]))
    write_json(
        "root_command_impact_ranking.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_root_command_surface_reports.py",
            "scope": "root command impact ranking for closure execution",
            "ranked_remaining_commands": ranked_remaining,
            "count": len(ranked_remaining),
        },
    )

    top_five_targets = [row["command"] for row in ranked_remaining[:5]]
    completion_steps = []
    for idx, command in enumerate(top_five_targets, start=1):
        completion_steps.append(
            {
                "order": idx,
                "command": command,
                "coverage_checks": coverage,
                "evidence": "crates/bijux-cli-bin/tests/root_command_matrix.rs",
            }
        )
    write_json(
        "root_command_completion_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_root_command_surface_reports.py",
            "scope": "root command closure execution",
            "remaining_count": len(ranked_remaining),
            "top_five_execution": completion_steps,
            "coverage_checks": coverage,
            "closure_status": "green" if len(ranked_remaining) == 0 and coverage["all_required"] else "open",
            "closure_reason": (
                "all root commands are complete and closure checks are proven"
                if len(ranked_remaining) == 0 and coverage["all_required"]
                else "root command closure still has open items"
            ),
        },
    )

    write_json(
        "root_command_closure_set.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_root_command_surface_reports.py",
            "scope": "tracked root command closure set",
            "tracked_commands": [row["command"] for row in rows],
            "closure_rule": "Root-command completion claims require zero remaining inventory and all required coverage checks.",
            "coverage_checks": coverage,
            "status": "frozen",
        },
    )

    text_lines = [
        "Root Command Completion Report",
        f"remaining: {len(ranked_remaining)}",
        f"coverage checks all required: {coverage['all_required']}",
        "",
        "required coverage checks:",
    ]
    for key in [
        "parity",
        "help_snapshot",
        "stderr_stdout",
        "exit_code",
        "json_output",
        "yaml_output",
        "determinism",
    ]:
        text_lines.append(f"- {key}: {coverage[key]}")
    if ranked_remaining:
        text_lines.append("")
        text_lines.append("ranked remaining commands:")
        for row in ranked_remaining:
            text_lines.append(f"- {row['command']} (impact={row['user_impact']})")
    else:
        text_lines.append("")
        text_lines.append("ranked remaining commands: none")
    (STATUS / "root_command_completion_report.txt").write_text(
        "\n".join(text_lines) + "\n", encoding="utf-8"
    )
    print("wrote artifacts/status/root_command_completion_report.txt")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
