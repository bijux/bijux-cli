#!/usr/bin/env python3
"""Generate state-audit/state-doctor hardening artifacts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    out = STATUS / name
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out.relative_to(ROOT)}")


def main() -> int:
    state_audit = read_json(STATUS / "state_audit_report.json")
    state_doctor = read_json(STATUS / "state_doctor_report.json")
    unified_corruption = read_json(STATUS / "unified_state_corruption_report.json")
    repeated_harness = read_json(STATUS / "repeated_run_corruption_harness.json")

    audit_checks = {
        "paths_present": isinstance(state_audit.get("paths"), dict),
        "corruption_health_present": isinstance(state_audit.get("corruption_health"), dict),
        "config_path_present": isinstance(state_audit.get("paths", {}).get("config", {}).get("path"), str),
        "plugin_registry_path_present": isinstance(
            state_audit.get("paths", {}).get("plugins_registry", {}).get("path"),
            str,
        ),
        "history_path_present": isinstance(state_audit.get("paths", {}).get("history", {}).get("path"), str),
        "memory_path_present": isinstance(state_audit.get("paths", {}).get("memory", {}).get("path"), str),
    }
    doctor_checks = {
        "doctor_object_present": isinstance(state_doctor.get("doctor"), dict),
        "issues_list_present": isinstance(state_doctor.get("doctor", {}).get("issues"), list),
        "repairs_list_present": isinstance(state_doctor.get("doctor", {}).get("repairs"), list),
        "runtime_marker_present": isinstance(state_doctor.get("runtime"), str),
    }

    harness_results = repeated_harness.get("results", []) if isinstance(repeated_harness, dict) else []
    has_corrupt_config_probe = any(
        isinstance(row, dict) and row.get("name") == "state_doctor_json_corrupt_config"
        for row in harness_results
    )
    all_harness_stable = all(
        isinstance(row, dict) and bool(row.get("stable")) for row in harness_results
    ) if harness_results else False
    harness_alignment_checks = {
        "corrupt_config_probe_present": has_corrupt_config_probe,
        "harness_results_stable": all_harness_stable,
        "unified_corruption_report_present": bool(unified_corruption),
    }

    all_checks = {**audit_checks, **doctor_checks, **harness_alignment_checks}
    drift_items = [name for name, ok in all_checks.items() if not ok]

    write_json(
        "state_audit_truth_artifact.json",
        {
            "scope": "state audit truth",
            "generator": "scripts/status/generate_dev_cli_state_diagnostics_reports.py",
            "checks": audit_checks,
            "status": "complete" if all(audit_checks.values()) else "partial",
        },
    )
    write_json(
        "state_doctor_truth_artifact.json",
        {
            "scope": "state doctor truth",
            "generator": "scripts/status/generate_dev_cli_state_diagnostics_reports.py",
            "checks": doctor_checks,
            "status": "complete" if all(doctor_checks.values()) else "partial",
        },
    )
    write_json(
        "corrupted_state_truth_artifact.json",
        {
            "scope": "corrupted state truth",
            "generator": "scripts/status/generate_dev_cli_state_diagnostics_reports.py",
            "checks": harness_alignment_checks,
            "status": "complete" if all(harness_alignment_checks.values()) else "partial",
        },
    )
    write_json(
        "state_diagnostics_drift_artifact.json",
        {
            "scope": "state diagnostics drift",
            "generator": "scripts/status/generate_dev_cli_state_diagnostics_reports.py",
            "drift_checks": drift_items,
            "drift_count": len(drift_items),
            "status": "clean" if not drift_items else "drift",
        },
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
