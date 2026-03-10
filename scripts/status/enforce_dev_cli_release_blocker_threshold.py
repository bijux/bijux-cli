#!/usr/bin/env python3
"""Fail CI when unresolved release blockers exceed threshold."""

from __future__ import annotations

import json
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BUNDLE = ROOT / "artifacts" / "status" / "dev_cli_release_truth_bundle.json"


def main() -> int:
    threshold = int(os.getenv("BIJUX_RELEASE_BLOCKER_THRESHOLD", "0"))
    if not BUNDLE.exists():
        print("release-blocker-failure: missing dev_cli_release_truth_bundle.json")
        return 1

    payload = json.loads(BUNDLE.read_text(encoding="utf-8"))
    summary = payload.get("summary", {}) if isinstance(payload, dict) else {}
    unresolved = int(summary.get("unresolved_gaps", 0))
    if unresolved > threshold:
        print(
            f"release-blocker-failure: unresolved_gaps={unresolved} exceeds threshold={threshold}"
        )
        return 1

    print("dev cli release blocker threshold policy satisfied")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
