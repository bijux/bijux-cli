#!/usr/bin/env python3
"""Fail CI when runtime/package diagnostics hardening drifts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    failures: list[str] = []
    required = [
        STATUS / "runtime_identity_diagnostics_artifact.json",
        STATUS / "package_health_diagnostics_artifact.json",
        STATUS / "install_ambiguity_diagnostics_artifact.json",
        STATUS / "runtime_package_diagnostics_drift_artifact.json",
    ]
    for path in required:
        if not path.exists():
            failures.append(f"missing artifact: {path.relative_to(ROOT)}")

    if not failures:
        runtime = read_json(STATUS / "runtime_identity_diagnostics_artifact.json")
        package = read_json(STATUS / "package_health_diagnostics_artifact.json")
        ambiguity = read_json(STATUS / "install_ambiguity_diagnostics_artifact.json")
        drift = read_json(STATUS / "runtime_package_diagnostics_drift_artifact.json")

        if runtime.get("status") != "complete":
            failures.append("runtime identity diagnostics artifact is not complete")
        if package.get("status") != "complete":
            failures.append("package health diagnostics artifact is not complete")
        if ambiguity.get("status") != "complete":
            failures.append("install ambiguity diagnostics artifact is not complete")
        if drift.get("status") != "clean" or int(drift.get("drift_count", 1)) != 0:
            failures.append("runtime/package diagnostics drift detected")

    if failures:
        print("RUNTIME PACKAGE DIAGNOSTICS GATE FAILED")
        for failure in failures:
            print(f" - {failure}")
        return 1
    print("Runtime/package diagnostics gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
