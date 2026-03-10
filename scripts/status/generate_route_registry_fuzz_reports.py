#!/usr/bin/env python3
"""Generate route and registry fuzz hardening artifacts for TODOs 21-40."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

ROUTE_TARGETS = ROOT / "crates" / "bijux-cli-routing" / "tests" / "route_fuzz_targets.rs"
ROUTE_REGRESSION = ROOT / "crates" / "bijux-cli-routing" / "tests" / "route_fuzz_regressions.rs"
REGISTRY_TARGETS = ROOT / "crates" / "bijux-cli-plugin" / "tests" / "registry_fuzz_targets.rs"
REGISTRY_REGRESSION = ROOT / "crates" / "bijux-cli-plugin" / "tests" / "registry_fuzz_regressions.rs"

ROUTE_MIN_DIR = ROOT / "crates" / "bijux-cli-routing" / "tests" / "fuzz" / "route_minimized_cases"
REGISTRY_MIN_DIR = ROOT / "crates" / "bijux-cli-plugin" / "tests" / "fuzz" / "registry_minimized_cases"

REQUIRED_TESTS = {
    21: (ROUTE_TARGETS, "fuzz_route_registration_order_is_deterministic"),
    22: (ROUTE_TARGETS, "fuzz_randomized_plugin_namespace_registration_is_safe_and_deterministic"),
    23: (ROUTE_TARGETS, "fuzz_normalized_collision_registration_rejects_equivalent_namespaces"),
    24: (ROUTE_TARGETS, "fuzz_hidden_alias_collision_registration_rejects_alias_roots"),
    25: (ROUTE_TARGETS, "fuzz_command_tree_export_under_randomized_registration_is_stable"),
    26: (ROUTE_TARGETS, "fuzz_route_inspection_payload_generation_is_json_stable"),
    27: (ROUTE_TARGETS, "fuzz_unknown_command_suggestion_generation_is_stable"),
    28: (ROUTE_TARGETS, "fuzz_command_metadata_rendering_is_stable"),
    29: (REGISTRY_TARGETS, "fuzz_plugin_registry_hydration_is_stable_under_malformed_inputs"),
    30: (REGISTRY_TARGETS, "fuzz_registry_discovery_disagreement_resolution_is_deterministic"),
    31: (REGISTRY_TARGETS, "fuzz_reserved_namespace_registry_loading_rejects_reserved_namespaces"),
    32: (ROUTE_TARGETS, "fuzz_route_tree_serialization_is_stable"),
    33: (ROUTE_TARGETS, "fuzz_route_tree_text_rendering_is_stable"),
    36: (ROUTE_REGRESSION, "minimized_route_cases_do_not_crash_and_are_deterministic"),
    37: (REGISTRY_REGRESSION, "minimized_registry_cases_replay_without_unexpected_errors"),
    38: (ROUTE_REGRESSION, "minimized_route_cases_do_not_crash_and_are_deterministic"),
    39: (REGISTRY_REGRESSION, "minimized_registry_cases_replay_without_unexpected_errors"),
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
        ROUTE_TARGETS: read_text(ROUTE_TARGETS),
        ROUTE_REGRESSION: read_text(ROUTE_REGRESSION),
        REGISTRY_TARGETS: read_text(REGISTRY_TARGETS),
        REGISTRY_REGRESSION: read_text(REGISTRY_REGRESSION),
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

    route_cases = sorted(str(p.relative_to(ROOT)).replace("\\", "/") for p in ROUTE_MIN_DIR.glob("*.txt"))
    registry_cases = sorted(str(p.relative_to(ROOT)).replace("\\", "/") for p in REGISTRY_MIN_DIR.glob("*.json"))

    route_replay = run_test(["cargo", "test", "-p", "bijux-cli-routing", "--test", "route_fuzz_regressions"])
    registry_replay = run_test(["cargo", "test", "-p", "bijux-cli-plugin", "--test", "registry_fuzz_regressions"])

    missing_todos = [row["todo"] for row in coverage_rows if row["status"] != "covered"]

    route_triage = {
        "generated_at": now,
        "generator": "scripts/status/generate_route_registry_fuzz_reports.py",
        "scope": "route fuzz crash triage",
        "tasks": [34],
        "status": "clean" if route_replay["ok"] else "needs-triage",
        "known_minimized_route_cases": len(route_cases),
        "regression_replay_ok": route_replay["ok"],
        "regression_replay_command": route_replay["command"],
    }

    registry_triage = {
        "generated_at": now,
        "generator": "scripts/status/generate_route_registry_fuzz_reports.py",
        "scope": "registry fuzz crash triage",
        "tasks": [35],
        "status": "clean" if registry_replay["ok"] else "needs-triage",
        "known_minimized_registry_cases": len(registry_cases),
        "regression_replay_ok": registry_replay["ok"],
        "regression_replay_command": registry_replay["command"],
    }

    route_regression = {
        "generated_at": now,
        "generator": "scripts/status/generate_route_registry_fuzz_reports.py",
        "scope": "route fuzz regressions",
        "tasks": [36, 38],
        "status": "clean" if route_replay["ok"] else "drift",
        "minimized_case_count": len(route_cases),
        "regression_replay_ok": route_replay["ok"],
    }

    registry_regression = {
        "generated_at": now,
        "generator": "scripts/status/generate_route_registry_fuzz_reports.py",
        "scope": "registry fuzz regressions",
        "tasks": [37, 39],
        "status": "clean" if registry_replay["ok"] else "drift",
        "minimized_case_count": len(registry_cases),
        "regression_replay_ok": registry_replay["ok"],
    }

    contract = {
        "generated_at": now,
        "generator": "scripts/status/generate_route_registry_fuzz_reports.py",
        "scope": "route-registry fuzz hardening",
        "tasks": list(range(21, 41)),
        "status": "frozen" if not missing_todos and route_replay["ok"] and registry_replay["ok"] else "partial",
        "todo_coverage": coverage_rows,
        "missing_todos": missing_todos,
        "route_minimized_cases": route_cases,
        "registry_minimized_cases": registry_cases,
        "policy": "route and registry fuzzing are permanent CI hardening checks",
    }

    write_json("route_crash_triage_artifact.json", route_triage)
    write_json("registry_crash_triage_artifact.json", registry_triage)
    write_json("route_fuzz_regression_artifact.json", route_regression)
    write_json("registry_fuzz_regression_artifact.json", registry_regression)
    write_json("route_registry_fuzz_contract.json", contract)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
