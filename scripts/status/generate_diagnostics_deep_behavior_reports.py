#!/usr/bin/env python3
"""Generate deep diagnostics behavior artifacts."""

from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILES = [
    ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "diagnostics_command_matrix.rs",
    ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "diagnostics_contract_consistency.rs",
    ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "diagnostics_deep_behavior_extra.rs",
]

REQUIRED_TESTS = {
    141: "doctor_findings_are_stable_and_do_not_reorder_nondeterministically",
    142: "doctor_findings_are_stable_and_do_not_reorder_nondeterministically",
    143: "doctor_json_and_text_are_stable_with_no_color_mode",
    144: "doctor_json_and_text_are_stable_with_no_color_mode",
    145: "inspect_and_doctor_agree_on_route_state_overlap_signals",
    146: "dev_cli_env_contracts_routes_and_registry_match_current_snapshots_and_resolution",
    147: "dev_cli_env_contracts_routes_and_registry_match_current_snapshots_and_resolution",
    148: "dev_cli_env_contracts_routes_and_registry_match_current_snapshots_and_resolution",
    149: "dev_cli_env_contracts_routes_and_registry_match_current_snapshots_and_resolution",
    150: "state_doctor_and_plugin_health_match_corruption_harness_findings",
    151: "state_doctor_and_plugin_health_match_corruption_harness_findings",
    152: "package_health_and_runtime_identity_are_consistent_with_active_binary_conditions",
    153: "package_health_and_runtime_identity_are_consistent_with_active_binary_conditions",
}


def run_cmd(args: list[str], env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    merged = os.environ.copy()
    if env:
        merged.update(env)
    return subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli", "--", *args],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
        env=merged,
    )


def run_json(args: list[str], env: dict[str, str] | None = None) -> dict[str, Any]:
    out = run_cmd(args + ["--format", "json", "--no-pretty"], env=env)
    if out.returncode != 0:
        return {}
    return json.loads(out.stdout or "{}")


def find_test(test_name: str, sources: dict[str, str]) -> str | None:
    needle = f"fn {test_name}("
    for path, source in sources.items():
        if needle in source:
            return path
    return None


def main() -> None:
    sources = {str(path.relative_to(ROOT)): path.read_text(encoding="utf-8") for path in TEST_FILES}

    doctor_a = run_json(["doctor"])
    doctor_b = run_json(["doctor"])
    state_doctor_a = run_json(["dev", "cli", "state-doctor"])
    state_doctor_b = run_json(["dev", "cli", "state-doctor"])
    inspect_payload = run_json(["inspect"])
    env_payload = run_json(["dev", "cli", "env"])
    contracts_payload = run_json(["dev", "cli", "contracts"])
    routes_payload = run_json(["dev", "cli", "routes"])
    registry_payload = run_json(["dev", "cli", "registry"])
    plugin_health_payload = run_json(["dev", "cli", "plugin-health"])
    package_health_payload = run_json(["dev", "cli", "package-health"])
    runtime_identity_payload = run_json(["dev", "cli", "runtime-identity"])

    coverage_rows = []
    for coverage_id, test_name in sorted(REQUIRED_TESTS.items()):
        evidence = find_test(test_name, sources)
        coverage_rows.append(
            {
                "coverage_id": coverage_id,
                "test_name": test_name,
                "status": "covered" if evidence else "missing",
                "evidence": evidence,
            }
        )
    missing_coverage_ids = [row for row in coverage_rows if row["status"] != "covered"]

    diagnostics_consistency = {
        "generator": "scripts/status/generate_diagnostics_deep_behavior_reports.py",
        "scope": "diagnostics consistency",
        "coverage_ids": [145, 146, 149, 150, 151, 152, 154],
        "status": "complete"
        if inspect_payload != {}
        and doctor_a != {}
        and env_payload != {}
        and routes_payload != {}
        and registry_payload != {}
        and package_health_payload != {}
        and runtime_identity_payload != {}
        else "partial",
        "sample": {
            "inspect_status": inspect_payload.get("status"),
            "doctor_status": doctor_a.get("status"),
            "env_keys": sorted(env_payload.keys()),
            "routes_keys": sorted(routes_payload.keys()),
            "registry_keys": sorted(registry_payload.keys()),
        },
    }

    doctor_determinism = {
        "generator": "scripts/status/generate_diagnostics_deep_behavior_reports.py",
        "scope": "doctor determinism",
        "coverage_ids": [141, 142, 143, 144, 155, 158],
        "status": "complete"
        if doctor_a == doctor_b
        and state_doctor_a == state_doctor_b
        and state_doctor_a.get("doctor", {}).get("issues") == state_doctor_b.get("doctor", {}).get("issues")
        else "partial",
        "byte_stable": doctor_a == doctor_b and state_doctor_a == state_doctor_b,
    }

    expected_contracts = json.loads(
        (ROOT / "crates" / "bijux-cli" / "tests" / "snapshots" / "ported" / "dev_cli_contracts.json").read_text(
            encoding="utf-8"
        )
    )
    expected_routes = json.loads(
        (ROOT / "crates" / "bijux-cli" / "tests" / "snapshots" / "ported" / "dev_cli_routes.json").read_text(
            encoding="utf-8"
        )
    )

    expected_route_set = {
        " ".join(segment for segment in row.get("segments", []))
        for row in expected_routes.get("routes", [])
        if isinstance(row, dict) and isinstance(row.get("segments"), list)
    }
    current_route_set = {
        " ".join(segment for segment in row.get("segments", []))
        for row in routes_payload.get("routes", [])
        if isinstance(row, dict) and isinstance(row.get("segments"), list)
    }

    diagnostics_schema_drift = {
        "generator": "scripts/status/generate_diagnostics_deep_behavior_reports.py",
        "scope": "diagnostics schema drift",
        "coverage_ids": [147, 148, 156],
        "status": "complete"
        if contracts_payload == expected_contracts and expected_route_set.issubset(current_route_set)
        else "partial",
        "contracts_matches_snapshot": contracts_payload == expected_contracts,
        "routes_matches_snapshot": expected_route_set.issubset(current_route_set),
    }

    diagnostics_source_of_truth = {
        "generator": "scripts/status/generate_diagnostics_deep_behavior_reports.py",
        "scope": "diagnostics source of truth",
        "coverage_ids": [146, 147, 148, 149, 157],
        "status": "complete"
        if env_payload != {} and contracts_payload != {} and routes_payload != {} and registry_payload != {}
        else "partial",
        "source_commands": [
            "dev cli env",
            "dev cli contracts",
            "dev cli routes",
            "dev cli registry",
        ],
    }

    findings_order = {
        "generator": "scripts/status/generate_diagnostics_deep_behavior_reports.py",
        "scope": "findings order",
        "coverage_ids": [141, 142, 150, 158],
        "status": "complete"
        if state_doctor_a.get("doctor", {}).get("issues") == state_doctor_b.get("doctor", {}).get("issues")
        else "partial",
        "stable_order": state_doctor_a.get("doctor", {}).get("issues")
        == state_doctor_b.get("doctor", {}).get("issues"),
    }

    diagnostics_contract = {
        "generator": "scripts/status/generate_diagnostics_deep_behavior_reports.py",
        "scope": "diagnostics contract",
        "coverage_ids": [143, 144, 145, 152, 153, 159],
        "status": "complete"
        if doctor_a != {}
        and plugin_health_payload != {}
        and package_health_payload != {}
        and runtime_identity_payload != {}
        else "partial",
        "contract_keys": {
            "doctor": sorted(doctor_a.keys()),
            "plugin_health": sorted(plugin_health_payload.keys()),
            "package_health": sorted(package_health_payload.keys()),
            "runtime_identity": sorted(runtime_identity_payload.keys()),
        },
    }

    drift_items: list[dict[str, Any]] = []
    for name, payload in [
        ("diagnostics_consistency_artifact.json", diagnostics_consistency),
        ("doctor_determinism_artifact.json", doctor_determinism),
        ("diagnostics_schema_drift_artifact.json", diagnostics_schema_drift),
        ("diagnostics_source_of_truth_artifact.json", diagnostics_source_of_truth),
        ("findings_order_artifact.json", findings_order),
        ("diagnostics_contract_artifact.json", diagnostics_contract),
    ]:
        if payload.get("status") != "complete":
            drift_items.append({"artifact": name, "reason": "status-not-complete"})
    if missing_coverage_ids:
        drift_items.append({"reason": "missing-coverage_id-coverage", "coverage_ids": [row["coverage_id"] for row in missing_coverage_ids]})

    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / "diagnostics_consistency_artifact.json").write_text(
        json.dumps(diagnostics_consistency, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    (STATUS / "doctor_determinism_artifact.json").write_text(
        json.dumps(doctor_determinism, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    (STATUS / "diagnostics_schema_drift_artifact.json").write_text(
        json.dumps(diagnostics_schema_drift, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    (STATUS / "diagnostics_source_of_truth_artifact.json").write_text(
        json.dumps(diagnostics_source_of_truth, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    (STATUS / "findings_order_artifact.json").write_text(
        json.dumps(findings_order, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    (STATUS / "diagnostics_contract_artifact.json").write_text(
        json.dumps(diagnostics_contract, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    (STATUS / "diagnostics_deep_behavior_drift_artifact.json").write_text(
        json.dumps(
            {
                "generator": "scripts/status/generate_diagnostics_deep_behavior_reports.py",
                "scope": "diagnostics deep behavior drift",
                "coverage_ids": [160],
                "status": "clean" if not drift_items else "drift-detected",
                "drift_count": len(drift_items),
                "drift_items": drift_items,
                "coverage_rows": coverage_rows,
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )

    print("wrote artifacts/status/diagnostics_consistency_artifact.json")
    print("wrote artifacts/status/doctor_determinism_artifact.json")
    print("wrote artifacts/status/diagnostics_schema_drift_artifact.json")
    print("wrote artifacts/status/diagnostics_source_of_truth_artifact.json")
    print("wrote artifacts/status/findings_order_artifact.json")
    print("wrote artifacts/status/diagnostics_contract_artifact.json")
    print("wrote artifacts/status/diagnostics_deep_behavior_drift_artifact.json")


if __name__ == "__main__":
    main()
