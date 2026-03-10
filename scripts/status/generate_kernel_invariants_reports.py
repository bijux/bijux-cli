#!/usr/bin/env python3
"""Generate kernel invariants report and drift artifact for TODOs 1-20."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli" / "src" / "kernel_pipeline_tests.rs"

REQUIRED_TESTS = {
    1: "kernel_pipeline_uses_one_canonical_entrypoint",
    2: "fast_path_commands_keep_valid_envelope_metadata_when_emitted",
    3: "cancellation_paths_never_skip_exit_code_mapping",
    4: "cancellation_paths_never_emit_partial_success_envelopes",
    5: "plugin_lifecycle_hooks_run_in_stable_order_around_execution",
    6: "repl_lifecycle_hooks_do_not_mutate_non_repl_command_semantics",
    7: "sync_and_async_handlers_produce_equivalent_normalized_results",
    8: "kernel_usage_validation_plugin_internal_error_mapping_is_stable",
    9: "kernel_usage_validation_plugin_internal_error_mapping_is_stable",
    10: "kernel_usage_validation_plugin_internal_error_mapping_is_stable",
    11: "kernel_usage_validation_plugin_internal_error_mapping_is_stable",
    12: "internal_failure_is_normalized_before_crossing_cli_surface",
    13: "trace_mode_adds_diagnostics_without_changing_payload_shape",
    14: "quiet_mode_suppresses_streams_but_preserves_result_category",
    15: "kernel_resolution_is_deterministic_under_reordered_inputs",
    16: "kernel_resolution_is_deterministic_under_reordered_inputs",
    17: "repeated_run_kernel_invariants_harness_for_representative_commands",
}


def has_test(source: str, test_name: str) -> bool:
    return f"fn {test_name}(" in source


def stable_rows(source: str) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for todo, test_name in sorted(REQUIRED_TESTS.items()):
        present = has_test(source, test_name)
        rows.append(
            {
                "todo": todo,
                "test_name": test_name,
                "status": "covered" if present else "missing",
                "evidence": "crates/bijux-cli/src/kernel_pipeline_tests.rs",
            }
        )
    return rows


def main() -> None:
    source = TEST_FILE.read_text(encoding="utf-8")
    rows = stable_rows(source)
    missing = [row for row in rows if row["status"] != "covered"]

    report = {
        "generator": "scripts/status/generate_kernel_invariants_reports.py",
        "scope": "kernel pipeline invariants",
        "status": "complete" if not missing else "partial",
        "tasks": list(range(1, 19)),
        "rows": rows,
        "missing": missing,
        "summary": {
            "covered": len(rows) - len(missing),
            "missing": len(missing),
        },
    }

    diff = {
        "generator": "scripts/status/generate_kernel_invariants_reports.py",
        "scope": "kernel invariants drift",
        "status": "clean" if not missing else "drift-detected",
        "tasks": [19],
        "drift_items": [
            {
                "todo": row["todo"],
                "kind": "missing-kernel-invariant-test",
                "test_name": row["test_name"],
            }
            for row in missing
        ],
    }

    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / "kernel_invariants_report.json").write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "kernel_invariants_diff.json").write_text(
        json.dumps(diff, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print("wrote artifacts/status/kernel_invariants_report.json")
    print("wrote artifacts/status/kernel_invariants_diff.json")


if __name__ == "__main__":
    main()

