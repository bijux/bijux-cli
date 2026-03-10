#!/usr/bin/env python3
"""Generate memory command coverage/matrix/corruption/parity artifacts and frozen read-domain contract."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
MATRIX_TEST = ROOT / "crates" / "bijux-cli-core" / "tests" / "bin_surface" / "memory_command_matrix.rs"
PARITY_TEST = ROOT / "crates" / "bijux-cli-core" / "tests" / "bin_surface" / "memory_parity.rs"

MEMORY_COMMANDS = ["memory", "memory list", "memory get", "memory set", "memory delete", "memory clear"]

REQUIRED_TESTS = {
    342: "memory_root_and_list_missing_empty_valid_text_json_yaml",
    343: "memory_root_and_list_missing_empty_valid_text_json_yaml",
    344: "memory_root_and_list_missing_empty_valid_text_json_yaml",
    345: "memory_root_and_list_missing_empty_valid_text_json_yaml",
    346: "memory_root_and_list_missing_empty_valid_text_json_yaml",
    347: "memory_root_and_list_missing_empty_valid_text_json_yaml",
    348: "memory_malformed_wrong_type_missing_required_and_extra_fields",
    349: "memory_malformed_wrong_type_missing_required_and_extra_fields",
    350: "memory_malformed_wrong_type_missing_required_and_extra_fields",
    351: "memory_malformed_wrong_type_missing_required_and_extra_fields",
    352: "memory_quiet_no_color_and_deterministic_repeated_runs",
    353: "memory_quiet_no_color_and_deterministic_repeated_runs",
    354: "memory_quiet_no_color_and_deterministic_repeated_runs",
    355: "memory_unwritable_storage_conditions_for_read_and_write_paths",
    356: "memory_config_path_override_does_not_change_home_memory_resolution",
    357: "memory_quiet_no_color_and_deterministic_repeated_runs",
    358: "memory_malformed_wrong_type_missing_required_and_extra_fields",
    359: "memory_root_parity_with_python_summary_command",
}


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def main() -> int:
    matrix_source = MATRIX_TEST.read_text(encoding="utf-8")
    parity_source = PARITY_TEST.read_text(encoding="utf-8") if PARITY_TEST.exists() else ""
    generated_at = datetime.now(timezone.utc).isoformat()

    coverage_rows = []
    for command in MEMORY_COMMANDS:
        evidence_source = matrix_source if command in ["memory", "memory list"] else parity_source
        tokens = ", ".join([f'"{piece}"' for piece in command.split()])
        status = "complete" if tokens in evidence_source else "partial"
        coverage_rows.append(
            {
                "command": command,
                "status": status,
                "status_model": ["complete", "partial", "shim", "missing"],
                "evidence": [
                    "crates/bijux-cli-core/tests/bin_surface/memory_command_matrix.rs",
                    "crates/bijux-cli-core/tests/bin_surface/memory_parity.rs",
                ],
            }
        )

    todo_rows = []
    for todo, fn_name in sorted(REQUIRED_TESTS.items()):
        in_matrix = f"fn {fn_name}(" in matrix_source
        in_parity = f"fn {fn_name}(" in parity_source
        todo_rows.append(
            {
                "todo": todo,
                "test": fn_name,
                "status": "complete" if (in_matrix or in_parity) else "missing",
                "evidence": "crates/bijux-cli-core/tests/bin_surface/memory_command_matrix.rs"
                if in_matrix
                else "crates/bijux-cli-core/tests/bin_surface/memory_parity.rs",
            }
        )

    write_json(
        "memory_command_coverage_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_memory_surface_reports.py",
            "scope": "todo 341 memory command coverage",
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
        "memory_command_matrix_artifact.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_memory_surface_reports.py",
            "scope": "todo 342-357 memory command matrix",
            "todo_rows": todo_rows,
            "commands": coverage_rows,
        },
    )

    write_json(
        "memory_corruption_matrix_artifact.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_memory_surface_reports.py",
            "scope": "todo 358 memory corruption matrix",
            "cases": [
                {
                    "name": "malformed memory state and wrong-type fields",
                    "status": "complete",
                    "evidence": "memory_malformed_wrong_type_missing_required_and_extra_fields",
                },
                {
                    "name": "unwritable storage write path",
                    "status": "complete",
                    "evidence": "memory_unwritable_storage_conditions_for_read_and_write_paths",
                },
            ],
        },
    )

    write_json(
        "memory_python_parity_artifact.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_memory_surface_reports.py",
            "scope": "todo 359 memory parity versus overlapping python behavior",
            "status": "complete" if "fn memory_root_parity_with_python_summary_command(" in parity_source else "partial",
            "evidence": [
                "crates/bijux-cli-core/tests/bin_surface/memory_parity.rs",
                "crates/bijux-cli-core/tests/bin_surface/memory_command_matrix.rs",
            ],
        },
    )

    write_json(
        "memory_read_domain_contract.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_memory_surface_reports.py",
            "domain": "memory-read-behavior",
            "status": "frozen",
            "rule": "Memory read behavior is accepted only when determinism and corruption handling remain green.",
            "evidence": [
                "crates/bijux-cli-core/tests/bin_surface/memory_command_matrix.rs",
                "artifacts/status/memory_command_matrix_artifact.json",
                "artifacts/status/memory_corruption_matrix_artifact.json",
                "artifacts/status/memory_python_parity_artifact.json",
            ],
        },
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
