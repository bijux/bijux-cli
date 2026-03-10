#!/usr/bin/env python3
"""Generate machine-readable and text plugin runtime health reports."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return {}


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)
    plugin_state = load_json(STATUS / "plugin_state_report.json")
    parity = load_json(ROOT / "artifacts" / "parity" / "command_parity_matrix.json")

    plugin_commands = plugin_state.get("plugin_commands", {})
    complete = plugin_commands.get("complete", [])
    partial = plugin_commands.get("partial", [])
    gaps = plugin_state.get("remaining_gaps", [])

    parity_rows = [
        row
        for row in parity.get("rows", [])
        if str(row.get("group", "")).startswith("plugin")
        or str(row.get("command", "")).startswith("cli plugins")
    ]
    parity_covered = sum(1 for row in parity_rows if row.get("status") == "complete")

    healthy = not partial and not gaps

    machine = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "schema": "plugin-health-report-v1",
        "status": "healthy" if healthy else "degraded",
        "plugin_commands": {
            "complete": complete,
            "partial": partial,
            "complete_count": len(complete),
            "partial_count": len(partial),
        },
        "parity": {
            "plugin_rows": len(parity_rows),
            "complete_rows": parity_covered,
        },
        "remaining_gaps": gaps,
        "overlap_parity_tests": plugin_state.get("overlap_parity_tests", []),
    }

    text = "\n".join(
        [
            "Plugin Runtime Health",
            f"status: {machine['status']}",
            f"complete commands: {len(complete)}",
            f"partial commands: {len(partial)}",
            f"plugin parity rows complete: {parity_covered}/{len(parity_rows)}",
            f"remaining gaps: {len(gaps)}",
            "",
            "Use `bijux dev cli plugin-health --format json` for machine-readable details.",
        ]
    ) + "\n"

    (STATUS / "plugin_health_report.json").write_text(
        json.dumps(machine, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    (STATUS / "plugin_health_report.txt").write_text(text, encoding="utf-8")
    print("wrote artifacts/status/plugin_health_report.json")
    print("wrote artifacts/status/plugin_health_report.txt")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
