#!/usr/bin/env python3
"""Generate stale-artifact/stale-evidence/stale-report hardening artifacts."""

from __future__ import annotations

import json
import os
import time
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class InputSpec:
    scenario_id: str
    command: str
    relative_path: str
    severity: str
    description: str


def _root() -> Path:
    override = os.environ.get("DEV_CLI_STALE_ARTIFACT_ROOT", "").strip()
    if override:
        return Path(override).resolve()
    return Path(__file__).resolve().parents[2]


ROOT = _root()
STATUS = ROOT / "artifacts" / "status"
NOW_EPOCH = int(float(os.environ.get("DEV_CLI_STALE_NOW_EPOCH", str(time.time()))))
MAX_AGE_SECONDS = int(os.environ.get("DEV_CLI_STALE_MAX_SECONDS", "86400"))

SPECS = [
    InputSpec(
        scenario_id="evidence_deleted_before_evidence_audit",
        command="dev cli evidence audit",
        relative_path="artifacts/status/evidence_integrity_artifact.json",
        severity="critical",
        description="Detect missing evidence artifact before evidence audit.",
    ),
    InputSpec(
        scenario_id="evidence_stale_before_evidence_stale",
        command="dev cli evidence stale",
        relative_path="artifacts/status/evidence_integrity_artifact.json",
        severity="critical",
        description="Detect stale evidence artifact before evidence stale command.",
    ),
    InputSpec(
        scenario_id="parity_stale_before_status",
        command="dev cli status",
        relative_path="artifacts/status/parity_drift_artifact.json",
        severity="critical",
        description="Detect stale parity artifact before status command.",
    ),
    InputSpec(
        scenario_id="migration_stale_before_truth",
        command="dev cli truth",
        relative_path="artifacts/status/migration_truth_artifact.json",
        severity="critical",
        description="Detect stale migration artifact before truth command.",
    ),
    InputSpec(
        scenario_id="package_health_stale_before_dashboard",
        command="dev cli dashboard",
        relative_path="artifacts/status/package_health_diagnostics_artifact.json",
        severity="critical",
        description="Detect stale package health artifact before dashboard command.",
    ),
    InputSpec(
        scenario_id="state_audit_stale_before_blockers",
        command="dev cli blockers",
        relative_path="artifacts/status/state_audit_truth_artifact.json",
        severity="critical",
        description="Detect stale state audit artifact before blockers command.",
    ),
    InputSpec(
        scenario_id="docs_audit_stale_before_repo_health",
        command="dev cli repo health",
        relative_path="artifacts/status/docs_audit.json",
        severity="critical",
        description="Detect stale docs-audit artifact before repo health command.",
    ),
    InputSpec(
        scenario_id="script_audit_stale_before_repo_health",
        command="dev cli repo health",
        relative_path="artifacts/status/script_only_behaviors.json",
        severity="critical",
        description="Detect stale script-audit artifact before repo health command.",
    ),
    InputSpec(
        scenario_id="crate_health_stale_before_crate_health",
        command="dev cli crate-health",
        relative_path="artifacts/status/duplication_hotspots.json",
        severity="critical",
        description="Detect stale crate-health artifact before crate-health command.",
    ),
    InputSpec(
        scenario_id="optional_next_report_stale_warning",
        command="dev cli next",
        relative_path="artifacts/status/dev_cli_next_report.json",
        severity="warning",
        description="Stale optional report is tolerated with warning.",
    ),
]


def _forced_stale_paths() -> set[str]:
    raw = os.environ.get("DEV_CLI_FORCE_STALE_FILES", "").strip()
    if not raw:
        return set()
    return {item.strip() for item in raw.split(",") if item.strip()}


def _inject_forced_stale_paths() -> set[str]:
    if os.environ.get("DEV_CLI_INJECT_STALE_ARTIFACT", "0") != "1":
        return set()
    # CI injection mode intentionally forces one stale critical artifact.
    return {"artifacts/status/parity_drift_artifact.json"}


def _evaluate(spec: InputSpec, forced: set[str]) -> dict:
    path = ROOT / spec.relative_path
    state = "fresh"
    age_seconds = None
    exists = path.exists()
    if not exists:
        state = "missing"
    else:
        mtime = int(path.stat().st_mtime)
        age_seconds = max(0, NOW_EPOCH - mtime)
        if spec.relative_path in forced or age_seconds > MAX_AGE_SECONDS:
            state = "stale"
    return {
        "scenario_id": spec.scenario_id,
        "command": spec.command,
        "path": spec.relative_path,
        "severity": spec.severity,
        "description": spec.description,
        "exists": exists,
        "state": state,
        "age_seconds": age_seconds,
        "max_age_seconds": MAX_AGE_SECONDS,
    }


def _write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    out = STATUS / name
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out.relative_to(ROOT)}")


def main() -> int:
    forced = _forced_stale_paths() | _inject_forced_stale_paths()
    evaluated = [_evaluate(spec, forced) for spec in SPECS]

    stale_or_missing = [row for row in evaluated if row["state"] in {"stale", "missing"}]
    fresh = [row for row in evaluated if row["state"] == "fresh"]
    critical_stale = [row for row in stale_or_missing if row["severity"] == "critical"]
    warning_stale = [row for row in stale_or_missing if row["severity"] == "warning"]

    status_value = "clean" if not stale_or_missing else "drift"
    summary = {
        "checks_total": len(evaluated),
        "fresh_count": len(fresh),
        "stale_or_missing_count": len(stale_or_missing),
        "critical_stale_count": len(critical_stale),
        "warning_stale_count": len(warning_stale),
        "status": status_value,
        "injection_mode": os.environ.get("DEV_CLI_INJECT_STALE_ARTIFACT", "0") == "1",
    }
    regression_suite = {
        "scope": "stale artifact regression suite",
        "generator": "scripts/status/generate_dev_cli_stale_artifact_reports.py",
        "cases": [
            {
                "scenario_id": row["scenario_id"],
                "command": row["command"],
                "state": row["state"],
                "severity": row["severity"],
            }
            for row in evaluated
        ],
        "status": "clean" if summary["critical_stale_count"] == 0 else "drift",
    }

    _write_json(
        "stale_artifact_artifact.json",
        {
            "scope": "stale artifact truth",
            "generator": "scripts/status/generate_dev_cli_stale_artifact_reports.py",
            "summary": summary,
            "checks": evaluated,
        },
    )
    _write_json(
        "stale_evidence_artifact.json",
        {
            "scope": "stale evidence truth",
            "generator": "scripts/status/generate_dev_cli_stale_artifact_reports.py",
            "checks": [
                row
                for row in evaluated
                if row["command"] in {"dev cli evidence audit", "dev cli evidence stale"}
            ],
            "status": "clean"
            if not any(
                row["state"] in {"stale", "missing"}
                and row["command"] in {"dev cli evidence audit", "dev cli evidence stale"}
                for row in evaluated
            )
            else "drift",
        },
    )
    _write_json(
        "stale_report_artifact.json",
        {
            "scope": "stale report truth",
            "generator": "scripts/status/generate_dev_cli_stale_artifact_reports.py",
            "checks": [
                row
                for row in evaluated
                if row["command"] not in {"dev cli evidence audit", "dev cli evidence stale"}
            ],
            "status": status_value,
        },
    )
    _write_json("stale_detection_regression_suite.json", regression_suite)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
