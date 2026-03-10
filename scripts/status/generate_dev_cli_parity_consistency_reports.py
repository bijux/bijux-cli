#!/usr/bin/env python3
"""Generate parity consistency and migration-truth hardening artifacts."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

VALID_MIGRATION_STATUSES = {
    "rust-complete",
    "rust-partial",
    "python-only",
    "intentionally-different",
}


def run_json(args: list[str]) -> dict:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-core", "--", *args, "--format", "json", "--no-pretty"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(proc.stdout or "{}")


def run_text(args: list[str]) -> str:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-core", "--", *args, "--format", "text"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return proc.stdout


def write_json(name: str, payload: dict) -> None:
    path = STATUS / name
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {path.relative_to(ROOT)}")


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)

    parity_first = run_json(["dev", "cli", "parity"])
    parity_second = run_json(["dev", "cli", "parity"])
    status_payload = run_json(["dev", "cli", "status"])
    parity_text_first = run_text(["dev", "cli", "parity"])
    parity_text_second = run_text(["dev", "cli", "parity"])

    migration_rows = status_payload.get("command_migration", {}).get("matrix", {}).get("commands", [])
    migration_rows = migration_rows if isinstance(migration_rows, list) else []
    parity_rows = parity_first.get("command_matrix", {}).get("commands", [])
    parity_rows = parity_rows if isinstance(parity_rows, list) else []

    invalid_status_rows = [
        row.get("command", "")
        for row in migration_rows
        if row.get("status") not in VALID_MIGRATION_STATUSES
    ]
    partial_without_blocker = []
    for row in migration_rows:
        if row.get("status") != "rust-partial":
            continue
        blocker = str(row.get("blocker", "")).strip()
        shim_alias = row.get("shim_alias_dependency", {})
        aliases = shim_alias.get("aliases", []) if isinstance(shim_alias, dict) else []
        shims = shim_alias.get("shims", []) if isinstance(shim_alias, dict) else []
        has_shim_alias = bool(aliases) or bool(shims)
        parity_coverage = row.get("parity_coverage", {})
        has_parity_mismatch = (
            isinstance(parity_coverage, dict) and any(value is False for value in parity_coverage.values())
        )
        if not blocker and not has_shim_alias and not has_parity_mismatch:
            partial_without_blocker.append(row.get("command", ""))
    intentional_without_reason = [
        row.get("command", "")
        for row in migration_rows
        if row.get("status") == "intentionally-different" and not str(row.get("reason", "")).strip()
    ]
    complete_without_evidence = [
        row.get("command", "")
        for row in migration_rows
        if row.get("status") == "rust-complete" and not row.get("evidence_links")
    ]

    parity_commands = {
        str(row.get("command", "")).strip()
        for row in parity_rows
        if isinstance(row, dict) and str(row.get("command", "")).strip()
    }
    migration_commands = {
        str(row.get("command", "")).strip()
        for row in migration_rows
        if isinstance(row, dict) and str(row.get("command", "")).strip()
    }
    missing_from_migration = sorted(command for command in parity_commands if command not in migration_commands)

    parity_complete = (
        parity_first.get("command_matrix", {}).get("summary", {}).get("complete", 0)
        if isinstance(parity_first, dict)
        else 0
    )
    migration_complete = (
        status_payload.get("command_migration", {}).get("matrix", {}).get("summary", {}).get("rust-complete", 0)
        if isinstance(status_payload, dict)
        else 0
    )

    consistency_checks = {
        "migration_rows_have_valid_status": len(invalid_status_rows) == 0,
        "partial_rows_have_blockers": len(partial_without_blocker) == 0,
        "intentional_rows_have_reasons": len(intentional_without_reason) == 0,
        "complete_rows_have_evidence_links": len(complete_without_evidence) == 0,
        "parity_commands_exist_in_migration_matrix": len(missing_from_migration) == 0,
        "parity_and_status_complete_counts_align": parity_complete == migration_complete,
        "parity_json_is_deterministic": parity_first == parity_second,
        "parity_text_is_deterministic": parity_text_first == parity_text_second,
    }

    migration_truth_artifact = {
        "scope": "migration truth",
        "generator": "scripts/status/generate_dev_cli_parity_consistency_reports.py",
        "rows_total": len(migration_rows),
        "checks": {
            "valid_status_rows": consistency_checks["migration_rows_have_valid_status"],
            "partial_rows_with_blockers": consistency_checks["partial_rows_have_blockers"],
            "intentional_rows_with_reasons": consistency_checks["intentional_rows_have_reasons"],
            "complete_rows_with_evidence_links": consistency_checks["complete_rows_have_evidence_links"],
        },
        "status": (
            "complete"
            if all(
                [
                    consistency_checks["migration_rows_have_valid_status"],
                    consistency_checks["partial_rows_have_blockers"],
                    consistency_checks["intentional_rows_have_reasons"],
                    consistency_checks["complete_rows_have_evidence_links"],
                ]
            )
            else "partial"
        ),
    }
    parity_evidence_consistency_artifact = {
        "scope": "parity evidence consistency",
        "generator": "scripts/status/generate_dev_cli_parity_consistency_reports.py",
        "checks": {
            "parity_commands_exist_in_migration_matrix": consistency_checks[
                "parity_commands_exist_in_migration_matrix"
            ],
            "parity_and_status_complete_counts_align": consistency_checks[
                "parity_and_status_complete_counts_align"
            ],
            "parity_json_is_deterministic": consistency_checks["parity_json_is_deterministic"],
            "parity_text_is_deterministic": consistency_checks["parity_text_is_deterministic"],
        },
        "status": (
            "complete"
            if all(
                [
                    consistency_checks["parity_commands_exist_in_migration_matrix"],
                    consistency_checks["parity_and_status_complete_counts_align"],
                    consistency_checks["parity_json_is_deterministic"],
                    consistency_checks["parity_text_is_deterministic"],
                ]
            )
            else "partial"
        ),
    }
    drift_items = [name for name, ok in consistency_checks.items() if not ok]
    parity_drift_artifact = {
        "scope": "parity and migration drift",
        "generator": "scripts/status/generate_dev_cli_parity_consistency_reports.py",
        "drift_checks": drift_items,
        "drift_count": len(drift_items),
        "status": "clean" if not drift_items else "drift",
        "details": {
            "invalid_status_rows": invalid_status_rows,
            "partial_without_blocker": partial_without_blocker,
            "intentional_without_reason": intentional_without_reason,
            "complete_without_evidence": complete_without_evidence,
            "parity_missing_from_migration": missing_from_migration,
        },
    }

    write_json("migration_truth_artifact.json", migration_truth_artifact)
    write_json("parity_evidence_consistency_artifact.json", parity_evidence_consistency_artifact)
    write_json("parity_drift_artifact.json", parity_drift_artifact)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
