#!/usr/bin/env python3
"""Generate plugin discovery determinism artifacts."""

from __future__ import annotations

import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "plugin_discovery_determinism_matrix.rs"


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
        (61, "deterministic_discovery_under_shuffled_install_order"),
        (62, "deterministic_plugin_list_ordering"),
        (63, "deterministic_plugin_inspect_ordering_multiple_plugins"),
        (64, "deterministic_help_ordering_with_plugins_installed"),
        (65, "deterministic_route_registration_with_different_install_orders"),
        (66, "deterministic_route_registration_after_uninstall_reinstall_cycles"),
        (67, "deterministic_namespace_conflict_resolution_messages"),
        (68, "deterministic_plugins_list_json_output"),
        (69, "deterministic_plugins_check_json_output"),
        (70, "deterministic_plugins_inspect_json_output"),
        (71, "discovery_ignores_unrelated_filesystem_clutter"),
        (72, "discovery_ignores_partially_written_temporary_files"),
        (73, "discovery_ignores_invalid_directories_cleanly"),
        (74, "discovery_is_stable_under_broken_symlink_entries"),
        (75, "broken_plugin_does_not_reorder_healthy_plugins"),
        (76, "broken_plugin_does_not_hide_healthy_plugins"),
        (77, "registry_and_discovery_disagreement_diagnostics_are_deterministic"),
        (78, "plugin_metadata_ordering_is_stable_in_machine_output"),
    ]

    STATUS.mkdir(parents=True, exist_ok=True)

    matrix = {
        "generated_at": stable_generated_at(),
        "generator": "scripts/status/generate_plugin_discovery_determinism_reports.py",
        "scope": "plugin discovery and ordering determinism",
        "rows": [
            {
                "coverage_id": coverage_id,
                "test_name": name,
                "status": "complete" if name in test_names else "missing",
                "evidence": "crates/bijux-cli/tests/bin_surface/plugin_discovery_determinism_matrix.rs",
            }
            for coverage_id, name in rows
        ],
    }
    matrix["summary"] = {
        "complete": sum(1 for row in matrix["rows"] if row["status"] == "complete"),
        "missing": sum(1 for row in matrix["rows"] if row["status"] == "missing"),
        "artifact_todo": 79,
        "artifact_path": "artifacts/status/plugin_discovery_determinism_report.json",
    }

    ordering_law = {
        "generated_at": matrix["generated_at"],
        "law": "plugin ordering is deterministic",
        "status": "frozen",
        "evidence": [
            "crates/bijux-cli/tests/bin_surface/plugin_discovery_determinism_matrix.rs",
            "artifacts/status/plugin_discovery_determinism_report.json",
        ],
        "covers_todo": 80,
    }

    (STATUS / "plugin_discovery_determinism_report.json").write_text(
        json.dumps(matrix, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "plugin_ordering_law.json").write_text(
        json.dumps(ordering_law, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print("wrote artifacts/status/plugin_discovery_determinism_report.json")
    print("wrote artifacts/status/plugin_ordering_law.json")


if __name__ == "__main__":
    main()
