#!/usr/bin/env python3
"""Generate packaging ambiguity and package-health artifacts."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def read_json(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def run_json_command(args: list[str]) -> dict:
    out = subprocess.run(args, cwd=ROOT, check=True, capture_output=True, text=True)
    return json.loads(out.stdout)


def main() -> None:
    generated_at = now_iso()

    install_source = read_json(STATUS / "install_source_diagnostics.json")
    ambiguous_runtime = read_json(STATUS / "ambiguous_runtime_diagnostics.json")

    package_health = run_json_command(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "bijux-cli-bin",
            "--",
            "dev",
            "cli",
            "package-health",
            "--format",
            "json",
            "--no-pretty",
        ]
    )

    runtime_identity = run_json_command(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "bijux-cli-bin",
            "--",
            "dev",
            "cli",
            "runtime-identity",
            "--format",
            "json",
            "--no-pretty",
        ]
    )

    packaging_ambiguity = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_packaging_ambiguity_reports.py",
        "scope": "packaging ambiguity",
        "status": "complete",
        "tasks": [536],
        "runtime_identity": {
            "active_binary_selection_is_ambiguous": runtime_identity.get(
                "active_binary_selection_is_ambiguous", False
            ),
            "active_path_is_shadowed": runtime_identity.get("active_path_is_shadowed", False),
            "diagnostics": runtime_identity.get("diagnostics", {}),
        },
        "install_source_diagnostics": install_source,
        "ambiguous_runtime_diagnostics": ambiguous_runtime,
        "evidence_tests": [
            "crates/bijux-cli-bin/tests/install_ambiguity_hardening.rs::pip_binary_shadowed_by_cargo_binary_is_reported",
            "crates/bijux-cli-bin/tests/install_ambiguity_hardening.rs::cargo_binary_shadowed_by_pip_binary_is_reported",
            "crates/bijux-cli-bin/tests/install_ambiguity_hardening.rs::package_health_and_runtime_identity_cover_ambiguous_install_state",
        ],
    }

    install_state_assumptions = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_packaging_ambiguity_reports.py",
        "scope": "install-state assumptions",
        "status": "complete",
        "tasks": [537],
        "install_state_assumptions": package_health.get("install_state_assumptions", []),
        "install_state_assumption_help": package_health.get("install_state_assumption_help", ""),
    }

    package_health_report = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_packaging_ambiguity_reports.py",
        "scope": "package health",
        "status": "complete",
        "tasks": [538],
        "payload": package_health,
    }

    package_health_text = "\n".join(
        [
            "Package Health",
            "",
            f"assumptions_count: {len(package_health.get('install_state_assumptions', []))}",
            f"help: {package_health.get('install_state_assumption_help', '')}",
        ]
    ) + "\n"

    write_json(STATUS / "packaging_ambiguity_report.json", packaging_ambiguity)
    write_json(STATUS / "install_state_assumptions_report.json", install_state_assumptions)
    write_json(STATUS / "package_health_report.json", package_health_report)
    write_text(STATUS / "package_health_report.txt", package_health_text)

    print("wrote artifacts/status/packaging_ambiguity_report.json")
    print("wrote artifacts/status/install_state_assumptions_report.json")
    print("wrote artifacts/status/package_health_report.json")
    print("wrote artifacts/status/package_health_report.txt")


if __name__ == "__main__":
    main()
