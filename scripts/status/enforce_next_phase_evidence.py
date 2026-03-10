#!/usr/bin/env python3
"""Enforce evidence-only next-priority planning requirements."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    failures: list[str] = []

    required = [
        STATUS / "next_phase.json",
        STATUS / "next_phase.txt",
        STATUS / "ranked_python_only_behaviors.json",
        STATUS / "ranked_parity_partial_behaviors.json",
        STATUS / "ranked_plugin_gaps.json",
        STATUS / "ranked_repl_gaps.json",
        STATUS / "ranked_packaging_gaps.json",
        STATUS / "ranked_state_management_gaps.json",
        STATUS / "ranked_untested_corruption_scenarios.json",
        STATUS / "ranked_untested_ambiguity_scenarios.json",
        STATUS / "ranked_crate_complexity.json",
        STATUS / "ranked_api_simplification_candidates.json",
        STATUS / "ranked_docs_deletion_candidates.json",
        STATUS / "ranked_script_deletion_candidates.json",
        STATUS / "ranked_weak_test_replacements.json",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        failures.append(f"missing next-phase artifacts: {', '.join(missing)}")

    next_phase = read_json(STATUS / "next_phase.json")
    policy = next_phase.get("evidence_first_policy", {}) if isinstance(next_phase, dict) else {}

    if policy.get("manual_curated_priority_lists_allowed") is not False:
        failures.append("manual curated next-phase priorities must be disabled")

    merge_sources = set(policy.get("crate_merge_reassessment_source", []))
    if {
        "artifacts/status/crate_boundary_metrics.json",
        "artifacts/status/crate_boundary_report.json",
    } - merge_sources:
        failures.append("crate merge reassessment must depend on generated crate complexity artifacts")

    api_sources = set(policy.get("public_api_trim_reassessment_source", []))
    if {
        "artifacts/status/internal_only_candidates_by_crate.json",
        "artifacts/status/cross_crate_api_usage.json",
        "artifacts/status/crate_boundary_metrics.json",
    } - api_sources:
        failures.append("public API trim reassessment must depend on usage and complexity artifacts")

    manual_priority_files = []
    for path in ROOT.rglob("*next*phase*"):
        if not path.is_file():
            continue
        rel = str(path.relative_to(ROOT))
        if rel.startswith("scripts/status/"):
            continue
        if rel.startswith("artifacts/status/"):
            continue
        if rel.endswith("next_phase.json") or rel.endswith("next_phase.txt"):
            continue
        manual_priority_files.append(rel)
    if manual_priority_files:
        failures.append(
            "manual next-phase files detected outside generated artifacts: "
            + ", ".join(sorted(manual_priority_files)[:20])
        )

    for msg in failures:
        print(f"NEXT PHASE GATE FAILURE: {msg}")

    return 2 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
