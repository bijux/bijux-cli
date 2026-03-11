#!/usr/bin/env python3
"""Generate evidence-ranked next priorities and priority_plan artifacts."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
PARITY = ROOT / "artifacts" / "parity"


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


def read_json(path: Path) -> dict[str, Any] | list[Any]:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT))


def parity_rows() -> list[dict[str, Any]]:
    matrix = read_json(PARITY / "command_parity_matrix.json")
    if not isinstance(matrix, dict):
        return []
    rows = matrix.get("commands", [])
    return [row for row in rows if isinstance(row, dict) and row.get("command")]


def ranked_python_only(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    python_rows = [
        row
        for row in rows
        if str(row.get("status", "missing")) == "missing"
        or str(row.get("owner", "")).lower().startswith("python")
    ]
    python_rows.sort(key=lambda row: (str(row.get("status", "missing")) != "missing", str(row.get("command", ""))))
    ranked = []
    for idx, row in enumerate(python_rows, start=1):
        ranked.append(
            {
                "rank": idx,
                "command": row.get("command"),
                "status": row.get("status"),
                "owner": row.get("owner"),
                "blocker": row.get("blocker", ""),
                "reason": row.get("reason", ""),
            }
        )
    return ranked


def ranked_parity_partial(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    partial = [row for row in rows if str(row.get("status", "")) == "partial"]
    partial.sort(
        key=lambda row: (
            float(row.get("confidence", 1.0)),
            str(row.get("blocker", "")),
            str(row.get("command", "")),
        )
    )
    return [
        {
            "rank": idx,
            "command": row.get("command"),
            "confidence": row.get("confidence", 0.0),
            "blocker": row.get("blocker", ""),
            "reason": row.get("reason", ""),
        }
        for idx, row in enumerate(partial, start=1)
    ]


def ranked_plugin_gaps() -> list[dict[str, Any]]:
    plugin = read_json(STATUS / "plugin_state_report.json")
    if not isinstance(plugin, dict):
        return []
    gaps: list[dict[str, Any]] = []
    for item in plugin.get("remaining_gaps", []):
        gaps.append({"gap": str(item), "severity": "high"})
    partial = plugin.get("plugin_commands", {}).get("partial", []) if isinstance(plugin.get("plugin_commands", {}), dict) else []
    for cmd in partial:
        gaps.append({"gap": f"partial command: {cmd}", "severity": "medium"})
    return [{"rank": i + 1, **item} for i, item in enumerate(gaps)]


def ranked_repl_gaps(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    repl = read_json(STATUS / "repl_only_behaviors.json")
    out: list[dict[str, Any]] = []
    if isinstance(repl, dict):
        for item in repl.get("repl_only_behaviors", []):
            if isinstance(item, dict):
                out.append(
                    {
                        "gap": item.get("name", "unknown"),
                        "category": item.get("category", "unknown"),
                        "defensible": bool(item.get("defensible", False)),
                    }
                )
    repl_partial = [
        row
        for row in rows
        if "repl" in str(row.get("command", "")).split() and str(row.get("status", "")) in {"partial", "missing"}
    ]
    for row in repl_partial:
        out.append(
            {
                "gap": f"parity: {row.get('command')}",
                "category": "parity",
                "defensible": False,
            }
        )
    out.sort(key=lambda item: (item.get("defensible", True), str(item.get("gap", ""))))
    return [{"rank": i + 1, **item} for i, item in enumerate(out)]


def ranked_packaging_gaps() -> list[dict[str, Any]]:
    report = read_json(STATUS / "packaging_ambiguity_report.json")
    out: list[dict[str, Any]] = []
    if not isinstance(report, dict):
        return out
    runtime = report.get("runtime_identity", {}) if isinstance(report.get("runtime_identity", {}), dict) else {}
    diagnostics = runtime.get("diagnostics", {}) if isinstance(runtime.get("diagnostics", {}), dict) else {}
    for key in [
        "path_shadowing_detected",
        "duplicate_installs_detected",
        "mismatched_wheel_binary_versions",
        "active_binary_missing",
        "broken_symlink_active_binary",
    ]:
        if diagnostics.get(key):
            out.append({"gap": key, "source": "runtime_identity.diagnostics"})
    if runtime.get("active_binary_selection_is_ambiguous"):
        out.append({"gap": "active_binary_selection_is_ambiguous", "source": "runtime_identity"})
    out.sort(key=lambda item: str(item["gap"]))
    return [{"rank": i + 1, **item} for i, item in enumerate(out)]


def ranked_state_management_gaps() -> list[dict[str, Any]]:
    state = read_json(STATUS / "status_state_corruption_health_report.json")
    out: list[dict[str, Any]] = []
    if isinstance(state, dict):
        areas = state.get("areas", {})
        if isinstance(areas, dict):
            for area, payload in areas.items():
                focus = payload.get("focus", []) if isinstance(payload, dict) else []
                out.append({"gap": area, "focus_count": len(focus), "focus": focus})
    out.sort(key=lambda item: (-int(item.get("focus_count", 0)), str(item.get("gap", ""))))
    return [{"rank": i + 1, **item} for i, item in enumerate(out)]


def ranked_untested_corruption() -> list[dict[str, Any]]:
    missing = read_json(STATUS / "top_20_missing_failure_cases.json")
    if not isinstance(missing, dict):
        return []
    cases = missing.get("cases", [])
    if not isinstance(cases, list):
        cases = []
    return [{"rank": i + 1, "case": item} for i, item in enumerate(cases)]


def ranked_untested_ambiguity() -> list[dict[str, Any]]:
    missing = read_json(STATUS / "top_20_missing_parity_cases.json")
    if not isinstance(missing, dict):
        return []
    cases = missing.get("cases", [])
    if not isinstance(cases, list):
        cases = []
    return [{"rank": i + 1, "case": item} for i, item in enumerate(cases)]


def ranked_crate_complexity() -> list[dict[str, Any]]:
    metrics = read_json(STATUS / "crate_boundary_metrics.json")
    per_crate = {}
    cross = []
    if isinstance(metrics, dict):
        m = metrics.get("metrics", {})
        if isinstance(m, dict):
            cross = m.get("cross_crate_change_frequency", []) if isinstance(m.get("cross_crate_change_frequency", []), list) else []
    for row in cross:
        if not isinstance(row, dict):
            continue
        left = row.get("left")
        right = row.get("right")
        shared = int(row.get("shared_commits", 0))
        if isinstance(left, str):
            per_crate[left] = per_crate.get(left, 0) + shared
        if isinstance(right, str):
            per_crate[right] = per_crate.get(right, 0) + shared
    ranked = sorted(per_crate.items(), key=lambda kv: (-kv[1], kv[0]))
    return [{"rank": i + 1, "crate": name, "complexity_score": score} for i, (name, score) in enumerate(ranked)]


def ranked_api_simplify() -> list[dict[str, Any]]:
    candidates = read_json(STATUS / "internal_only_candidates_by_crate.json")
    usage = read_json(STATUS / "cross_crate_api_usage.json")
    zero_usage = set()
    if isinstance(usage, dict):
        for item in usage.get("items", []):
            if isinstance(item, dict) and int(item.get("cross_crate_user_count", 0)) == 0:
                zero_usage.add(str(item.get("symbol", "")))
    ranked = []
    if isinstance(candidates, dict):
        crates = candidates.get("crates", {})
        if isinstance(crates, dict):
            for crate, items in crates.items():
                if not isinstance(items, list):
                    continue
                for item in items:
                    if not isinstance(item, dict):
                        continue
                    symbol = str(item.get("symbol", ""))
                    ranked.append(
                        {
                            "crate": crate,
                            "symbol": symbol,
                            "defined_at": item.get("defined_at", ""),
                            "cross_crate_user_count": 0 if symbol in zero_usage else 1,
                            "reason": item.get("reason", ""),
                        }
                    )
    ranked.sort(key=lambda row: (row["cross_crate_user_count"], row["crate"], row["symbol"]))
    return [{"rank": i + 1, **row} for i, row in enumerate(ranked)]


def ranked_docs_delete() -> list[dict[str, Any]]:
    report = read_json(STATUS / "docs_duplication_report.json")
    ranked: list[dict[str, Any]] = []
    if not isinstance(report, dict):
        return ranked
    for group in report.get("duplicate_stem_groups", []):
        if not isinstance(group, list):
            continue
        group = [str(item) for item in group]
        if len(group) < 2:
            continue
        ranked.append({"group": group, "duplicate_count": len(group)})
    ranked.sort(key=lambda item: (-item["duplicate_count"], item["group"][0]))
    return [{"rank": i + 1, **item} for i, item in enumerate(ranked)]


def ranked_scripts_delete() -> list[dict[str, Any]]:
    scripts = read_json(STATUS / "script_only_behaviors.json")
    ranked: list[dict[str, Any]] = []
    if not isinstance(scripts, dict):
        return ranked
    for path in scripts.get("remaining_script_only_behaviors", []):
        ranked.append({"path": str(path), "reason": "still script-only"})
    ranked.sort(key=lambda item: item["path"])
    return [{"rank": i + 1, **item} for i, item in enumerate(ranked)]


def ranked_weak_tests() -> list[dict[str, Any]]:
    weak = read_json(STATUS / "top_20_weakest_tests.json")
    tests = weak.get("tests", []) if isinstance(weak, dict) else []
    rows = [row for row in tests if isinstance(row, dict)]
    rows.sort(
        key=lambda row: (
            -int(row.get("shallow_score", 0)),
            int(row.get("assert_count", 0)),
            str(row.get("path", "")),
        )
    )
    return [{"rank": i + 1, **row} for i, row in enumerate(rows)]


def emit_ranked(name: str, title: str, items: list[dict[str, Any]], source: list[str], coverage_id: int, generated_at: str) -> dict[str, Any]:
    payload = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_priority_rankings.py",
        "title": title,
        "coverage_id": coverage_id,
        "source": source,
        "items": items,
    }
    write_json(STATUS / f"{name}.json", payload)
    return payload


def main() -> None:
    generated_at = stable_generated_at()
    rows = parity_rows()

    outputs = {
        "ranked_python_only_behaviors": emit_ranked(
            "ranked_python_only_behaviors",
            "remaining python-only behaviors",
            ranked_python_only(rows),
            [rel(PARITY / "command_parity_matrix.json")],
            581,
            generated_at,
        ),
        "ranked_parity_partial_behaviors": emit_ranked(
            "ranked_parity_partial_behaviors",
            "parity-partial behaviors",
            ranked_parity_partial(rows),
            [rel(PARITY / "command_parity_matrix.json")],
            582,
            generated_at,
        ),
        "ranked_plugin_gaps": emit_ranked(
            "ranked_plugin_gaps",
            "highest-value unresolved plugin gaps",
            ranked_plugin_gaps(),
            [rel(STATUS / "plugin_state_report.json")],
            583,
            generated_at,
        ),
        "ranked_repl_gaps": emit_ranked(
            "ranked_repl_gaps",
            "highest-value unresolved repl gaps",
            ranked_repl_gaps(rows),
            [rel(STATUS / "repl_only_behaviors.json"), rel(PARITY / "command_parity_matrix.json")],
            584,
            generated_at,
        ),
        "ranked_packaging_gaps": emit_ranked(
            "ranked_packaging_gaps",
            "highest-value unresolved packaging gaps",
            ranked_packaging_gaps(),
            [rel(STATUS / "packaging_ambiguity_report.json")],
            585,
            generated_at,
        ),
        "ranked_state_management_gaps": emit_ranked(
            "ranked_state_management_gaps",
            "highest-value unresolved state-management gaps",
            ranked_state_management_gaps(),
            [rel(STATUS / "status_state_corruption_health_report.json")],
            586,
            generated_at,
        ),
        "ranked_untested_corruption_scenarios": emit_ranked(
            "ranked_untested_corruption_scenarios",
            "highest-risk corruption scenarios still untested",
            ranked_untested_corruption(),
            [rel(STATUS / "top_20_missing_failure_cases.json")],
            587,
            generated_at,
        ),
        "ranked_untested_ambiguity_scenarios": emit_ranked(
            "ranked_untested_ambiguity_scenarios",
            "highest-risk ambiguity scenarios still untested",
            ranked_untested_ambiguity(),
            [rel(STATUS / "top_20_missing_parity_cases.json")],
            588,
            generated_at,
        ),
        "ranked_crate_complexity": emit_ranked(
            "ranked_crate_complexity",
            "crates with the most accidental complexity",
            ranked_crate_complexity(),
            [rel(STATUS / "crate_boundary_metrics.json")],
            589,
            generated_at,
        ),
        "ranked_api_simplification_candidates": emit_ranked(
            "ranked_api_simplification_candidates",
            "apis most likely to be merged or simplified",
            ranked_api_simplify(),
            [rel(STATUS / "internal_only_candidates_by_crate.json"), rel(STATUS / "cross_crate_api_usage.json")],
            590,
            generated_at,
        ),
        "ranked_docs_deletion_candidates": emit_ranked(
            "ranked_docs_deletion_candidates",
            "docs most likely to be deleted",
            ranked_docs_delete(),
            [rel(STATUS / "docs_duplication_report.json")],
            591,
            generated_at,
        ),
        "ranked_script_deletion_candidates": emit_ranked(
            "ranked_script_deletion_candidates",
            "scripts with highest deletion priority",
            ranked_scripts_delete(),
            [rel(STATUS / "script_only_behaviors.json")],
            592,
            generated_at,
        ),
        "ranked_weak_test_replacements": emit_ranked(
            "ranked_weak_test_replacements",
            "weak tests with highest replacement priority",
            ranked_weak_tests(),
            [rel(STATUS / "top_20_weakest_tests.json")],
            593,
            generated_at,
        ),
    }

    priority_plan = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_priority_rankings.py",
        "coverage_ids": [594, 596, 597, 598, 599, 600],
        "ranked_reports": {name: rel(STATUS / f"{name}.json") for name in outputs},
        "top_priorities": {
            name: payload.get("items", [])[:5] for name, payload in outputs.items()
        },
        "evidence_first_policy": {
            "manual_curated_priority_lists_allowed": False,
            "crate_merge_reassessment_source": [
                rel(STATUS / "crate_boundary_metrics.json"),
                rel(STATUS / "crate_boundary_report.json"),
            ],
            "public_api_trim_reassessment_source": [
                rel(STATUS / "internal_only_candidates_by_crate.json"),
                rel(STATUS / "cross_crate_api_usage.json"),
                rel(STATUS / "crate_boundary_metrics.json"),
            ],
            "required_artifacts": [
                rel(STATUS / "priority_plan.json"),
                rel(STATUS / "priority_plan.txt"),
            ],
        },
    }
    write_json(STATUS / "priority_plan.json", priority_plan)

    lines = [
        "Priority Rankings (Evidence-Ranked)",
        "",
    ]
    for name, payload in outputs.items():
        lines.append(f"{payload['title']}:")
        for item in payload.get("items", [])[:5]:
            label = item.get("command") or item.get("gap") or item.get("path") or item.get("crate") or item.get("symbol") or item.get("case")
            lines.append(f"- {label}")
        lines.append("")
    write_text(STATUS / "priority_plan.txt", "\n".join(lines))

    print("wrote artifacts/status/priority_plan.json")
    print("wrote artifacts/status/priority_plan.txt")
    for name in outputs:
        print(f"wrote artifacts/status/{name}.json")


if __name__ == "__main__":
    main()
