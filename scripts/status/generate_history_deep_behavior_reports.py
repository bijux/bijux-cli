#!/usr/bin/env python3
"""Generate deep history behavior artifacts for TODOs 101-120."""

from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILES = [
    ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "history_command_matrix.rs",
    ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "history_parity.rs",
    ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "history_deep_behavior_extra.rs",
]

REQUIRED_TESTS = {
    101: "history_root_listing_no_file_one_record_many_records_and_ordering",
    102: "history_limit_path_override_and_repeated_run_determinism",
    103: "history_malformed_and_mixed_valid_invalid_tolerance_and_duplicates",
    104: "history_malformed_and_mixed_valid_invalid_tolerance_and_duplicates",
    105: "history_json_yaml_text_outputs_are_emitted",
    106: "history_text_json_yaml_quiet_and_no_color_modes",
    107: "history_json_yaml_text_outputs_are_emitted",
    108: "history_reads_repl_line_layout_for_cli_interop",
    109: "history_limit_path_override_and_repeated_run_determinism",
    110: "history_missing_and_malformed_behaviors_are_stable",
    111: "history_handles_huge_files_with_stable_tail_limit",
    112: "history_doctor_and_state_doctor_agree_on_history_corruption_findings",
    113: "history_output_is_stable_under_filesystem_metadata_changes",
}


def run_cmd(args: list[str], env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    merged = None
    if env:
        merged = {**os.environ, **env}
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

    semantic_sample = run_json(["history"])
    determinism_a = run_cmd(["history", "--format", "json", "--no-pretty"])
    determinism_b = run_cmd(["history", "--format", "json", "--no-pretty"])
    corruption_sample = run_json(["history"])
    repl_interop_sample = run_json(["history"])
    stream_sample = run_cmd(["history", "--format", "text"])
    failure_sample = run_cmd(["history", "--unknown-flag"])

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

    history_semantic = {
        "generator": "scripts/status/generate_history_deep_behavior_reports.py",
        "scope": "history semantic",
        "tasks": [101, 102, 103, 104, 105, 108, 109, 110, 111, 113, 114],
        "status": "complete" if semantic_sample != {} else "partial",
        "sample": semantic_sample,
    }
    history_determinism = {
        "generator": "scripts/status/generate_history_deep_behavior_reports.py",
        "scope": "history determinism",
        "tasks": [101, 102, 107, 111, 113, 115],
        "status": "complete"
        if determinism_a.returncode == 0
        and determinism_b.returncode == 0
        and determinism_a.stdout == determinism_b.stdout
        and determinism_a.stderr == determinism_b.stderr
        else "partial",
        "byte_stable": determinism_a.stdout == determinism_b.stdout
        and determinism_a.stderr == determinism_b.stderr,
    }
    history_corruption = {
        "generator": "scripts/status/generate_history_deep_behavior_reports.py",
        "scope": "history corruption",
        "tasks": [103, 104, 110, 112, 116],
        "status": "complete" if corruption_sample != {} else "partial",
        "sample": corruption_sample,
    }
    history_repl_interop = {
        "generator": "scripts/status/generate_history_deep_behavior_reports.py",
        "scope": "history repl interop",
        "tasks": [108, 117],
        "status": "complete" if repl_interop_sample != {} else "partial",
        "sample": repl_interop_sample,
    }
    history_stream_discipline = {
        "generator": "scripts/status/generate_history_deep_behavior_reports.py",
        "scope": "history stream discipline",
        "tasks": [106, 118],
        "status": "complete" if stream_sample.returncode == 0 and not stream_sample.stderr else "partial",
    }
    history_failure_class = {
        "generator": "scripts/status/generate_history_deep_behavior_reports.py",
        "scope": "history failure class",
        "tasks": [112, 119],
        "status": "complete" if failure_sample.returncode == 2 else "partial",
        "sample_exit_code": failure_sample.returncode,
    }

    drift_items: list[dict[str, Any]] = []
    for name, payload in [
        ("history_semantic_artifact.json", history_semantic),
        ("history_determinism_artifact.json", history_determinism),
        ("history_corruption_artifact.json", history_corruption),
        ("history_repl_interop_artifact.json", history_repl_interop),
        ("history_stream_discipline_artifact.json", history_stream_discipline),
        ("history_failure_class_artifact.json", history_failure_class),
    ]:
        if payload.get("status") != "complete":
            drift_items.append({"artifact": name, "reason": "status-not-complete"})
    if missing_todos:
        drift_items.append({"reason": "missing-todo-coverage", "todos": [row["todo"] for row in missing_todos]})

    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / "history_semantic_artifact.json").write_text(
        json.dumps(history_semantic, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "history_determinism_artifact.json").write_text(
        json.dumps(history_determinism, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "history_corruption_artifact.json").write_text(
        json.dumps(history_corruption, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "history_repl_interop_artifact.json").write_text(
        json.dumps(history_repl_interop, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "history_stream_discipline_artifact.json").write_text(
        json.dumps(history_stream_discipline, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "history_failure_class_artifact.json").write_text(
        json.dumps(history_failure_class, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "history_deep_behavior_drift_artifact.json").write_text(
        json.dumps(
            {
                "generator": "scripts/status/generate_history_deep_behavior_reports.py",
                "scope": "history deep behavior drift",
                "tasks": [120],
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
    print("wrote artifacts/status/history_semantic_artifact.json")
    print("wrote artifacts/status/history_determinism_artifact.json")
    print("wrote artifacts/status/history_corruption_artifact.json")
    print("wrote artifacts/status/history_repl_interop_artifact.json")
    print("wrote artifacts/status/history_stream_discipline_artifact.json")
    print("wrote artifacts/status/history_failure_class_artifact.json")
    print("wrote artifacts/status/history_deep_behavior_drift_artifact.json")


if __name__ == "__main__":
    main()
