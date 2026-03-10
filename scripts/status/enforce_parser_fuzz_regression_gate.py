#!/usr/bin/env python3
"""Fail CI when known parser crash cases or parser fuzz regressions drift."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
REQUIRED = [
    STATUS / "parser_crash_triage_artifact.json",
    STATUS / "parser_fuzz_regression_artifact.json",
    STATUS / "parser_fuzz_campaign_artifact.json",
]


def main() -> int:
    missing = [path for path in REQUIRED if not path.exists()]
    if missing:
        raise SystemExit("missing parser fuzz artifacts: " + ", ".join(str(p.relative_to(ROOT)) for p in missing))

    triage = json.loads((STATUS / "parser_crash_triage_artifact.json").read_text(encoding="utf-8"))
    if not triage.get("regression_test_ok", False):
        raise SystemExit("parser crash triage regression replay failed")

    regression = json.loads((STATUS / "parser_fuzz_regression_artifact.json").read_text(encoding="utf-8"))
    if regression.get("status") != "clean":
        raise SystemExit(
            f"parser fuzz regression drift detected: missing_todos={regression.get('missing_todos', [])}"
        )

    campaign = json.loads((STATUS / "parser_fuzz_campaign_artifact.json").read_text(encoding="utf-8"))
    if campaign.get("status") != "complete":
        raise SystemExit("parser fuzz campaign is not complete")

    print("parser fuzz regression gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
