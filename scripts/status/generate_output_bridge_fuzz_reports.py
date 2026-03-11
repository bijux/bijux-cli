#!/usr/bin/env python3
"""Generate output/envelope and bridge-conversion fuzz hardening artifacts."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

OUTPUT_TARGETS = ROOT / "crates" / "bijux-cli-output" / "tests" / "output_envelope_fuzz_targets.rs"
OUTPUT_REGRESSION = ROOT / "crates" / "bijux-cli-output" / "tests" / "output_envelope_fuzz_regressions.rs"
BRIDGE_TARGETS = ROOT / "crates" / "bijux-cli-python" / "tests" / "bridge_conversion_fuzz_targets.rs"
BRIDGE_REGRESSION = ROOT / "crates" / "bijux-cli-python" / "tests" / "bridge_conversion_fuzz_regressions.rs"

OUTPUT_MIN_DIR = ROOT / "crates" / "bijux-cli-output" / "tests" / "fuzz" / "output_minimized_cases"
BRIDGE_MIN_DIR = ROOT / "crates" / "bijux-cli-python" / "tests" / "fuzz" / "bridge_conversion_minimized_cases"

REQUIRED_TESTS = {
    81: (OUTPUT_TARGETS, "fuzz_success_envelope_serialization_is_stable"),
    82: (OUTPUT_TARGETS, "fuzz_error_envelope_serialization_is_stable"),
    83: (OUTPUT_TARGETS, "fuzz_json_yaml_text_emitters_render_without_corruption"),
    84: (OUTPUT_TARGETS, "fuzz_json_yaml_text_emitters_render_without_corruption"),
    85: (OUTPUT_TARGETS, "fuzz_json_yaml_text_emitters_render_without_corruption"),
    86: (OUTPUT_TARGETS, "fuzz_nested_diagnostics_multiline_unicode_empty_and_large_payload_rendering"),
    87: (OUTPUT_TARGETS, "fuzz_nested_diagnostics_multiline_unicode_empty_and_large_payload_rendering"),
    88: (OUTPUT_TARGETS, "fuzz_nested_diagnostics_multiline_unicode_empty_and_large_payload_rendering"),
    89: (OUTPUT_TARGETS, "fuzz_nested_diagnostics_multiline_unicode_empty_and_large_payload_rendering"),
    90: (OUTPUT_TARGETS, "fuzz_malformed_envelope_deserialization_is_rejected"),
    91: (BRIDGE_TARGETS, "fuzz_bridge_conversion_of_success_envelopes_is_stable"),
    92: (BRIDGE_TARGETS, "fuzz_bridge_conversion_of_error_envelopes_is_stable"),
    93: (OUTPUT_TARGETS, "fuzz_route_inspection_json_rendering_is_deterministic"),
    96: (OUTPUT_REGRESSION, "minimized_output_cases_replay_with_stable_parse_behavior"),
    97: (BRIDGE_REGRESSION, "minimized_bridge_conversion_cases_replay_deterministically"),
    98: (OUTPUT_REGRESSION, "minimized_output_cases_replay_with_stable_parse_behavior"),
    99: (OUTPUT_TARGETS, "fuzz_output_field_order_invariant_for_machine_rendering"),
}


def write_json(name: str, payload: dict[str, Any]) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def run_test(args: list[str]) -> dict[str, Any]:
    result = subprocess.run(args, cwd=ROOT, check=False, capture_output=True, text=True)
    return {
        "command": args,
        "exit_code": result.returncode,
        "ok": result.returncode == 0,
        "stdout": result.stdout[-4000:],
        "stderr": result.stderr[-4000:],
    }


def main() -> int:
    now = datetime.now(timezone.utc).isoformat()
    texts = {
        OUTPUT_TARGETS: OUTPUT_TARGETS.read_text(encoding="utf-8") if OUTPUT_TARGETS.exists() else "",
        OUTPUT_REGRESSION: OUTPUT_REGRESSION.read_text(encoding="utf-8") if OUTPUT_REGRESSION.exists() else "",
        BRIDGE_TARGETS: BRIDGE_TARGETS.read_text(encoding="utf-8") if BRIDGE_TARGETS.exists() else "",
        BRIDGE_REGRESSION: BRIDGE_REGRESSION.read_text(encoding="utf-8") if BRIDGE_REGRESSION.exists() else "",
    }

    coverage = []
    for coverage_id, (path, test_name) in sorted(REQUIRED_TESTS.items()):
        covered = f"fn {test_name}(" in texts[path]
        coverage.append(
            {
                "coverage_id": coverage_id,
                "test": test_name,
                "status": "covered" if covered else "missing",
                "evidence": str(path.relative_to(ROOT)).replace("\\", "/"),
            }
        )

    output_cases = sorted(str(p.relative_to(ROOT)).replace("\\", "/") for p in OUTPUT_MIN_DIR.glob("*.json"))
    bridge_cases = sorted(str(p.relative_to(ROOT)).replace("\\", "/") for p in BRIDGE_MIN_DIR.glob("*.json"))

    output_targets_run = run_test(["cargo", "test", "-p", "bijux-cli-output", "--test", "output_envelope_fuzz_targets"])
    output_reg_run = run_test(["cargo", "test", "-p", "bijux-cli-output", "--test", "output_envelope_fuzz_regressions"])
    bridge_targets_run = run_test(["cargo", "test", "-p", "bijux-cli-python", "--test", "bridge_conversion_fuzz_targets"])
    bridge_reg_run = run_test(["cargo", "test", "-p", "bijux-cli-python", "--test", "bridge_conversion_fuzz_regressions"])

    missing = [row["coverage_id"] for row in coverage if row["status"] != "covered"]

    output_triage = {
        "generated_at": now,
        "generator": "scripts/status/generate_output_bridge_fuzz_reports.py",
        "scope": "output crash triage",
        "coverage_ids": [94],
        "status": "clean" if output_targets_run["ok"] and output_reg_run["ok"] else "needs-triage",
        "target_suite_ok": output_targets_run["ok"],
        "regression_suite_ok": output_reg_run["ok"],
        "minimized_case_count": len(output_cases),
    }

    bridge_triage = {
        "generated_at": now,
        "generator": "scripts/status/generate_output_bridge_fuzz_reports.py",
        "scope": "bridge conversion crash triage",
        "coverage_ids": [95],
        "status": "clean" if bridge_targets_run["ok"] and bridge_reg_run["ok"] else "needs-triage",
        "target_suite_ok": bridge_targets_run["ok"],
        "regression_suite_ok": bridge_reg_run["ok"],
        "minimized_case_count": len(bridge_cases),
    }

    output_regression = {
        "generated_at": now,
        "generator": "scripts/status/generate_output_bridge_fuzz_reports.py",
        "scope": "output fuzz regressions",
        "coverage_ids": [96, 98],
        "status": "clean" if output_reg_run["ok"] else "drift",
        "minimized_cases": output_cases,
    }

    bridge_regression = {
        "generated_at": now,
        "generator": "scripts/status/generate_output_bridge_fuzz_reports.py",
        "scope": "bridge conversion fuzz regressions",
        "coverage_ids": [97],
        "status": "clean" if bridge_reg_run["ok"] else "drift",
        "minimized_cases": bridge_cases,
    }

    contract = {
        "generated_at": now,
        "generator": "scripts/status/generate_output_bridge_fuzz_reports.py",
        "scope": "output and envelope fuzz hardening",
        "coverage_ids": list(range(81, 101)),
        "status": "frozen"
        if not missing
        and output_targets_run["ok"]
        and output_reg_run["ok"]
        and bridge_targets_run["ok"]
        and bridge_reg_run["ok"]
        and len(output_cases) > 0
        and len(bridge_cases) > 0
        else "partial",
        "coverage_rows": coverage,
        "missing_coverage_ids": missing,
        "output_minimized_case_count": len(output_cases),
        "bridge_minimized_case_count": len(bridge_cases),
        "policy": "envelope/output fuzzing is contract hardening and remains permanently gated",
    }

    write_json("output_crash_triage_artifact.json", output_triage)
    write_json("bridge_conversion_crash_triage_artifact.json", bridge_triage)
    write_json("output_fuzz_regression_artifact.json", output_regression)
    write_json("bridge_conversion_fuzz_regression_artifact.json", bridge_regression)
    write_json("output_envelope_fuzz_contract.json", contract)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
