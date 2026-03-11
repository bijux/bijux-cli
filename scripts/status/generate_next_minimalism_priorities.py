#!/usr/bin/env python3
"""Generate evidence-ranked simplification priorities for 781-800."""

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
    payload = json.loads(path.read_text(encoding="utf-8"))
    return payload if isinstance(payload, dict) else {}


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT))


def ranked_from(path: Path) -> list[dict[str, Any]]:
    return read_json(path).get("items", [])


def emit(name: str, title: str, items: list[dict[str, Any]], sources: list[str], coverage_id: int, generated_at: str) -> dict[str, Any]:
    payload = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_next_minimalism_priorities.py",
        "title": title,
        "coverage_id": coverage_id,
        "sources": sources,
        "items": items,
    }
    write_json(STATUS / f"{name}.json", payload)
    return payload


def build_shim_alias_ranking() -> list[dict[str, Any]]:
    shims = read_json(STATUS / "live_compatibility_shims.json").get("items", [])
    aliases = read_json(STATUS / "live_compatibility_aliases.json").get("items", [])
    rows: list[dict[str, Any]] = []
    for item in shims if isinstance(shims, list) else []:
        if isinstance(item, dict):
            rows.append(
                {
                    "kind": "shim",
                    "id": item.get("command") or item.get("path") or item.get("shim"),
                    "justification": item.get("justification", ""),
                    "removal_plan": item.get("removal_plan", ""),
                }
            )
    for item in aliases if isinstance(aliases, list) else []:
        if isinstance(item, dict):
            rows.append(
                {
                    "kind": "alias",
                    "id": item.get("alias") or item.get("path") or item.get("command"),
                    "justification": item.get("justification", ""),
                    "removal_plan": item.get("removal_plan", ""),
                }
            )
    rows.sort(key=lambda row: (row.get("kind", ""), str(row.get("id", ""))))
    return [{"rank": i + 1, **row} for i, row in enumerate(rows)]


def main() -> None:
    generated_at = stable_generated_at()

    ranked_crate_complexity = ranked_from(STATUS / "ranked_crate_complexity.json")
    ranked_python_only = ranked_from(STATUS / "ranked_python_only_behaviors.json")
    ranked_api = ranked_from(STATUS / "ranked_api_simplification_candidates.json")
    ranked_state = ranked_from(STATUS / "ranked_state_management_gaps.json")
    ranked_plugin = ranked_from(STATUS / "ranked_plugin_gaps.json")
    ranked_packaging = ranked_from(STATUS / "ranked_packaging_gaps.json")
    ranked_repl = ranked_from(STATUS / "ranked_repl_gaps.json")
    ranked_docs = ranked_from(STATUS / "ranked_docs_deletion_candidates.json")
    ranked_scripts = ranked_from(STATUS / "ranked_script_deletion_candidates.json")
    ranked_weak_tests = ranked_from(STATUS / "ranked_weak_test_replacements.json")
    ranked_untested_corruption = ranked_from(STATUS / "ranked_untested_corruption_scenarios.json")

    outputs = {
        "ranked_accidental_complexity_hotspots": emit(
            "ranked_accidental_complexity_hotspots",
            "remaining accidental complexity hotspots",
            ranked_crate_complexity,
            [rel(STATUS / "ranked_crate_complexity.json")],
            781,
            generated_at,
        ),
        "ranked_python_era_leftovers": emit(
            "ranked_python_era_leftovers",
            "remaining python-era leftovers",
            ranked_python_only,
            [rel(STATUS / "ranked_python_only_behaviors.json")],
            782,
            generated_at,
        ),
        "ranked_shim_alias_leftovers": emit(
            "ranked_shim_alias_leftovers",
            "remaining shims and aliases",
            build_shim_alias_ranking(),
            [
                rel(STATUS / "live_compatibility_shims.json"),
                rel(STATUS / "live_compatibility_aliases.json"),
            ],
            783,
            generated_at,
        ),
        "ranked_public_api_removal_candidates": emit(
            "ranked_public_api_removal_candidates",
            "remaining public APIs likely to be removed",
            ranked_api,
            [rel(STATUS / "ranked_api_simplification_candidates.json")],
            784,
            generated_at,
        ),
        "ranked_remaining_state_risks": emit(
            "ranked_remaining_state_risks",
            "remaining state risks",
            ranked_state + ranked_untested_corruption,
            [
                rel(STATUS / "ranked_state_management_gaps.json"),
                rel(STATUS / "ranked_untested_corruption_scenarios.json"),
            ],
            785,
            generated_at,
        ),
        "ranked_remaining_plugin_lifecycle_risks": emit(
            "ranked_remaining_plugin_lifecycle_risks",
            "remaining plugin lifecycle risks",
            ranked_plugin,
            [rel(STATUS / "ranked_plugin_gaps.json")],
            786,
            generated_at,
        ),
        "ranked_remaining_packaging_ambiguity_risks": emit(
            "ranked_remaining_packaging_ambiguity_risks",
            "remaining packaging ambiguity risks",
            ranked_packaging,
            [rel(STATUS / "ranked_packaging_gaps.json")],
            787,
            generated_at,
        ),
        "ranked_remaining_repl_divergence_risks": emit(
            "ranked_remaining_repl_divergence_risks",
            "remaining repl divergence risks",
            ranked_repl,
            [rel(STATUS / "ranked_repl_gaps.json")],
            788,
            generated_at,
        ),
        "ranked_remaining_crate_boundary_problems": emit(
            "ranked_remaining_crate_boundary_problems",
            "remaining crate-boundary problems",
            ranked_crate_complexity,
            [rel(STATUS / "ranked_crate_complexity.json")],
            789,
            generated_at,
        ),
        "ranked_remaining_docs_worth_deleting": emit(
            "ranked_remaining_docs_worth_deleting",
            "remaining docs worth deleting",
            ranked_docs,
            [rel(STATUS / "ranked_docs_deletion_candidates.json")],
            790,
            generated_at,
        ),
        "ranked_remaining_scripts_worth_deleting": emit(
            "ranked_remaining_scripts_worth_deleting",
            "remaining scripts worth deleting",
            ranked_scripts,
            [rel(STATUS / "ranked_script_deletion_candidates.json")],
            791,
            generated_at,
        ),
        "ranked_remaining_weak_tests_worth_replacing": emit(
            "ranked_remaining_weak_tests_worth_replacing",
            "remaining weak tests worth replacing",
            ranked_weak_tests,
            [rel(STATUS / "ranked_weak_test_replacements.json")],
            792,
            generated_at,
        ),
    }

    simplification_priorities = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_next_minimalism_priorities.py",
        "coverage_ids": [793, 795, 796, 797, 798, 799, 800],
        "ranked_reports": {name: rel(STATUS / f"{name}.json") for name in outputs},
        "top_priorities": {name: payload.get("items", [])[:5] for name, payload in outputs.items()},
        "evidence_first_policy": {
            "manual_curated_priority_lists_allowed": False,
            "roadmap_requires_generated_artifacts": True,
            "crate_merge_reassessment_source": [
                rel(STATUS / "ranked_crate_complexity.json"),
                rel(STATUS / "ranked_remaining_crate_boundary_problems.json"),
            ],
            "docs_survival_reassessment_source": [
                rel(STATUS / "ranked_docs_deletion_candidates.json"),
                rel(STATUS / "ranked_remaining_docs_worth_deleting.json"),
                rel(STATUS / "cleanup_report.json"),
            ],
            "shim_retention_reassessment_source": [
                rel(STATUS / "live_compatibility_shims.json"),
                rel(STATUS / "live_compatibility_aliases.json"),
                rel(STATUS / "ranked_shim_alias_leftovers.json"),
            ],
            "next_wave_requires_artifacts": [
                rel(STATUS / "simplification_priorities.json"),
                rel(STATUS / "simplification_priorities.txt"),
            ],
        },
    }
    write_json(STATUS / "simplification_priorities.json", simplification_priorities)

    lines = ["Next Simplification Priorities (Evidence-Ranked)", ""]
    for name, payload in outputs.items():
        lines.append(f"{payload['title']}:")
        for item in payload.get("items", [])[:5]:
            label = (
                item.get("command")
                or item.get("gap")
                or item.get("path")
                or item.get("crate")
                or item.get("symbol")
                or item.get("case")
                or item.get("id")
            )
            lines.append(f"- {label}")
        lines.append("")
    write_text(STATUS / "simplification_priorities.txt", "\n".join(lines))

    print("wrote artifacts/status/simplification_priorities.json")
    print("wrote artifacts/status/simplification_priorities.txt")
    for name in outputs:
        print(f"wrote artifacts/status/{name}.json")


if __name__ == "__main__":
    main()
