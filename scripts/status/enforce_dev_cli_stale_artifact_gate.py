#!/usr/bin/env python3
"""Fail CI when critical stale-artifact scenarios are detected."""

from __future__ import annotations

import json
import os
from pathlib import Path


def _root() -> Path:
    override = os.environ.get("DEV_CLI_STALE_ARTIFACT_ROOT", "").strip()
    if override:
        return Path(override).resolve()
    return Path(__file__).resolve().parents[2]


ROOT = _root()
STATUS = ROOT / "artifacts" / "status"


def _read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    failures: list[str] = []
    required = [
        STATUS / "stale_artifact_artifact.json",
        STATUS / "stale_evidence_artifact.json",
        STATUS / "stale_report_artifact.json",
        STATUS / "stale_detection_regression_suite.json",
    ]
    for path in required:
        if not path.exists():
            failures.append(f"missing stale artifact: {path.relative_to(ROOT)}")

    if failures:
        print("DEV CLI STALE ARTIFACT GATE FAILED")
        for failure in failures:
            print(f" - {failure}")
        return 1

    payload = _read_json(STATUS / "stale_artifact_artifact.json")
    summary = payload.get("summary", {})
    critical = int(summary.get("critical_stale_count", 0))
    warnings = int(summary.get("warning_stale_count", 0))
    injection_mode = bool(summary.get("injection_mode"))

    if injection_mode and os.environ.get("DEV_CLI_ALLOW_INJECTION_DRIFT", "0") == "1":
        # Dedicated CI verification mode intentionally creates stale drift.
        if critical <= 0:
            failures.append("injection mode expected critical stale detection but got none")
        else:
            print(f"stale injection mode verified: critical_stale_count={critical}")
            return 0

    if critical > 0:
        failures.append(f"critical stale artifacts detected: {critical}")

    if warnings > 0:
        print(f"stale warnings tolerated by policy: warning_stale_count={warnings}")

    if failures:
        print("DEV CLI STALE ARTIFACT GATE FAILED")
        for failure in failures:
            print(f" - {failure}")
        return 1

    print("Dev CLI stale artifact gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
