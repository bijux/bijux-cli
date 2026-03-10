#!/usr/bin/env python3
"""Generate command-level parity matrix and side-by-side diffs."""

from __future__ import annotations

import json
import re
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MATRIX_OUT = ROOT / "artifacts" / "parity" / "command_parity_matrix.json"
DIFFS_OUT = ROOT / "artifacts" / "parity" / "command_parity_diffs.json"
SUMMARY_TXT = ROOT / "artifacts" / "parity" / "command_parity_summary.txt"
PLUGIN_MATRIX_OUT = ROOT / "artifacts" / "parity" / "plugin_parity_matrix.json"
REPL_MATRIX_OUT = ROOT / "artifacts" / "parity" / "repl_parity_matrix.json"
BRIDGE_MATRIX_OUT = ROOT / "artifacts" / "parity" / "python_bridge_parity_matrix.json"
STATE_MATRIX_OUT = ROOT / "artifacts" / "parity" / "state_behavior_parity_matrix.json"
OWNED_OUT = ROOT / "artifacts" / "parity" / "commands_fully_rust_owned.json"
SHIMS_OUT = ROOT / "artifacts" / "parity" / "commands_using_compatibility_shims.json"
PYTHON_ONLY_OUT = ROOT / "artifacts" / "parity" / "commands_python_only.json"
COVERAGE_OUT = ROOT / "artifacts" / "parity" / "parity_coverage_matrix.json"
STDOUT_MD = ROOT / "artifacts" / "parity" / "stdout_diff.md"
STDERR_MD = ROOT / "artifacts" / "parity" / "stderr_diff.md"
EXIT_MD = ROOT / "artifacts" / "parity" / "exit_code_diff.md"
HELP_MD = ROOT / "artifacts" / "parity" / "help_diff.md"


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        return ""


def run_cmd(args: list[str], cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, cwd=cwd or ROOT, text=True, capture_output=True, check=False)


def parse_python_command_tree() -> list[str]:
    local = ROOT / "bin" / "bijux"
    if local.exists():
        exe = str(local)
    else:
        resolved = shutil.which("bijux")
        if resolved is None:
            return []
        exe = resolved

    def subcommands(path_tokens: tuple[str, ...]) -> list[str]:
        proc = run_cmd([exe, *path_tokens, "--help"])
        if proc.returncode != 0:
            return []
        out = []
        in_commands = False
        for line in proc.stdout.splitlines():
            stripped = line.strip()
            if stripped == "Commands:":
                in_commands = True
                continue
            if in_commands and stripped.startswith("Options:"):
                break
            if not in_commands or not stripped:
                continue
            token = stripped.split()[0]
            if token == "help":
                continue
            if re.match(r"^[a-z][a-z0-9_-]*$", token):
                out.append(token)
        return out

    seen: set[tuple[str, ...]] = set()
    queue: list[tuple[str, ...]] = [tuple()]
    while queue:
        path = queue.pop(0)
        if path in seen:
            continue
        seen.add(path)
        if len(path) >= 4:
            continue
        for sub in subcommands(path):
            child = (*path, sub)
            if child not in seen:
                queue.append(child)
    return sorted(" ".join(parts) for parts in seen if parts)


def parse_rust_surface_commands() -> list[str]:
    report = ROOT / "artifacts" / "status" / "current_rust_state.json"
    if not report.exists():
        return []
    data = json.loads(read_text(report))
    return sorted(set(data.get("rust_routed_commands", {}).get("surface", [])))


def load_parity_results() -> dict[str, dict]:
    path = ROOT / "artifacts" / "parity" / "rust_python_parity_report.json"
    if not path.exists():
        return {}
    data = json.loads(read_text(path))
    mapping = {}
    for row in data.get("commands", []):
        argv = row.get("argv", [])
        command = " ".join(argv[1:]) if isinstance(argv, list) else ""
        if command:
            mapping[command] = row
    return mapping


def normalize_command(name: str) -> str:
    return " ".join(name.split())


def classify_group(command: str) -> str:
    if command.startswith("dev cli "):
        return "dev-cli"
    if command.startswith("cli "):
        if command.startswith("cli config "):
            return "config"
        if command.startswith("cli plugins "):
            return "plugin"
        return "cli"
    if command.startswith("plugins "):
        return "plugin"
    if command.startswith("config"):
        return "config"
    if command.startswith("history"):
        return "history"
    if command.startswith("memory"):
        return "memory"
    return "root"


def load_intentional_differences() -> dict[str, str]:
    path = ROOT / "docs" / "architecture" / "parity" / "intentional_differences.json"
    if not path.exists():
        return {}
    data = json.loads(read_text(path))
    return {normalize_command(k): v for k, v in data.items()}


def confidence_for(row: dict | None, status: str) -> float:
    if status == "missing":
        return 0.0
    if row is None:
        return 0.35
    score = 0.2
    score += 0.25 if row.get("exit_match") else 0.0
    score += 0.25 if row.get("stdout_match") else 0.0
    score += 0.25 if row.get("stderr_match") else 0.0
    if row.get("status") == "rust-complete":
        score += 0.05
    return round(min(score, 1.0), 2)


def build_matrix() -> tuple[list[dict], dict]:
    python_commands = {normalize_command(c) for c in parse_python_command_tree()}
    rust_commands = {normalize_command(c) for c in parse_rust_surface_commands()}
    parity_rows = load_parity_results()
    intentional = load_intentional_differences()

    universe = sorted(python_commands | rust_commands)
    matrix: list[dict] = []

    for command in universe:
        row = parity_rows.get(command)
        in_python = command in python_commands
        in_rust = command in rust_commands

        if command in intentional:
            status = "different-by-decision"
            reason = intentional[command]
            blocker = ""
            owner = "parity-council"
        elif not in_rust and in_python:
            status = "missing"
            reason = ""
            blocker = "not routed by rust yet"
            owner = "routing-core"
        elif row and row.get("status") == "rust-complete":
            status = "complete"
            reason = ""
            blocker = ""
            owner = ""
        else:
            status = "partial"
            reason = ""
            blocker = "parity coverage incomplete"
            owner = "rust-foundation"

        evidence_links = [
            "artifacts/parity/rust_python_parity_report.json",
            "artifacts/parity/command_parity_diffs.json",
        ]
        if status == "different-by-decision":
            evidence_links.append("docs/architecture/parity/intentional_differences.json")
        diff_links = (
            {
                "stdout": "artifacts/parity/stdout_diff.md",
                "stderr": "artifacts/parity/stderr_diff.md",
                "exit_code": "artifacts/parity/exit_code_diff.md",
                "help": "artifacts/parity/help_diff.md",
            }
            if row is not None
            else {}
        )
        matrix.append(
            {
                "command": command,
                "group": classify_group(command),
                "status": status,
                "reason": reason,
                "blocker": blocker,
                "owner": owner,
                "confidence": confidence_for(row, status),
                "python_available": in_python,
                "rust_available": in_rust,
                "evidence_links": evidence_links,
                "diff_links": diff_links,
            }
        )

    grouped = {
        "root": [m for m in matrix if m["group"] == "root"],
        "cli": [m for m in matrix if m["group"] == "cli"],
        "dev_cli": [m for m in matrix if m["group"] == "dev-cli"],
        "plugin": [m for m in matrix if m["group"] == "plugin"],
        "config": [m for m in matrix if m["group"] == "config"],
        "history": [m for m in matrix if m["group"] == "history"],
        "memory": [m for m in matrix if m["group"] == "memory"],
    }
    return matrix, grouped


def diff_rows() -> list[dict]:
    path = ROOT / "artifacts" / "parity" / "rust_python_parity_report.json"
    if not path.exists():
        return []
    data = json.loads(read_text(path))
    out = []
    for row in data.get("commands", []):
        argv = row.get("argv", [])
        command = " ".join(argv[1:]) if isinstance(argv, list) else ""
        if not command:
            continue
        out.append(
            {
                "command": command,
                "stdout": {
                    "match": bool(row.get("stdout_match")),
                    "python": row.get("python_stdout", ""),
                    "rust": row.get("rust_stdout", ""),
                },
                "stderr": {
                    "match": bool(row.get("stderr_match")),
                    "python": row.get("python_stderr", ""),
                    "rust": row.get("rust_stderr", ""),
                },
                "exit_code": {
                    "match": bool(row.get("exit_match")),
                    "python": row.get("python_exit"),
                    "rust": row.get("rust_exit"),
                },
                "help": {
                    "is_help_command": "help" in command or "--help" in command,
                    "match": bool(row.get("stdout_match")) and bool(row.get("stderr_match")),
                },
            }
        )
    return out


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_markdown_diffs(diffs: list[dict]) -> None:
    def snippet(text: str) -> str:
        compact = text.replace("\n", "\\n")
        if len(compact) > 180:
            return compact[:177] + "..."
        return compact

    stdout_lines = ["# Stdout Diff", "", "| Command | Match | Python | Rust |", "|---|---|---|---|"]
    stderr_lines = ["# Stderr Diff", "", "| Command | Match | Python | Rust |", "|---|---|---|---|"]
    exit_lines = ["# Exit Code Diff", "", "| Command | Match | Python | Rust |", "|---|---|---|---|"]
    help_lines = ["# Help Diff", "", "| Command | Help Command | Match |", "|---|---|---|"]

    for row in diffs:
        command = row["command"]
        stdout_lines.append(
            f"| `{command}` | {'yes' if row['stdout']['match'] else 'no'} | `{snippet(str(row['stdout']['python']))}` | `{snippet(str(row['stdout']['rust']))}` |"
        )
        stderr_lines.append(
            f"| `{command}` | {'yes' if row['stderr']['match'] else 'no'} | `{snippet(str(row['stderr']['python']))}` | `{snippet(str(row['stderr']['rust']))}` |"
        )
        exit_lines.append(
            f"| `{command}` | {'yes' if row['exit_code']['match'] else 'no'} | `{row['exit_code']['python']}` | `{row['exit_code']['rust']}` |"
        )
        help_lines.append(
            f"| `{command}` | {'yes' if row['help']['is_help_command'] else 'no'} | {'yes' if row['help']['match'] else 'no'} |"
        )

    for path, lines in [
        (STDOUT_MD, stdout_lines),
        (STDERR_MD, stderr_lines),
        (EXIT_MD, exit_lines),
        (HELP_MD, help_lines),
    ]:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_text_summary(matrix: list[dict]) -> None:
    total = len(matrix)
    counts = {
        "complete": sum(1 for row in matrix if row["status"] == "complete"),
        "partial": sum(1 for row in matrix if row["status"] == "partial"),
        "missing": sum(1 for row in matrix if row["status"] == "missing"),
        "different-by-decision": sum(
            1 for row in matrix if row["status"] == "different-by-decision"
        ),
    }
    plugin_rows = [row for row in matrix if row["group"] == "plugin"]
    lines = [
        "Command parity summary",
        f"total: {total}",
        f"complete: {counts['complete']}",
        f"partial: {counts['partial']}",
        f"missing: {counts['missing']}",
        f"different-by-decision: {counts['different-by-decision']}",
        "",
        f"plugin-commands-total: {len(plugin_rows)}",
        f"plugin-commands-complete: {sum(1 for row in plugin_rows if row['status'] == 'complete')}",
        f"plugin-commands-partial: {sum(1 for row in plugin_rows if row['status'] == 'partial')}",
        "",
        "truth-source: artifacts/parity/command_parity_matrix.json",
    ]
    SUMMARY_TXT.parent.mkdir(parents=True, exist_ok=True)
    SUMMARY_TXT.write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_specialized_matrices(matrix: list[dict], diffs: list[dict]) -> None:
    by_command = {row["command"]: row for row in matrix}
    diff_by_command = {row["command"]: row for row in diffs}

    plugin_rows = [row for row in matrix if row["group"] == "plugin"]
    repl_rows = [row for row in matrix if "repl" in row["command"].split()]
    state_rows = [row for row in matrix if row["group"] in {"config", "history", "memory"}]
    bridge_rows = []
    for command, row in by_command.items():
        diff = diff_by_command.get(command)
        if not diff:
            continue
        bridge_rows.append(
            {
                "command": command,
                "status": row["status"],
                "exit_match": diff["exit_code"]["match"],
                "stdout_match": diff["stdout"]["match"],
                "stderr_match": diff["stderr"]["match"],
            }
        )

    write_json(
        PLUGIN_MATRIX_OUT,
        {
            "generated_at": datetime.now(timezone.utc).isoformat(),
            "source": "artifacts/parity/command_parity_matrix.json",
            "rows": plugin_rows,
            "summary": {
                "total": len(plugin_rows),
                "complete": sum(1 for row in plugin_rows if row["status"] == "complete"),
                "partial": sum(1 for row in plugin_rows if row["status"] == "partial"),
                "missing": sum(1 for row in plugin_rows if row["status"] == "missing"),
            },
        },
    )
    write_json(
        REPL_MATRIX_OUT,
        {
            "generated_at": datetime.now(timezone.utc).isoformat(),
            "source": "artifacts/parity/command_parity_matrix.json",
            "rows": repl_rows,
            "summary": {
                "total": len(repl_rows),
                "complete": sum(1 for row in repl_rows if row["status"] == "complete"),
                "partial": sum(1 for row in repl_rows if row["status"] == "partial"),
                "missing": sum(1 for row in repl_rows if row["status"] == "missing"),
            },
        },
    )
    write_json(
        BRIDGE_MATRIX_OUT,
        {
            "generated_at": datetime.now(timezone.utc).isoformat(),
            "source": "artifacts/parity/command_parity_diffs.json",
            "rows": bridge_rows,
            "summary": {
                "total": len(bridge_rows),
                "exit_match": sum(1 for row in bridge_rows if row["exit_match"]),
                "stdout_match": sum(1 for row in bridge_rows if row["stdout_match"]),
                "stderr_match": sum(1 for row in bridge_rows if row["stderr_match"]),
            },
        },
    )
    write_json(
        STATE_MATRIX_OUT,
        {
            "generated_at": datetime.now(timezone.utc).isoformat(),
            "source": "artifacts/parity/command_parity_matrix.json",
            "rows": state_rows,
            "summary": {
                "total": len(state_rows),
                "complete": sum(1 for row in state_rows if row["status"] == "complete"),
                "partial": sum(1 for row in state_rows if row["status"] == "partial"),
                "missing": sum(1 for row in state_rows if row["status"] == "missing"),
            },
        },
    )

    state_report = ROOT / "artifacts" / "status" / "current_rust_state.json"
    aliases: set[str] = set()
    if state_report.exists():
        data = json.loads(read_text(state_report))
        aliases = set(data.get("rust_routed_commands", {}).get("aliases", []))

    owned_rows = [row for row in matrix if row["status"] == "complete"]
    shim_rows = [row for row in matrix if row["command"] in aliases]
    python_only_rows = [row for row in matrix if row["status"] == "missing" and row["python_available"]]
    write_json(OWNED_OUT, {"generated_at": datetime.now(timezone.utc).isoformat(), "commands": owned_rows})
    write_json(SHIMS_OUT, {"generated_at": datetime.now(timezone.utc).isoformat(), "commands": shim_rows})
    write_json(
        PYTHON_ONLY_OUT,
        {"generated_at": datetime.now(timezone.utc).isoformat(), "commands": python_only_rows},
    )

    write_json(
        COVERAGE_OUT,
        {
            "generated_at": datetime.now(timezone.utc).isoformat(),
            "coverage": [
                {
                    "command": command,
                    "parity_tests": command in diff_by_command,
                    "failure_tests": row.get("status") in {"complete", "partial"},
                    "output_snapshots": bool(row.get("diff_links", {}).get("stdout")),
                    "exit_code_checks": bool(row.get("diff_links", {}).get("exit_code")),
                    "stderr_stdout_checks": bool(row.get("diff_links", {}).get("stderr")),
                }
                for command, row in sorted(by_command.items())
            ],
        },
    )


def main() -> int:
    matrix, grouped = build_matrix()
    diffs = diff_rows()
    plugin_rows = [row for row in matrix if row["group"] == "plugin"]

    matrix_payload = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "generator": "scripts/parity/generate_command_parity_matrix.py",
        "commands": matrix,
        "groups": grouped,
        "plugin_lifecycle": {
            "commands": plugin_rows,
            "summary": {
                "total": len(plugin_rows),
                "complete": sum(1 for row in plugin_rows if row["status"] == "complete"),
                "partial": sum(1 for row in plugin_rows if row["status"] == "partial"),
                "missing": sum(1 for row in plugin_rows if row["status"] == "missing"),
                "different_by_decision": sum(
                    1 for row in plugin_rows if row["status"] == "different-by-decision"
                ),
            },
        },
        "summary": {
            "total": len(matrix),
            "complete": sum(1 for row in matrix if row["status"] == "complete"),
            "partial": sum(1 for row in matrix if row["status"] == "partial"),
            "missing": sum(1 for row in matrix if row["status"] == "missing"),
            "different_by_decision": sum(
                1 for row in matrix if row["status"] == "different-by-decision"
            ),
        },
    }
    diffs_payload = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "generator": "scripts/parity/generate_command_parity_matrix.py",
        "diffs": diffs,
    }

    write_json(MATRIX_OUT, matrix_payload)
    write_json(DIFFS_OUT, diffs_payload)
    write_markdown_diffs(diffs)
    write_text_summary(matrix)
    write_specialized_matrices(matrix, diffs)
    print(f"wrote {MATRIX_OUT.relative_to(ROOT)}")
    print(f"wrote {DIFFS_OUT.relative_to(ROOT)}")
    print(f"wrote {SUMMARY_TXT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
