#!/usr/bin/env python3
"""Fail CI if install/runtime identity behavior regresses for covered cases."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
REQUIRED = [
    STATUS / "install_runtime_identity_artifact.json",
    STATUS / "install_ambiguity_artifact.json",
    STATUS / "package_health_artifact.json",
    STATUS / "install_runtime_identity_drift_artifact.json",
    STATUS / "install_runtime_identity_contract.json",
]


def main() -> int:
    tests = subprocess.run(
        [
            "cargo",
            "test",
            "-q",
            "-p",
            "bijux-cli",
            "--test",
            "install_ambiguity_hardening",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if tests.returncode != 0:
        raise SystemExit("install/runtime identity tests failed:\n" + tests.stdout + tests.stderr)

    missing = [path for path in REQUIRED if not path.exists()]
    if missing:
        raise SystemExit(
            "missing install/runtime identity artifacts: "
            + ", ".join(str(path.relative_to(ROOT)) for path in missing)
        )

    drift = json.loads((STATUS / "install_runtime_identity_drift_artifact.json").read_text(encoding="utf-8"))
    if drift.get("status") != "clean" or int(drift.get("drift_count", 1)) != 0:
        raise SystemExit(f"install/runtime identity drift detected: todos={drift.get('drift_todos', [])}")

    contract = json.loads((STATUS / "install_runtime_identity_contract.json").read_text(encoding="utf-8"))
    if contract.get("status") != "frozen":
        raise SystemExit("install/runtime identity contract is not frozen")

    print("install/runtime identity gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
