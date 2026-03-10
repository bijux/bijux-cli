#!/usr/bin/env python3
"""Generate cross-surface equivalence and drift artifacts from required coverage tests."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TEST_FILE = ROOT / "crates" / "bijux-cli-core" / "tests" / "bin_surface" / "cross_surface_equivalence.rs"
STATUS_DIR = ROOT / "artifacts" / "status"

REQUIRED_TESTS: list[tuple[int, str, str]] = [
    (161, "binary_vs_direct_core_version_result_matches", "binary vs direct-core version"),
    (162, "binary_vs_direct_core_status_result_matches", "binary vs direct-core status"),
    (163, "binary_vs_direct_core_doctor_result_matches", "binary vs direct-core doctor"),
    (164, "binary_vs_direct_core_plugins_list_result_matches", "binary vs direct-core plugins list"),
    (165, "binary_vs_direct_core_config_get_result_matches", "binary vs direct-core config get"),
    (166, "binary_vs_python_bridge_version_result_matches", "binary vs python bridge version"),
    (167, "binary_vs_python_bridge_status_result_matches", "binary vs python bridge status"),
    (168, "binary_vs_python_bridge_doctor_result_matches", "binary vs python bridge doctor"),
    (169, "binary_vs_python_bridge_plugins_list_result_matches", "binary vs python bridge plugins list"),
    (170, "binary_vs_python_bridge_config_get_result_matches", "binary vs python bridge config get"),
    (171, "binary_vs_repl_status_result_matches_where_sensible", "binary vs repl result where sensible"),
    (172, "binary_vs_repl_unknown_command_exit_semantics_match_where_sensible", "binary vs repl exit semantics where sensible"),
    (173, "binary_vs_python_bridge_namespace_rejection_behavior_matches", "binary vs python bridge namespace rejection"),
    (174, "binary_vs_python_bridge_error_envelope_shape_matches", "binary vs python bridge error envelope shape"),
    (175, "binary_vs_python_bridge_stdout_stderr_discipline_matches", "binary vs python bridge stdout/stderr discipline"),
    (176, "route_registry_snapshots_match_across_binary_core_and_bridge", "route registry snapshots across surfaces"),
]


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    text = TEST_FILE.read_text(encoding="utf-8")

    covered: list[dict[str, object]] = []
    missing: list[dict[str, object]] = []
    for todo, fn_name, law in REQUIRED_TESTS:
        row = {
            "todo": todo,
            "law": law,
            "test": f"crates/bijux-cli-core/tests/bin_surface/cross_surface_equivalence.rs::{fn_name}",
        }
        if f"fn {fn_name}(" in text:
            covered.append(row)
        else:
            missing.append(row)

    equivalence = {
        "generator": "scripts/status/generate_cross_surface_reports.py",
        "scope": "cross-surface equivalence",
        "rule": "binary, direct-core, python bridge, and repl must agree for covered commands",
        "verification_command": "cargo test -q -p bijux-cli-core --test bin_surface cross_surface_equivalence::",
        "covered": covered,
        "missing": missing,
        "summary": {
            "required": len(REQUIRED_TESTS),
            "covered": len(covered),
            "missing": len(missing),
        },
    }

    drift = {
        "generator": "scripts/status/generate_cross_surface_reports.py",
        "scope": "cross-surface drift",
        "status": "drift-detected" if missing else "clean",
        "drift_count": len(missing),
        "drift_items": missing,
        "gate": "scripts/parity/check_cross_surface_drift_gate.py --enforce",
    }

    contract = {
        "generator": "scripts/status/generate_cross_surface_reports.py",
        "contract": "Cross-surface equivalence",
        "law": "One command law across binary, core, python bridge, and repl for covered commands.",
        "freeze_rule": "New covered command paths must add cross-surface equivalence tests before merge.",
        "evidence": [
            "crates/bijux-cli-core/tests/bin_surface/cross_surface_equivalence.rs",
            "artifacts/status/cross_surface_equivalence_report.json",
            "artifacts/status/cross_surface_drift_report.json",
        ],
    }

    write_json(STATUS_DIR / "cross_surface_equivalence_report.json", equivalence)
    write_json(STATUS_DIR / "cross_surface_drift_report.json", drift)
    write_json(STATUS_DIR / "cross_surface_duality_contract.json", contract)

    print("wrote artifacts/status/cross_surface_equivalence_report.json")
    print("wrote artifacts/status/cross_surface_drift_report.json")
    print("wrote artifacts/status/cross_surface_duality_contract.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
