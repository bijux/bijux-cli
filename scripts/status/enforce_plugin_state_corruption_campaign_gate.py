#!/usr/bin/env python3
"""Fail CI when plugin/history/memory corruption campaign evidence drifts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

REQUIRED = [
    STATUS / "plugin_state_corruption_campaign_artifact.json",
    STATUS / "plugin_state_corruption_corpus_retention_artifact.json",
    STATUS / "plugin_state_corruption_triage_artifact.json",
    STATUS / "plugin_state_corruption_regression_artifact.json",
    STATUS / "plugin_state_corruption_severity_classification.json",
    STATUS / "plugin_state_corruption_contract.json",
]


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    missing = [p for p in REQUIRED if not p.exists()]
    if missing:
        raise SystemExit(
            "missing plugin/history/memory corruption artifacts: "
            + ", ".join(str(p.relative_to(ROOT)) for p in missing)
        )

    campaign = read_json(STATUS / "plugin_state_corruption_campaign_artifact.json")
    corpus = read_json(STATUS / "plugin_state_corruption_corpus_retention_artifact.json")
    triage = read_json(STATUS / "plugin_state_corruption_triage_artifact.json")
    regressions = read_json(STATUS / "plugin_state_corruption_regression_artifact.json")
    contract = read_json(STATUS / "plugin_state_corruption_contract.json")

    if campaign.get("status") != "complete":
        raise SystemExit("plugin/history/memory corruption campaigns are incomplete")
    if corpus.get("status") != "complete":
        raise SystemExit("plugin/history/memory corruption corpus retention is incomplete")
    if triage.get("status") != "clean":
        raise SystemExit("plugin/history/memory corruption triage requires attention")
    if regressions.get("status") != "clean":
        raise SystemExit("plugin/history/memory corruption regression drift detected")
    if contract.get("status") != "frozen":
        raise SystemExit(f"plugin/history/memory corruption contract is not frozen: {contract.get('missing_todos', [])}")

    print("plugin/history/memory corruption gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
