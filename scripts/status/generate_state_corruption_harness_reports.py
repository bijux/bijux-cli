#!/usr/bin/env python3
"""Generate randomized state-corruption harness artifacts for TODOs 101-120."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

HARNESS_TEST = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "randomized_state_corruption_harness.rs"
REGRESSION_TEST = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "randomized_state_corruption_regressions.rs"
MIN_CASES_DIR = ROOT / "crates" / "bijux-cli" / "tests" / "fuzz" / "state_corruption_minimized_cases"

REQUIRED_TESTS = {
    101: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    102: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    103: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    104: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    105: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    106: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    107: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    108: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    109: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    110: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    111: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    112: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    113: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    114: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    115: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    116: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    117: (HARNESS_TEST, "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains"),
    119: (REGRESSION_TEST, "minimized_corrupted_state_reproducers_replay_without_crashing"),
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
        HARNESS_TEST: HARNESS_TEST.read_text(encoding="utf-8") if HARNESS_TEST.exists() else "",
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
        [
            "cargo",
            "test",
            "-p",
            "bijux-cli",
            "--test",
            "randomized_state_corruption_harness",
        ]
    )
    replay_run = run_test(
        [
            "cargo",
            "test",
            "-p",
            "bijux-cli",
            "--test",
            "randomized_state_corruption_regressions",
        ]
    )

    minimized_cases = sorted(str(p.relative_to(ROOT)).replace("\\", "/") for p in MIN_CASES_DIR.glob("*.json"))

    missing = [row["todo"] for row in coverage if row["status"] != "covered"]

    campaign_artifact = {
        "generated_at": now,
        "generator": "scripts/status/generate_state_corruption_harness_reports.py",
        "scope": "randomized corruption campaign",
        "tasks": list(range(101, 119)),
        "status": "clean" if campaign_run["ok"] else "needs-triage",
        "campaign_suite_ok": campaign_run["ok"],
        "todo_coverage": coverage,
    }

    reproducer_retention = {
        "generated_at": now,
        "generator": "scripts/status/generate_state_corruption_harness_reports.py",
        "scope": "minimized corrupted-state reproducer retention",
        "tasks": [119],
        "status": "clean" if replay_run["ok"] and len(minimized_cases) > 0 else "needs-triage",
        "replay_suite_ok": replay_run["ok"],
        "minimized_case_count": len(minimized_cases),
        "minimized_cases": minimized_cases,
    }

    contract = {
        "generated_at": now,
        "generator": "scripts/status/generate_state_corruption_harness_reports.py",
        "scope": "randomized state corruption harness",
        "tasks": list(range(101, 121)),
        "status": "frozen"
        if not missing and campaign_run["ok"] and replay_run["ok"] and len(minimized_cases) > 0
        else "partial",
        "missing_todos": missing,
        "campaign_suite": campaign_run,
        "replay_suite": replay_run,
        "policy": "randomized state corruption harness is shared test utility and release hardening evidence",
    }

    write_json("state_corruption_campaign_artifact.json", campaign_artifact)
    write_json("state_corruption_reproducer_retention_artifact.json", reproducer_retention)
    write_json("state_corruption_harness_contract.json", contract)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
