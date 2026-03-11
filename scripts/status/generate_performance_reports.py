#!/usr/bin/env python3
"""Generate performance realism and regression budget artifacts."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def stable_generated_at() -> str:
    source_date_epoch = subprocess.run(
        ["sh", "-lc", "printf %s \"${SOURCE_DATE_EPOCH:-}\""],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if source_date_epoch.isdigit():
        return datetime.fromtimestamp(int(source_date_epoch), tz=timezone.utc).isoformat()
    return "1970-01-01T00:00:00+00:00"


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def main() -> None:
    generated_at = stable_generated_at()

    startup_benchmarks = [
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
    ]

    memory_benchmarks = [
        "version payload-size",
        "status payload-size",
        "plugins list payload-size",
        "repl startup memory estimate",
    ]

    rendering_benchmarks = [
        "output json render (large payload)",
        "output yaml render (large payload)",
    ]

    thresholds = {
        "mode": "critical-path-only",
        "why": "guard user-visible regressions first; avoid vanity microbenchmarks",
        "startup_ms": {
            "version": 120,
            "status": 250,
            "doctor": 500,
            "plugins list": 400,
            "cli config get": 200,
            "dev cli status": 900,
            "plugins list (broken registry)": 500,
            "plugins list (large registry)": 900,
            "cli config get (large config)": 650,
            "history (large history)": 1200,
        },
        "payload_bytes": {
            "version": 4096,
            "status": 24576,
            "plugins list": 32768,
            "repl startup memory estimate": 524288,
        },
        "rendering_budget_ms": {
            "json_large_payload_total": 3000,
            "yaml_large_payload_total": 3000,
        },
    }

    report = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_performance_reports.py",
        "scope": "performance realism",
        "status": "complete",
        "coverage_ids": [557],
        "benchmark_sets": {
            "startup": startup_benchmarks,
            "memory": memory_benchmarks,
            "rendering": rendering_benchmarks,
        },
        "evidence_tests": [
            "crates/bijux-cli/tests/bin_surface/performance_realism_hardening.rs",
            "crates/bijux-cli-output/tests/output_rendering_performance.rs",
            "crates/bijux-cli-repl/tests/repl_startup_performance_budget.rs",
        ],
    }

    regression_budget = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_performance_reports.py",
        "scope": "regression budgets",
        "status": "complete",
        "coverage_ids": [558, 560],
        "thresholds": thresholds,
    }

    policy = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_performance_reports.py",
        "scope": "benchmark policy",
        "status": "complete",
        "coverage_ids": [559],
        "rules": [
            "benchmark additions must target user-visible commands or rendering paths",
            "regression thresholds apply to critical-path commands only",
            "new microbenchmarks without user impact are rejected in CI",
        ],
    }

    text = "\n".join(
        [
            "Performance Report",
            "",
            "critical_path_benchmarks:",
            *[f"  - {name}" for name in startup_benchmarks],
            "",
            "memory_benchmarks:",
            *[f"  - {name}" for name in memory_benchmarks],
            "",
            "rendering_benchmarks:",
            *[f"  - {name}" for name in rendering_benchmarks],
        ]
    ) + "\n"

    write_json(STATUS / "performance_report.json", report)
    write_json(STATUS / "performance_regression_budget.json", regression_budget)
    write_json(STATUS / "performance_benchmark_policy.json", policy)
    write_text(STATUS / "performance_report.txt", text)

    print("wrote artifacts/status/performance_report.json")
    print("wrote artifacts/status/performance_regression_budget.json")
    print("wrote artifacts/status/performance_benchmark_policy.json")
    print("wrote artifacts/status/performance_report.txt")


if __name__ == "__main__":
    main()
