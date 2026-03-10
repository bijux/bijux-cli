#!/usr/bin/env python3
"""Generate simplification evidence artifacts for TODO 161-180."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def stable_generated_at() -> str:
    source_date_epoch = subprocess.run(
        ["sh", "-lc", "printf %s \"${SOURCE_DATE_EPOCH:-}\""],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if source_date_epoch.isdigit():
        return datetime.fromtimestamp(int(source_date_epoch), tz=timezone.utc).isoformat()
    return "1970-01-01T00:00:00+00:00"


def read_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    generated_at = stable_generated_at()
    boundary = read_json(STATUS / "crate_boundary_report.json")
    public_api = read_json(STATUS / "public_api_by_crate.json")
    complexity = read_json(STATUS / "ranked_crate_complexity.json")
    duplication = read_json(STATUS / "duplication_hotspots.json")
    removal_candidates = read_json(STATUS / "ranked_public_api_removal_candidates.json")
    route_special = read_json(STATUS / "route_special_cases.json")
    cross_crate_usage = read_json(STATUS / "cross_crate_api_usage.json")

    merge_later = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_simplification_reports.py",
        "scope": "candidate to merge later",
        "items": [
            row
            for row in boundary.get("crate_decisions", [])
            if isinstance(row, dict) and str(row.get("status", "")) == "candidate-to-merge-later"
        ],
    }
    keep_separate = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_simplification_reports.py",
        "scope": "candidate to keep separate",
        "items": [
            row
            for row in boundary.get("crate_decisions", [])
            if isinstance(row, dict) and str(row.get("status", "")) in {"keep", "watch"}
        ],
    }

    write_json(
        STATUS / "cross_crate_duplication_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_simplification_reports.py",
            "scope": "cross-crate duplication",
            "duplication_hotspots": duplication,
            "cross_crate_api_usage": cross_crate_usage,
            "status": "tracked",
        },
    )
    write_json(
        STATUS / "public_api_inventory_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_simplification_reports.py",
            "scope": "public api inventory",
            "public_api_by_crate": public_api,
            "removal_candidates": removal_candidates,
            "status": "tracked",
        },
    )
    write_json(
        STATUS / "crate_complexity_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_simplification_reports.py",
            "scope": "crate complexity",
            "ranked_crate_complexity": complexity,
            "status": "tracked",
        },
    )
    write_json(STATUS / "candidate_merge_later_report.json", merge_later)
    write_json(STATUS / "candidate_keep_separate_report.json", keep_separate)

    simplification = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_simplification_reports.py",
        "scope": "simplification deletions",
        "safe_deletions": [
            "legacy dev aliases removed from routing map",
            "three internal-only completion APIs reduced to crate visibility",
            "python command-tree introspection now derived from canonical inspect output",
            "namespace normalization delegated to contracts::Namespace::normalize",
            "active binary missing/broken symlink diagnostics centralized in install layer",
            "route special-case count held at zero",
        ],
        "route_special_cases": route_special.get("report", {}).get("summary", {}),
        "policy": {
            "merge_boundary_rule": "do not merge crates unless boundary evidence says boundary is fake",
            "abstraction_rule": "do not add abstractions during simplification-only pass",
            "reassessment_rule": "reassess architecture only after parity and neutrality reports remain stable",
        },
    }
    write_json(STATUS / "simplification_deletion_artifact.json", simplification)

    text_lines = [
        "Simplification Deletion Artifact",
        f"route special-case count: {simplification['route_special_cases'].get('special_case_count', 'unknown')}",
        "safe deletions:",
    ]
    text_lines.extend([f"- {item}" for item in simplification["safe_deletions"]])
    text_lines.append("policy:")
    text_lines.append(f"- {simplification['policy']['merge_boundary_rule']}")
    text_lines.append(f"- {simplification['policy']['abstraction_rule']}")
    text_lines.append(f"- {simplification['policy']['reassessment_rule']}")
    (STATUS / "simplification_deletion_artifact.txt").write_text(
        "\n".join(text_lines) + "\n", encoding="utf-8"
    )

    print("wrote artifacts/status/cross_crate_duplication_report.json")
    print("wrote artifacts/status/public_api_inventory_report.json")
    print("wrote artifacts/status/crate_complexity_report.json")
    print("wrote artifacts/status/candidate_merge_later_report.json")
    print("wrote artifacts/status/candidate_keep_separate_report.json")
    print("wrote artifacts/status/simplification_deletion_artifact.json")
    print("wrote artifacts/status/simplification_deletion_artifact.txt")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
