#!/usr/bin/env python3
"""Fail CI when repo/docs/scripts/crate-health hardening drifts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    failures: list[str] = []
    truth = STATUS / "repo_docs_scripts_crate_health_artifact.json"
    drift = STATUS / "repo_docs_scripts_crate_health_drift_artifact.json"
    if not truth.exists():
        failures.append("missing artifact: artifacts/status/repo_docs_scripts_crate_health_artifact.json")
    if not drift.exists():
        failures.append("missing artifact: artifacts/status/repo_docs_scripts_crate_health_drift_artifact.json")

    if not failures:
        truth_payload = read_json(truth)
        drift_payload = read_json(drift)
        if truth_payload.get("status") != "complete":
            failures.append("repo/docs/scripts/crate-health artifact is not complete")
        if drift_payload.get("status") != "clean" or int(drift_payload.get("drift_count", 1)) != 0:
            failures.append("repo/docs/scripts/crate-health drift detected")

    if failures:
        print("DEV CLI REPO DOCS SCRIPTS CRATE HEALTH GATE FAILED")
        for failure in failures:
            print(f" - {failure}")
        return 1
    print("Dev CLI repo/docs/scripts/crate-health gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
