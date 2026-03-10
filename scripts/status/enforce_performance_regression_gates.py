#!/usr/bin/env python3
"""Enforce critical-path performance regression gate definitions."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
BIN_TEST = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "performance_realism_hardening.rs"
OUTPUT_TEST = ROOT / "crates" / "bijux-cli-output" / "tests" / "output_rendering_performance.rs"
REPL_TEST = ROOT / "crates" / "bijux-cli-repl" / "tests" / "repl_startup_performance_budget.rs"


def read_json(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def required_snippets(path: Path, snippets: list[str]) -> list[str]:
    if not path.exists():
        return [f"missing file: {path.relative_to(ROOT)}"]
    text = path.read_text(encoding="utf-8")
    missing = []
    for snippet in snippets:
        if snippet not in text:
            missing.append(f"{path.relative_to(ROOT)} missing snippet: {snippet}")
    return missing


def main() -> int:
    failures: list[str] = []

    budget = read_json(STATUS / "performance_regression_budget.json")
    thresholds = budget.get("thresholds", {}) if isinstance(budget, dict) else {}
    if thresholds.get("mode") != "critical-path-only":
        failures.append("performance threshold mode must be critical-path-only")

    startup = thresholds.get("startup_ms", {})
    payload = thresholds.get("payload_bytes", {})
    rendering = thresholds.get("rendering_budget_ms", {})

    required_startup = {
        "version",
        "status",
        "doctor",
        "plugins list",
        "cli config get",
        "dev cli status",
        "plugins list (broken registry)",
        "plugins list (large registry)",
        "cli config get (large config)",
        "history (large history)",
    }
    missing_startup = sorted(required_startup - set(startup.keys()))
    if missing_startup:
        failures.append(f"missing startup thresholds: {', '.join(missing_startup)}")

    required_payload = {
        "version",
        "status",
        "plugins list",
        "repl startup memory estimate",
    }
    missing_payload = sorted(required_payload - set(payload.keys()))
    if missing_payload:
        failures.append(f"missing payload thresholds: {', '.join(missing_payload)}")

    required_rendering = {"json_large_payload_total", "yaml_large_payload_total"}
    missing_rendering = sorted(required_rendering - set(rendering.keys()))
    if missing_rendering:
        failures.append(f"missing rendering thresholds: {', '.join(missing_rendering)}")

    failures.extend(
        required_snippets(
            BIN_TEST,
            [
                "startup_benchmarks_for_key_commands_stay_within_budget",
                "startup_benchmarks_under_registry_config_and_history_stress_stay_within_budget",
                "payload_size_benchmarks_for_key_commands_stay_within_budget",
            ],
        )
    )
    failures.extend(
        required_snippets(
            OUTPUT_TEST,
            [
                "large_json_rendering_stays_within_budget",
                "large_yaml_rendering_stays_within_budget",
            ],
        )
    )
    failures.extend(
        required_snippets(
            REPL_TEST,
            [
                "repl_startup_latency_stays_within_budget",
                "repl_startup_memory_estimate_stays_within_budget",
            ],
        )
    )

    for failure in failures:
        print(f"PERF GATE FAILURE: {failure}")

    return 2 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
