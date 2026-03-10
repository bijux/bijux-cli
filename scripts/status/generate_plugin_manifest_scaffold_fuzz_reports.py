#!/usr/bin/env python3
"""Generate plugin manifest/scaffold fuzz hardening artifacts for TODOs 61-80."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

MANIFEST_TARGETS = ROOT / "crates" / "bijux-cli-plugin" / "tests" / "plugin_manifest_fuzz_targets.rs"
MANIFEST_REGRESSION = ROOT / "crates" / "bijux-cli-plugin" / "tests" / "plugin_manifest_fuzz_regressions.rs"
SCAFFOLD_TARGETS = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "plugin_scaffold_fuzz_targets.rs"
SCAFFOLD_REGRESSION = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "plugin_scaffold_fuzz_regressions.rs"

MANIFEST_MIN_DIR = ROOT / "crates" / "bijux-cli-plugin" / "tests" / "fuzz" / "plugin_manifest_minimized_cases"
SCAFFOLD_MIN_DIR = ROOT / "crates" / "bijux-cli" / "tests" / "fuzz" / "plugin_scaffold_minimized_cases"

REQUIRED_TESTS = {
    61: (MANIFEST_TARGETS, "fuzz_plugin_manifest_parsing_is_stable"),
    62: (MANIFEST_TARGETS, "fuzz_plugin_manifest_validation_covers_required_and_optional_fields"),
    63: (MANIFEST_TARGETS, "fuzz_compatibility_range_parsing_is_enforced"),
    64: (MANIFEST_TARGETS, "fuzz_plugin_entrypoint_path_parsing_by_kind_is_enforced"),
    65: (MANIFEST_TARGETS, "fuzz_plugin_metadata_optional_fields_and_duplicate_aliases"),
    66: (SCAFFOLD_TARGETS, "fuzz_scaffold_option_parsing_and_template_expansion_inputs_are_stable"),
    67: (SCAFFOLD_TARGETS, "fuzz_scaffold_option_parsing_and_template_expansion_inputs_are_stable"),
    68: (SCAFFOLD_TARGETS, "fuzz_python_and_rust_scaffold_manifest_generation_are_correct"),
    69: (SCAFFOLD_TARGETS, "fuzz_python_and_rust_scaffold_manifest_generation_are_correct"),
    70: (SCAFFOLD_TARGETS, "fuzz_scaffold_path_sanitization_rejects_parent_segments"),
    71: (SCAFFOLD_TARGETS, "fuzz_plugin_inspect_payload_and_check_diagnostics_rendering_are_stable"),
    72: (SCAFFOLD_TARGETS, "fuzz_plugin_inspect_payload_and_check_diagnostics_rendering_are_stable"),
    73: (SCAFFOLD_TARGETS, "fuzz_plugin_reserved_name_error_rendering_is_stable"),
    76: (MANIFEST_REGRESSION, "minimized_plugin_manifest_cases_replay_deterministically"),
    77: (SCAFFOLD_REGRESSION, "minimized_scaffold_cases_replay_with_deterministic_exit_codes"),
    78: (MANIFEST_REGRESSION, "minimized_plugin_manifest_cases_replay_deterministically"),
    79: (SCAFFOLD_REGRESSION, "minimized_scaffold_cases_replay_with_deterministic_exit_codes"),
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
        MANIFEST_TARGETS: MANIFEST_TARGETS.read_text(encoding="utf-8") if MANIFEST_TARGETS.exists() else "",
        MANIFEST_REGRESSION: MANIFEST_REGRESSION.read_text(encoding="utf-8") if MANIFEST_REGRESSION.exists() else "",
        SCAFFOLD_TARGETS: SCAFFOLD_TARGETS.read_text(encoding="utf-8") if SCAFFOLD_TARGETS.exists() else "",
        SCAFFOLD_REGRESSION: SCAFFOLD_REGRESSION.read_text(encoding="utf-8") if SCAFFOLD_REGRESSION.exists() else "",
    }

    coverage = []
    for todo, (path, test_name) in sorted(REQUIRED_TESTS.items()):
        covered = f"fn {test_name}(" in texts[path]
        coverage.append(
            {
                "todo": todo,
                "test": test_name,
                "status": "covered" if covered else "missing",
                "evidence": str(path.relative_to(ROOT)).replace("\\", "/"),
            }
        )

    manifest_cases = sorted(str(p.relative_to(ROOT)).replace("\\", "/") for p in MANIFEST_MIN_DIR.glob("*.json"))
    scaffold_cases = sorted(str(p.relative_to(ROOT)).replace("\\", "/") for p in SCAFFOLD_MIN_DIR.glob("*.argv"))

    manifest_targets_run = run_test(["cargo", "test", "-p", "bijux-cli-plugin", "--test", "plugin_manifest_fuzz_targets"])
    manifest_reg_run = run_test(["cargo", "test", "-p", "bijux-cli-plugin", "--test", "plugin_manifest_fuzz_regressions"])
    scaffold_targets_run = run_test(
        ["cargo", "test", "-p", "bijux-cli", "--test", "bin_surface", "plugin_scaffold_fuzz_targets::"]
    )
    scaffold_reg_run = run_test(
        [
            "cargo",
            "test",
            "-p",
            "bijux-cli",
            "--test",
            "bin_surface",
            "plugin_scaffold_fuzz_regressions::",
        ]
    )

    missing = [row["todo"] for row in coverage if row["status"] != "covered"]

    manifest_triage = {
        "generated_at": now,
        "generator": "scripts/status/generate_plugin_manifest_scaffold_fuzz_reports.py",
        "scope": "plugin manifest fuzz crash triage",
        "tasks": [74],
        "status": "clean" if manifest_targets_run["ok"] and manifest_reg_run["ok"] else "needs-triage",
        "target_suite_ok": manifest_targets_run["ok"],
        "regression_suite_ok": manifest_reg_run["ok"],
        "minimized_case_count": len(manifest_cases),
    }

    scaffold_triage = {
        "generated_at": now,
        "generator": "scripts/status/generate_plugin_manifest_scaffold_fuzz_reports.py",
        "scope": "plugin scaffold fuzz crash triage",
        "tasks": [75],
        "status": "clean" if scaffold_targets_run["ok"] and scaffold_reg_run["ok"] else "needs-triage",
        "target_suite_ok": scaffold_targets_run["ok"],
        "regression_suite_ok": scaffold_reg_run["ok"],
        "minimized_case_count": len(scaffold_cases),
    }

    manifest_regression = {
        "generated_at": now,
        "generator": "scripts/status/generate_plugin_manifest_scaffold_fuzz_reports.py",
        "scope": "plugin manifest fuzz regressions",
        "tasks": [76, 78],
        "status": "clean" if manifest_reg_run["ok"] else "drift",
        "minimized_cases": manifest_cases,
    }

    scaffold_regression = {
        "generated_at": now,
        "generator": "scripts/status/generate_plugin_manifest_scaffold_fuzz_reports.py",
        "scope": "plugin scaffold fuzz regressions",
        "tasks": [77, 79],
        "status": "clean" if scaffold_reg_run["ok"] else "drift",
        "minimized_cases": scaffold_cases,
    }

    contract = {
        "generated_at": now,
        "generator": "scripts/status/generate_plugin_manifest_scaffold_fuzz_reports.py",
        "scope": "plugin manifest and scaffold fuzzing",
        "tasks": list(range(61, 81)),
        "status": "frozen"
        if not missing
        and manifest_targets_run["ok"]
        and manifest_reg_run["ok"]
        and scaffold_targets_run["ok"]
        and scaffold_reg_run["ok"]
        and len(manifest_cases) > 0
        and len(scaffold_cases) > 0
        else "partial",
        "todo_coverage": coverage,
        "missing_todos": missing,
        "manifest_minimized_case_count": len(manifest_cases),
        "scaffold_minimized_case_count": len(scaffold_cases),
        "policy": "plugin manifest and scaffold fuzzing remain maintenance-required hardening checks",
    }

    write_json("plugin_manifest_crash_triage_artifact.json", manifest_triage)
    write_json("plugin_scaffold_crash_triage_artifact.json", scaffold_triage)
    write_json("plugin_manifest_fuzz_regression_artifact.json", manifest_regression)
    write_json("plugin_scaffold_fuzz_regression_artifact.json", scaffold_regression)
    write_json("plugin_manifest_scaffold_fuzz_contract.json", contract)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
