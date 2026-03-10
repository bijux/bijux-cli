#!/usr/bin/env python3
"""Generate dev-cli release command artifacts for CI and maintainer review."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

REPORTS = {
    "dev_cli_release_status_report.json": ["dev", "cli", "release", "status"],
    "dev_cli_release_evidence_report.json": ["dev", "cli", "release", "evidence"],
    "dev_cli_release_readiness_report.json": ["dev", "cli", "release", "readiness"],
    "dev_cli_release_diff_report.json": ["dev", "cli", "release", "diff"],
    "dev_cli_release_gaps_report.json": ["dev", "cli", "release", "gaps"],
    "dev_cli_release_summary_report.json": ["dev", "cli", "release", "summary"],
    "dev_cli_release_manifest_report.json": ["dev", "cli", "release", "manifest"],
    "dev_cli_release_notes_report.json": ["dev", "cli", "release", "notes"],
    "dev_cli_release_behavior_changes_report.json": ["dev", "cli", "release", "behavior-changes"],
    "dev_cli_release_intentional_differences_report.json": ["dev", "cli", "release", "intentional-differences"],
    "dev_cli_release_unresolved_gaps_report.json": ["dev", "cli", "release", "unresolved-gaps"],
    "dev_cli_release_compatibility_leftovers_report.json": ["dev", "cli", "release", "compatibility-leftovers"],
}


def run_json(args: list[str]) -> dict:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-bin", "--", *args, "--format", "json", "--no-pretty"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout or "{}")


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)
    for filename, command in REPORTS.items():
        payload = run_json(command)
        out = STATUS / filename
        out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"wrote {out.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
