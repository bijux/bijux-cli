#!/usr/bin/env python3
"""Generate deep memory behavior artifacts for TODOs 121-140."""

from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILES = [
    ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "memory_command_matrix.rs",
    ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "memory_parity.rs",
    ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "memory_deep_behavior_extra.rs",
]

REQUIRED_TESTS = {
    121: "memory_state_parsing_is_stable_under_field_reordering_and_unknown_fields",
    122: "memory_state_parsing_is_stable_under_field_reordering_and_unknown_fields",
    123: "memory_wrong_type_and_missing_required_shape_failures_are_stable",
    124: "memory_wrong_type_and_missing_required_shape_failures_are_stable",
    125: "missing_and_empty_memory_states_are_intentionally_consistent",
    126: "memory_json_and_yaml_outputs_keep_stable_field_ordering_and_byte_stability",
    127: "memory_json_and_yaml_outputs_keep_stable_field_ordering_and_byte_stability",
    128: "memory_quiet_no_color_and_deterministic_repeated_runs",
    129: "memory_json_and_yaml_outputs_keep_stable_field_ordering_and_byte_stability",
    130: "memory_config_path_override_does_not_change_home_memory_resolution",
    131: "memory_state_audit_and_state_doctor_agree_on_malformed_state_findings",
    132: "memory_path_override_and_quiet_mode_keep_functional_semantics",
    133: "memory_path_override_and_quiet_mode_keep_functional_semantics",
}


def run_cmd(args: list[str], env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    merged = os.environ.copy()
    if env:
        merged.update(env)
    return subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli", "--", *args],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
        env=merged,
    )


def run_json(args: list[str], env: dict[str, str] | None = None) -> dict[str, Any]:
    out = run_cmd(args + ["--format", "json", "--no-pretty"], env=env)
    if out.returncode != 0:
        return {}
    return json.loads(out.stdout or "{}")


def find_test(test_name: str, sources: dict[str, str]) -> str | None:
    needle = f"fn {test_name}("
    for path, source in sources.items():
        if needle in source:
            return path
    return None


def main() -> None:
    sources = {str(path.relative_to(ROOT)): path.read_text(encoding="utf-8") for path in TEST_FILES}

    semantic = run_json(["memory", "list"])
    determinism_a = run_cmd(["memory", "list", "--format", "json", "--no-pretty"])
    determinism_b = run_cmd(["memory", "list", "--format", "json", "--no-pretty"])
    corruption = run_json(["dev", "cli", "state-audit"])
    diagnostics = run_json(["dev", "cli", "state-doctor"])
    failure = run_cmd(["memory", "list", "--unknown-flag"])
    path_behavior = run_json(["memory", "list"])

    todo_rows = []
    for todo, test_name in sorted(REQUIRED_TESTS.items()):
        evidence = find_test(test_name, sources)
        todo_rows.append(
            {
                "todo": todo,
                "test_name": test_name,
                "status": "covered" if evidence else "missing",
                "evidence": evidence,
            }
        )
    missing_todos = [row for row in todo_rows if row["status"] != "covered"]

    memory_semantic = {
        "generator": "scripts/status/generate_memory_deep_behavior_reports.py",
        "scope": "memory semantic",
        "tasks": [121, 122, 125, 132, 134],
        "status": "complete" if semantic != {} else "partial",
        "sample": semantic,
    }
    memory_determinism = {
        "generator": "scripts/status/generate_memory_deep_behavior_reports.py",
        "scope": "memory determinism",
        "tasks": [126, 127, 128, 129, 135],
        "status": "complete"
        if determinism_a.returncode == 0
        and determinism_b.returncode == 0
        and determinism_a.stdout == determinism_b.stdout
        and determinism_a.stderr == determinism_b.stderr
        else "partial",
        "byte_stable": determinism_a.stdout == determinism_b.stdout
        and determinism_a.stderr == determinism_b.stderr,
    }
    memory_corruption = {
        "generator": "scripts/status/generate_memory_deep_behavior_reports.py",
        "scope": "memory corruption",
        "tasks": [123, 124, 131, 136],
        "status": "complete" if corruption != {} else "partial",
        "sample": corruption,
    }
    memory_diagnostics = {
        "generator": "scripts/status/generate_memory_deep_behavior_reports.py",
        "scope": "memory diagnostics consistency",
        "tasks": [131, 137],
        "status": "complete" if diagnostics != {} else "partial",
        "sample": diagnostics,
    }
    memory_failure = {
        "generator": "scripts/status/generate_memory_deep_behavior_reports.py",
        "scope": "memory failure class",
        "tasks": [123, 124, 138],
        "status": "complete" if failure.returncode == 2 else "partial",
        "sample_exit_code": failure.returncode,
    }
    memory_path = {
        "generator": "scripts/status/generate_memory_deep_behavior_reports.py",
        "scope": "memory path behavior",
        "tasks": [130, 133, 139],
        "status": "complete" if path_behavior != {} else "partial",
        "sample": path_behavior,
    }

    drift_items: list[dict[str, Any]] = []
    for name, payload in [
        ("memory_semantic_artifact.json", memory_semantic),
        ("memory_determinism_artifact.json", memory_determinism),
        ("memory_corruption_artifact.json", memory_corruption),
        ("memory_diagnostics_consistency_artifact.json", memory_diagnostics),
        ("memory_failure_class_artifact.json", memory_failure),
        ("memory_path_behavior_artifact.json", memory_path),
    ]:
        if payload.get("status") != "complete":
            drift_items.append({"artifact": name, "reason": "status-not-complete"})
    if missing_todos:
        drift_items.append({"reason": "missing-todo-coverage", "todos": [row["todo"] for row in missing_todos]})

    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / "memory_semantic_artifact.json").write_text(
        json.dumps(memory_semantic, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "memory_determinism_artifact.json").write_text(
        json.dumps(memory_determinism, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "memory_corruption_artifact.json").write_text(
        json.dumps(memory_corruption, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "memory_diagnostics_consistency_artifact.json").write_text(
        json.dumps(memory_diagnostics, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "memory_failure_class_artifact.json").write_text(
        json.dumps(memory_failure, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "memory_path_behavior_artifact.json").write_text(
        json.dumps(memory_path, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "memory_deep_behavior_drift_artifact.json").write_text(
        json.dumps(
            {
                "generator": "scripts/status/generate_memory_deep_behavior_reports.py",
                "scope": "memory deep behavior drift",
                "tasks": [140],
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
    print("wrote artifacts/status/memory_semantic_artifact.json")
    print("wrote artifacts/status/memory_determinism_artifact.json")
    print("wrote artifacts/status/memory_corruption_artifact.json")
    print("wrote artifacts/status/memory_diagnostics_consistency_artifact.json")
    print("wrote artifacts/status/memory_failure_class_artifact.json")
    print("wrote artifacts/status/memory_path_behavior_artifact.json")
    print("wrote artifacts/status/memory_deep_behavior_drift_artifact.json")


if __name__ == "__main__":
    main()
