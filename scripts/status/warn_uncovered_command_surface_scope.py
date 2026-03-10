#!/usr/bin/env python3
"""Warn on newly introduced uncovered commands in command/config/history/memory/diagnostics scope."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONSISTENCY = ROOT / "artifacts" / "status" / "command_surface_consistency_artifact.json"


def main() -> int:
    if not CONSISTENCY.exists():
        print("warning: command_surface_consistency_artifact.json missing; skipping uncovered command warning")
        return 0

    payload = json.loads(CONSISTENCY.read_text(encoding="utf-8"))
    missing = [row for row in payload.get("todo_rows", []) if row.get("status") != "complete"]
    if missing:
        todos = ", ".join(str(row.get("todo")) for row in missing)
        print(f"warning: uncovered command-surface consistency todos detected: {todos}")
    else:
        print("no uncovered command-surface consistency todos detected")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
