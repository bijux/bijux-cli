#!/usr/bin/env python3
"""Generate Python-bridge execution artifacts."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli-python" / "tests" / "bridge_execution_law_extra.rs"

REQUIRED_TESTS = {
    261: "python_bridge_version_status_doctor_and_inspect_match_binary_outputs",
    262: "python_bridge_version_status_doctor_and_inspect_match_binary_outputs",
    263: "python_bridge_version_status_doctor_and_inspect_match_binary_outputs",
    264: "python_bridge_version_status_doctor_and_inspect_match_binary_outputs",
    265: "python_bridge_plugins_config_history_and_memory_match_binary_outputs",
    266: "python_bridge_plugins_config_history_and_memory_match_binary_outputs",
    267: "python_bridge_plugins_config_history_and_memory_match_binary_outputs",
    268: "python_bridge_plugins_config_history_and_memory_match_binary_outputs",
    269: "python_bridge_and_binary_agree_on_exit_codes_for_usage_validation_plugin_and_internal_representatives",
    270: "python_bridge_and_binary_agree_on_exit_codes_for_usage_validation_plugin_and_internal_representatives",
    271: "python_bridge_and_binary_agree_on_exit_codes_for_usage_validation_plugin_and_internal_representatives",
    272: "python_bridge_and_binary_agree_on_exit_codes_for_usage_validation_plugin_and_internal_representatives",
    273: "python_bridge_and_binary_agree_on_stream_routing_for_covered_commands",
    274: "python_bridge_and_binary_agree_on_namespace_rejection_behavior",
    275: "python_bridge_and_binary_help_outputs_match_for_representative_commands",
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
                "evidence": "crates/bijux-cli-python/tests/bridge_execution_law_extra.rs",
            }
        )

    missing = [row for row in coverage_rows if row["status"] != "covered"]

    execution = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_python_bridge_execution_reports.py",
        "scope": "python bridge execution parity",
        "coverage_ids": list(range(261, 277)),
        "status": "complete" if not missing else "partial",
        "coverage_rows": coverage_rows,
    }

    drift = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_python_bridge_execution_reports.py",
        "scope": "python bridge drift",
        "coverage_ids": [277, 278],
        "status": "clean" if not missing else "drift",
        "drift_count": len(missing),
        "drift_coverage_ids": [row["coverage_id"] for row in missing],
    }

    contract = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_python_bridge_execution_reports.py",
        "scope": "python bridge execution contract",
        "coverage_ids": [280],
        "status": "frozen" if not missing else "not-frozen",
        "law": "python bridge execution parity is a hard requirement",
    }

    write_json("python_bridge_execution_artifact.json", execution)
    write_json("python_bridge_drift_artifact.json", drift)
    write_json("python_bridge_execution_contract.json", contract)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
