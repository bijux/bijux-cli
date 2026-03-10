#!/usr/bin/env python3
"""Generate repo health control-plane artifacts."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

REPORTS = {
    "repo_health_report.json": ["dev", "cli", "repo", "health"],
    "repo_drift_report.json": ["dev", "cli", "repo", "drift"],
    "repo_inventories_report.json": ["dev", "cli", "repo", "inventories"],
    "repo_generated_report.json": ["dev", "cli", "repo", "generated"],
    "repo_stale_report.json": ["dev", "cli", "repo", "stale"],
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


def write(name: str, payload: dict) -> None:
    out = STATUS / name
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out.relative_to(ROOT)}")


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)
    for filename, command in REPORTS.items():
        write(filename, run_json(command))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
