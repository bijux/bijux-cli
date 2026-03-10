#!/usr/bin/env python3
"""Generate command migration matrix and surface summaries for maintainer status."""

from __future__ import annotations

import json
import subprocess
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS_DIR = ROOT / "artifacts" / "status"
PARITY_DIR = ROOT / "artifacts" / "parity"

STATUS_RUST_COMPLETE = "rust-complete"
STATUS_RUST_PARTIAL = "rust-partial"
STATUS_PYTHON_ONLY = "python-only"
STATUS_INTENTIONAL = "intentionally-different"


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


def write_json(name: str, payload: dict[str, Any]) -> None:
    target = STATUS_DIR / name
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_text(name: str, body: str) -> None:
    target = STATUS_DIR / name
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(body.rstrip() + "\n", encoding="utf-8")


def normalize_status(status: str) -> str:
    if status == "complete":
        return STATUS_RUST_COMPLETE
    if status == "partial":
        return STATUS_RUST_PARTIAL
    if status == "missing":
        return STATUS_PYTHON_ONLY
    if status == "different-by-decision":
        return STATUS_INTENTIONAL
    return STATUS_RUST_PARTIAL


def command_surface(command: str) -> str:
    parts = command.split()
    if not parts:
        return "unknown"
    if parts[0] == "dev" and len(parts) >= 2 and parts[1] == "cli":
        return "dev-cli"
    if parts[0] == "cli":
        return "cli"
    if parts[0] == "plugins" or (parts[0] == "cli" and len(parts) >= 2 and parts[1] == "plugins"):
        return "plugin"
    if "repl" in parts:
        return "repl"
    return "root"


def evidence_links(row: dict[str, Any], surface: str) -> list[str]:
    links = list(row.get("evidence_links", []))
    if not links:
        links = ["artifacts/parity/command_parity_matrix.json"]
    if surface == "repl":
        links.append("artifacts/parity/repl_parity_matrix.json")
    if row.get("command", "").startswith("dev cli"):
        links.append("artifacts/status/dev_cli_inventory.json")
    deduped: list[str] = []
    for link in links:
        if link not in deduped:
            deduped.append(link)
    return deduped


def select_owner(row: dict[str, Any], status: str) -> str:
    owner = str(row.get("owner", "")).strip()
    if owner:
        return owner
    if status == STATUS_RUST_PARTIAL:
        return "rust-foundation"
    return ""


def select_blocker(row: dict[str, Any], status: str) -> str:
    blocker = str(row.get("blocker", "")).strip()
    if blocker:
        return blocker
    if status == STATUS_PYTHON_ONLY:
        return "missing rust route or implementation"
    return ""


def select_reason(row: dict[str, Any], status: str) -> str:
    reason = str(row.get("reason", "")).strip()
    if reason:
        return reason
    if status == STATUS_INTENTIONAL:
        return "documented behavior divergence"
    return ""


def summary_rows(rows: list[dict[str, Any]], status: str) -> list[dict[str, Any]]:
    return [
        {
            "command": row["command"],
            "surface": row["surface"],
            "owner": row.get("owner", ""),
            "blocker": row.get("blocker", ""),
            "reason": row.get("reason", ""),
            "evidence": row.get("evidence", []),
        }
        for row in rows
        if row["status"] == status
    ]


def render_text_report(rows: list[dict[str, Any]]) -> str:
    counts = Counter(row["status"] for row in rows)
    surface_counts = Counter(row["surface"] for row in rows)
    lines = [
        "Command Migration Matrix",
        f"total: {len(rows)}",
        f"rust-complete: {counts.get(STATUS_RUST_COMPLETE, 0)}",
        f"rust-partial: {counts.get(STATUS_RUST_PARTIAL, 0)}",
        f"python-only: {counts.get(STATUS_PYTHON_ONLY, 0)}",
        f"intentionally-different: {counts.get(STATUS_INTENTIONAL, 0)}",
        "",
        "surface counts:",
        f"root: {surface_counts.get('root', 0)}",
        f"cli: {surface_counts.get('cli', 0)}",
        f"dev-cli: {surface_counts.get('dev-cli', 0)}",
        f"plugin: {surface_counts.get('plugin', 0)}",
        f"repl: {surface_counts.get('repl', 0)}",
        "",
        "python-only commands:",
    ]
    for row in rows:
        if row["status"] != STATUS_PYTHON_ONLY:
            continue
        lines.append(f"- {row['command']} (surface={row['surface']}, blocker={row.get('blocker', '')})")
    return "\n".join(lines)


def main() -> int:
    generated_at = stable_generated_at()
    parity = read_json(PARITY_DIR / "command_parity_matrix.json")
    source_rows = parity.get("commands", []) if isinstance(parity, dict) else []

    rows: list[dict[str, Any]] = []
    for item in source_rows:
        if not isinstance(item, dict):
            continue
        command = str(item.get("command", "")).strip()
        if not command:
            continue
        status = normalize_status(str(item.get("status", "partial")))
        surface = command_surface(command)
        rows.append(
            {
                "command": command,
                "surface": surface,
                "status": status,
                "owner": select_owner(item, status),
                "blocker": select_blocker(item, status),
                "reason": select_reason(item, status),
                "evidence": evidence_links(item, surface),
            }
        )

    rows.sort(key=lambda row: row["command"])

    counts = Counter(row["status"] for row in rows)
    surfaces: dict[str, list[dict[str, Any]]] = {
        "root": [r for r in rows if r["surface"] == "root"],
        "cli": [r for r in rows if r["surface"] == "cli"],
        "dev_cli": [r for r in rows if r["surface"] == "dev-cli"],
        "plugin": [r for r in rows if r["surface"] == "plugin"],
        "repl": [r for r in rows if r["surface"] == "repl"],
    }

    matrix_payload = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_command_migration_matrix.py",
        "status_model": [
            STATUS_RUST_COMPLETE,
            STATUS_RUST_PARTIAL,
            STATUS_PYTHON_ONLY,
            STATUS_INTENTIONAL,
        ],
        "summary": {
            "total": len(rows),
            STATUS_RUST_COMPLETE: counts.get(STATUS_RUST_COMPLETE, 0),
            STATUS_RUST_PARTIAL: counts.get(STATUS_RUST_PARTIAL, 0),
            STATUS_PYTHON_ONLY: counts.get(STATUS_PYTHON_ONLY, 0),
            STATUS_INTENTIONAL: counts.get(STATUS_INTENTIONAL, 0),
        },
        "commands": rows,
        "surfaces": surfaces,
    }
    write_json("command_migration_matrix.json", matrix_payload)

    rust_partial = summary_rows(rows, STATUS_RUST_PARTIAL)
    python_only = summary_rows(rows, STATUS_PYTHON_ONLY)
    intentional = summary_rows(rows, STATUS_INTENTIONAL)

    write_json(
        "command_migration_rust_partial.json",
        {"generated_at": generated_at, "commands": rust_partial, "count": len(rust_partial)},
    )
    write_json(
        "command_migration_python_only.json",
        {"generated_at": generated_at, "commands": python_only, "count": len(python_only)},
    )
    write_json(
        "command_migration_intentional_differences.json",
        {"generated_at": generated_at, "commands": intentional, "count": len(intentional)},
    )

    write_text("command_migration_matrix.txt", render_text_report(rows))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
