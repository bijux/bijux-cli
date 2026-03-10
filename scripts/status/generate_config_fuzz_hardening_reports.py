#!/usr/bin/env python3
"""Generate config fuzz hardening artifacts for TODOs 41-60."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

TARGETS = ROOT / "crates" / "bijux-cli-core" / "tests" / "bin_surface" / "config_fuzz_targets.rs"
REGRESSION = ROOT / "crates" / "bijux-cli-core" / "tests" / "bin_surface" / "config_fuzz_regressions.rs"
MINIMIZED = ROOT / "crates" / "bijux-cli-core" / "tests" / "fuzz" / "config_minimized_cases"

REQUIRED_TESTS = {
    41: "fuzz_dotenv_style_config_parsing_is_stable",
    42: "fuzz_malformed_config_lines_fail_consistently",
    43: "fuzz_duplicate_key_handling_keeps_last_value",
    44: "fuzz_weird_whitespace_handling_is_stable",
    45: "fuzz_quote_parsing_and_escape_parsing_are_stable",
    46: "fuzz_quote_parsing_and_escape_parsing_are_stable",
    47: "fuzz_null_byte_and_control_characters_are_handled_deterministically",
    48: "fuzz_mixed_valid_invalid_content_never_silently_succeeds",
    49: "fuzz_config_export_serialization_roundtrips_for_random_inputs",
    50: "fuzz_config_load_import_parsing_is_deterministic",
    51: "fuzz_roundtrip_parse_serialize_parse_is_semantically_stable",
    52: "fuzz_key_normalization_and_value_validation_are_stable",
    53: "fuzz_key_normalization_and_value_validation_are_stable",
    57: "minimized_config_cases_replay_with_stable_exit_behavior",
    58: "fuzz_roundtrip_parse_serialize_parse_is_semantically_stable",
    59: "fuzz_no_silent_key_loss_invariant_holds_under_repeated_exports",
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
    targets_text = TARGETS.read_text(encoding="utf-8") if TARGETS.exists() else ""
    regression_text = REGRESSION.read_text(encoding="utf-8") if REGRESSION.exists() else ""

    coverage = []
    for todo, test_name in sorted(REQUIRED_TESTS.items()):
        source = regression_text if "minimized_config_cases" in test_name else targets_text
        covered = f"fn {test_name}(" in source
        coverage.append(
            {
                "todo": todo,
                "test": test_name,
                "status": "covered" if covered else "missing",
                "evidence": str((REGRESSION if "minimized_config_cases" in test_name else TARGETS).relative_to(ROOT)),
            }
        )

    minimized_cases = sorted(str(p.relative_to(ROOT)) for p in MINIMIZED.glob("*.env"))

    replay = run_test(
        ["cargo", "test", "-p", "bijux-cli-core", "--test", "bin_surface", "config_fuzz_regressions::"]
    )
    targets = run_test(["cargo", "test", "-p", "bijux-cli-core", "--test", "bin_surface", "config_fuzz_targets::"])

    missing = [row["todo"] for row in coverage if row["status"] != "covered"]

    parser_triage = {
        "generated_at": now,
        "generator": "scripts/status/generate_config_fuzz_hardening_reports.py",
        "scope": "config parser fuzz triage",
        "tasks": [54],
        "status": "clean" if targets["ok"] and replay["ok"] else "needs-triage",
        "regression_replay_ok": replay["ok"],
        "target_suite_ok": targets["ok"],
    }

    serializer_triage = {
        "generated_at": now,
        "generator": "scripts/status/generate_config_fuzz_hardening_reports.py",
        "scope": "config serializer fuzz triage",
        "tasks": [55],
        "status": "clean" if targets["ok"] else "needs-triage",
        "target_suite_ok": targets["ok"],
    }

    regression = {
        "generated_at": now,
        "generator": "scripts/status/generate_config_fuzz_hardening_reports.py",
        "scope": "config fuzz regression",
        "tasks": [56, 57],
        "status": "clean" if replay["ok"] else "drift",
        "minimized_case_count": len(minimized_cases),
        "regression_replay_ok": replay["ok"],
    }

    contract = {
        "generated_at": now,
        "generator": "scripts/status/generate_config_fuzz_hardening_reports.py",
        "scope": "config fuzz hardening",
        "tasks": list(range(41, 61)),
        "status": "frozen" if not missing and replay["ok"] and targets["ok"] and len(minimized_cases) > 0 else "partial",
        "todo_coverage": coverage,
        "missing_todos": missing,
        "minimized_cases": minimized_cases,
        "policy": "config fuzzing is required before release claims",
    }

    write_json("config_parser_crash_triage_artifact.json", parser_triage)
    write_json("config_serializer_crash_triage_artifact.json", serializer_triage)
    write_json("config_fuzz_regression_artifact.json", regression)
    write_json("config_fuzz_contract.json", contract)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
