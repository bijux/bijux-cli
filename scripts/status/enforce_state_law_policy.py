#!/usr/bin/env python3
"""Enforce canonical state-law reports and write-path rules."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(read(path))


def main() -> int:
    failures: list[str] = []

    required = [
        STATUS / "state_file_inventory.json",
        STATUS / "state_file_readers.json",
        STATUS / "state_file_writers.json",
        STATUS / "state_file_mutation_paths.json",
        STATUS / "state_write_guarantees.json",
        STATUS / "state_recovery_guarantees.json",
        STATUS / "state_complexity_report.json",
        STATUS / "state_migration_status.json",
        STATUS / "unified_state_behavior_report.json",
        STATUS / "unified_state_corruption_report.json",
        STATUS / "unified_state_rollback_report.json",
        STATUS / "unified_state_path_resolution_report.json",
        STATUS / "unified_state_doctor_snapshots.json",
        STATUS / "unified_state_audit_payload.json",
    ]
    for item in required:
        if not item.exists():
            failures.append(f"missing required artifact: {item.relative_to(ROOT)}")

    core_storage = read(ROOT / "crates/bijux-cli-core/src/config/storage.rs")
    install_compat = read(ROOT / "crates/bijux-cli-install/src/compatibility.rs")

    if "atomic_write_text(" not in core_storage:
        failures.append("core config repository does not use atomic_write_text")
    if "atomic_write_text(" not in install_compat:
        failures.append("install compatibility config does not use atomic_write_text")

    if "with_extension(\"tmp\")" in core_storage:
        failures.append("core config repository still carries ad-hoc temp-file logic")
    if "with_extension(\"tmp\")" in install_compat:
        failures.append("install compatibility config still carries ad-hoc temp-file logic")

    inventory = read_json(STATUS / "state_file_inventory.json")
    files = inventory.get("state_files", []) if isinstance(inventory, dict) else []
    if len(files) < 4:
        failures.append("state inventory has fewer than four files, likely incomplete")

    for failure in failures:
        print(f"STATE LAW FAILURE: {failure}")

    return 2 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
