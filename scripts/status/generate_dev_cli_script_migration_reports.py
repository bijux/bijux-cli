#!/usr/bin/env python3
"""Generate dev-cli script migration reports from canonical command outputs."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def run_json(args: list[str]) -> dict:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli", "--", *args, "--format", "json", "--no-pretty"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(proc.stdout or "{}")


def write(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    remaining = run_json(["dev", "cli", "scripts", "remaining"])
    migrated = run_json(["dev", "cli", "scripts", "migrated"])
    diff = run_json(["dev", "cli", "scripts", "diff"])

    write(STATUS / "dev_cli_scripts_remaining_report.json", remaining)
    write(STATUS / "dev_cli_scripts_migrated_report.json", migrated)
    write(STATUS / "dev_cli_scripts_diff_report.json", diff)

    ranking = {
        "ranking": [
            {
                "script": row.get("from"),
                "replacement": row.get("to"),
                "maintainer_value_rank": row.get("maintainer_value_rank", 0),
            }
            for row in migrated.get("migrated", [])
        ]
    }
    ranking["ranking"].sort(key=lambda row: int(row["maintainer_value_rank"]), reverse=True)
    write(STATUS / "dev_cli_script_value_ranking.json", ranking)

    make_target_inventory = {
        "make_targets": remaining.get("make_targets", []),
        "count": len(remaining.get("make_targets", [])),
    }
    write(STATUS / "dev_cli_make_target_inventory.json", make_target_inventory)

    print("wrote artifacts/status/dev_cli_scripts_remaining_report.json")
    print("wrote artifacts/status/dev_cli_scripts_migrated_report.json")
    print("wrote artifacts/status/dev_cli_scripts_diff_report.json")
    print("wrote artifacts/status/dev_cli_script_value_ranking.json")
    print("wrote artifacts/status/dev_cli_make_target_inventory.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
