#!/usr/bin/env python3
"""Enforce install-neutrality evidence and regression gates."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def run_install_tests() -> tuple[bool, str]:
    cmd = [
        "cargo",
        "test",
        "-q",
        "-p",
        "bijux-cli-core",
        "--test",
        "install_ambiguity_hardening",
        "--",
        "--nocapture",
    ]
    proc = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, check=False)
    if proc.returncode != 0:
        return False, proc.stdout + proc.stderr
    return True, "install ambiguity hardening tests passed"


def main() -> int:
    failures: list[str] = []

    ok, detail = run_install_tests()
    if not ok:
        failures.append(f"install tests failed:\n{detail}")

    neutrality = read_json(STATUS / "install_neutrality_report.json")
    runtime = read_json(STATUS / "active_runtime_report.json")
    remaining = read_json(STATUS / "remaining_install_ambiguities.json")

    if not neutrality:
        failures.append("missing artifacts/status/install_neutrality_report.json")
    if not runtime:
        failures.append("missing artifacts/status/active_runtime_report.json")
    if not remaining:
        failures.append("missing artifacts/status/remaining_install_ambiguities.json")

    if neutrality and neutrality.get("status") != "complete":
        failures.append("install neutrality report status is not complete")

    known = neutrality.get("known_remaining_install_ambiguities", []) if neutrality else []
    if neutrality and not isinstance(known, list):
        failures.append("known_remaining_install_ambiguities must be a list")

    if failures:
        print("INSTALL NEUTRALITY GATE FAILED")
        for failure in failures:
            print(f" - {failure}")
        return 1

    print("Install neutrality gate passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
