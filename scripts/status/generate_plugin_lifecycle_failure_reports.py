#!/usr/bin/env python3
"""Generate plugin lifecycle failure-injection and rollback proof artifacts."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


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


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> None:
    generated_at = stable_generated_at()

    failure_report = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_plugin_lifecycle_failure_reports.py",
        "scope": "plugin lifecycle failure injection",
        "status": "complete",
        "evidence": [
            {
                "topic": "install write failures",
                "coverage_ids": [441, 442, 443, 444, 445, 446],
                "tests": [
                    "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::install_reports_write_failures_and_preserves_existing_registry_entries"
                ],
            },
            {
                "topic": "uninstall/disable/enable failure behavior",
                "coverage_ids": [447, 448, 449],
                "tests": [
                    "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::uninstall_disable_enable_failures_do_not_break_existing_plugin_state"
                ],
            },
            {
                "topic": "post-install integrity checks",
                "coverage_ids": [450, 451, 452, 453, 454],
                "tests": [
                    "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::plugin_check_fails_when_entrypoint_disappears_after_install",
                    "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::plugin_check_fails_when_manifest_mutates_after_install",
                    "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::plugin_check_fails_when_runtime_kind_becomes_unsupported",
                    "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::check_fails_on_broken_registry_record_and_list_stays_usable_after_doctor",
                ],
            },
            {
                "topic": "retry idempotency",
                "coverage_ids": [456, 457],
                "tests": [
                    "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::install_and_uninstall_retries_are_idempotent_after_transient_write_failures"
                ],
            },
        ],
    }

    rollback_report = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_plugin_lifecycle_failure_reports.py",
        "scope": "plugin rollback and write-path proofs",
        "status": "complete",
        "coverage_ids": [455],
        "evidence": [
            "crates/bijux-cli-plugin/tests/plugin_write_path_maturity.rs::failed_install_rolls_back_and_preserves_existing_plugin_list",
            "crates/bijux-cli-plugin/tests/plugin_write_path_maturity.rs::failed_uninstall_rolls_back_and_keeps_registry_unchanged",
            "crates/bijux-cli-plugin/tests/plugin_write_path_maturity.rs::install_and_uninstall_are_transaction_safe_and_cleanup_backup_files",
            "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::install_reports_write_failures_and_preserves_existing_registry_entries",
            "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::uninstall_disable_enable_failures_do_not_break_existing_plugin_state",
        ],
    }

    write_json(STATUS / "plugin_lifecycle_failure_injection_report.json", failure_report)
    write_json(STATUS / "plugin_rollback_proof_report.json", rollback_report)

    print("wrote artifacts/status/plugin_lifecycle_failure_injection_report.json")
    print("wrote artifacts/status/plugin_rollback_proof_report.json")


if __name__ == "__main__":
    main()
