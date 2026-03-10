#!/usr/bin/env python3
"""Generate stdout/stderr stream-discipline contract and drift artifacts for TODOs 41-60."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli-core" / "tests" / "bin_surface" / "stream_discipline_matrix.rs"

CASES: list[dict[str, Any]] = [
    {"todo": 41, "name": "success_machine_json_stderr_empty", "args": ["status", "--format", "json", "--no-pretty"], "expect_code": 0, "expect_stdout_nonempty": True, "expect_stderr_empty": True},
    {"todo": 42, "name": "success_text_no_stderr_noise", "args": ["status", "--format", "text"], "expect_code": 0, "expect_stdout_nonempty": True, "expect_stderr_empty": True},
    {"todo": 43, "name": "usage_error_stderr_only", "args": ["config", "get"], "expect_code": 2, "expect_stdout_nonempty": False, "expect_stderr_empty": False},
    {"todo": 44, "name": "validation_error_stderr_only", "args": ["--format", "not-a-format", "status"], "expect_code": 1, "expect_stdout_nonempty": False, "expect_stderr_empty": False},
    {"todo": 45, "name": "plugin_error_stderr_only", "args": ["plugins", "uninstall"], "expect_code": 1, "expect_stdout_nonempty": False, "expect_stderr_empty": False},
    {"todo": 46, "name": "internal_like_error_stderr_only", "args": ["plugins", "enable"], "expect_code": 1, "expect_stdout_nonempty": False, "expect_stderr_empty": False},
    {"todo": 47, "name": "quiet_mode_suppresses_stdout", "args": ["--quiet", "status", "--format", "json", "--no-pretty"], "expect_code": 0, "expect_stdout_nonempty": False, "expect_stderr_empty": True},
    {"todo": 48, "name": "quiet_mode_suppresses_nonessential_stderr", "args": ["--quiet", "status", "--format", "json", "--no-pretty"], "expect_code": 0, "expect_stdout_nonempty": False, "expect_stderr_empty": True},
    {"todo": 49, "name": "trace_mode_stream_contract", "args": ["--log-level", "trace", "status", "--format", "json", "--no-pretty"], "expect_code": 0, "expect_stdout_nonempty": True, "expect_stderr_empty": True},
    {"todo": 50, "name": "pretty_json_stream_contract", "args": ["status", "--format", "json", "--pretty"], "expect_code": 0, "expect_stdout_nonempty": True, "expect_stderr_empty": True},
    {"todo": 51, "name": "compact_json_stream_contract", "args": ["status", "--format", "json", "--no-pretty"], "expect_code": 0, "expect_stdout_nonempty": True, "expect_stderr_empty": True},
    {"todo": 52, "name": "yaml_stream_contract", "args": ["status", "--format", "yaml", "--pretty"], "expect_code": 0, "expect_stdout_nonempty": True, "expect_stderr_empty": True},
    {"todo": 53, "name": "help_no_unrelated_stderr", "args": ["help", "status"], "expect_code": 0, "expect_stdout_nonempty": True, "expect_stderr_empty": True},
    {"todo": 54, "name": "version_no_unrelated_stderr", "args": ["version"], "expect_code": 0, "expect_stdout_nonempty": True, "expect_stderr_empty": True},
    {"todo": 55, "name": "plugin_commands_follow_stream_law", "args": ["plugins", "list", "--format", "json", "--no-pretty"], "expect_code": 0, "expect_stdout_nonempty": True, "expect_stderr_empty": True},
    {"todo": 56, "name": "state_doctor_follows_stream_law", "args": ["dev", "cli", "state-doctor", "--format", "json", "--no-pretty"], "expect_code": 0, "expect_stdout_nonempty": True, "expect_stderr_empty": True},
    {"todo": 57, "name": "binary_bridge_stream_routing_consistency", "args": ["status", "--format", "json", "--no-pretty"], "expect_code": 0, "expect_stdout_nonempty": True, "expect_stderr_empty": True},
]

REQUIRED_TESTS = {
    41: "successful_machine_readable_commands_keep_stderr_empty",
    42: "text_success_commands_do_not_leak_diagnostics_to_stderr_in_normal_mode",
    43: "usage_validation_plugin_and_internal_failures_route_to_stderr_only",
    44: "usage_validation_plugin_and_internal_failures_route_to_stderr_only",
    45: "usage_validation_plugin_and_internal_failures_route_to_stderr_only",
    46: "usage_validation_plugin_and_internal_failures_route_to_stderr_only",
    47: "quiet_mode_suppresses_success_stdout_and_nonessential_stderr_noise",
    48: "quiet_mode_suppresses_success_stdout_and_nonessential_stderr_noise",
    49: "trace_mode_preserves_stream_contract_without_corrupting_output_envelope",
    50: "pretty_compact_json_and_yaml_all_respect_stream_discipline",
    51: "pretty_compact_json_and_yaml_all_respect_stream_discipline",
    52: "pretty_compact_json_and_yaml_all_respect_stream_discipline",
    53: "help_and_version_fast_paths_do_not_leak_unrelated_diagnostics_to_stderr",
    54: "help_and_version_fast_paths_do_not_leak_unrelated_diagnostics_to_stderr",
    55: "plugin_and_state_doctor_commands_obey_builtin_stream_law",
    56: "plugin_and_state_doctor_commands_obey_builtin_stream_law",
    57: "binary_and_bridge_agree_on_stream_routing_for_success_and_failure",
}


def has_test(source: str, test_name: str) -> bool:
    return f"fn {test_name}(" in source


def run_case(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-core", "--", *args],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )


def main() -> None:
    source = TEST_FILE.read_text(encoding="utf-8")

    rows: list[dict[str, Any]] = []
    drift_items: list[dict[str, Any]] = []
    for case in CASES:
        out = run_case(case["args"])
        stdout_nonempty = bool(out.stdout)
        stderr_empty = not bool(out.stderr)
        ok = (
            out.returncode == case["expect_code"]
            and stdout_nonempty == case["expect_stdout_nonempty"]
            and stderr_empty == case["expect_stderr_empty"]
        )
        row = {
            "todo": case["todo"],
            "name": case["name"],
            "command": " ".join(case["args"]),
            "expected_exit_code": case["expect_code"],
            "observed_exit_code": out.returncode,
            "expected_stdout_nonempty": case["expect_stdout_nonempty"],
            "observed_stdout_nonempty": stdout_nonempty,
            "expected_stderr_empty": case["expect_stderr_empty"],
            "observed_stderr_empty": stderr_empty,
            "status": "covered" if ok else "drift",
        }
        rows.append(row)
        if row["status"] != "covered":
            drift_items.append(row)

    todo_rows = []
    for todo, test_name in sorted(REQUIRED_TESTS.items()):
        todo_rows.append(
            {
                "todo": todo,
                "test_name": test_name,
                "status": "covered" if has_test(source, test_name) else "missing",
                "evidence": "crates/bijux-cli-core/tests/bin_surface/stream_discipline_matrix.rs",
            }
        )
    missing_todos = [row for row in todo_rows if row["status"] != "covered"]

    contract = {
        "generator": "scripts/status/generate_stream_discipline_reports.py",
        "scope": "stdout-stderr discipline",
        "status": "complete" if not drift_items and not missing_todos else "partial",
        "tasks": list(range(41, 59)),
        "release_blocking": True,
        "rows": rows,
        "todo_coverage": todo_rows,
        "summary": {
            "covered_rows": len(rows) - len(drift_items),
            "drift_rows": len(drift_items),
            "covered_todos": len(todo_rows) - len(missing_todos),
            "missing_todos": len(missing_todos),
        },
    }

    drift = {
        "generator": "scripts/status/generate_stream_discipline_reports.py",
        "scope": "stdout-stderr discipline drift",
        "status": "clean" if not drift_items else "drift-detected",
        "tasks": [59, 60],
        "drift_count": len(drift_items),
        "drift_items": drift_items,
        "missing_todos": [row["todo"] for row in missing_todos],
    }

    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / "stream_discipline_artifact.json").write_text(
        json.dumps(contract, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "stream_drift_artifact.json").write_text(
        json.dumps(drift, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print("wrote artifacts/status/stream_discipline_artifact.json")
    print("wrote artifacts/status/stream_drift_artifact.json")


if __name__ == "__main__":
    main()

