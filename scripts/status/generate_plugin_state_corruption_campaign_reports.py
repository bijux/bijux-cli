#!/usr/bin/env python3
"""Generate plugin/history/memory corruption campaign artifacts for TODOs 141-160."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

CAMPAIGN_TEST = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "randomized_plugin_state_corruption_campaigns.rs"
REGRESSION_TEST = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "plugin_state_corruption_campaign_regressions.rs"
MIN_CASES_DIR = ROOT / "crates" / "bijux-cli" / "tests" / "fuzz" / "plugin_state_corruption_minimized_cases"

REQUIRED_TESTS = {
    141: (CAMPAIGN_TEST, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths"),
    142: (CAMPAIGN_TEST, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths"),
    143: (CAMPAIGN_TEST, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths"),
    144: (CAMPAIGN_TEST, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths"),
    145: (CAMPAIGN_TEST, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths"),
    146: (CAMPAIGN_TEST, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths"),
    147: (CAMPAIGN_TEST, "one_broken_plugin_never_hides_unrelated_healthy_plugins"),
    148: (CAMPAIGN_TEST, "plugin_list_is_deterministic_for_identical_corrupted_registry"),
    149: (CAMPAIGN_TEST, "plugin_registry_rollback_preserves_coherence_after_failed_mutation_paths"),
    150: (CAMPAIGN_TEST, "plugin_registry_rollback_preserves_coherence_after_failed_mutation_paths"),
    151: (CAMPAIGN_TEST, "plugin_doctor_reports_corruption_injected_by_campaign"),
    152: (CAMPAIGN_TEST, "history_and_memory_corruption_recovery_remains_stable_and_policy_compliant"),
    153: (CAMPAIGN_TEST, "history_and_memory_corruption_recovery_remains_stable_and_policy_compliant"),
    154: (CAMPAIGN_TEST, "history_and_memory_corruption_recovery_remains_stable_and_policy_compliant"),
    155: (CAMPAIGN_TEST, "history_and_memory_corruption_recovery_remains_stable_and_policy_compliant"),
    158: (REGRESSION_TEST, "minimized_plugin_state_corruption_cases_replay_without_crashing"),
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
        [
            "cargo",
            "test",
            "-p",
            "bijux-cli",
            "--test",
            "bin_surface",
            "randomized_plugin_state_corruption_campaigns::",
        ]
    )
    regression_run = run_test(
        [
            "cargo",
            "test",
            "-p",
            "bijux-cli",
            "--test",
            "bin_surface",
            "plugin_state_corruption_campaign_regressions::",
        ]
    )

    minimized_cases = sorted(str(p.relative_to(ROOT)).replace("\\", "/") for p in MIN_CASES_DIR.glob("*.json"))
    missing = [row["todo"] for row in coverage if row["status"] != "covered"]

    campaign = {
        "generated_at": now,
        "generator": "scripts/status/generate_plugin_state_corruption_campaign_reports.py",
        "scope": "plugin/history/memory corruption campaigns",
        "tasks": list(range(141, 156)),
        "status": "complete" if campaign_run["ok"] else "partial",
        "campaign_suite": campaign_run,
    }

    corpus_retention = {
        "generated_at": now,
        "generator": "scripts/status/generate_plugin_state_corruption_campaign_reports.py",
        "scope": "plugin/history/memory corruption corpus retention",
        "tasks": [156],
        "status": "complete" if minimized_cases else "partial",
        "minimized_case_count": len(minimized_cases),
        "minimized_cases": minimized_cases,
    }

    triage = {
        "generated_at": now,
        "generator": "scripts/status/generate_plugin_state_corruption_campaign_reports.py",
        "scope": "plugin/history/memory corruption triage",
        "tasks": [157],
        "status": "clean" if campaign_run["ok"] and regression_run["ok"] else "needs-triage",
        "campaign_suite_ok": campaign_run["ok"],
        "regression_suite_ok": regression_run["ok"],
    }

    regressions = {
        "generated_at": now,
        "generator": "scripts/status/generate_plugin_state_corruption_campaign_reports.py",
        "scope": "plugin/history/memory corruption regression replay",
        "tasks": [158],
        "status": "clean" if regression_run["ok"] else "drift",
        "minimized_cases": minimized_cases,
    }

    severity = {
        "generated_at": now,
        "generator": "scripts/status/generate_plugin_state_corruption_campaign_reports.py",
        "scope": "plugin/history/memory corruption severity classification",
        "tasks": [159],
        "status": "complete",
        "classes": {
            "critical": ["plugin registry write rollback failure", "state read panic"],
            "high": ["nondeterministic plugin list under identical corrupted input", "memory recovery drift"],
            "medium": ["history malformed entries with degraded but successful read"],
            "low": ["doctor self-repair with stable output"],
        },
    }

    contract = {
        "generated_at": now,
        "generator": "scripts/status/generate_plugin_state_corruption_campaign_reports.py",
        "scope": "plugin/history/memory corruption hardening contract",
        "tasks": list(range(141, 161)),
        "status": "frozen"
        if campaign_run["ok"] and regression_run["ok"] and minimized_cases and not missing
        else "partial",
        "missing_todos": missing,
        "policy": "plugin/history/memory corruption campaigns are required hardening coverage",
    }

    write_json("plugin_state_corruption_campaign_artifact.json", campaign)
    write_json("plugin_state_corruption_corpus_retention_artifact.json", corpus_retention)
    write_json("plugin_state_corruption_triage_artifact.json", triage)
    write_json("plugin_state_corruption_regression_artifact.json", regressions)
    write_json("plugin_state_corruption_severity_classification.json", severity)
    write_json("plugin_state_corruption_contract.json", contract)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
