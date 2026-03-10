#!/usr/bin/env python3
"""Generate install-neutrality and active-runtime truth artifacts."""

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


def main() -> int:
    generated_at = stable_generated_at()

    runtime_identity = read_json(STATUS / "install_source_diagnostics.json")
    ambiguous = read_json(STATUS / "ambiguous_runtime_diagnostics.json")
    install_health = read_json(STATUS / "install_health_report.json")
    package_health = read_json(STATUS / "package_health_report.json")
    remaining = read_json(STATUS / "remaining_install_ambiguities.json")

    install_neutrality = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_install_neutrality_reports.py",
        "schema": "install-neutrality-v1",
        "source_reports": [
            "artifacts/status/install_source_diagnostics.json",
            "artifacts/status/ambiguous_runtime_diagnostics.json",
            "artifacts/status/install_health_report.json",
            "artifacts/status/package_health_report.json",
            "artifacts/status/remaining_install_ambiguities.json",
        ],
        "channels": ["cargo", "pip", "pipx"],
        "diagnostics": {
            "active_binary_selection_is_ambiguous": ambiguous.get(
                "active_binary_selection_is_ambiguous", False
            ),
            "path_shadowing_detected": ambiguous.get("path_shadowing_detected", False),
            "mixed_pip_cargo_install_detected": ambiguous.get(
                "mixed_pip_cargo_install_detected", False
            ),
            "stale_wrapper_detected": ambiguous.get("stale_wrapper_detected", False),
            "active_binary_mismatch_detected": ambiguous.get(
                "active_binary_mismatch_detected", False
            ),
            "python_bridge_supported": ambiguous.get("python_bridge_supported", True),
        },
        "active_runtime": {
            "active_binary": runtime_identity.get("active_binary"),
            "install_source": runtime_identity.get("install_source"),
            "path_binaries": runtime_identity.get("path_binaries", []),
        },
        "known_remaining_install_ambiguities": remaining.get("ambiguities", []),
        "known_remaining_install_ambiguities_count": remaining.get("count", 0),
        "status": "complete" if install_health else "incomplete",
    }

    active_runtime = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_install_neutrality_reports.py",
        "schema": "active-runtime-v1",
        "source": "artifacts/status/install_source_diagnostics.json",
        "active_binary": runtime_identity.get("active_binary"),
        "install_source": runtime_identity.get("install_source", "unknown"),
        "path_binaries": runtime_identity.get("path_binaries", []),
        "diagnostics": runtime_identity.get("diagnostics", {}),
    }

    write_json(STATUS / "install_neutrality_report.json", install_neutrality)
    write_json(STATUS / "active_runtime_report.json", active_runtime)
    print("wrote artifacts/status/install_neutrality_report.json")
    print("wrote artifacts/status/active_runtime_report.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
