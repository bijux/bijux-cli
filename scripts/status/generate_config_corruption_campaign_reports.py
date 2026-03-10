#!/usr/bin/env python3
"""Generate randomized config-corruption campaign artifacts for TODOs 121-140."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

CAMPAIGN_TEST = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "randomized_config_corruption_campaigns.rs"
REGRESSION_TEST = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "config_corruption_campaign_regressions.rs"
MIN_CASES_DIR = ROOT / "crates" / "bijux-cli" / "tests" / "fuzz" / "config_corruption_minimized_cases"

REQUIRED_TESTS = {
    121: (CAMPAIGN_TEST, "randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands"),
    122: (CAMPAIGN_TEST, "randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands"),
    123: (CAMPAIGN_TEST, "randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands"),
    124: (CAMPAIGN_TEST, "randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands"),
    125: (CAMPAIGN_TEST, "randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands"),
    126: (CAMPAIGN_TEST, "randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands"),
    127: (CAMPAIGN_TEST, "randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands"),
    128: (CAMPAIGN_TEST, "randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands"),
    129: (CAMPAIGN_TEST, "config_mutations_never_silently_destroy_unrelated_valid_keys"),
    130: (CAMPAIGN_TEST, "config_corruption_has_stable_failure_class_and_recovery_path"),
    131: (CAMPAIGN_TEST, "failed_config_load_rolls_back_and_preserves_coherent_state"),
    132: (CAMPAIGN_TEST, "state_doctor_reports_corruption_introduced_by_campaign_harness"),
    133: (CAMPAIGN_TEST, "repeated_run_corruption_inputs_are_deterministic_for_config_command_set"),
    136: (REGRESSION_TEST, "minimized_config_corruption_campaign_cases_replay_without_crashing"),
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
        ["cargo", "test", "-p", "bijux-cli", "--test", "bin_surface", "randomized_config_corruption_campaigns::"]
    )
    regression_run = run_test(
        ["cargo", "test", "-p", "bijux-cli", "--test", "bin_surface", "config_corruption_campaign_regressions::"]
    )

    minimized_cases = sorted(str(p.relative_to(ROOT)).replace("\\", "/") for p in MIN_CASES_DIR.glob("*.json"))
    missing = [row["todo"] for row in coverage if row["status"] != "covered"]

    campaign = {
        "generated_at": now,
        "generator": "scripts/status/generate_config_corruption_campaign_reports.py",
        "scope": "randomized config corruption campaigns",
        "tasks": list(range(121, 129)),
        "status": "complete" if campaign_run["ok"] else "partial",
        "campaign_suite": campaign_run,
    }

    invariants = {
        "generated_at": now,
        "generator": "scripts/status/generate_config_corruption_campaign_reports.py",
        "scope": "config corruption invariants",
        "tasks": [129, 130, 131, 132, 133],
        "status": "complete"
        if campaign_run["ok"] and not any(todo in missing for todo in [129, 130, 131, 132, 133])
        else "partial",
        "todo_coverage": [row for row in coverage if 129 <= int(row["todo"]) <= 133],
    }

    corpus_retention = {
        "generated_at": now,
        "generator": "scripts/status/generate_config_corruption_campaign_reports.py",
        "scope": "config corruption corpus retention",
        "tasks": [134],
        "status": "complete" if minimized_cases else "partial",
        "minimized_case_count": len(minimized_cases),
        "minimized_cases": minimized_cases,
    }

    triage = {
        "generated_at": now,
        "generator": "scripts/status/generate_config_corruption_campaign_reports.py",
        "scope": "config corruption triage",
        "tasks": [135],
        "status": "clean" if campaign_run["ok"] and regression_run["ok"] else "needs-triage",
        "campaign_suite_ok": campaign_run["ok"],
        "regression_suite_ok": regression_run["ok"],
    }

    regression = {
        "generated_at": now,
        "generator": "scripts/status/generate_config_corruption_campaign_reports.py",
        "scope": "config corruption regression replay",
        "tasks": [136],
        "status": "clean" if regression_run["ok"] else "drift",
        "minimized_cases": minimized_cases,
    }

    severity = {
        "generated_at": now,
        "generator": "scripts/status/generate_config_corruption_campaign_reports.py",
        "scope": "config corruption severity classification",
        "tasks": [137],
        "status": "complete",
        "classes": {
            "critical": ["write-path panic", "state file replacement with empty content"],
            "high": ["rollback failure", "nondeterministic failure class"],
            "medium": ["malformed input with clean failure"],
            "low": ["recoverable duplicate-key or whitespace anomalies"],
        },
    }

    recovery = {
        "generated_at": now,
        "generator": "scripts/status/generate_config_corruption_campaign_reports.py",
        "scope": "config corruption recovery classification",
        "tasks": [138],
        "status": "complete",
        "paths": {
            "stable_failure": ["usage/validation failure with unchanged file content"],
            "self_recovery": ["repair input and rerun command to success"],
            "rollback_preserved": ["failed load keeps previous coherent config"],
        },
    }

    determinism = {
        "generated_at": now,
        "generator": "scripts/status/generate_config_corruption_campaign_reports.py",
        "scope": "config corruption determinism",
        "tasks": [139],
        "status": "complete" if campaign_run["ok"] else "partial",
        "deterministic_failure_class_required": True,
        "evidence": "crates/bijux-cli/tests/bin_surface/randomized_config_corruption_campaigns.rs::repeated_run_corruption_inputs_are_deterministic_for_config_command_set",
    }

    contract = {
        "generated_at": now,
        "generator": "scripts/status/generate_config_corruption_campaign_reports.py",
        "scope": "config corruption release-blocking contract",
        "tasks": list(range(121, 141)),
        "status": "frozen"
        if campaign_run["ok"] and regression_run["ok"] and minimized_cases and not missing
        else "partial",
        "missing_todos": missing,
        "release_blocking": True,
        "policy": "config corruption campaign coverage and deterministic rollback behavior are required before release",
    }

    write_json("config_corruption_campaign_artifact.json", campaign)
    write_json("config_corruption_invariants_artifact.json", invariants)
    write_json("config_corruption_corpus_retention_artifact.json", corpus_retention)
    write_json("config_corruption_triage_artifact.json", triage)
    write_json("config_corruption_regression_artifact.json", regression)
    write_json("config_corruption_severity_classification.json", severity)
    write_json("config_corruption_recovery_classification.json", recovery)
    write_json("config_corruption_determinism_artifact.json", determinism)
    write_json("config_corruption_release_blocking_contract.json", contract)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
