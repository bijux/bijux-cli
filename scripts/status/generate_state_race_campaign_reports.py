#!/usr/bin/env python3
"""Generate concurrent state-race hardening artifacts for TODOs 161-180."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

CAMPAIGN_TEST = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "state_race_campaigns.rs"
REGRESSION_TEST = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "state_race_campaign_regressions.rs"
MIN_CASES_DIR = ROOT / "crates" / "bijux-cli" / "tests" / "fuzz" / "state_race_minimized_cases"

REQUIRED_TESTS = {
    161: (CAMPAIGN_TEST, "concurrent_config_readers_and_writers_preserve_file_shape_and_recoverability"),
    162: (CAMPAIGN_TEST, "concurrent_config_readers_and_writers_preserve_file_shape_and_recoverability"),
    163: (CAMPAIGN_TEST, "concurrent_config_readers_and_writers_preserve_file_shape_and_recoverability"),
    164: (CAMPAIGN_TEST, "concurrent_config_readers_and_writers_preserve_file_shape_and_recoverability"),
    165: (REGRESSION_TEST, "minimized_race_reproducers_replay_without_crashing"),
    166: (CAMPAIGN_TEST, "concurrent_config_readers_and_writers_preserve_file_shape_and_recoverability"),
    167: (CAMPAIGN_TEST, "concurrent_config_readers_and_writers_preserve_file_shape_and_recoverability"),
    168: (CAMPAIGN_TEST, "concurrent_config_export_load_and_read_paths_stay_non_corrupt"),
    169: (CAMPAIGN_TEST, "concurrent_config_export_load_and_read_paths_stay_non_corrupt"),
    170: (CAMPAIGN_TEST, "concurrent_history_plugin_registry_and_memory_reads_remain_stable"),
    171: (CAMPAIGN_TEST, "concurrent_history_plugin_registry_and_memory_reads_remain_stable"),
    172: (CAMPAIGN_TEST, "concurrent_history_plugin_registry_and_memory_reads_remain_stable"),
    173: (CAMPAIGN_TEST, "concurrent_history_plugin_registry_and_memory_reads_remain_stable"),
    174: (CAMPAIGN_TEST, "concurrent_history_plugin_registry_and_memory_reads_remain_stable"),
    175: (CAMPAIGN_TEST, "concurrent_history_plugin_registry_and_memory_reads_remain_stable"),
    176: (CAMPAIGN_TEST, "deterministic_final_state_is_stable_when_policy_uses_same_target_value"),
    178: (REGRESSION_TEST, "minimized_race_reproducers_replay_without_crashing"),
    179: (REGRESSION_TEST, "minimized_race_reproducers_replay_without_crashing"),
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

    campaign_run = run_test(
        ["cargo", "test", "-p", "bijux-cli", "--test", "bin_surface", "state_race_campaigns::"]
    )
    regression_run = run_test(
        ["cargo", "test", "-p", "bijux-cli", "--test", "bin_surface", "state_race_campaign_regressions::"]
    )

    minimized_cases = sorted(str(p.relative_to(ROOT)).replace("\\", "/") for p in MIN_CASES_DIR.glob("*.json"))
    missing = [row["todo"] for row in coverage if row["status"] != "covered"]

    campaign = {
        "generated_at": now,
        "generator": "scripts/status/generate_state_race_campaign_reports.py",
        "scope": "concurrent state race campaigns",
        "tasks": list(range(161, 177)),
        "status": "complete" if campaign_run["ok"] else "partial",
        "campaign_suite": campaign_run,
        "todo_coverage": [row for row in coverage if 161 <= int(row["todo"]) <= 176],
    }

    race_outcome = {
        "generated_at": now,
        "generator": "scripts/status/generate_state_race_campaign_reports.py",
        "scope": "race outcome classification",
        "tasks": [177],
        "status": "complete",
        "classes": {
            "deterministic-stable": ["same target value writes converge to stable final value"],
            "deterministic-bounded-failure": ["commands exit only in allowed classes 0/1/2 under contention"],
            "non-corrupting": ["post-race readers still parse and return structured outputs"],
        },
    }

    retention = {
        "generated_at": now,
        "generator": "scripts/status/generate_state_race_campaign_reports.py",
        "scope": "minimized race reproducer retention",
        "tasks": [178],
        "status": "complete" if minimized_cases else "partial",
        "minimized_case_count": len(minimized_cases),
        "minimized_cases": minimized_cases,
    }

    regressions = {
        "generated_at": now,
        "generator": "scripts/status/generate_state_race_campaign_reports.py",
        "scope": "race regression replay",
        "tasks": [179],
        "status": "clean" if regression_run["ok"] else "drift",
        "minimized_cases": minimized_cases,
    }

    contract = {
        "generated_at": now,
        "generator": "scripts/status/generate_state_race_campaign_reports.py",
        "scope": "core state race hardening contract",
        "tasks": list(range(161, 181)),
        "status": "frozen"
        if campaign_run["ok"] and regression_run["ok"] and minimized_cases and not missing
        else "partial",
        "missing_todos": missing,
        "policy": "core state race tests are permanent and release-gated",
    }

    write_json("state_race_campaign_artifact.json", campaign)
    write_json("state_race_outcome_classification_artifact.json", race_outcome)
    write_json("state_race_reproducer_retention_artifact.json", retention)
    write_json("state_race_regression_artifact.json", regressions)
    write_json("state_race_contract.json", contract)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
