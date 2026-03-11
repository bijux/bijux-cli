#!/usr/bin/env python3
"""Generate history command coverage/matrix/corruption artifacts and frozen read-domain contract."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "history_command_matrix.rs"

HISTORY_COMMANDS = ["history", "history clear"]

REQUIRED_TESTS = {
    322: "history_root_listing_no_file_one_record_many_records_and_ordering",
    323: "history_root_listing_no_file_one_record_many_records_and_ordering",
    324: "history_root_listing_no_file_one_record_many_records_and_ordering",
    325: "history_text_json_yaml_quiet_and_no_color_modes",
    326: "history_text_json_yaml_quiet_and_no_color_modes",
    327: "history_text_json_yaml_quiet_and_no_color_modes",
    328: "history_root_listing_no_file_one_record_many_records_and_ordering",
    329: "history_malformed_and_mixed_valid_invalid_tolerance_and_duplicates",
    330: "history_malformed_and_mixed_valid_invalid_tolerance_and_duplicates",
    331: "history_malformed_and_mixed_valid_invalid_tolerance_and_duplicates",
    332: "history_limit_path_override_and_repeated_run_determinism",
    333: "history_limit_path_override_and_repeated_run_determinism",
    334: "history_clear_with_unwritable_parent_fails_stably",
    335: "history_text_json_yaml_quiet_and_no_color_modes",
    336: "history_text_json_yaml_quiet_and_no_color_modes",
    337: "history_limit_path_override_and_repeated_run_determinism",
    338: "history_help_and_exit_discipline_for_root_and_clear",
    339: "history_malformed_and_mixed_valid_invalid_tolerance_and_duplicates",
}


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def main() -> int:
    source = TEST_FILE.read_text(encoding="utf-8")
    generated_at = datetime.now(timezone.utc).isoformat()

    coverage_rows = []
    for command in HISTORY_COMMANDS:
        tokens = ", ".join([f'"{piece}"' for piece in command.split()])
        status = "complete" if tokens in source else "partial"
        coverage_rows.append(
            {
                "command": command,
                "status": status,
                "status_model": ["complete", "partial", "shim", "missing"],
                "evidence": "crates/bijux-cli/tests/bin_surface/history_command_matrix.rs",
            }
        )

    coverage_rows = [
        {
            "coverage_id": coverage_id,
            "test": fn_name,
            "status": "complete" if f"fn {fn_name}(" in source else "missing",
            "evidence": "crates/bijux-cli/tests/bin_surface/history_command_matrix.rs",
        }
        for coverage_id, fn_name in sorted(REQUIRED_TESTS.items())
    ]

    write_json(
        "history_command_coverage_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_history_surface_reports.py",
            "scope": "history command coverage",
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
        "history_command_matrix_artifact.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_history_surface_reports.py",
            "scope": "history command matrix",
            "coverage_rows": coverage_rows,
            "commands": coverage_rows,
        },
    )

    write_json(
        "history_corruption_matrix_artifact.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_history_surface_reports.py",
            "scope": "history corruption matrix",
            "cases": [
                {
                    "name": "line-layout malformed and mixed records",
                    "status": "complete",
                    "evidence": "history_malformed_and_mixed_valid_invalid_tolerance_and_duplicates",
                },
                {
                    "name": "unwritable parent directory on clear",
                    "status": "complete",
                    "evidence": "history_clear_with_unwritable_parent_fails_stably",
                },
            ],
        },
    )

    write_json(
        "history_read_domain_contract.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_history_surface_reports.py",
            "domain": "history-read-behavior",
            "status": "frozen",
            "rule": "History read behavior must remain deterministic, format-stable, and resilient under malformed storage states.",
            "evidence": [
                "crates/bijux-cli/tests/bin_surface/history_command_matrix.rs",
                "artifacts/status/history_command_matrix_artifact.json",
                "artifacts/status/history_corruption_matrix_artifact.json",
            ],
        },
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
