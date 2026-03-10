#!/usr/bin/env python3
"""Generate deterministic output artifacts for TODOs 121-140."""

from __future__ import annotations

import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "deterministic_output_matrix.rs"


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


def main() -> None:
    text = TEST_FILE.read_text(encoding="utf-8") if TEST_FILE.exists() else ""
    test_names = set(re.findall(r"fn\s+([a-z0-9_]+)\s*\(", text))

    rows = [
        (121, "status_json_is_byte_stable_across_runs"),
        (122, "plugins_list_json_is_byte_stable_across_runs"),
        (123, "config_get_json_is_byte_stable_across_runs"),
        (124, "inspect_json_is_byte_stable_across_runs"),
        (125, "help_text_is_stable_across_runs"),
        (126, "json_envelope_field_order_is_stable"),
        (127, "yaml_envelope_field_order_is_stable"),
        (128, "plugin_list_machine_output_order_is_stable"),
        (129, "diagnostic_ordering_is_stable_in_machine_output"),
        (130, "state_doctor_ordering_is_stable_in_machine_output"),
        (131, "repeated_runs_do_not_introduce_timestamp_noise_when_disallowed"),
        (132, "repeated_runs_do_not_introduce_path_order_noise"),
        (133, "repeated_runs_do_not_introduce_plugin_discovery_order_noise"),
        (134, "repeated_runs_do_not_introduce_environment_order_noise"),
        (135, "text_output_stability_holds_under_no_color_mode"),
        (136, "stderr_payloads_are_stable_for_identical_failures"),
        (137, "exit_codes_are_stable_for_identical_failures"),
    ]

    generated_at = stable_generated_at()

    report = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_deterministic_output_reports.py",
        "scope": "todo 121-140 deterministic output tests",
        "rows": [
            {
                "todo": todo,
                "test_name": name,
                "status": "complete" if name in test_names else "missing",
                "evidence": "crates/bijux-cli/tests/bin_surface/deterministic_output_matrix.rs",
            }
            for todo, name in rows
        ],
        "summary": {
            "complete": sum(1 for _, name in rows if name in test_names),
            "missing": sum(1 for _, name in rows if name not in test_names),
            "artifact_todo": 138,
            "artifact_path": "artifacts/status/deterministic_output_report.json",
        },
    }

    dashboard = {
        "generated_at": generated_at,
        "dashboard": "command-by-command determinism",
        "commands": [
            "status --format json --no-pretty",
            "cli plugins list --format json --no-pretty",
            "cli config get alpha --format json --no-pretty",
            "inspect --format json --no-pretty",
            "help cli plugins",
            "dev cli state-doctor --format json --no-pretty",
        ],
        "evidence": [
            "crates/bijux-cli/tests/bin_surface/deterministic_output_matrix.rs",
            "artifacts/status/deterministic_output_report.json",
        ],
        "covers_todo": 139,
    }

    frozen = {
        "generated_at": generated_at,
        "expectation": "byte stability is required where explicitly claimed",
        "status": "frozen",
        "evidence": [
            "artifacts/status/deterministic_output_report.json",
            "artifacts/status/determinism_dashboard.json",
        ],
        "covers_todo": 140,
    }

    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / "deterministic_output_report.json").write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "determinism_dashboard.json").write_text(
        json.dumps(dashboard, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "determinism_expectations.json").write_text(
        json.dumps(frozen, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    print("wrote artifacts/status/deterministic_output_report.json")
    print("wrote artifacts/status/determinism_dashboard.json")
    print("wrote artifacts/status/determinism_expectations.json")


if __name__ == "__main__":
    main()
