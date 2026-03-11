#!/usr/bin/env python3
"""Generate dev-cli extraction boundary inventories and leakage report."""

from __future__ import annotations

import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
DEV_FIXTURE = ROOT / "crates" / "bijux-cli" / "tests" / "routing" / "fixtures" / "dev_cli_subcommands.txt"
CORE_APP = ROOT / "crates" / "bijux-cli" / "src" / "app.rs"

MAINTAINER_DIAGNOSTIC_COMMANDS = {
    "dev cli routes",
    "dev cli route-audit",
    "dev cli registry",
    "dev cli parity",
    "dev cli status",
    "dev cli script-audit",
    "dev cli crate-health",
    "dev cli package-health",
    "dev cli env",
    "dev cli doctor",
    "dev cli contracts",
    "dev cli runtime-identity",
    "dev cli state-audit",
    "dev cli state-doctor",
    "dev cli docs-audit",
}

RUNTIME_OWNED_BEHAVIORS = [
    {
        "behavior": "command routing and normalization",
        "owner": "bijux-cli",
        "evidence": "crates/bijux-cli/src/routing/catalog.rs",
    },
    {
        "behavior": "runtime command execution kernel",
        "owner": "bijux-cli",
        "evidence": "crates/bijux-cli/src/app.rs",
    },
    {
        "behavior": "config persistence and state law",
        "owner": "bijux-cli",
        "evidence": "crates/bijux-cli/src/config",
    },
    {
        "behavior": "plugin registry lifecycle",
        "owner": "bijux-cli-plugin",
        "evidence": "crates/bijux-cli-plugin/src",
    },
    {
        "behavior": "install and runtime identity primitives",
        "owner": "bijux-cli::install",
        "evidence": "crates/bijux-cli/src/install",
    },
    {
        "behavior": "output envelope and rendering",
        "owner": "bijux-cli-output",
        "evidence": "crates/bijux-cli-output/src/lib.rs",
    },
]


def stable_generated_at() -> str:
    source_date_epoch = subprocess.run(
        ["sh", "-lc", 'printf %s "${SOURCE_DATE_EPOCH:-}"'],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if source_date_epoch.isdigit():
        return datetime.fromtimestamp(int(source_date_epoch), tz=timezone.utc).isoformat()
    return datetime.now(timezone.utc).isoformat()


def write_json(name: str, payload: dict[str, Any]) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def read_dev_cli_commands() -> list[str]:
    commands = [
        line.strip()
        for line in DEV_FIXTURE.read_text(encoding="utf-8").splitlines()
        if line.strip().startswith("dev cli ")
    ]
    return sorted(set(commands))


def parse_dev_cli_implementations() -> dict[str, str]:
    source = CORE_APP.read_text(encoding="utf-8")
    pattern = re.compile(
        r'\[a, b, c\]\s+if\s+a\s*==\s*"dev"\s*&&\s*b\s*==\s*"cli"\s*&&\s*c\s*==\s*"([^"]+)"'
    )
    implementations: dict[str, str] = {}
    for command in pattern.findall(source):
        key = f"dev cli {command}"
        implementations[key] = "bijux-cli"

    for command in ["dev cli route-audit"]:
        if command in implementations:
            implementations[command] = "bijux-cli::routing + bijux-cli"

    for command in ["dev cli runtime-identity", "dev cli package-health", "dev cli state-audit", "dev cli state-doctor"]:
        if command in implementations:
            implementations[command] = "bijux-cli + bijux-cli::install + bijux-cli-plugin"

    delegated = {
        "dev cli routes": "dev_routes::build_report",
        "dev cli registry": "dev_registry::build_report",
        "dev cli env": "dev_env::build_report",
        "dev cli contracts": "dev_contracts::build_report",
        "dev cli parity": "dev_parity::build_report",
        "dev cli status": "dev_status::build_report",
        "dev cli runtime-identity": "dev_runtime_identity::build_report",
        "dev cli package-health": "dev_package_health::build_report",
        "dev cli state-audit": "dev_state_audit::build_report",
        "dev cli state-doctor": "dev_state_audit::build_doctor_report",
        "dev cli script-audit": "dev_script_audit::build_report",
        "dev cli docs-audit": "dev_docs_audit::build_report",
        "dev cli crate-health": "dev_crate_health::build_report",
        "dev cli inventory": "dev_script_audit::build_inventory_report",
    }
    for command, marker in delegated.items():
        if command in implementations and marker in source:
            implementations[command] = "bijux-dev-cli + runtime-data-providers"

    delegated_query_path = {
        "dev cli routes": "dev_routes::build_report_from_query",
        "dev cli registry": "dev_registry::build_report_from_query",
        "dev cli route-audit": "dev_route_audit::build_report_from_query",
    }
    for command, marker in delegated_query_path.items():
        if command in implementations and marker in source:
            implementations[command] = "bijux-dev-cli + runtime-data-providers"

    return implementations


def parse_script_replacements() -> list[dict[str, str]]:
    source = CORE_APP.read_text(encoding="utf-8")
    pattern = re.compile(r'\{"from":\s*"([^"]+)",\s*"to":\s*"([^"]+)"\}')
    replacements = [{"from": src, "to": dst} for src, dst in pattern.findall(source)]
    replacements.sort(key=lambda row: row["from"])
    return replacements


def classify_script(path: str) -> str:
    if path.startswith("scripts/status/"):
        return "should-move-to-dev-cli"
    if path.startswith("scripts/docs_builder/"):
        return "keep-as-script"
    if path == "scripts/__init__.py":
        return "delete"
    return "should-move-to-dev-cli"


def script_inventory() -> tuple[list[dict[str, str]], list[dict[str, str]]]:
    replaced = parse_script_replacements()
    replacements = {row["from"]: row["to"] for row in replaced}

    remaining: list[dict[str, str]] = []
    for path in sorted((ROOT / "scripts").rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(ROOT).as_posix()
        classification = classify_script(rel)
        if classification == "should-move-to-dev-cli" and rel not in replacements:
            remaining.append(
                {
                    "path": rel,
                    "target": "bijux dev cli <subcommand>",
                    "classification": classification,
                }
            )

    return replaced, remaining


def main() -> int:
    generated_at = stable_generated_at()
    generator = "scripts/status/generate_dev_cli_boundary_reports.py"
    commands = read_dev_cli_commands()
    implementations = parse_dev_cli_implementations()
    replaced_scripts, remaining_scripts = script_inventory()

    dev_rows: list[dict[str, Any]] = []
    misplaced_rows: list[dict[str, Any]] = []
    missing_implementations: list[str] = []

    for command in commands:
        implementation = implementations.get(command, "unmapped")
        if implementation == "unmapped":
            missing_implementations.append(command)
        behavior_kind = "diagnostic" if command in MAINTAINER_DIAGNOSTIC_COMMANDS else "automation"
        row: dict[str, Any] = {
            "command": command,
            "behavior_kind": behavior_kind,
            "intended_owner": "maintainer-control-plane",
            "current_owner": implementation,
            "leaks_through_runtime": not implementation.startswith("bijux-dev-cli"),
            "exposed_through_binary": True,
            "evidence": [
                "crates/bijux-cli/tests/routing/fixtures/dev_cli_subcommands.txt",
                "crates/bijux-cli/src/app.rs",
            ],
        }
        dev_rows.append(row)

        if row["leaks_through_runtime"]:
            misplaced_rows.append(
                {
                    "behavior": command,
                    "expected_owner": "bijux-dev-cli",
                    "current_owner": implementation,
                    "reason": "maintainer behavior still implemented in runtime crates",
                    "severity": "must-move",
                }
            )

    boundary_rules = {
        "control_plane_owner": "bijux-dev-cli owns maintainer automation and report assembly",
        "runtime_scope": "runtime crates own runtime law and structured-data services, not maintainer workflows",
        "canonical_surface": "bijux dev cli remains the canonical maintainer command surface",
        "distribution": "bijux-dev-cli is a workspace crate, not a second public binary package",
        "binary_identity": "bijux remains the only canonical executable",
        "law_center": "bijux-dev-cli does not become a second runtime law center",
    }

    write_json(
        "dev_cli_owned_behaviors_inventory.json",
        {
            "generated_at": generated_at,
            "generator": generator,
            "scope": "dev-cli maintainer-owned behavior inventory",
            "commands": dev_rows,
            "maintainer_only_commands_implemented_in_runtime_crates": [
                row["command"] for row in dev_rows if row["leaks_through_runtime"]
            ],
            "maintainer_only_diagnostics_exposed_from_bin": sorted(MAINTAINER_DIAGNOSTIC_COMMANDS),
            "maintainer_report_artifact_generators": [
                "dev cli parity",
                "dev cli status",
                "dev cli route-audit",
                "dev cli state-audit",
                "dev cli script-audit",
                "dev cli crate-health",
                "dev cli package-health",
                "dev cli docs-audit",
            ],
            "script_replacements_already_covered_by_dev_cli": replaced_scripts,
            "remaining_scripts_to_move_into_dev_cli": remaining_scripts,
            "boundary_rules": boundary_rules,
            "boundary_frozen": True,
            "missing_implementation_mappings": missing_implementations,
        },
    )

    write_json(
        "runtime_owned_behaviors_inventory.json",
        {
            "generated_at": generated_at,
            "generator": generator,
            "scope": "runtime-owned behaviors",
            "behaviors": RUNTIME_OWNED_BEHAVIORS,
            "rules": {
                "runtime_crates_do_not_own_maintainer_workflows": True,
                "runtime_crates_expose_structured_data_only_for_maintainer_reports": True,
            },
        },
    )

    write_json(
        "misplaced_dev_behaviors_report.json",
        {
            "generated_at": generated_at,
            "generator": generator,
            "scope": "misplaced maintainer behavior still implemented in runtime crates",
            "misplaced_behaviors": misplaced_rows,
            "summary": {
                "total_dev_cli_commands": len(dev_rows),
                "misplaced_count": len(misplaced_rows),
            },
            "boundary_freeze": {
                "status": "frozen-before-extraction",
                "rule": "boundary inventory must be generated and reviewed before moving implementation",
            },
        },
    )

    write_json(
        "dev_cli_maintainer_command_ownership_report.json",
        {
            "generated_at": generated_at,
            "generator": generator,
            "scope": "maintainer inventory command ownership",
            "maintainer_inventory_commands": [
                "dev cli inventory",
                "dev cli script-audit",
                "dev cli docs-audit",
                "dev cli crate-health",
                "dev cli package-health",
                "dev cli runtime-identity",
                "dev cli state-audit",
                "dev cli state-doctor",
            ],
            "owned_by_bijux_dev_cli": [
                row["command"] for row in dev_rows if str(row["current_owner"]).startswith("bijux-dev-cli")
            ],
            "not_yet_owned_by_bijux_dev_cli": [
                row["command"] for row in dev_rows if not str(row["current_owner"]).startswith("bijux-dev-cli")
            ],
            "owned_maintainer_inventory_commands": [
                cmd
                for cmd in [
                    "dev cli inventory",
                    "dev cli script-audit",
                    "dev cli docs-audit",
                    "dev cli crate-health",
                    "dev cli package-health",
                    "dev cli runtime-identity",
                    "dev cli state-audit",
                    "dev cli state-doctor",
                ]
                if cmd
                in {
                    row["command"] for row in dev_rows if str(row["current_owner"]).startswith("bijux-dev-cli")
                }
            ],
        },
    )

    print("wrote artifacts/status/dev_cli_owned_behaviors_inventory.json")
    print("wrote artifacts/status/runtime_owned_behaviors_inventory.json")
    print("wrote artifacts/status/misplaced_dev_behaviors_report.json")
    print("wrote artifacts/status/dev_cli_maintainer_command_ownership_report.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
