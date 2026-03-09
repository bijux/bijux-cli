#!/usr/bin/env python3
"""Generate config/history/memory parity status artifacts."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
PARITY_REPORT = ROOT / "artifacts" / "parity" / "rust_python_parity_report.json"
OUT_DIR = ROOT / "artifacts" / "parity"

CONFIG_REPORT = OUT_DIR / "config_parity_report.json"
HISTORY_REPORT = OUT_DIR / "history_parity_report.json"
MEMORY_REPORT = OUT_DIR / "memory_parity_report.json"


def load_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def command_rows() -> list[dict[str, Any]]:
    data = load_json(PARITY_REPORT)
    rows = data.get("commands", []) if isinstance(data, dict) else []
    return [row for row in rows if isinstance(row, dict)]


def summarize(rows: list[dict[str, Any]], command_tokens: tuple[str, ...]) -> dict[str, Any]:
    def match(row: dict[str, Any]) -> bool:
        argv = row.get("argv", [])
        if not isinstance(argv, list) or len(argv) < 2:
            return False
        args = tuple(str(x) for x in argv[1:])
        return args[: len(command_tokens)] == command_tokens

    scoped = [row for row in rows if match(row)]
    if not scoped:
        return {
            "covered": False,
            "count": 0,
            "complete": 0,
            "partial": 0,
            "python_only": 0,
            "commands": [],
        }

    complete = sum(1 for row in scoped if row.get("status") == "rust-complete")
    partial = sum(1 for row in scoped if row.get("status") == "rust-partial")
    python_only = sum(1 for row in scoped if row.get("status") == "python-only")
    command_rows = [
        {
            "name": row.get("name", ""),
            "status": row.get("status", "unknown"),
            "exit_match": bool(row.get("exit_match")),
            "stdout_match": bool(row.get("stdout_match")),
            "stderr_match": bool(row.get("stderr_match")),
        }
        for row in scoped
    ]
    command_rows.sort(key=lambda item: str(item["name"]))

    return {
        "covered": True,
        "count": len(scoped),
        "complete": complete,
        "partial": partial,
        "python_only": python_only,
        "commands": command_rows,
    }


def write_report(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    generated_at = datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    rows = command_rows()

    config_summary = summarize(rows, ("config",))
    history_summary = summarize(rows, ("history",))
    memory_summary = summarize(rows, ("memory",))

    config_report = {
        "artifact": "config_parity_report",
        "generated_at": generated_at,
        "source": str(PARITY_REPORT.relative_to(ROOT)),
        "summary": config_summary,
        "better_than_python": [
            "typed output modes are consistent across json/yaml/text for config commands",
            "config error envelopes include explicit machine-readable code and command path",
            "config diagnostics integrate with dev-cli doctor checks",
        ],
        "weaker_than_python": [
            "full command-by-command parity for every config edge case still depends on more capture coverage",
            "historical python compatibility aliases are still present and should shrink over time",
        ],
    }

    history_report = {
        "artifact": "history_parity_report",
        "generated_at": generated_at,
        "source": str(PARITY_REPORT.relative_to(ROOT)),
        "summary": history_summary,
        "resilience_focus": [
            "malformed history lines",
            "huge history files",
            "missing history storage",
        ],
    }

    memory_report = {
        "artifact": "memory_parity_report",
        "generated_at": generated_at,
        "source": str(PARITY_REPORT.relative_to(ROOT)),
        "summary": memory_summary,
        "resilience_focus": [
            "malformed memory state",
            "empty memory state",
            "missing memory storage",
        ],
    }

    write_report(CONFIG_REPORT, config_report)
    write_report(HISTORY_REPORT, history_report)
    write_report(MEMORY_REPORT, memory_report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
