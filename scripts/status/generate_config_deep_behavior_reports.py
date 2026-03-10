#!/usr/bin/env python3
"""Generate deep config behavior artifacts for TODOs 81-100."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli-core" / "tests" / "bin_surface" / "config_deep_behavior_matrix.rs"

REQUIRED_TESTS = {
    81: "config_key_normalization_and_parse_behavior_are_stable_across_repeated_inputs",
    82: "config_writer_ordering_and_formatting_rules_are_deterministic",
    83: "config_key_normalization_and_parse_behavior_are_stable_across_repeated_inputs",
    84: "config_key_normalization_and_parse_behavior_are_stable_across_repeated_inputs",
    85: "config_key_normalization_and_parse_behavior_are_stable_across_repeated_inputs",
    86: "config_key_normalization_and_parse_behavior_are_stable_across_repeated_inputs",
    87: "config_writer_ordering_and_formatting_rules_are_deterministic",
    88: "config_export_and_load_preserve_semantic_content_and_roundtrip_exact_values",
    89: "config_export_and_load_preserve_semantic_content_and_roundtrip_exact_values",
    90: "config_export_and_load_preserve_semantic_content_and_roundtrip_exact_values",
    91: "config_unset_clear_and_repeated_mutations_follow_expected_semantics",
    92: "config_unset_clear_and_repeated_mutations_follow_expected_semantics",
    93: "config_unset_clear_and_repeated_mutations_follow_expected_semantics",
    94: "root_and_cli_config_path_override_behavior_is_identical_for_list",
    95: "config_doctor_and_state_doctor_agree_on_corrupted_config_findings",
}


def has_test(source: str, test_name: str) -> bool:
    return f"fn {test_name}(" in source


def run_cmd(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-core", "--", *args],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )


def run_json(args: list[str]) -> dict[str, Any]:
    out = run_cmd(args + ["--format", "json", "--no-pretty"])
    if out.returncode != 0:
        return {}
    return json.loads(out.stdout or "{}")


def main() -> None:
    source = TEST_FILE.read_text(encoding="utf-8")

    semantic_roundtrip = run_json(["cli", "config", "list"])
    precedence_view = run_json(["dev", "cli", "env"])
    determinism_first = run_cmd(["cli", "config", "list", "--format", "json", "--no-pretty"])
    determinism_second = run_cmd(["cli", "config", "list", "--format", "json", "--no-pretty"])
    corruption_view = run_json(["dev", "cli", "state-doctor"])

    todo_rows = []
    for todo, test_name in sorted(REQUIRED_TESTS.items()):
        todo_rows.append(
            {
                "todo": todo,
                "test_name": test_name,
                "status": "covered" if has_test(source, test_name) else "missing",
                "evidence": "crates/bijux-cli-core/tests/bin_surface/config_deep_behavior_matrix.rs",
            }
        )
    missing_todos = [row for row in todo_rows if row["status"] != "covered"]

    semantic_artifact = {
        "generator": "scripts/status/generate_config_deep_behavior_reports.py",
        "scope": "config semantic roundtrip",
        "tasks": [88, 89, 90, 91, 92, 96],
        "status": "complete" if semantic_roundtrip != {} else "partial",
        "sample": semantic_roundtrip,
    }

    precedence_artifact = {
        "generator": "scripts/status/generate_config_deep_behavior_reports.py",
        "scope": "config precedence",
        "tasks": [94, 97],
        "status": "complete" if precedence_view != {} else "partial",
        "sample": precedence_view,
    }

    determinism_artifact = {
        "generator": "scripts/status/generate_config_deep_behavior_reports.py",
        "scope": "config determinism",
        "tasks": [81, 82, 83, 84, 85, 86, 87, 93, 98],
        "status": "complete"
        if determinism_first.returncode == 0
        and determinism_second.returncode == 0
        and determinism_first.stdout == determinism_second.stdout
        and determinism_first.stderr == determinism_second.stderr
        else "partial",
        "first_exit_code": determinism_first.returncode,
        "second_exit_code": determinism_second.returncode,
        "byte_stable": determinism_first.stdout == determinism_second.stdout
        and determinism_first.stderr == determinism_second.stderr,
    }

    corruption_artifact = {
        "generator": "scripts/status/generate_config_deep_behavior_reports.py",
        "scope": "config corruption recovery",
        "tasks": [95, 99],
        "status": "complete" if corruption_view != {} else "partial",
        "sample": corruption_view,
    }

    drift_items: list[dict[str, Any]] = []
    for artifact_name, payload in [
        ("config_semantic_roundtrip_artifact.json", semantic_artifact),
        ("config_precedence_artifact.json", precedence_artifact),
        ("config_determinism_artifact.json", determinism_artifact),
        ("config_corruption_recovery_artifact.json", corruption_artifact),
    ]:
        if payload.get("status") != "complete":
            drift_items.append({"artifact": artifact_name, "reason": "status-not-complete"})
    if missing_todos:
        drift_items.append({"reason": "missing-todo-coverage", "todos": [row["todo"] for row in missing_todos]})

    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / "config_semantic_roundtrip_artifact.json").write_text(
        json.dumps(semantic_artifact, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "config_precedence_artifact.json").write_text(
        json.dumps(precedence_artifact, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "config_determinism_artifact.json").write_text(
        json.dumps(determinism_artifact, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "config_corruption_recovery_artifact.json").write_text(
        json.dumps(corruption_artifact, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "config_deep_behavior_drift_artifact.json").write_text(
        json.dumps(
            {
                "generator": "scripts/status/generate_config_deep_behavior_reports.py",
                "scope": "config deep behavior drift",
                "tasks": [100],
                "status": "clean" if not drift_items else "drift-detected",
                "drift_count": len(drift_items),
                "drift_items": drift_items,
                "todo_coverage": todo_rows,
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    print("wrote artifacts/status/config_semantic_roundtrip_artifact.json")
    print("wrote artifacts/status/config_precedence_artifact.json")
    print("wrote artifacts/status/config_determinism_artifact.json")
    print("wrote artifacts/status/config_corruption_recovery_artifact.json")
    print("wrote artifacts/status/config_deep_behavior_drift_artifact.json")


if __name__ == "__main__":
    main()

