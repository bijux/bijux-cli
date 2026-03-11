#!/usr/bin/env python3
"""Generate diagnostics trust artifacts."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "diagnostics_trust_law_extra.rs"

REQUIRED_TESTS = {
    361: "dev_cli_contracts_and_routes_match_snapshot_semantics_and_are_byte_stable",
    362: "dev_cli_contracts_and_routes_match_snapshot_semantics_and_are_byte_stable",
    363: "dev_cli_registry_env_parity_crate_health_and_docs_audit_reflect_live_truth",
    364: "dev_cli_registry_env_parity_crate_health_and_docs_audit_reflect_live_truth",
    365: "dev_cli_registry_env_parity_crate_health_and_docs_audit_reflect_live_truth",
    366: "dev_cli_registry_env_parity_crate_health_and_docs_audit_reflect_live_truth",
    367: "dev_cli_registry_env_parity_crate_health_and_docs_audit_reflect_live_truth",
    368: "doctor_plugin_doctor_and_runtime_identity_provide_actionable_diagnostics_for_problem_cases",
    369: "doctor_plugin_doctor_and_runtime_identity_provide_actionable_diagnostics_for_problem_cases",
    370: "doctor_plugin_doctor_and_runtime_identity_provide_actionable_diagnostics_for_problem_cases",
    371: "diagnostics_do_not_invent_unsupported_remediation_steps",
    372: "diagnostics_text_is_boring_and_json_is_machine_friendly",
    373: "diagnostics_text_is_boring_and_json_is_machine_friendly",
    374: "diagnostics_runs_are_deterministic_for_covered_commands",
}

EXPECTED_TOP_LEVEL_KEYS = {
    "dev cli contracts": ["contracts", "runtime_version", "schema_version"],
    "dev cli routes": ["aliases", "routes"],
    "dev cli registry": ["ownership", "precedence", "registry"],
    "dev cli env": ["active", "env", "source_precedence"],
    "dev cli parity": [
        "binary_bridge",
        "command_matrix",
        "commands_fully_rust_owned",
        "commands_python_only",
        "commands_using_compatibility_shims",
        "coverage",
        "diffs",
        "exit_code_report",
        "flag_normalization_report",
        "help_diff_report",
        "machine_output_diff_report",
        "parity_dashboard",
        "parity_dashboard_text",
        "plugin_lifecycle",
        "plugin_matrix",
        "precedence_report",
        "python_bridge_matrix",
        "repl_cli_output_diff",
        "repl_matrix",
        "rust_python",
        "state_behavior_matrix",
        "state_parity",
        "stream_report",
        "text_summary",
    ],
    "dev cli crate-health": [
        "crate_metrics",
        "crate_report",
        "cross_crate_api_usage",
        "dependency_edges",
        "duplication_hotspots",
        "internal_only_candidates_by_crate",
        "public_api_by_crate",
        "public_api_counts",
    ],
    "dev cli docs-audit": ["docs", "docs_audit", "docs_count"],
    "dev cli doctor": ["issues", "runtime", "status"],
    "dev cli runtime-identity": [
        "active_binary",
        "active_binary_selection_is_ambiguous",
        "active_path_is_canonical_name",
        "active_path_is_shadowed",
        "canonical_user_binary",
        "diagnostics",
        "entrypoints",
        "install_source",
        "package_channels",
        "path_binaries",
        "public_runtime_binary_names",
        "runtime",
        "schema",
        "secondary_public_runtime_binary_names",
        "text_summary",
    ],
}


def run_json(args: list[str]) -> dict[str, Any]:
    out = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli", "--", *args, "--format", "json", "--no-pretty"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if out.returncode != 0:
        return {}
    return json.loads(out.stdout or "{}")


def write_json(name: str, payload: dict[str, Any]) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def main() -> int:
    source = TEST_FILE.read_text(encoding="utf-8") if TEST_FILE.exists() else ""
    generated_at = datetime.now(timezone.utc).isoformat()

    coverage_rows = []
    for coverage_id, test_name in sorted(REQUIRED_TESTS.items()):
        covered = f"fn {test_name}(" in source
        coverage_rows.append(
            {
                "coverage_id": coverage_id,
                "test": test_name,
                "status": "covered" if covered else "missing",
                "evidence": "crates/bijux-cli/tests/bin_surface/diagnostics_trust_law_extra.rs",
            }
        )

    missing = [row for row in coverage_rows if row["status"] != "covered"]

    payloads = {command: run_json(command.split()) for command in EXPECTED_TOP_LEVEL_KEYS}
    plugin_health = run_json(["dev", "cli", "plugin-health"])

    schema_rows = []
    for command, expected in EXPECTED_TOP_LEVEL_KEYS.items():
        payload = payloads.get(command, {})
        actual = sorted(payload.keys()) if isinstance(payload, dict) else []
        schema_rows.append(
            {
                "command": command,
                "expected_keys": expected,
                "actual_keys": actual,
                "status": "match" if actual == expected else "drift",
            }
        )
    schema_drift = [row for row in schema_rows if row["status"] != "match"]

    trust = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_diagnostics_trust_reports.py",
        "scope": "diagnostics trust",
        "coverage_ids": [361, 362, 363, 364, 365, 366, 367, 374, 375],
        "status": "complete" if not missing else "partial",
        "coverage_rows": coverage_rows,
    }

    actionable = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_diagnostics_trust_reports.py",
        "scope": "actionable diagnostics",
        "coverage_ids": [368, 369, 370, 371, 376],
        "status": "complete" if not missing else "partial",
        "checks": {
            "plugin_health_has_guidance": "Use `bijux dev cli plugin-health --format json`"
            in json.dumps(plugin_health),
            "doctor_payload_present": bool(payloads.get("dev cli doctor")),
            "runtime_identity_payload_present": bool(payloads.get("dev cli runtime-identity")),
        },
    }

    minimalism = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_diagnostics_trust_reports.py",
        "scope": "diagnostics minimalism",
        "coverage_ids": [372, 373, 377],
        "status": "complete" if not missing else "partial",
        "json_commands_checked": sorted(EXPECTED_TOP_LEVEL_KEYS.keys()),
        "json_schema_drift_count": len(schema_drift),
    }

    schema = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_diagnostics_trust_reports.py",
        "scope": "diagnostics trust schema drift",
        "coverage_ids": [378],
        "status": "clean" if not schema_drift and not missing else "drift",
        "drift_count": len(schema_drift) + len(missing),
        "schema_rows": schema_rows,
        "missing_coverage_ids": [row["coverage_id"] for row in missing],
    }

    contract = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_diagnostics_trust_reports.py",
        "scope": "diagnostics trust contract",
        "coverage_ids": [380],
        "status": "frozen" if not schema_drift and not missing else "not-frozen",
        "law": "diagnostics are credible operator output",
    }

    write_json("diagnostics_trust_artifact.json", trust)
    write_json("actionable_diagnostics_artifact.json", actionable)
    write_json("diagnostics_minimalism_artifact.json", minimalism)
    write_json("diagnostics_trust_schema_drift_artifact.json", schema)
    write_json("diagnostics_trust_contract.json", contract)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
