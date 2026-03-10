#!/usr/bin/env python3
"""Generate unified state audit artifacts for maintainer status and release evidence."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
PARITY = ROOT / "artifacts" / "parity"
SNAPSHOT_DIR = ROOT / "crates" / "bijux-cli" / "tests" / "snapshots"


def stable_generated_at() -> str:
    source_date_epoch = subprocess.run(
        ["sh", "-lc", "printf %s \"${SOURCE_DATE_EPOCH:-}\""],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if source_date_epoch.isdigit():
        return datetime.fromtimestamp(int(source_date_epoch), tz=timezone.utc).isoformat()
    return "1970-01-01T00:00:00+00:00"


def read_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def module_status_from_matrix(matrix: dict[str, Any], prefixes: tuple[str, ...]) -> dict[str, Any]:
    rows = [row for row in matrix.get("commands", []) if isinstance(row, dict)]
    matched = [row for row in rows if str(row.get("command", "")).startswith(prefixes)]
    if not matched:
        return {"status": "still-changing", "reason": "no command rows found", "counts": {}}

    counts = {
        "rust-complete": 0,
        "rust-partial": 0,
        "python-only": 0,
        "intentionally-different": 0,
    }
    for row in matched:
        status = str(row.get("status", "")).strip()
        if status in counts:
            counts[status] += 1
    if counts["python-only"] > 0:
        status = "still-changing"
        reason = "python-only commands remain"
    elif counts["rust-partial"] > 0:
        status = "partial"
        reason = "rust-partial commands remain"
    else:
        status = "complete"
        reason = "all command rows are rust-complete or intentionally-different"
    return {"status": status, "reason": reason, "counts": counts, "total": len(matched)}


def find_doctor_snapshots() -> list[str]:
    candidates = [
        "dev_cli_state_doctor_text.txt",
        "dev_cli_state_doctor_no_color.txt",
        "dev_cli_state_audit_text.txt",
        "dev_cli_state_audit_no_color.txt",
    ]
    return [f"crates/bijux-cli/tests/snapshots/{name}" for name in candidates if (SNAPSHOT_DIR / name).exists()]


def main() -> None:
    generated_at = stable_generated_at()
    base = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_state_audit_reports.py",
    }

    migration = read_json(STATUS / "command_migration_matrix.json")
    state_behavior = read_json(STATUS / "status_state_behavior_coverage.json")
    state_paths = read_json(STATUS / "status_state_paths_report.json")
    state_corruption = read_json(STATUS / "status_state_corruption_health_report.json")
    state_audit = read_json(STATUS / "state_audit_report.json")
    state_doctor = read_json(STATUS / "state_doctor_report.json")
    state_write_guarantees = read_json(STATUS / "state_write_guarantees.json")
    state_recovery_guarantees = read_json(STATUS / "state_recovery_guarantees.json")
    state_inventory = read_json(STATUS / "state_file_inventory.json")
    parity_matrix = read_json(PARITY / "state_behavior_parity_matrix.json")

    module_status = {
        "config": module_status_from_matrix(migration, ("config", "cli config")),
        "history": module_status_from_matrix(migration, ("history", "cli history")),
        "memory": module_status_from_matrix(migration, ("memory", "cli memory")),
        "plugin_registry_behavior": module_status_from_matrix(
            migration, ("plugins", "cli plugins")
        ),
    }
    write_json(
        STATUS / "state_migration_status.json",
        {
            **base,
            "modules": module_status,
            "source_matrix": "artifacts/status/command_migration_matrix.json",
        },
    )

    write_json(
        STATUS / "unified_state_behavior_report.json",
        {
            **base,
            "module_status": module_status,
            "state_behavior_coverage": state_behavior,
            "state_behavior_parity_matrix": parity_matrix,
        },
    )
    write_json(
        STATUS / "unified_state_corruption_report.json",
        {
            **base,
            "status_corruption_health": state_corruption,
            "runtime_state_audit": state_audit.get("corruption_health", {}),
        },
    )
    write_json(
        STATUS / "unified_state_rollback_report.json",
        {
            **base,
            "recovery_guarantees": state_recovery_guarantees,
            "write_guarantees": state_write_guarantees,
            "doctor_repairs": state_doctor.get("doctor", {}).get("repairs", []),
        },
    )
    write_json(
        STATUS / "unified_state_path_resolution_report.json",
        {
            **base,
            "path_resolution": state_paths,
            "runtime_paths": state_audit.get("paths", {}),
            "inventory": state_inventory.get("state_files", []),
        },
    )
    write_json(
        STATUS / "unified_state_doctor_snapshots.json",
        {
            **base,
            "snapshots": find_doctor_snapshots(),
            "runtime_reports": [
                "artifacts/status/state_audit_report.json",
                "artifacts/status/state_doctor_report.json",
                "artifacts/status/state_doctor_report.txt",
            ],
        },
    )
    write_json(
        STATUS / "unified_state_audit_payload.json",
        {
            **base,
            "behavior_report": read_json(STATUS / "unified_state_behavior_report.json"),
            "corruption_report": read_json(STATUS / "unified_state_corruption_report.json"),
            "rollback_report": read_json(STATUS / "unified_state_rollback_report.json"),
            "path_resolution_report": read_json(STATUS / "unified_state_path_resolution_report.json"),
            "doctor_snapshots": read_json(STATUS / "unified_state_doctor_snapshots.json"),
        },
    )

    print("wrote artifacts/status/state_migration_status.json")
    print("wrote artifacts/status/unified_state_behavior_report.json")
    print("wrote artifacts/status/unified_state_corruption_report.json")
    print("wrote artifacts/status/unified_state_rollback_report.json")
    print("wrote artifacts/status/unified_state_path_resolution_report.json")
    print("wrote artifacts/status/unified_state_doctor_snapshots.json")
    print("wrote artifacts/status/unified_state_audit_payload.json")


if __name__ == "__main__":
    main()
