#!/usr/bin/env python3
"""Generate top-level dev-cli cockpit command artifacts."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

REPORTS = {
    "dev_cli_dashboard_report.json": ["dev", "cli", "dashboard"],
    "dev_cli_quickcheck_report.json": ["dev", "cli", "quickcheck"],
    "dev_cli_truth_report.json": ["dev", "cli", "truth"],
    "dev_cli_blockers_report.json": ["dev", "cli", "blockers"],
    "dev_cli_next_report.json": ["dev", "cli", "next"],
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


def run_text(args: list[str]) -> str:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-bin", "--", *args, "--format", "text"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return proc.stdout


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)
    text_heads: dict[str, str] = {}
    for filename, command in REPORTS.items():
        payload = run_json(command)
        out = STATUS / filename
        out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"wrote {out.relative_to(ROOT)}")
        text = run_text(command)
        text_heads[" ".join(command)] = "\n".join(text.splitlines()[:3])

    (STATUS / "dev_cli_cockpit_text_heads.json").write_text(
        json.dumps(text_heads, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print("wrote artifacts/status/dev_cli_cockpit_text_heads.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
