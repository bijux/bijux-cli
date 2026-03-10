#!/usr/bin/env python3
"""Generate command-law parity reports and consolidated dashboard."""

from __future__ import annotations

import json
import subprocess
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
PARITY = ROOT / "artifacts" / "parity"
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


def read_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_text(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body.rstrip() + "\n", encoding="utf-8")


def main() -> int:
    generated_at = stable_generated_at()
    matrix = read_json(PARITY / "command_parity_matrix.json")
    diffs = read_json(PARITY / "command_parity_diffs.json")
    coverage = read_json(PARITY / "parity_coverage_matrix.json")

    commands = matrix.get("commands", []) if isinstance(matrix, dict) else []
    diff_rows = diffs.get("diffs", []) if isinstance(diffs, dict) else []
    coverage_rows = coverage.get("coverage", []) if isinstance(coverage, dict) else []

    diff_map = {row.get("command"): row for row in diff_rows if isinstance(row, dict)}
    coverage_map = {row.get("command"): row for row in coverage_rows if isinstance(row, dict)}

    precedence_rows = []
    flag_rows = []
    stream_rows = []
    exit_rows = []
    help_rows = []
    machine_rows = []

    for row in commands:
        if not isinstance(row, dict):
            continue
        command = row.get("command", "")
        if not command:
            continue
        diff = diff_map.get(command, {})
        cov = coverage_map.get(command, {})

        precedence_rows.append(
            {
                "command": command,
                "group": row.get("group", "unknown"),
                "status": row.get("status", "missing"),
                "source_precedence": ["flags", "env", "config", "defaults"],
                "coverage": bool(cov.get("parity_tests", False)),
            }
        )
        flag_rows.append(
            {
                "command": command,
                "group": row.get("group", "unknown"),
                "status": row.get("status", "missing"),
                "global_flags_supported": [
                    "--format/-f",
                    "--pretty/--no-pretty",
                    "--color",
                    "--log-level",
                    "--quiet/-q",
                ],
                "coverage": bool(cov.get("parity_tests", False)),
            }
        )
        stream_rows.append(
            {
                "command": command,
                "stdout_match": bool(diff.get("stdout", {}).get("match", False)),
                "stderr_match": bool(diff.get("stderr", {}).get("match", False)),
                "coverage": bool(cov.get("stderr_stdout_checks", False)),
            }
        )
        exit_rows.append(
            {
                "command": command,
                "exit_code_match": bool(diff.get("exit_code", {}).get("match", False)),
                "coverage": bool(cov.get("exit_code_checks", False)),
            }
        )
        help_rows.append(
            {
                "command": command,
                "is_help_command": bool(diff.get("help", {}).get("is_help_command", False)),
                "help_match": bool(diff.get("help", {}).get("match", False)),
                "coverage": bool(cov.get("output_snapshots", False)),
            }
        )
        machine_rows.append(
            {
                "command": command,
                "stdout_match": bool(diff.get("stdout", {}).get("match", False)),
                "stderr_match": bool(diff.get("stderr", {}).get("match", False)),
                "exit_code_match": bool(diff.get("exit_code", {}).get("match", False)),
                "coverage": bool(cov.get("parity_tests", False)),
            }
        )

    def by_surface() -> dict[str, dict[str, int]]:
        grouped: dict[str, Counter] = defaultdict(Counter)
        for row in commands:
            if not isinstance(row, dict):
                continue
            group = str(row.get("group", "unknown"))
            status = str(row.get("status", "missing"))
            grouped[group][status] += 1
        return {
            key: {
                "complete": values.get("complete", 0),
                "partial": values.get("partial", 0),
                "missing": values.get("missing", 0),
                "different_by_decision": values.get("different-by-decision", 0)
                + values.get("intentionally-different", 0),
            }
            for key, values in grouped.items()
        }

    coverage_summary = Counter()
    for row in coverage_rows:
        if not isinstance(row, dict):
            continue
        if row.get("parity_tests"):
            coverage_summary["parity_tests"] += 1
        if row.get("failure_tests"):
            coverage_summary["failure_tests"] += 1
        if row.get("stderr_stdout_checks"):
            coverage_summary["stderr_stdout_checks"] += 1
        if row.get("exit_code_checks"):
            coverage_summary["exit_code_checks"] += 1
        if row.get("output_snapshots"):
            coverage_summary["output_snapshots"] += 1

    write_json(
        PARITY / "command_precedence_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/parity/generate_command_law_reports.py",
            "rows": precedence_rows,
        },
    )
    write_json(
        PARITY / "command_flag_normalization_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/parity/generate_command_law_reports.py",
            "rows": flag_rows,
        },
    )
    write_json(
        PARITY / "command_stream_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/parity/generate_command_law_reports.py",
            "rows": stream_rows,
        },
    )
    write_json(
        PARITY / "command_exit_code_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/parity/generate_command_law_reports.py",
            "rows": exit_rows,
        },
    )
    write_json(
        PARITY / "command_help_diff_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/parity/generate_command_law_reports.py",
            "rows": help_rows,
        },
    )
    write_json(
        PARITY / "command_machine_output_diff_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/parity/generate_command_law_reports.py",
            "rows": machine_rows,
        },
    )

    dashboard = {
        "generated_at": generated_at,
        "generator": "scripts/parity/generate_command_law_reports.py",
        "summary": {
            "total_commands": len(commands),
            "surfaces": by_surface(),
            "coverage": {
                "parity_tests": coverage_summary.get("parity_tests", 0),
                "failure_tests": coverage_summary.get("failure_tests", 0),
                "stderr_stdout_checks": coverage_summary.get("stderr_stdout_checks", 0),
                "exit_code_checks": coverage_summary.get("exit_code_checks", 0),
                "output_snapshots": coverage_summary.get("output_snapshots", 0),
            },
        },
        "reports": {
            "precedence": "artifacts/parity/command_precedence_report.json",
            "flag_normalization": "artifacts/parity/command_flag_normalization_report.json",
            "stdout_stderr": "artifacts/parity/command_stream_report.json",
            "exit_code": "artifacts/parity/command_exit_code_report.json",
            "help_diff": "artifacts/parity/command_help_diff_report.json",
            "machine_output_diff": "artifacts/parity/command_machine_output_diff_report.json",
            "parity_matrix": "artifacts/parity/command_parity_matrix.json",
            "coverage_matrix": "artifacts/parity/parity_coverage_matrix.json",
        },
    }

    write_json(PARITY / "parity_dashboard.json", dashboard)
    write_text(
        PARITY / "parity_dashboard.txt",
        "\n".join(
            [
                "Parity Dashboard",
                f"total_commands: {dashboard['summary']['total_commands']}",
                f"parity_tests: {dashboard['summary']['coverage']['parity_tests']}",
                f"exit_code_checks: {dashboard['summary']['coverage']['exit_code_checks']}",
                f"stderr_stdout_checks: {dashboard['summary']['coverage']['stderr_stdout_checks']}",
                f"output_snapshots: {dashboard['summary']['coverage']['output_snapshots']}",
                "source: artifacts/parity/parity_dashboard.json",
            ]
        ),
    )

    print("wrote artifacts/parity/command_precedence_report.json")
    print("wrote artifacts/parity/command_flag_normalization_report.json")
    print("wrote artifacts/parity/command_stream_report.json")
    print("wrote artifacts/parity/command_exit_code_report.json")
    print("wrote artifacts/parity/command_help_diff_report.json")
    print("wrote artifacts/parity/command_machine_output_diff_report.json")
    print("wrote artifacts/parity/parity_dashboard.json")
    print("wrote artifacts/parity/parity_dashboard.txt")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
