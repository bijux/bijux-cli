#!/usr/bin/env python3
"""Generate plugin lifecycle test matrix artifact for TODOs 21-40."""

from __future__ import annotations

import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "plugin_lifecycle_matrix.rs"


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
        (21, "python_scaffold_install_list_inspect_uninstall_end_to_end"),
        (22, "rust_scaffold_install_list_inspect_uninstall_end_to_end"),
        (23, "installed_plugin_help_entrypoint_is_deterministic"),
        (24, "installed_plugin_disable_rejects_plugin_check"),
        (25, "disabled_plugin_enable_restores_plugin_check"),
        (26, "duplicate_install_without_force_is_deterministic_rejection"),
        (27, "duplicate_install_force_flag_behavior_is_deterministic_when_unsupported"),
        (28, "uninstall_missing_plugin_returns_stable_failure"),
        (29, "inspect_broken_registry_returns_stable_diagnostics"),
        (30, "plugin_check_after_entrypoint_deletion_reports_stable_failure"),
        (31, "plugin_help_flows_through_root_help_tree"),
        (32, "plugin_command_output_uses_core_envelope_rules"),
        (33, "plugin_command_stderr_stdout_discipline_is_stable"),
        (34, "plugin_command_exit_codes_map_through_core_rules"),
        (35, "two_plugins_keep_stable_ordering_in_list"),
        (36, "uninstalling_one_plugin_does_not_affect_other"),
        (37, "registry_survives_restart_after_successful_install"),
        (38, "registry_survives_restart_after_successful_uninstall"),
        (39, "plugin_check_reports_healthy_and_unhealthy_in_same_registry"),
    ]

    payload = {
        "generated_at": stable_generated_at(),
        "generator": "scripts/status/generate_plugin_lifecycle_test_matrix.py",
        "scope": "todo 21-40 plugin lifecycle integration tests",
        "rows": [
            {
                "todo": todo,
                "test_name": name,
                "status": "complete" if name in test_names else "missing",
                "evidence": "crates/bijux-cli/tests/bin_surface/plugin_lifecycle_matrix.rs",
            }
            for todo, name in rows
        ],
    }
    payload["summary"] = {
        "complete": sum(1 for row in payload["rows"] if row["status"] == "complete"),
        "missing": sum(1 for row in payload["rows"] if row["status"] == "missing"),
        "artifact_todo": 40,
        "artifact_path": "artifacts/status/plugin_lifecycle_test_matrix.json",
    }

    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / "plugin_lifecycle_test_matrix.json").write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print("wrote artifacts/status/plugin_lifecycle_test_matrix.json")


if __name__ == "__main__":
    main()
