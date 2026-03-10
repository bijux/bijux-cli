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
    bridge_dup = run(["python3", "scripts/status/generate_bridge_duplicate_law_report.py"])
    if bridge_dup.returncode != 0:
        print(bridge_dup.stderr.strip() or bridge_dup.stdout.strip())
        return bridge_dup.returncode
    install_neutrality = run(["python3", "scripts/status/generate_install_neutrality_reports.py"])
    if install_neutrality.returncode != 0:
        print(install_neutrality.stderr.strip() or install_neutrality.stdout.strip())
        return install_neutrality.returncode
    compatibility = run(["python3", "scripts/status/generate_compatibility_shim_reports.py"])
    if compatibility.returncode != 0:
        print(compatibility.stderr.strip() or compatibility.stdout.strip())
        return compatibility.returncode
    parity_law = run(["python3", "scripts/parity/generate_command_law_reports.py"])
    if parity_law.returncode != 0:
        print(parity_law.stderr.strip() or parity_law.stdout.strip())
        return parity_law.returncode

    diff = run(["git", "diff", "--name-only", "--", "artifacts/status"])
    parity_diff = run(["git", "diff", "--name-only", "--", "artifacts/parity"])
    changed = [
        line.strip()
        for line in diff.stdout.splitlines()
        if (
            (line.strip().startswith("artifacts/status/status") and line.strip().endswith(".json"))
            or line.strip().startswith("artifacts/status/command_migration_")
            or line.strip() == "artifacts/status/command_migration_matrix.json"
            or line.strip() == "artifacts/status/command_migration_matrix.txt"
            or line.strip() == "artifacts/status/command_migration_repl_paths.json"
            or line.strip() == "artifacts/status/command_migration_python_bridge_entrypoints.json"
            or line.strip() == "artifacts/status/documented_python_commands_not_proven_in_rust.json"
            or line.strip() == "artifacts/status/public_python_paths_still_reachable.json"
            or line.strip() == "artifacts/status/legacy_alias_paths_still_accepted.json"
            or line.strip() == "artifacts/status/compatibility_shims_still_active.json"
            or line.strip() == "artifacts/status/bridge_duplicate_law_report.json"
            or line.strip() == "artifacts/status/install_neutrality_report.json"
            or line.strip() == "artifacts/status/active_runtime_report.json"
            or line.strip() == "artifacts/status/compatibility_shim_inventory.json"
            or line.strip() == "artifacts/status/compatibility_alias_inventory.json"
            or line.strip() == "artifacts/status/compatibility_shim_count_delta.json"
            or line.strip() == "artifacts/status/compatibility_alias_count_delta.json"
            or line.strip() == "artifacts/status/compatibility_shim_count_report.json"
            or line.strip() == "artifacts/status/compatibility_alias_count_report.json"
            or line.strip() == "artifacts/status/hidden_alias_inventory.json"
            or line.strip() == "artifacts/status/old_python_path_tolerance_inventory.json"
            or line.strip() == "artifacts/status/live_compatibility_shims.json"
            or line.strip() == "artifacts/status/live_compatibility_aliases.json"
        )
    ]
    if changed:
        print("STATUS REPORT STALE: regenerate and commit updated artifacts:")
        for item in changed:
            print(f" - {item}")
        return 1

    parity_changed = [
        line.strip()
        for line in parity_diff.stdout.splitlines()
        if line.strip()
        in {
            "artifacts/parity/command_precedence_report.json",
            "artifacts/parity/command_flag_normalization_report.json",
            "artifacts/parity/command_stream_report.json",
            "artifacts/parity/command_exit_code_report.json",
            "artifacts/parity/command_help_diff_report.json",
            "artifacts/parity/command_machine_output_diff_report.json",
            "artifacts/parity/parity_dashboard.json",
            "artifacts/parity/parity_dashboard.txt",
        }
    ]
    if parity_changed:
        print("PARITY REPORT STALE: regenerate and commit updated parity artifacts:")
        for item in parity_changed:
            print(f" - {item}")
        return 1

    print("Status report freshness check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
