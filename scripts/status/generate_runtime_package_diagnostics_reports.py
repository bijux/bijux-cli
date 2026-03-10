#!/usr/bin/env python3
"""Generate runtime-identity/package-health diagnostics hardening artifacts."""

from __future__ import annotations

import json
import os
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def run_json(args: list[str], env: dict[str, str] | None = None) -> dict:
    merged = os.environ.copy()
    if env:
        merged.update(env)
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-bin", "--", *args, "--format", "json", "--no-pretty"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        env=merged,
        check=True,
    )
    return json.loads(proc.stdout or "{}")


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    out = STATUS / name
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out.relative_to(ROOT)}")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="bijux-runtime-diagnostics-") as raw_tmp:
        tmp = Path(raw_tmp)
        cargo = tmp / ".cargo" / "bin"
        pip = tmp / "site-packages" / "bin"
        wrappers = tmp / "wrappers"
        cargo.mkdir(parents=True, exist_ok=True)
        pip.mkdir(parents=True, exist_ok=True)
        wrappers.mkdir(parents=True, exist_ok=True)
        (cargo / "bijux").write_text("#!/bin/sh\n", encoding="utf-8")
        (pip / "bijux").write_text("#!/bin/sh\n", encoding="utf-8")
        (wrappers / "bijux.sh").write_text("#!/bin/sh\nexec /missing/bijux\n", encoding="utf-8")
        path_mixed = os.pathsep.join([str(cargo), str(pip), os.environ.get("PATH", "")])

        runtime_payload = run_json(
            ["dev", "cli", "runtime-identity"],
            env={
                "PATH": path_mixed,
                "BIJUX_BIN": str(tmp / "missing-bijux"),
                "BIJUX_WHEEL_VERSION": "0.0.1",
                "BIJUX_PYTHON_BRIDGE_SUPPORTED": "0",
            },
        )
        package_payload = run_json(
            ["dev", "cli", "package-health"],
            env={"PATH": path_mixed, "BIJUX_PYTHON_BRIDGE_SUPPORTED": "0"},
        )
        runtime_second = run_json(
            ["dev", "cli", "runtime-identity"],
            env={
                "PATH": path_mixed,
                "BIJUX_BIN": str(tmp / "missing-bijux"),
                "BIJUX_WHEEL_VERSION": "0.0.1",
                "BIJUX_PYTHON_BRIDGE_SUPPORTED": "0",
            },
        )
        package_second = run_json(
            ["dev", "cli", "package-health"],
            env={"PATH": path_mixed, "BIJUX_PYTHON_BRIDGE_SUPPORTED": "0"},
        )

    runtime_checks = {
        "has_entrypoints": isinstance(runtime_payload.get("entrypoints"), dict),
        "detects_mixed_install": runtime_payload.get("diagnostics", {}).get("mixed_pip_cargo_install_detected") is True,
        "detects_path_shadowing": runtime_payload.get("diagnostics", {}).get("path_shadowing_detected") is True,
        "detects_stale_wrapper_or_missing_binary": runtime_payload.get("diagnostics", {}).get("active_binary_missing")
        is True,
        "detects_wheel_binary_mismatch": runtime_payload.get("diagnostics", {}).get("mismatched_wheel_binary_versions")
        is True,
        "runtime_output_deterministic": runtime_payload == runtime_second,
    }
    package_checks = {
        "has_install_assumptions": isinstance(package_payload.get("install_state_assumptions"), list),
        "has_runtime_identity_rules": isinstance(package_payload.get("runtime_identity_rules"), dict),
        "package_output_deterministic": package_payload == package_second,
    }
    ambiguity_checks = {
        "runtime_identity_operator_truth": runtime_payload.get("runtime_truth_default") == "bijux dev cli runtime-identity",
        "package_health_reports_assumptions": len(package_payload.get("install_state_assumptions", [])) > 0,
        "python_runtime_relevance_present": isinstance(package_payload.get("runtime_identity_rules"), dict),
    }
    all_checks = {**runtime_checks, **package_checks, **ambiguity_checks}
    drift = [name for name, ok in all_checks.items() if not ok]

    write_json(
        "runtime_identity_diagnostics_artifact.json",
        {
            "scope": "runtime identity diagnostics",
            "generator": "scripts/status/generate_runtime_package_diagnostics_reports.py",
            "checks": runtime_checks,
            "status": "complete" if all(runtime_checks.values()) else "partial",
        },
    )
    write_json(
        "package_health_diagnostics_artifact.json",
        {
            "scope": "package health diagnostics",
            "generator": "scripts/status/generate_runtime_package_diagnostics_reports.py",
            "checks": package_checks,
            "status": "complete" if all(package_checks.values()) else "partial",
        },
    )
    write_json(
        "install_ambiguity_diagnostics_artifact.json",
        {
            "scope": "install ambiguity diagnostics",
            "generator": "scripts/status/generate_runtime_package_diagnostics_reports.py",
            "checks": ambiguity_checks,
            "status": "complete" if all(ambiguity_checks.values()) else "partial",
        },
    )
    write_json(
        "runtime_package_diagnostics_drift_artifact.json",
        {
            "scope": "runtime/package diagnostics drift",
            "generator": "scripts/status/generate_runtime_package_diagnostics_reports.py",
            "drift_checks": drift,
            "drift_count": len(drift),
            "status": "clean" if not drift else "drift",
        },
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
