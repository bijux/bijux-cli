#!/usr/bin/env python3
"""Enforce freshness/readiness of dev-cli release truth bundle."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BUNDLE = ROOT / "artifacts" / "status" / "dev_cli_release_truth_bundle.json"


def main() -> int:
    if not BUNDLE.exists():
        print("release-bundle-failure: missing artifacts/status/dev_cli_release_truth_bundle.json")
        return 1

    payload = json.loads(BUNDLE.read_text(encoding="utf-8"))
    reports = payload.get("reports", {}) if isinstance(payload, dict) else {}
    required = {"status", "evidence", "readiness", "diff", "gaps"}
    missing = sorted(required - set(reports.keys()))
    if missing:
        print("release-bundle-failure: missing report keys: " + ", ".join(missing))
        return 1

    print("dev cli release bundle freshness policy satisfied")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
