#!/usr/bin/env python3
"""Fail CI when config corruption campaign hardening drifts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

REQUIRED = [
    STATUS / "config_corruption_campaign_artifact.json",
    STATUS / "config_corruption_invariants_artifact.json",
    STATUS / "config_corruption_corpus_retention_artifact.json",
    STATUS / "config_corruption_triage_artifact.json",
    STATUS / "config_corruption_regression_artifact.json",
    STATUS / "config_corruption_severity_classification.json",
    STATUS / "config_corruption_recovery_classification.json",
    STATUS / "config_corruption_determinism_artifact.json",
    STATUS / "config_corruption_release_blocking_contract.json",
]


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    missing = [p for p in REQUIRED if not p.exists()]
    if missing:
        raise SystemExit(
            "missing config corruption campaign artifacts: "
            + ", ".join(str(p.relative_to(ROOT)) for p in missing)
        )

    campaign = read_json(STATUS / "config_corruption_campaign_artifact.json")
    invariants = read_json(STATUS / "config_corruption_invariants_artifact.json")
    corpus = read_json(STATUS / "config_corruption_corpus_retention_artifact.json")
    triage = read_json(STATUS / "config_corruption_triage_artifact.json")
    regression = read_json(STATUS / "config_corruption_regression_artifact.json")
    determinism = read_json(STATUS / "config_corruption_determinism_artifact.json")
    contract = read_json(STATUS / "config_corruption_release_blocking_contract.json")

    if campaign.get("status") != "complete":
        raise SystemExit("config corruption campaign coverage is not complete")
    if invariants.get("status") != "complete":
        raise SystemExit("config corruption invariants are not complete")
    if corpus.get("status") != "complete":
        raise SystemExit("config corruption corpus retention is not complete")
    if triage.get("status") != "clean":
        raise SystemExit("config corruption triage requires attention")
    if regression.get("status") != "clean":
        raise SystemExit("config corruption regression replay drift detected")
    if determinism.get("status") != "complete":
        raise SystemExit("config corruption determinism evidence is incomplete")
    if contract.get("status") != "frozen":
        raise SystemExit(
            f"config corruption release-blocking contract is not frozen: {contract.get('missing_todos', [])}"
        )

    print("config corruption campaign gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
