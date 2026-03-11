#!/usr/bin/env python3
"""Generate top-level dev-cli cockpit command artifacts."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

REPORTS = {
    "dev_cli_status_report.json": ["dev", "cli", "status"],
    "dev_cli_dashboard_report.json": ["dev", "cli", "dashboard"],
    "dev_cli_quickcheck_report.json": ["dev", "cli", "quickcheck"],
    "dev_cli_truth_report.json": ["dev", "cli", "truth"],
    "dev_cli_blockers_report.json": ["dev", "cli", "blockers"],
    "dev_cli_next_report.json": ["dev", "cli", "next"],
}


def run_json(args: list[str]) -> dict:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli", "--", *args, "--format", "json", "--no-pretty"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout or "{}")


def run_text(args: list[str]) -> str:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli", "--", *args, "--format", "text"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return proc.stdout


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)
    payloads: dict[str, dict] = {}
    text_heads: dict[str, str] = {}
    for filename, command in REPORTS.items():
        payload = run_json(command)
        payloads[filename] = payload
        out = STATUS / filename
        out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"wrote {out.relative_to(ROOT)}")
        text = run_text(command)
        text_heads[" ".join(command)] = "\n".join(text.splitlines()[:3])

    (STATUS / "dev_cli_cockpit_text_heads.json").write_text(
        json.dumps(text_heads, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print("wrote artifacts/status/dev_cli_cockpit_text_heads.json")

    status_summary = payloads["dev_cli_status_report.json"]["status_report"]["summary"]
    truth_payload = payloads["dev_cli_truth_report.json"]["truth"]
    truth_done = truth_payload["done"]["summary"]["count"]
    truth_missing = truth_payload["missing"]["summary"]["count"]
    truth_partial = truth_payload["partial"]["summary"]["count"]
    truth_intentional = truth_payload["intentional_differences"]["summary"]["count"]
    blockers = payloads["dev_cli_blockers_report.json"]["blockers"]
    unresolved = {
        row.get("command", "")
        for row in payloads["dev_cli_status_report.json"]["status_report"]["commands"]
        if row.get("status") != "complete"
    }
    blocker_commands: list[str] = []
    for row in blockers:
        if isinstance(row, dict) and isinstance(row.get("command"), str):
            blocker_commands.append(row["command"])
        elif isinstance(row, str):
            blocker_commands.append(row)
    blocker_subset_ok = all(command in unresolved for command in blocker_commands)

    next_policy = payloads["dev_cli_next_report.json"]["next"]["minimalism"]["evidence_first_policy"]
    next_derived_ok = (
        next_policy.get("manual_curated_priority_lists_allowed") is False
        and next_policy.get("roadmap_requires_generated_artifacts") is True
        and bool(next_policy.get("required_artifacts"))
    )
    dashboard_status_match = (
        payloads["dev_cli_dashboard_report.json"]["dashboard"]["status"]["summary"] == status_summary
    )
    count_alignment_ok = (
        status_summary.get("complete", -1) == truth_done
        and status_summary.get("missing", -1) == truth_missing
        and status_summary.get("partial", 0) + status_summary.get("shim", 0) == truth_partial + truth_intentional
    )
    summary_checks = {
        "status_truth_count_alignment": count_alignment_ok,
        "blockers_subset_of_unresolved_status": blocker_subset_ok,
        "next_derived_from_generated_evidence_status": next_derived_ok,
        "dashboard_matches_standalone_status_summary": dashboard_status_match,
    }
    summary_artifact = {
        "scope": "dev cli summary surface",
        "generator": "scripts/status/generate_dev_cli_cockpit_reports.py",
        "checks": summary_checks,
        "status": "complete" if all(summary_checks.values()) else "partial",
    }
    drift = [name for name, ok in summary_checks.items() if not ok]
    drift_artifact = {
        "scope": "dev cli summary surface drift",
        "generator": "scripts/status/generate_dev_cli_cockpit_reports.py",
        "drift_checks": drift,
        "drift_count": len(drift),
        "status": "clean" if not drift else "drift",
    }
    (STATUS / "dev_cli_summary_surface_artifact.json").write_text(
        json.dumps(summary_artifact, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "dev_cli_summary_surface_drift_artifact.json").write_text(
        json.dumps(drift_artifact, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print("wrote artifacts/status/dev_cli_summary_surface_artifact.json")
    print("wrote artifacts/status/dev_cli_summary_surface_drift_artifact.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
