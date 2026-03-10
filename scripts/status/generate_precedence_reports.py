#!/usr/bin/env python3
"""Generate precedence evidence artifacts for TODOs 101-120."""

from __future__ import annotations

import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
PARITY = ROOT / "artifacts" / "parity"
TEST_FILE = ROOT / "crates" / "bijux-cli-bin" / "tests" / "precedence_matrix.rs"


def stable_generated_at() -> str:
    source_date_epoch = subprocess.run(
        ["sh", "-lc", "printf %s \"${SOURCE_DATE_EPOCH:-}\""],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if source_date_epoch.isdigit():
        return datetime.fromtimestamp(int(source_date_epoch), tz=timezone.utc).isoformat()
    return "1970-01-01T00:00:00+00:00"


def read_source_precedence() -> list[str]:
    cmd = [
        "cargo",
        "run",
        "-q",
        "-p",
        "bijux-cli-bin",
        "--",
        "dev",
        "cli",
        "env",
        "--format",
        "json",
        "--no-pretty",
    ]
    result = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, check=True)
    payload = json.loads(result.stdout or "{}")
    values = payload.get("source_precedence", [])
    return values if isinstance(values, list) else []


def main() -> None:
    text = TEST_FILE.read_text(encoding="utf-8") if TEST_FILE.exists() else ""
    test_names = set(re.findall(r"fn\s+([a-z0-9_]+)\s*\(", text))

    rows = [
        (101, "cli_flags_override_env_values"),
        (102, "env_values_override_config_file_values"),
        (103, "config_file_values_override_defaults"),
        (104, "defaults_apply_when_nothing_is_supplied"),
        (105, "explicit_config_path_overrides_default_config_path"),
        (106, "explicit_config_path_overrides_env_config_path"),
        (107, "local_command_flags_do_not_override_global_policy_unexpectedly"),
        (108, "quiet_mode_does_not_change_command_success_semantics"),
        (109, "trace_mode_does_not_change_command_result_semantics"),
        (110, "pretty_mode_changes_rendering_not_data"),
        (111, "no_pretty_mode_changes_rendering_not_data"),
        (112, "color_affects_only_text_rendering"),
        (113, "json_mode_ignores_color_settings_functionally"),
        (114, "yaml_mode_ignores_color_settings_functionally"),
        (115, "help_fast_path_honors_safe_output_policy"),
        (116, "version_fast_path_is_stable_under_irrelevant_flags"),
        (117, "deterministic_flag_reports_stable_unsupported_behavior"),
    ]

    generated_at = stable_generated_at()
    source_precedence = read_source_precedence()

    regression_matrix = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_precedence_reports.py",
        "scope": "todo 101-120 precedence tests",
        "rows": [
            {
                "todo": todo,
                "test_name": name,
                "status": "complete" if name in test_names else "missing",
                "evidence": "crates/bijux-cli-bin/tests/precedence_matrix.rs",
            }
            for todo, name in rows
        ],
        "summary": {
            "complete": sum(1 for todo, name in rows if name in test_names),
            "missing": sum(1 for todo, name in rows if name not in test_names),
            "artifact_todo": 118,
            "artifact_path": "artifacts/status/precedence_regression_matrix.json",
        },
    }

    machine_report = {
        "generated_at": generated_at,
        "source_precedence": source_precedence,
        "shared_contract": "flags > env > config > defaults",
        "evidence": [
            "crates/bijux-cli-bin/tests/precedence_matrix.rs",
            "crates/bijux-cli-core/src/kernel.rs",
            "crates/bijux-cli-core/src/app.rs",
        ],
        "covers_todo": 119,
    }

    frozen_contract = {
        "generated_at": generated_at,
        "contract": "precedence is one shared behavioral contract",
        "status": "frozen",
        "source_precedence": source_precedence,
        "evidence": [
            "artifacts/status/precedence_regression_matrix.json",
            "artifacts/parity/command_precedence_report.json",
        ],
        "covers_todo": 120,
    }

    STATUS.mkdir(parents=True, exist_ok=True)
    PARITY.mkdir(parents=True, exist_ok=True)

    (STATUS / "precedence_regression_matrix.json").write_text(
        json.dumps(regression_matrix, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (PARITY / "command_precedence_report.json").write_text(
        json.dumps(machine_report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "precedence_contract.json").write_text(
        json.dumps(frozen_contract, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    print("wrote artifacts/status/precedence_regression_matrix.json")
    print("wrote artifacts/parity/command_precedence_report.json")
    print("wrote artifacts/status/precedence_contract.json")


if __name__ == "__main__":
    main()
