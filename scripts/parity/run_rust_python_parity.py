#!/usr/bin/env python3
"""Generate a unified Rust-vs-Python parity report for covered command captures."""

from __future__ import annotations

import argparse
import json
import subprocess
import time
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Any

CAPTURE_KEYS = [
    "bijux_help",
    "bijux_version",
    "bijux_doctor",
    "bijux_status_text",
    "bijux_status_json_no_pretty",
    "bijux_status_yaml_pretty",
    "bijux_plugins_list",
    "bijux_config_root",
    "bijux_history_root",
    "bijux_dev_help",
    "behavior_plugins_check",
    "behavior_config_precedence_config_only",
    "behavior_config_precedence_env_override",
    "behavior_config_precedence_cli_override",
]

NEXT_FIVE_PORTS = [
    {"command": "history", "category": "read-only", "status": "implemented"},
    {"command": "plugins check", "category": "diagnostics", "status": "implemented"},
    {"command": "config", "category": "config", "status": "implemented"},
    {"command": "plugins list", "category": "plugin-read", "status": "implemented"},
    {"command": "repl --help", "category": "repl-gap", "status": "covered"},
]


@dataclass
class CommandResult:
    name: str
    argv: list[str]
    python_exit: int
    rust_exit: int
    python_stdout: str
    rust_stdout: str
    python_stderr: str
    rust_stderr: str
    rust_ms: float
    exit_match: bool
    stdout_match: bool
    stderr_match: bool
    status: str


@dataclass
class CrateCheck:
    name: str
    ok: bool


def run_cmd(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, check=False, text=True, capture_output=True)


def classify(exit_match: bool, stdout_match: bool, stderr_match: bool, rust_exit: int, python_exit: int) -> str:
    if exit_match and stdout_match and stderr_match:
        return "rust-complete"
    if python_exit == 0 and rust_exit != 0:
        return "python-only"
    return "rust-partial"


def markdown_table(results: list[CommandResult]) -> str:
    lines = [
        "| Capture | Command | Status | Exit | Stdout | Stderr | Rust ms |",
        "|---|---|---|---|---|---|---:|",
    ]
    for r in results:
        lines.append(
            f"| {r.name} | `{' '.join(r.argv[1:])}` | {r.status} | "
            f"{'match' if r.exit_match else 'diff'} | {'match' if r.stdout_match else 'diff'} | "
            f"{'match' if r.stderr_match else 'diff'} | {r.rust_ms:.2f} |"
        )
    return "\n".join(lines)


def load_baseline(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    return json.loads(path.read_text())


def enforce_baseline(current: dict[str, Any], baseline: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    if not baseline:
        return failures

    base_by_name = {item["name"]: item for item in baseline.get("commands", [])}
    for item in current.get("commands", []):
        prev = base_by_name.get(item["name"])
        if prev is None:
            continue
        prev_rank = {"rust-complete": 3, "rust-partial": 2, "python-only": 1}.get(prev["status"], 0)
        curr_rank = {"rust-complete": 3, "rust-partial": 2, "python-only": 1}.get(item["status"], 0)
        if curr_rank < prev_rank:
            failures.append(
                f"status regressed for {item['name']}: {prev['status']} -> {item['status']}"
            )
    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--captures", default="artifacts/current-python-behavior-lock.json")
    parser.add_argument("--json-out", default="artifacts/parity/rust_python_parity_report.json")
    parser.add_argument("--md-out", default="docs/architecture/rust-parity-status-table.md")
    parser.add_argument("--baseline", default="docs/architecture/parity/baseline-parity-v1.json")
    parser.add_argument("--enforce-baseline", action="store_true")
    args = parser.parse_args()

    captures = json.loads(Path(args.captures).read_text())["captures"]
    results: list[CommandResult] = []

    for key in CAPTURE_KEYS:
        capture = captures[key]
        argv = capture["argv"]
        started = time.perf_counter()
        proc = run_cmd(["cargo", "run", "-q", "-p", "bijux-cli-bin", "--", *argv[1:]])
        elapsed_ms = (time.perf_counter() - started) * 1000.0

        py_stdout = capture.get("stdout", "")
        py_stderr = capture.get("stderr", "")
        rs_stdout = proc.stdout
        rs_stderr = proc.stderr

        exit_match = capture["exit_code"] == proc.returncode
        stdout_match = py_stdout == rs_stdout
        stderr_match = py_stderr == rs_stderr

        results.append(
            CommandResult(
                name=key,
                argv=argv,
                python_exit=capture["exit_code"],
                rust_exit=proc.returncode,
                python_stdout=py_stdout,
                rust_stdout=rs_stdout,
                python_stderr=py_stderr,
                rust_stderr=rs_stderr,
                rust_ms=elapsed_ms,
                exit_match=exit_match,
                stdout_match=stdout_match,
                stderr_match=stderr_match,
                status=classify(exit_match, stdout_match, stderr_match, proc.returncode, capture["exit_code"]),
            )
        )

    crate_checks = [
        ("bin", ["cargo", "test", "-q", "-p", "bijux-cli-bin"]),
        ("core", ["cargo", "test", "-q", "-p", "bijux-cli-core"]),
        ("output", ["cargo", "test", "-q", "-p", "bijux-cli-output", "--test", "python_parity"]),
        ("plugin", ["cargo", "test", "-q", "-p", "bijux-cli-plugin", "--test", "plugin_parity_read_paths"]),
        ("repl", ["cargo", "test", "-q", "-p", "bijux-cli-repl", "--test", "transcript_parity"]),
        ("python", ["cargo", "test", "-q", "-p", "bijux-cli-python", "--test", "bridge_bindings"]),
    ]
    checks: list[CrateCheck] = []
    for name, cmd in crate_checks:
        proc = run_cmd(cmd)
        checks.append(CrateCheck(name=name, ok=proc.returncode == 0))

    report = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "commands": [asdict(item) for item in results],
        "crate_checks": [asdict(c) for c in checks],
        "next_five_ports": NEXT_FIVE_PORTS,
    }

    json_out = Path(args.json_out)
    json_out.parent.mkdir(parents=True, exist_ok=True)
    json_out.write_text(json.dumps(report, indent=2) + "\n")

    md_out = Path(args.md_out)
    md_out.parent.mkdir(parents=True, exist_ok=True)
    md_out.write_text(
        "# Rust Parity Status Table\n\n"
        + markdown_table(results)
        + "\n\n## Crate Checks\n\n"
        + "\n".join(
            f"- `{c.name}`: {'pass' if c.ok else 'fail'}" for c in checks
        )
        + "\n"
    )

    baseline = load_baseline(Path(args.baseline))
    if args.enforce_baseline:
        failures = enforce_baseline(report, baseline)
        if failures:
            for failure in failures:
                print(f"PARITY REGRESSION: {failure}")
            return 2

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
