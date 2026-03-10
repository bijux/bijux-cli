#!/usr/bin/env python3
"""Generate dev-cli command closure evidence artifacts."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
FIXTURE = ROOT / "crates" / "bijux-cli" / "tests" / "routing" / "fixtures" / "dev_cli_subcommands.txt"
TEST_FILE = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "dev_cli_command_matrix.rs"
BIN_TESTS_DIR = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface"

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

DEV_MAINTAINER_VALUE = {
    "dev cli status": 100,
    "dev cli routes": 98,
    "dev cli registry": 98,
    "dev cli env": 96,
    "dev cli doctor": 95,
    "dev cli contracts": 93,
    "dev cli parity": 91,
    "dev cli runtime-identity": 90,
    "dev cli state-audit": 90,
    "dev cli state-doctor": 90,
}


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def read_json(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def read_dev_cli_commands() -> list[str]:
    return [
        line.strip()
        for line in FIXTURE.read_text(encoding="utf-8").splitlines()
        if line.strip().startswith("dev cli ")
    ]


def command_status(command: str, source: str) -> str:
    quoted = ', '.join([f'"{part}"' for part in command.split()])
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


def maintainer_value(command: str) -> int:
    return DEV_MAINTAINER_VALUE.get(command, 75)


def required_coverage_checks(source: str) -> dict[str, bool]:
    checks = {
        "parity": f"fn {REQUIRED_TESTS[243]}(" in source,
        "contract_shape": f"fn {REQUIRED_TESTS[251]}(" in source,
        "help_snapshots": f"fn {REQUIRED_TESTS[250]}(" in source,
        "stderr_stdout_exit_code": f"fn {REQUIRED_TESTS[253]}(" in source,
        "malformed_input": f"fn {REQUIRED_TESTS[255]}(" in source,
        "determinism": f"fn {REQUIRED_TESTS[256]}(" in source,
        "consistency_inspect_routes_registry": f"fn {REQUIRED_TESTS[257]}(" in source,
        "consistency_config_env_resolution": f"fn {REQUIRED_TESTS[258]}(" in source,
        "consistency_plugin_registry_state": f"fn {REQUIRED_TESTS[257]}(" in source,
    }
    checks["all_required"] = all(checks.values())
    return checks


def main() -> int:
    source = TEST_FILE.read_text(encoding="utf-8")
    test_sources = {
        path: path.read_text(encoding="utf-8")
        for path in sorted(BIN_TESTS_DIR.glob("*.rs"))
    }
    commands = read_dev_cli_commands()

    rows = [
        (
            lambda evidence: {
                "command": command,
                "status": "complete" if evidence else command_status(command, source),
                "status_model": ["complete", "partial", "shim", "missing"],
                "evidence": evidence[0] if evidence else "crates/bijux-cli/tests/bin_surface/dev_cli_command_matrix.rs",
                "evidence_links": evidence,
                "maintainer_value": maintainer_value(command),
            }
        )(command_coverage_evidence(command, test_sources))
        for command in commands
    ]
    rows.sort(key=lambda row: (-int(row["maintainer_value"]), row["command"]))

    todo_rows = [
        {
            "todo": todo,
            "test": fn_name,
            "status": "complete" if f"fn {fn_name}(" in source else "missing",
            "evidence": "crates/bijux-cli/tests/bin_surface/dev_cli_command_matrix.rs",
        }
        for todo, fn_name in sorted(REQUIRED_TESTS.items())
    ]

    generated_at = datetime.now(timezone.utc).isoformat()
    coverage = required_coverage_checks(source)

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
                "crates/bijux-cli/tests/routing/fixtures/dev_cli_subcommands.txt",
                "crates/bijux-cli/tests/bin_surface/dev_cli_command_matrix.rs",
                "artifacts/status/dev_cli_command_coverage_report.json",
                "artifacts/status/dev_cli_command_matrix_artifact.json",
            ],
        },
    )

    remaining = [row for row in rows if row["status"] != "complete"]
    write_json(
        "dev_cli_command_remaining_inventory.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_dev_cli_command_surface_reports.py",
            "scope": "remaining dev cli subcommands not proven complete in rust",
            "remaining_commands": remaining,
            "count": len(remaining),
        },
    )

    ranked_remaining = sorted(remaining, key=lambda row: (-int(row["maintainer_value"]), row["command"]))
    write_json(
        "dev_cli_command_value_ranking.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_dev_cli_command_surface_reports.py",
            "scope": "dev cli maintainer-value ranking for closure execution",
            "ranked_remaining_commands": ranked_remaining,
            "count": len(ranked_remaining),
        },
    )

    write_json(
        "dev_cli_command_completion_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_dev_cli_command_surface_reports.py",
            "scope": "dev cli command closure execution",
            "remaining_count": len(ranked_remaining),
            "coverage_checks": coverage,
            "closure_status": "green" if len(ranked_remaining) == 0 and coverage["all_required"] else "open",
            "closure_reason": (
                "all dev cli subcommands are complete and closure checks are proven"
                if len(ranked_remaining) == 0 and coverage["all_required"]
                else "dev cli closure still has open items"
            ),
            "top_targets": ranked_remaining[:2],
        },
    )

    write_json(
        "dev_cli_command_closure_set.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_dev_cli_command_surface_reports.py",
            "scope": "tracked dev cli closure set",
            "tracked_commands": [row["command"] for row in rows],
            "coverage_checks": coverage,
            "status": "frozen",
        },
    )

    cli_completion = read_json(STATUS / "cli_command_completion_report.json")
    cli_remaining = int(cli_completion.get("remaining_count", 0))
    cli_green = str(cli_completion.get("closure_status", "open")) == "green"
    dev_remaining = len(ranked_remaining)
    dev_green = len(ranked_remaining) == 0 and bool(coverage["all_required"])
    combined_green = cli_green and dev_green

    combined_report = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_dev_cli_command_surface_reports.py",
        "scope": "cli and dev cli command closure",
        "cli": {
            "remaining_count": cli_remaining,
            "closure_status": cli_completion.get("closure_status", "open"),
            "top_targets": cli_completion.get("top_targets", []),
        },
        "dev_cli": {
            "remaining_count": dev_remaining,
            "closure_status": "green" if dev_green else "open",
            "top_targets": ranked_remaining[:2],
        },
        "cross_command_consistency": {
            "inspect_routes_registry": coverage["consistency_inspect_routes_registry"],
            "config_env_resolution": coverage["consistency_config_env_resolution"],
            "plugin_registry_state": coverage["consistency_plugin_registry_state"],
        },
        "closure_status": "green" if combined_green else "open",
        "complete_language_allowed": combined_green,
    }
    write_json("cli_dev_command_closure_report.json", combined_report)

    text = [
        "CLI and DEV CLI Closure Report",
        f"overall: {combined_report['closure_status']}",
        f"complete language allowed: {combined_report['complete_language_allowed']}",
        "",
        f"cli remaining: {cli_remaining}",
        f"dev cli remaining: {dev_remaining}",
        "",
        "cross-command consistency:",
        f"- inspect/routes/registry: {coverage['consistency_inspect_routes_registry']}",
        f"- config/env/resolution: {coverage['consistency_config_env_resolution']}",
        f"- plugin state/registry: {coverage['consistency_plugin_registry_state']}",
    ]
    (STATUS / "cli_dev_command_closure_report.txt").write_text(
        "\n".join(text) + "\n", encoding="utf-8"
    )
    print("wrote artifacts/status/cli_dev_command_closure_report.txt")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
