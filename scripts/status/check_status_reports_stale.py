#!/usr/bin/env python3
"""Fail when generated status reports are stale in git working tree."""

from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
def run(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, cwd=ROOT, check=False, capture_output=True, text=True)


def main() -> int:
    gen = run(["python3", "scripts/status/generate_status_reports.py"])
    if gen.returncode != 0:
        print(gen.stderr.strip() or gen.stdout.strip())
        return gen.returncode
    migration = run(["python3", "scripts/status/generate_command_migration_matrix.py"])
    if migration.returncode != 0:
        print(migration.stderr.strip() or migration.stdout.strip())
        return migration.returncode
    inventory = run(["python3", "scripts/status/generate_command_surface_inventory.py"])
    if inventory.returncode != 0:
        print(inventory.stderr.strip() or inventory.stdout.strip())
        return inventory.returncode

    diff = run(["git", "diff", "--name-only", "--", "artifacts/status"])
    changed = [
        line.strip()
        for line in diff.stdout.splitlines()
        if (
            (line.strip().startswith("artifacts/status/status") and line.strip().endswith(".json"))
            or line.strip().startswith("artifacts/status/command_migration_")
            or line.strip() == "artifacts/status/command_migration_matrix.json"
            or line.strip() == "artifacts/status/command_migration_matrix.txt"
            or line.strip() == "artifacts/status/documented_python_commands_not_proven_in_rust.json"
            or line.strip() == "artifacts/status/public_python_paths_still_reachable.json"
            or line.strip() == "artifacts/status/legacy_alias_paths_still_accepted.json"
            or line.strip() == "artifacts/status/compatibility_shims_still_active.json"
        )
    ]
    if changed:
        print("STATUS REPORT STALE: regenerate and commit updated artifacts:")
        for item in changed:
            print(f" - {item}")
        return 1

    print("Status report freshness check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
