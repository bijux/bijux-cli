#!/usr/bin/env python3
"""Generate diagnostics taxonomy and consistency artifacts."""

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


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def rg(pattern: str) -> list[str]:
    result = subprocess.run(
        ["rg", "-n", pattern, "crates", "scripts", "-S"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    return [line.strip() for line in result.stdout.splitlines() if line.strip()]


def main() -> None:
    generated_at = stable_generated_at()

    buckets = {
        "runtime": rg(r"runtime-identity|runtime_unity|execution_outcome"),
        "state": rg(r"state-audit|state-doctor|history|memory"),
        "plugin": rg(r"plugins doctor|plugin-health|load_time_diagnostics|plugin_doctor"),
        "package": rg(r"package-health|install_health_report|packaging"),
        "parity": rg(r"parity|binary_vs_python_bridge"),
        "route": rg(r"route-audit|routes_report|registry_report"),
        "health": rg(r"doctor|diagnostics"),
    }

    taxonomy = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_diagnostics_law_reports.py",
        "taxonomy": [
            {"type": kind, "evidence_count": len(lines), "examples": lines[:20]}
            for kind, lines in buckets.items()
        ],
    }

    usefulness = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_diagnostics_law_reports.py",
        "severity_model": ["error", "warning", "info"],
        "actionable_next_step_model": {
            "required_fields": ["area", "severity", "message"],
            "optional_fields": ["path", "action", "next_step"],
        },
        "removed_low_value_diagnostics": [
            "legacy dev routes hidden alias diagnostics",
            "legacy dev registry hidden alias diagnostics",
            "duplicate route special-case counters not tied to canonical paths",
        ],
        "consistency_targets": {
            "json_shape": [
                "status",
                "diagnostics",
            ],
            "text_output": [
                "header line",
                "plain action lines",
            ],
            "exit_code_expectations": {
                "usage_error": 2,
                "runtime_error": 1,
                "success": 0,
            },
        },
    }

    write_json(STATUS / "diagnostics_taxonomy.json", taxonomy)
    write_json(STATUS / "diagnostics_usefulness_review.json", usefulness)
    print("wrote artifacts/status/diagnostics_taxonomy.json")
    print("wrote artifacts/status/diagnostics_usefulness_review.json")


if __name__ == "__main__":
    main()

