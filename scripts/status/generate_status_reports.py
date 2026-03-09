#!/usr/bin/env python3
"""Generate command status reports used by `bijux dev cli status`."""

from __future__ import annotations

import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS_DIR = ROOT / "artifacts" / "status"
PARITY_DIR = ROOT / "artifacts" / "parity"


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

    commit_time = subprocess.run(
        ["git", "log", "-1", "--format=%cI"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if commit_time:
        return commit_time

    return datetime.now(timezone.utc).isoformat()


def read_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(name: str, payload: dict[str, Any]) -> None:
    out = STATUS_DIR / name
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def command_from_test_line(line: str) -> str:
    quoted = re.findall(r'"([a-z0-9_-]+)"', line)
    if not quoted:
        return ""
    if quoted[0] in {"bijux", "env!", "CARGO_BIN_EXE_bijux-rs"}:
        quoted = quoted[1:]
    return " ".join(quoted).strip()


def failure_path_coverage() -> list[str]:
    command_hits: set[str] = set()
    patterns = (
        "error",
        "failure",
        "invalid",
        "malformed",
        "missing",
        "reject",
        "rollback",
        "corrupt",
        "unsafe",
        "duplicate",
        "conflict",
        "shadow",
    )
    for path in (ROOT / "crates").rglob("tests/*.rs"):
        text = path.read_text(encoding="utf-8")
        lowered = text.lower()
        if not any(token in lowered for token in patterns):
            continue
        for line in text.splitlines():
            if "[\"" not in line:
                continue
            if not any(token in line.lower() for token in patterns):
                continue
            command = command_from_test_line(line)
            if command:
                command_hits.add(command)
    return sorted(command_hits)


def top_level_command(command: str) -> str:
    return command.split()[0] if command else ""


def direct_cli_subcommand(command: str) -> str:
    if not command.startswith("cli "):
        return ""
    parts = command.split()
    return " ".join(parts[:3]) if len(parts) >= 3 else command


def direct_dev_cli_subcommand(command: str) -> str:
    if not command.startswith("dev cli "):
        return ""
    parts = command.split()
    return " ".join(parts[:4]) if len(parts) >= 4 else command


def plugin_command_name(command: str) -> str:
    if command.startswith("plugins "):
        parts = command.split()
        return " ".join(parts[:2]) if len(parts) >= 2 else command
    if command.startswith("cli plugins "):
        parts = command.split()
        return " ".join(parts[:3]) if len(parts) >= 3 else command
    return ""


def status_for(command: str, matrix_status: str, aliases: set[str]) -> str:
    if command in aliases:
        return "shim"
    if matrix_status == "missing":
        return "missing"
    if matrix_status == "partial":
        return "partial"
    return "complete"


def main() -> int:
    generated_at = stable_generated_at()

    current_state = read_json(STATUS_DIR / "current_rust_state.json")
    parity_matrix = read_json(PARITY_DIR / "command_parity_matrix.json")
    bridge_report = read_json(PARITY_DIR / "binary_vs_python_bridge_parity_report.json")
    runtime_unity = read_json(STATUS_DIR / "runtime_unity_report.json")
    state_config = read_json(PARITY_DIR / "config_parity_report.json")
    state_history = read_json(PARITY_DIR / "history_parity_report.json")
    state_memory = read_json(PARITY_DIR / "memory_parity_report.json")
    plugin_state = read_json(STATUS_DIR / "plugin_state_report.json")
    intentional_differences = read_json(
        ROOT / "docs" / "architecture" / "parity" / "intentional_differences.json"
    )

    matrix_rows = parity_matrix.get("commands", []) if isinstance(parity_matrix, dict) else []
    aliases = set(current_state.get("rust_routed_commands", {}).get("aliases", []))

    command_rows: list[dict[str, Any]] = []
    for row in matrix_rows:
        if not isinstance(row, dict):
            continue
        command = str(row.get("command", "")).strip()
        if not command:
            continue
        command_rows.append(
            {
                "command": command,
                "group": row.get("group", "unknown"),
                "status": status_for(command, str(row.get("status", "missing")), aliases),
                "matrix_status": row.get("status", "missing"),
                "owner": row.get("owner", ""),
                "reason": row.get("reason", ""),
                "blocker": row.get("blocker", ""),
                "confidence": row.get("confidence", 0.0),
            }
        )
    command_rows.sort(key=lambda item: str(item["command"]))

    root_commands = sorted({top_level_command(c["command"]) for c in command_rows if c["command"]})
    cli_commands = sorted(
        {
            direct_cli_subcommand(c["command"])
            for c in command_rows
            if c["command"].startswith("cli ") and direct_cli_subcommand(c["command"])
        }
    )
    dev_cli_commands = sorted(
        {
            direct_dev_cli_subcommand(c["command"])
            for c in command_rows
            if c["command"].startswith("dev cli ") and direct_dev_cli_subcommand(c["command"])
        }
    )
    plugin_commands = sorted(
        {
            plugin_command_name(c["command"])
            for c in command_rows
            if plugin_command_name(c["command"])
        }
    )

    snapshot_covered = sorted(current_state.get("snapshot_covered_commands", []))
    stream_covered = sorted(current_state.get("stderr_stdout_covered_commands", []))
    exit_covered = sorted(current_state.get("exit_code_covered_commands", []))
    fail_covered = failure_path_coverage()

    known_gaps = [
        {
            "command": row["command"],
            "status": row["status"],
            "blocker": row.get("blocker", ""),
            "owner": row.get("owner", ""),
        }
        for row in command_rows
        if row["status"] in {"missing", "partial", "shim"}
    ]

    base = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_status_reports.py",
    }

    write_json(
        "status.json",
        {
            **base,
            "commands": command_rows,
            "summary": {
                "total": len(command_rows),
                "complete": sum(1 for row in command_rows if row["status"] == "complete"),
                "partial": sum(1 for row in command_rows if row["status"] == "partial"),
                "shim": sum(1 for row in command_rows if row["status"] == "shim"),
                "missing": sum(1 for row in command_rows if row["status"] == "missing"),
            },
        },
    )

    write_json("status_root_commands.json", {**base, "commands": root_commands})
    write_json("status_cli_subcommands.json", {**base, "commands": cli_commands})
    write_json("status_dev_cli_subcommands.json", {**base, "commands": dev_cli_commands})
    write_json("status_plugin_commands.json", {**base, "commands": plugin_commands})

    repl_commands = [row for row in command_rows if "repl" in row["command"].split()]
    write_json(
        "status_repl_parity_coverage.json",
        {
            **base,
            "summary": {
                "count": len(repl_commands),
                "statuses": {
                    "complete": sum(1 for row in repl_commands if row["status"] == "complete"),
                    "partial": sum(1 for row in repl_commands if row["status"] == "partial"),
                    "shim": sum(1 for row in repl_commands if row["status"] == "shim"),
                    "missing": sum(1 for row in repl_commands if row["status"] == "missing"),
                },
            },
            "commands": repl_commands,
            "evidence_files": [
                "crates/bijux-cli-repl/tests/transcript_parity.rs",
                "crates/bijux-cli-repl/tests/transcript_cases.rs",
            ],
        },
    )

    write_json("status_python_bridge_parity_coverage.json", {**base, "report": bridge_report})
    write_json(
        "status_install_packaging_parity_coverage.json",
        {
            **base,
            "runtime_unity": runtime_unity,
            "runtime_identity_rules": current_state.get("runtime_identity_rules", {}),
            "package_entrypoints": current_state.get("package_entrypoints", []),
        },
    )
    write_json(
        "status_state_behavior_coverage.json",
        {
            **base,
            "config": state_config,
            "history": state_history,
            "memory": state_memory,
            "plugin_state": plugin_state,
        },
    )

    write_json("status_snapshot_coverage.json", {**base, "commands": snapshot_covered})
    write_json("status_stream_coverage.json", {**base, "commands": stream_covered})
    write_json("status_exit_code_coverage.json", {**base, "commands": exit_covered})
    write_json("status_failure_path_coverage.json", {**base, "commands": fail_covered})

    write_json(
        "status_compatibility_aliases.json",
        {
            **base,
            "aliases": sorted(aliases),
        },
    )
    write_json("status_known_parity_gaps.json", {**base, "gaps": known_gaps})
    write_json(
        "status_intentional_differences.json",
        {
            **base,
            "commands": intentional_differences,
        },
    )
    write_json(
        "status_unowned_scripts.json",
        {
            **base,
            "scripts": sorted(current_state.get("scripts_outside_dev_cli", [])),
        },
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
