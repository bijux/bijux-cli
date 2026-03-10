#!/usr/bin/env python3
"""Generate adversarial filesystem/process hardening artifacts for TODOs 181-200."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

CAMPAIGN_TEST = ROOT / "crates" / "bijux-cli-bin" / "tests" / "adversarial_fs_process_campaigns.rs"
REGRESSION_TEST = ROOT / "crates" / "bijux-cli-bin" / "tests" / "adversarial_fs_process_campaign_regressions.rs"
MIN_CASES_DIR = ROOT / "crates" / "bijux-cli-bin" / "tests" / "fuzz" / "adversarial_fs_process_minimized_cases"

REQUIRED_TESTS = {
    181: (CAMPAIGN_TEST, "missing_parent_and_type_flip_path_cases_are_handled_without_corruption"),
    182: (CAMPAIGN_TEST, "missing_parent_and_type_flip_path_cases_are_handled_without_corruption"),
    183: (CAMPAIGN_TEST, "missing_parent_and_type_flip_path_cases_are_handled_without_corruption"),
    184: (CAMPAIGN_TEST, "missing_parent_and_type_flip_path_cases_are_handled_without_corruption"),
    185: (CAMPAIGN_TEST, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
    186: (CAMPAIGN_TEST, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
    187: (CAMPAIGN_TEST, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
    188: (CAMPAIGN_TEST, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
    189: (CAMPAIGN_TEST, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
    190: (CAMPAIGN_TEST, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
    191: (CAMPAIGN_TEST, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
    192: (CAMPAIGN_TEST, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
    193: (CAMPAIGN_TEST, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
    194: (CAMPAIGN_TEST, "rename_race_and_temp_leftovers_keep_commands_non_panicking"),
    195: (CAMPAIGN_TEST, "rename_race_and_temp_leftovers_keep_commands_non_panicking"),
    196: (CAMPAIGN_TEST, "rename_race_and_temp_leftovers_keep_commands_non_panicking"),
    197: (CAMPAIGN_TEST, "child_process_failure_paths_surface_normalized_failures_when_plugins_are_broken"),
    198: (CAMPAIGN_TEST, "interrupted_process_behavior_is_normalized_for_interactive_entrypoint"),
}


def write_json(name: str, payload: dict[str, Any]) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def run_test(args: list[str]) -> dict[str, Any]:
    proc = subprocess.run(args, cwd=ROOT, check=False, capture_output=True, text=True)
    return {
        "command": args,
        "exit_code": proc.returncode,
        "ok": proc.returncode == 0,
        "stdout": proc.stdout[-4000:],
        "stderr": proc.stderr[-4000:],
    }


def main() -> int:
    now = datetime.now(timezone.utc).isoformat()
    texts = {
        CAMPAIGN_TEST: CAMPAIGN_TEST.read_text(encoding="utf-8") if CAMPAIGN_TEST.exists() else "",
        REGRESSION_TEST: REGRESSION_TEST.read_text(encoding="utf-8") if REGRESSION_TEST.exists() else "",
    }

    coverage = []
    for todo, (path, test_name) in sorted(REQUIRED_TESTS.items()):
        covered = f"fn {test_name}(" in texts[path]
        coverage.append(
            {
                "todo": todo,
                "test": test_name,
                "status": "covered" if covered else "missing",
                "evidence": str(path.relative_to(ROOT)).replace("\\", "/"),
            }
        )

    campaign_run = run_test(["cargo", "test", "-p", "bijux-cli-bin", "--test", "adversarial_fs_process_campaigns"])
    regression_run = run_test(["cargo", "test", "-p", "bijux-cli-bin", "--test", "adversarial_fs_process_campaign_regressions"])

    minimized_cases = sorted(str(p.relative_to(ROOT)).replace("\\", "/") for p in MIN_CASES_DIR.glob("*.json"))
    missing = [row["todo"] for row in coverage if row["status"] != "covered"]

    matrix = {
        "generated_at": now,
        "generator": "scripts/status/generate_adversarial_fs_process_reports.py",
        "scope": "adversarial filesystem/process matrix",
        "tasks": list(range(181, 199)),
        "status": "complete" if campaign_run["ok"] and not missing else "partial",
        "todo_coverage": coverage,
        "campaign_suite": campaign_run,
    }

    artifact = {
        "generated_at": now,
        "generator": "scripts/status/generate_adversarial_fs_process_reports.py",
        "scope": "adversarial filesystem/process evidence artifact",
        "tasks": [199],
        "status": "complete" if campaign_run["ok"] and regression_run["ok"] else "partial",
        "minimized_case_count": len(minimized_cases),
        "minimized_cases": minimized_cases,
        "regression_suite": regression_run,
    }

    contract = {
        "generated_at": now,
        "generator": "scripts/status/generate_adversarial_fs_process_reports.py",
        "scope": "adversarial filesystem/process hardening contract",
        "tasks": list(range(181, 201)),
        "status": "frozen"
        if campaign_run["ok"] and regression_run["ok"] and minimized_cases and not missing
        else "partial",
        "missing_todos": missing,
        "policy": "adversarial fs/process behavior is first-class hardening and permanently gated",
    }

    write_json("adversarial_fs_process_matrix.json", matrix)
    write_json("adversarial_fs_process_artifact.json", artifact)
    write_json("adversarial_fs_process_contract.json", contract)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
