#!/usr/bin/env python3
"""Generate plugin state report with overlap parity and beyond-python markers."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "artifacts" / "status" / "plugin_state_report.json"


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


def main() -> int:
    report = {
        "generated_at": stable_generated_at(),
        "generator": "scripts/status/generate_plugin_state_report.py",
        "plugin_commands": {
            "complete": [
                "plugins list",
                "plugins inspect",
                "plugins check",
                "plugins reserved-names",
                "plugins where",
                "plugins explain",
                "plugins schema",
            ],
            "partial": [
                "plugins scaffold",
                "plugins install",
                "plugins uninstall",
                "plugins enable",
                "plugins disable",
            ],
            "python_only": [],
        },
        "beyond_python": [
            "reserved namespace diagnostics surface",
            "plugin registry origin metadata",
            "transaction rollback assertions for install/uninstall failures",
            "explicit plugin schema command",
        ],
        "overlap_parity_tests": [
            "crates/bijux-cli-plugin/tests/plugin_parity_read_paths.rs",
            "crates/bijux-cli/tests/bin_surface/plugin_command_parity.rs",
        ],
        "remaining_gaps": [
            "scaffold command parity against Python templates",
            "full CLI lifecycle command parity for install/uninstall/enable/disable",
            "end-to-end CLI plugin diagnostics parity for all failure classes",
        ],
        "frozen_law": "plugin v1 contract is frozen before expanding command cleverness",
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
