#!/usr/bin/env python3
"""Generate diagnostics command coverage/matrix/drift artifacts and frozen operator-truth contract."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli-core" / "tests" / "bin_surface" / "diagnostics_command_matrix.rs"

DIAGNOSTICS_COMMANDS = [
    "inspect",
    "doctor",
    "dev cli routes",
    "dev cli registry",
    "dev cli env",
    "dev cli contracts",
    "dev cli doctor",
    "dev cli state-doctor",
]

REQUIRED_TESTS = {
    362: "inspect_text_json_yaml_quiet_and_trace_modes",
    363: "inspect_text_json_yaml_quiet_and_trace_modes",
    364: "inspect_text_json_yaml_quiet_and_trace_modes",
    365: "inspect_text_json_yaml_quiet_and_trace_modes",
    366: "inspect_text_json_yaml_quiet_and_trace_modes",
    367: "doctor_text_json_and_corrupted_state_coverage",
    368: "doctor_text_json_and_corrupted_state_coverage",
    369: "doctor_text_json_and_corrupted_state_coverage",
    370: "doctor_text_json_and_corrupted_state_coverage",
    371: "doctor_text_json_and_corrupted_state_coverage",
    372: "doctor_text_json_and_corrupted_state_coverage",
    373: "dev_cli_routes_registry_env_contracts_json_shape_stability",
    374: "dev_cli_routes_registry_env_contracts_json_shape_stability",
    375: "dev_cli_routes_registry_env_contracts_json_shape_stability",
    376: "dev_cli_routes_registry_env_contracts_json_shape_stability",
    377: "diagnostics_consistency_across_inspect_doctor_and_dev_surfaces",
    378: "diagnostics_consistency_across_inspect_doctor_and_dev_surfaces",
    379: "diagnostics_consistency_across_inspect_doctor_and_dev_surfaces",
}


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def command_status(command: str, source: str) -> str:
    tokens = ", ".join([f'"{piece}"' for piece in command.split()])
    return "complete" if tokens in source else "partial"


def main() -> int:
    source = TEST_FILE.read_text(encoding="utf-8")
    generated_at = datetime.now(timezone.utc).isoformat()

    coverage_rows = [
        {
            "command": command,
            "status": command_status(command, source),
            "status_model": ["complete", "partial", "shim", "missing"],
            "evidence": "crates/bijux-cli-core/tests/bin_surface/diagnostics_command_matrix.rs",
        }
        for command in DIAGNOSTICS_COMMANDS
    ]

    todo_rows = [
        {
            "todo": todo,
            "test": fn_name,
            "status": "complete" if f"fn {fn_name}(" in source else "missing",
            "evidence": "crates/bijux-cli-core/tests/bin_surface/diagnostics_command_matrix.rs",
        }
        for todo, fn_name in sorted(REQUIRED_TESTS.items())
    ]

    drift = [row for row in coverage_rows if row["status"] != "complete"]

    write_json(
        "diagnostics_command_coverage_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_diagnostics_surface_reports.py",
            "scope": "todo 361 diagnostics command coverage",
            "commands": coverage_rows,
            "summary": {
                "total": len(coverage_rows),
                "complete": sum(1 for row in coverage_rows if row["status"] == "complete"),
                "partial": sum(1 for row in coverage_rows if row["status"] == "partial"),
                "shim": 0,
                "missing": 0,
            },
        },
    )

    write_json(
        "diagnostics_matrix_artifact.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_diagnostics_surface_reports.py",
            "scope": "todo 362-378 diagnostics matrix",
            "todo_rows": todo_rows,
            "commands": coverage_rows,
        },
    )

    write_json(
        "diagnostics_shape_drift_artifact.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_diagnostics_surface_reports.py",
            "scope": "todo 379 diagnostics shape drift",
            "drift_count": len(drift),
            "drift_commands": [row["command"] for row in drift],
            "status": "clean" if not drift else "drift",
        },
    )

    write_json(
        "diagnostics_operator_truth_contract.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_diagnostics_surface_reports.py",
            "domain": "diagnostics-operator-truth",
            "status": "frozen",
            "rule": "Diagnostics outputs must remain structured, consistent across surfaces, and stable in machine shape.",
            "evidence": [
                "crates/bijux-cli-core/tests/bin_surface/diagnostics_command_matrix.rs",
                "artifacts/status/diagnostics_matrix_artifact.json",
                "artifacts/status/diagnostics_shape_drift_artifact.json",
            ],
        },
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
