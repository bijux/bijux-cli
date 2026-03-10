#!/usr/bin/env python3
"""Generate install/runtime truth artifacts for maintainers."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS_DIR = ROOT / "artifacts" / "status"


def run_bijux(args: list[str], text: bool = False) -> dict[str, Any] | str:
    cmd = ["cargo", "run", "-q", "-p", "bijux-cli", "--", *args]
    proc = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, check=True)
    if text:
        return proc.stdout
    return json.loads(proc.stdout)


def write_json(name: str, payload: dict[str, Any]) -> None:
    path = STATUS_DIR / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {path.relative_to(ROOT)}")


def write_text(name: str, payload: str) -> None:
    path = STATUS_DIR / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(payload, encoding="utf-8")
    print(f"wrote {path.relative_to(ROOT)}")


def main() -> int:
    generated_at = datetime.now(timezone.utc).isoformat()
    runtime_identity = run_bijux(["dev", "cli", "runtime-identity", "--json", "--no-pretty"])
    package_health = run_bijux(["dev", "cli", "package-health", "--json", "--no-pretty"])
    install_text = run_bijux(["dev", "cli", "runtime-identity", "--text"], text=True)

    assert isinstance(runtime_identity, dict)
    assert isinstance(package_health, dict)

    install_source_payload = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_install_truth_reports.py",
        "source_command": "bijux dev cli runtime-identity --json --no-pretty",
        "active_binary": runtime_identity.get("active_binary"),
        "install_source": runtime_identity.get("install_source"),
        "path_binaries": runtime_identity.get("path_binaries", []),
        "diagnostics": runtime_identity.get("diagnostics", {}),
    }
    write_json("install_source_diagnostics.json", install_source_payload)

    diagnostics = runtime_identity.get("diagnostics", {})
    ambiguous_payload = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_install_truth_reports.py",
        "source_command": "bijux dev cli runtime-identity --json --no-pretty",
        "active_binary_selection_is_ambiguous": runtime_identity.get(
            "active_binary_selection_is_ambiguous", False
        ),
        "active_path_is_shadowed": runtime_identity.get("active_path_is_shadowed", False),
        "duplicate_install_detected": diagnostics.get("duplicate_install_detected", False),
        "mixed_pip_cargo_install_detected": diagnostics.get(
            "mixed_pip_cargo_install_detected", False
        ),
        "path_shadowing_detected": diagnostics.get("path_shadowing_detected", False),
        "stale_wrapper_detected": diagnostics.get("stale_wrapper_detected", False),
        "active_binary_mismatch_detected": diagnostics.get(
            "active_binary_mismatch_detected", False
        ),
        "python_bridge_supported": diagnostics.get("python_bridge_supported", True),
    }
    write_json("ambiguous_runtime_diagnostics.json", ambiguous_payload)

    install_health_payload = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_install_truth_reports.py",
        "source_commands": [
            "bijux dev cli runtime-identity --json --no-pretty",
            "bijux dev cli package-health --json --no-pretty",
        ],
        "runtime_identity": runtime_identity,
        "install_state_assumptions": package_health.get("install_state_assumptions", []),
        "install_state_assumption_help": package_health.get("install_state_assumption_help", ""),
    }
    write_json("install_health_report.json", install_health_payload)
    write_text("install_health_report.txt", str(install_text))

    ambiguities: list[str] = []
    if ambiguous_payload["active_binary_selection_is_ambiguous"]:
        ambiguities.append("multiple bijux binaries detected in PATH order")
    if ambiguous_payload["path_shadowing_detected"]:
        ambiguities.append("PATH shadowing detected for canonical bijux executable")
    if ambiguous_payload["mixed_pip_cargo_install_detected"]:
        ambiguities.append("cargo and pip installations both appear active")
    if ambiguous_payload["stale_wrapper_detected"]:
        ambiguities.append("stale wrapper scripts found in PATH")
    if ambiguous_payload["active_binary_mismatch_detected"]:
        ambiguities.append("runtime binary version does not match wheel version")
    if not ambiguous_payload["python_bridge_supported"]:
        ambiguities.append("python bridge support is unavailable for current runtime")

    write_json(
        "remaining_install_ambiguities.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_install_truth_reports.py",
            "count": len(ambiguities),
            "ambiguities": ambiguities,
            "status": "clear" if not ambiguities else "attention-required",
        },
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
