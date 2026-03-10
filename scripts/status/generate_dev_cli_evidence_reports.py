#!/usr/bin/env python3
"""Generate dev-cli evidence command artifacts for CI and maintainer review."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

REPORTS = {
    "dev_cli_evidence_list_report.json": ["dev", "cli", "evidence", "list"],
    "dev_cli_evidence_audit_report.json": ["dev", "cli", "evidence", "audit"],
    "dev_cli_evidence_stale_report.json": ["dev", "cli", "evidence", "stale"],
    "dev_cli_evidence_matrix_report.json": ["dev", "cli", "evidence", "matrix"],
    "dev_cli_evidence_website_export_report.json": ["dev", "cli", "evidence", "website-export"],
    "dev_cli_evidence_ci_export_report.json": ["dev", "cli", "evidence", "ci-export"],
    "dev_cli_evidence_release_export_report.json": ["dev", "cli", "evidence", "release-export"],
    "dev_cli_evidence_command_map_report.json": ["dev", "cli", "evidence", "command-map"],
    "dev_cli_evidence_parity_map_report.json": ["dev", "cli", "evidence", "parity-map"],
}


def run_json(args: list[str]) -> dict:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-core", "--", *args, "--format", "json", "--no-pretty"],
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
