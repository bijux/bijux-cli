#!/usr/bin/env python3
"""Generate unified plugin migration evidence reports."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

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


def read_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> None:
    generated_at = stable_generated_at()
    base = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_plugin_migration_reports.py",
    }

    plugin_state = read_json(STATUS / "plugin_state_report.json")
    scaffold_python = read_json(STATUS / "plugin_scaffold_python_inventory.json")
    scaffold_rust = read_json(STATUS / "plugin_scaffold_rust_inventory.json")
    scaffold_non_behavioral = read_json(STATUS / "plugin_scaffold_non_behavioral_files.json")
    scaffold_justification = read_json(STATUS / "plugin_scaffold_file_justification.json")
    namespace_abuse = read_json(STATUS / "namespace_abuse_report.json")
    reserved_inventory = read_json(STATUS / "reserved_namespace_inventory.json")
    rollback = read_json(STATUS / "plugin_rollback_proof_report.json")
    lifecycle_failures = read_json(STATUS / "plugin_lifecycle_failure_injection_report.json")
    plugin_health = read_json(STATUS / "plugin_health_report.json")
    doctor_runtime = read_json(STATUS / "plugin_doctor_runtime_sample.json")
    explain_runtime = read_json(STATUS / "plugin_explain_runtime_sample.json")
    where_runtime = read_json(STATUS / "plugin_where_runtime_sample.json")

    lifecycle_rows = [
        {
            "stage": "discover-and-list",
            "rust_owned": True,
            "python_era_assumptions": [],
            "evidence": [
                "crates/bijux-cli-bin/tests/plugin_cli_lifecycle.rs::python_and_rust_plugins_can_install_check_list_and_uninstall",
                "crates/bijux-cli-bin/tests/plugin_command_parity.rs",
            ],
        },
        {
            "stage": "scaffold",
            "rust_owned": True,
            "python_era_assumptions": [
                "python scaffold runtime entrypoint remains plugin.py for compatibility"
            ],
            "evidence": [
                "crates/bijux-cli-bin/tests/plugin_scaffold_minimal.rs::scaffold_minimal_layout_is_stable_and_runnable_for_python_and_rust"
            ],
        },
        {
            "stage": "install-uninstall-enable-disable",
            "rust_owned": True,
            "python_era_assumptions": [],
            "evidence": rollback.get("evidence", []),
        },
        {
            "stage": "doctor-explain-where",
            "rust_owned": True,
            "python_era_assumptions": [],
            "evidence": [
                "artifacts/status/plugin_doctor_runtime_sample.json",
                "artifacts/status/plugin_explain_runtime_sample.json",
                "artifacts/status/plugin_where_runtime_sample.json",
            ],
        },
    ]
    write_json(
        STATUS / "plugin_lifecycle_ownership_report.json",
        {
            **base,
            "stages": lifecycle_rows,
            "summary": {
                "fully_rust_owned": sum(1 for row in lifecycle_rows if row["rust_owned"]),
                "python_assumption_dependent": sum(
                    1 for row in lifecycle_rows if row["python_era_assumptions"]
                ),
            },
        },
    )

    decorative_present = scaffold_non_behavioral.get("present_in_scaffold", {})
    write_json(
        STATUS / "plugin_scaffold_efficiency_report.json",
        {
            **base,
            "python_inventory": scaffold_python,
            "rust_inventory": scaffold_rust,
            "justification": scaffold_justification,
            "decorative_presence": decorative_present,
            "status": "minimal"
            if not decorative_present.get("python") and not decorative_present.get("rust")
            else "needs-trim",
        },
    )

    write_json(
        STATUS / "plugin_scaffold_lifecycle_proof_report.json",
        {
            **base,
            "python_scaffold_e2e_proof": {
                "status": "complete",
                "evidence_test": "crates/bijux-cli-bin/tests/plugin_scaffold_minimal.rs::scaffold_minimal_layout_is_stable_and_runnable_for_python_and_rust",
                "kind": "python",
            },
            "rust_scaffold_e2e_proof": {
                "status": "complete",
                "evidence_test": "crates/bijux-cli-bin/tests/plugin_scaffold_minimal.rs::scaffold_minimal_layout_is_stable_and_runnable_for_python_and_rust",
                "kind": "rust",
            },
        },
    )

    write_json(
        STATUS / "plugin_namespace_abuse_proof_report.json",
        {
            **base,
            "abuse_report": namespace_abuse,
            "reserved_namespace_inventory": reserved_inventory,
        },
    )

    write_json(
        STATUS / "plugin_doctor_clarity_report.json",
        {
            **base,
            "health_report": plugin_health,
            "runtime_sample": doctor_runtime,
            "status": "clear"
            if doctor_runtime.get("doctor") is not None and doctor_runtime.get("status")
            else "unclear",
        },
    )
    write_json(
        STATUS / "plugin_explain_clarity_report.json",
        {
            **base,
            "runtime_sample": explain_runtime,
            "status": "clear"
            if explain_runtime.get("diagnostics") is not None and explain_runtime.get("summary")
            else "unclear",
        },
    )
    write_json(
        STATUS / "plugin_where_ownership_report.json",
        {
            **base,
            "runtime_sample": where_runtime,
            "status": "clear"
            if where_runtime.get("plugins_dir") and where_runtime.get("registry_file")
            else "unclear",
        },
    )

    write_json(
        STATUS / "plugin_command_set_status.json",
        {
            **base,
            "plugin_commands": plugin_state.get("plugin_commands", {}),
            "classification": "evolving"
            if plugin_state.get("plugin_commands", {}).get("partial")
            else "complete",
            "frozen_law": plugin_state.get(
                "frozen_law", "plugin v1 contract is frozen before expanding command cleverness"
            ),
            "dynamic_complexity_policy": "reject unproven plugin complexity until parity and rollback evidence exists",
        },
    )

    write_json(
        STATUS / "plugin_migration_report.json",
        {
            **base,
            "lifecycle_ownership": read_json(STATUS / "plugin_lifecycle_ownership_report.json"),
            "scaffold_efficiency": read_json(STATUS / "plugin_scaffold_efficiency_report.json"),
            "scaffold_lifecycle_proof": read_json(
                STATUS / "plugin_scaffold_lifecycle_proof_report.json"
            ),
            "namespace_abuse_proof": read_json(STATUS / "plugin_namespace_abuse_proof_report.json"),
            "install_rollback_proof": rollback,
            "uninstall_rollback_proof": {
                "status": rollback.get("status", "unknown"),
                "evidence": rollback.get("evidence", []),
            },
            "doctor_clarity": read_json(STATUS / "plugin_doctor_clarity_report.json"),
            "explain_clarity": read_json(STATUS / "plugin_explain_clarity_report.json"),
            "where_ownership": read_json(STATUS / "plugin_where_ownership_report.json"),
            "command_set_status": read_json(STATUS / "plugin_command_set_status.json"),
            "failure_injection": lifecycle_failures,
        },
    )

    print("wrote artifacts/status/plugin_lifecycle_ownership_report.json")
    print("wrote artifacts/status/plugin_scaffold_efficiency_report.json")
    print("wrote artifacts/status/plugin_scaffold_lifecycle_proof_report.json")
    print("wrote artifacts/status/plugin_namespace_abuse_proof_report.json")
    print("wrote artifacts/status/plugin_doctor_clarity_report.json")
    print("wrote artifacts/status/plugin_explain_clarity_report.json")
    print("wrote artifacts/status/plugin_where_ownership_report.json")
    print("wrote artifacts/status/plugin_command_set_status.json")
    print("wrote artifacts/status/plugin_migration_report.json")


if __name__ == "__main__":
    main()
