#!/usr/bin/env python3
"""Generate plugin failure/rollback test matrix artifact for TODOs 41-60."""

from __future__ import annotations

import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli-bin" / "tests" / "plugin_failure_rollback_matrix.rs"


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


def main() -> None:
    text = TEST_FILE.read_text(encoding="utf-8") if TEST_FILE.exists() else ""
    test_names = set(re.findall(r"fn\s+([a-z0-9_]+)\s*\(", text))

    rows = [
        (41, "simulated_disk_write_failure_during_install"),
        (42, "simulated_partial_copy_failure_during_install"),
        (43, "simulated_registry_write_failure_during_install"),
        (44, "simulated_manifest_parse_failure_during_install"),
        (45, "simulated_compatibility_range_failure_during_install"),
        (46, "simulated_missing_entrypoint_failure_during_install"),
        (47, "simulated_permission_denied_failure_during_install"),
        (48, "simulated_partial_uninstall_failure"),
        (49, "simulated_registry_write_failure_during_uninstall"),
        (50, "simulated_enable_failure_when_plugin_files_missing"),
        (51, "simulated_disable_failure_when_registry_is_corrupted"),
        (52, "rollback_proof_install_failure_preserves_existing_plugins"),
        (53, "rollback_proof_uninstall_failure_preserves_existing_plugins"),
        (54, "retry_install_after_partial_failure_is_idempotent"),
        (55, "retry_uninstall_after_partial_failure_is_idempotent"),
        (56, "failed_install_does_not_leave_claimed_namespace"),
        (57, "failed_uninstall_does_not_orphan_registry_state_silently"),
        (58, "plugin_doctor_reports_rollback_relevant_damage_clearly"),
        (59, "machine_readable_rollback_diagnostics_are_stable"),
    ]

    payload = {
        "generated_at": stable_generated_at(),
        "generator": "scripts/status/generate_plugin_failure_rollback_test_matrix.py",
        "scope": "todo 41-60 plugin failure and rollback tests",
        "rows": [
            {
                "todo": todo,
                "test_name": name,
                "status": "complete" if name in test_names else "missing",
                "evidence": "crates/bijux-cli-bin/tests/plugin_failure_rollback_matrix.rs",
            }
            for todo, name in rows
        ],
    }
    payload["summary"] = {
        "complete": sum(1 for row in payload["rows"] if row["status"] == "complete"),
        "missing": sum(1 for row in payload["rows"] if row["status"] == "missing"),
        "artifact_todo": 60,
        "artifact_path": "artifacts/status/plugin_failure_rollback_test_matrix.json",
    }

    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / "plugin_failure_rollback_test_matrix.json").write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print("wrote artifacts/status/plugin_failure_rollback_test_matrix.json")


if __name__ == "__main__":
    main()
