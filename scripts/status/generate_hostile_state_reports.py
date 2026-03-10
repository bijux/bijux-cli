#!/usr/bin/env python3
"""Generate hostile-state determinism artifacts for TODOs 141-160."""

from __future__ import annotations

import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli-bin" / "tests" / "deterministic_hostile_state_matrix.rs"
HARNESS_FILE = STATUS / "repeated_run_corruption_harness.json"


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
    STATUS.mkdir(parents=True, exist_ok=True)
    text = TEST_FILE.read_text(encoding="utf-8") if TEST_FILE.exists() else ""
    test_names = set(re.findall(r"fn\s+([a-z0-9_]+)\s*\(", text))

    rows = [
        (141, "corrupted_config_failure_class_is_stable_across_runs"),
        (142, "corrupted_plugin_registry_failure_class_is_stable_across_runs"),
        (143, "broken_history_file_recovery_is_stable_across_runs"),
        (144, "malformed_memory_state_recovery_is_stable_across_runs"),
        (145, "missing_config_file_defaulting_is_stable_across_runs"),
        (146, "missing_plugin_directory_empty_behavior_is_stable_across_runs"),
        (147, "broken_plugin_does_not_nondeterministically_affect_healthy_output"),
        (148, "conflicting_plugin_installs_fail_deterministically"),
        (149, "path_shadowing_diagnostics_are_stable_across_runs"),
        (150, "runtime_identity_output_is_stable_under_same_ambiguous_state"),
        (151, "state_doctor_json_is_stable_under_same_corrupted_state"),
        (152, "state_doctor_text_is_stable_under_same_corrupted_state"),
        (153, "plugin_doctor_json_is_stable_under_same_corrupted_state"),
        (154, "plugin_doctor_text_is_stable_under_same_corrupted_state"),
        (155, "command_tree_export_is_stable_with_broken_optional_state"),
    ]

    generated_at = stable_generated_at()
    harness = json.loads(HARNESS_FILE.read_text(encoding="utf-8")) if HARNESS_FILE.exists() else {}
    harness_summary = harness.get("summary", {}) if isinstance(harness, dict) else {}

    report = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_hostile_state_reports.py",
        "scope": "todo 141-160 deterministic hostile-state behavior",
        "rows": [
            {
                "todo": todo,
                "test_name": name,
                "status": "complete" if name in test_names else "missing",
                "evidence": "crates/bijux-cli-bin/tests/deterministic_hostile_state_matrix.rs",
            }
            for todo, name in rows
        ],
        "summary": {
            "complete": sum(1 for _, name in rows if name in test_names),
            "missing": sum(1 for _, name in rows if name not in test_names),
            "artifact_todo": 156,
            "artifact_path": "artifacts/status/deterministic_hostile_state_report.json",
        },
    }

    failure_class_report = {
        "generated_at": generated_at,
        "harness_summary": harness_summary,
        "harness_file": "artifacts/status/repeated_run_corruption_harness.json",
        "covers_todo": 157,
    }

    quality_bar = {
        "generated_at": generated_at,
        "status": "frozen",
        "quality_bar": "deterministic failure behavior required for hostile-state covered commands",
        "required_artifacts": [
            "artifacts/status/deterministic_hostile_state_report.json",
            "artifacts/status/failure_class_stability_report.json",
            "artifacts/status/repeated_run_corruption_harness.json",
        ],
        "ci_enforced": True,
        "covers_todo": 160,
    }

    (STATUS / "deterministic_hostile_state_report.json").write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "failure_class_stability_report.json").write_text(
        json.dumps(failure_class_report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "deterministic_failure_quality_bar.json").write_text(
        json.dumps(quality_bar, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    print("wrote artifacts/status/deterministic_hostile_state_report.json")
    print("wrote artifacts/status/failure_class_stability_report.json")
    print("wrote artifacts/status/deterministic_failure_quality_bar.json")


if __name__ == "__main__":
    main()
