#!/usr/bin/env python3
"""Generate migration-focused command inventories for maintainer closure work."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS_DIR = ROOT / "artifacts" / "status"
ROUTING_FIXTURES = ROOT / "crates" / "bijux-cli-routing" / "tests" / "fixtures"


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


def read_lines(path: Path) -> list[str]:
    if not path.exists():
        return []
    return [line.strip() for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]


def main() -> int:
    generated_at = stable_generated_at()

    matrix = read_json(STATUS_DIR / "command_migration_matrix.json")
    matrix_rows = matrix.get("commands", []) if isinstance(matrix, dict) else []

    python_documented = set(read_lines(ROUTING_FIXTURES / "python_documented_commands.txt"))
    matrix_by_command = {str(row.get("command", "")).strip(): row for row in matrix_rows if isinstance(row, dict)}

    documented_not_proven = []
    for command in sorted(python_documented):
        row = matrix_by_command.get(command)
        if row is None or row.get("status") != "rust-complete":
            documented_not_proven.append(
                {
                    "command": command,
                    "status": row.get("status", "python-only") if row else "python-only",
                    "surface": row.get("surface", "root") if row else "root",
                    "blocker": row.get("blocker", "missing rust route or implementation") if row else "missing rust route or implementation",
                }
            )

    alias_inventory = read_json(STATUS_DIR / "compatibility_alias_inventory.json")
    shim_inventory = read_json(STATUS_DIR / "compatibility_shim_inventory.json")

    aliases = alias_inventory.get("aliases", []) if isinstance(alias_inventory, dict) else []
    active_aliases = [
        {
            "alias": entry.get("alias", ""),
            "canonical": entry.get("canonical", ""),
            "justification": entry.get("justification", "compatibility path"),
        }
        for entry in aliases
        if isinstance(entry, dict)
    ]

    shims = shim_inventory.get("shims", []) if isinstance(shim_inventory, dict) else []
    active_shims = [
        {
            "path": entry.get("path", ""),
            "kind": entry.get("kind", "compatibility-shim"),
            "justification": entry.get("justification", "compatibility path"),
        }
        for entry in shims
        if isinstance(entry, dict)
    ]

    python_only_rows = [
        {
            "command": row.get("command", ""),
            "surface": row.get("surface", "root"),
            "blocker": row.get("blocker", ""),
        }
        for row in matrix_rows
        if isinstance(row, dict) and row.get("status") == "python-only"
    ]

    write_json(
        "documented_python_commands_not_proven_in_rust.json",
        {
            "generated_at": generated_at,
            "source": "crates/bijux-cli-routing/tests/fixtures/python_documented_commands.txt",
            "commands": documented_not_proven,
            "count": len(documented_not_proven),
        },
    )
    write_json(
        "public_python_paths_still_reachable.json",
        {
            "generated_at": generated_at,
            "source": "artifacts/status/command_migration_matrix.json",
            "commands": python_only_rows,
            "count": len(python_only_rows),
        },
    )
    write_json(
        "legacy_alias_paths_still_accepted.json",
        {
            "generated_at": generated_at,
            "source": "artifacts/status/compatibility_alias_inventory.json",
            "aliases": active_aliases,
            "count": len(active_aliases),
        },
    )
    write_json(
        "compatibility_shims_still_active.json",
        {
            "generated_at": generated_at,
            "source": "artifacts/status/compatibility_shim_inventory.json",
            "shims": active_shims,
            "count": len(active_shims),
        },
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
