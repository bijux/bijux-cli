#!/usr/bin/env python3
"""Generate parser fuzz hardening artifacts for TODOs 1-20."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

ROUTING_TEST_FILE = ROOT / "crates" / "bijux-cli-routing" / "tests" / "parser_fuzz_targets.rs"
BIN_TEST_FILE = ROOT / "crates" / "bijux-cli-core" / "tests" / "bin_surface" / "parser_invalid_utf8_argv.rs"
REGRESSION_FILE = ROOT / "crates" / "bijux-cli-routing" / "tests" / "parser_fuzz_regressions.rs"
CORPUS_DIR = ROOT / "crates" / "bijux-cli-routing" / "tests" / "fuzz" / "parser_interesting_inputs"
MINIMIZED_DIR = ROOT / "crates" / "bijux-cli-routing" / "tests" / "fuzz" / "parser_minimized_cases"

REQUIRED_TESTS = {
    1: (ROUTING_TEST_FILE, "fuzz_root_argv_parsing_does_not_panic"),
    2: (ROUTING_TEST_FILE, "fuzz_cli_argv_parsing_does_not_panic"),
    3: (ROUTING_TEST_FILE, "fuzz_dev_cli_argv_parsing_does_not_panic"),
    4: (ROUTING_TEST_FILE, "fuzz_plugin_command_argv_parsing_does_not_panic"),
    5: (ROUTING_TEST_FILE, "fuzz_config_command_argv_parsing_does_not_panic"),
    6: (ROUTING_TEST_FILE, "fuzz_diagnostics_command_argv_parsing_does_not_panic"),
    7: (ROUTING_TEST_FILE, "fuzz_mixed_global_local_flag_ordering_is_deterministic"),
    8: (ROUTING_TEST_FILE, "fuzz_repeated_conflicting_flags_stays_safe_and_deterministic"),
    9: (BIN_TEST_FILE, "malformed_utf8_argv_is_rejected_without_panic"),
    10: (ROUTING_TEST_FILE, "fuzz_huge_tokens_and_values_does_not_panic"),
    11: (ROUTING_TEST_FILE, "fuzz_typo_suggestion_paths_are_stable"),
    12: (ROUTING_TEST_FILE, "fuzz_help_path_parsing_and_alias_resolution_is_safe"),
    13: (ROUTING_TEST_FILE, "fuzz_help_path_parsing_and_alias_resolution_is_safe"),
    14: (ROUTING_TEST_FILE, "fuzz_namespace_normalization_and_reserved_rejection_stays_safe"),
    15: (ROUTING_TEST_FILE, "fuzz_reserved_name_rejection_and_normalization_are_deterministic"),
    17: (REGRESSION_FILE, "interesting_corpus_cases_do_not_crash_or_corrupt_route_resolution"),
    18: (REGRESSION_FILE, "minimized_parser_cases_do_not_crash_and_are_deterministic"),
    19: (REGRESSION_FILE, "minimized_parser_cases_do_not_crash_and_are_deterministic"),
}


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.exists() else ""


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

    sources = {
        ROUTING_TEST_FILE: read_text(ROUTING_TEST_FILE),
        BIN_TEST_FILE: read_text(BIN_TEST_FILE),
        REGRESSION_FILE: read_text(REGRESSION_FILE),
    }

    coverage_rows = []
    for todo, (path, test_name) in sorted(REQUIRED_TESTS.items()):
        covered = f"fn {test_name}(" in sources[path]
        coverage_rows.append(
            {
                "todo": todo,
                "test": test_name,
                "status": "covered" if covered else "missing",
                "evidence": str(path.relative_to(ROOT)).replace("\\", "/"),
            }
        )

    corpus_files = sorted(
        str(path.relative_to(ROOT)).replace("\\", "/")
        for path in CORPUS_DIR.glob("*.txt")
        if path.is_file()
    )
    minimized_files = sorted(
        str(path.relative_to(ROOT)).replace("\\", "/")
        for path in MINIMIZED_DIR.glob("*.argv")
        if path.is_file()
    )

    regression_run = run_test(["cargo", "test", "-p", "bijux-cli-routing", "--test", "parser_fuzz_regressions"])

    missing_todos = [row["todo"] for row in coverage_rows if row["status"] != "covered"]

    triage = {
        "generated_at": now,
        "generator": "scripts/status/generate_parser_fuzz_hardening_reports.py",
        "scope": "parser crash triage",
        "tasks": [16],
        "status": "clean" if regression_run["ok"] else "needs-triage",
        "known_crash_case_count": len(minimized_files),
        "regression_test_ok": regression_run["ok"],
        "regression_test_command": regression_run["command"],
        "triage_notes": [
            "minimized cases are retained and replayed on every gate run",
            "new parser crashes must be added as minimized reproducer cases",
        ],
    }

    regression = {
        "generated_at": now,
        "generator": "scripts/status/generate_parser_fuzz_hardening_reports.py",
        "scope": "parser fuzz regressions",
        "tasks": [19, 20],
        "status": "clean" if regression_run["ok"] and not missing_todos else "drift",
        "missing_todos": missing_todos,
        "corpus_file_count": len(corpus_files),
        "minimized_case_count": len(minimized_files),
        "regression_test_ok": regression_run["ok"],
    }

    campaign = {
        "generated_at": now,
        "generator": "scripts/status/generate_parser_fuzz_hardening_reports.py",
        "scope": "parser fuzzing",
        "tasks": list(range(1, 21)),
        "status": "complete" if not missing_todos and len(corpus_files) > 0 and len(minimized_files) > 0 else "partial",
        "todo_coverage": coverage_rows,
        "corpus_directory": str(CORPUS_DIR.relative_to(ROOT)).replace("\\", "/"),
        "corpus_files": corpus_files,
        "minimized_directory": str(MINIMIZED_DIR.relative_to(ROOT)).replace("\\", "/"),
        "minimized_files": minimized_files,
    }

    write_json("parser_crash_triage_artifact.json", triage)
    write_json("parser_fuzz_regression_artifact.json", regression)
    write_json("parser_fuzz_campaign_artifact.json", campaign)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
