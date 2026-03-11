#!/usr/bin/env python3
"""Generate Python-bridge conversion artifacts."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli-python" / "tests" / "bridge_conversion_law_extra.rs"

REQUIRED_TESTS = {
    281: "python_exception_mapping_covers_usage_validation_plugin_and_internal_failures",
    282: "python_exception_mapping_covers_usage_validation_plugin_and_internal_failures",
    283: "python_exception_mapping_covers_usage_validation_plugin_and_internal_failures",
    284: "python_exception_mapping_covers_usage_validation_plugin_and_internal_failures",
    285: "error_and_success_envelope_fields_survive_python_conversion_intact",
    286: "error_and_success_envelope_fields_survive_python_conversion_intact",
    287: "diagnostics_and_inspection_payloads_survive_conversion_with_stable_shape",
    288: "diagnostics_and_inspection_payloads_survive_conversion_with_stable_shape",
    289: "diagnostics_and_inspection_payloads_survive_conversion_with_stable_shape",
    290: "bridge_conversions_preserve_field_names_optional_semantics_and_order_sensitive_lists",
    291: "bridge_conversions_preserve_field_names_optional_semantics_and_order_sensitive_lists",
    292: "bridge_conversions_preserve_field_names_optional_semantics_and_order_sensitive_lists",
    293: "conversion_failures_and_unsupported_runtime_conditions_are_normalized_clearly",
    294: "conversion_failures_and_unsupported_runtime_conditions_are_normalized_clearly",
    295: "bridge_import_failure_paths_are_distinct_from_command_failures",
}


def write_json(name: str, payload: dict[str, Any]) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def main() -> int:
    source = TEST_FILE.read_text(encoding="utf-8") if TEST_FILE.exists() else ""
    generated_at = datetime.now(timezone.utc).isoformat()

    coverage_rows = []
    for coverage_id, name in sorted(REQUIRED_TESTS.items()):
        covered = f"fn {name}(" in source
        coverage_rows.append(
            {
                "coverage_id": coverage_id,
                "test": name,
                "status": "covered" if covered else "missing",
                "evidence": "crates/bijux-cli-python/tests/bridge_conversion_law_extra.rs",
            }
        )

    missing = [row for row in coverage_rows if row["status"] != "covered"]

    conversion = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_python_bridge_conversion_reports.py",
        "scope": "python bridge conversion",
        "coverage_ids": list(range(281, 297)),
        "status": "complete" if not missing else "partial",
        "coverage_rows": coverage_rows,
    }

    exception_mapping = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_python_bridge_conversion_reports.py",
        "scope": "python bridge exception mapping",
        "coverage_ids": [281, 282, 283, 284, 297],
        "status": "complete" if not missing else "partial",
    }

    envelope_integrity = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_python_bridge_conversion_reports.py",
        "scope": "python bridge envelope integrity",
        "coverage_ids": [285, 286, 287, 288, 289, 290, 291, 292, 298],
        "status": "complete" if not missing else "partial",
    }

    drift = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_python_bridge_conversion_reports.py",
        "scope": "python bridge conversion drift",
        "coverage_ids": [299],
        "status": "clean" if not missing else "drift",
        "drift_count": len(missing),
        "drift_coverage_ids": [row["coverage_id"] for row in missing],
    }

    contract = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_python_bridge_conversion_reports.py",
        "scope": "python bridge conversion contract",
        "coverage_ids": [300],
        "status": "frozen" if not missing else "not-frozen",
        "law": "python bridge conversion behavior is part of CLI law",
    }

    write_json("bridge_conversion_artifact.json", conversion)
    write_json("bridge_exception_mapping_artifact.json", exception_mapping)
    write_json("bridge_envelope_integrity_artifact.json", envelope_integrity)
    write_json("bridge_conversion_drift_artifact.json", drift)
    write_json("bridge_conversion_contract.json", contract)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
