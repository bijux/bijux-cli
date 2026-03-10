#!/usr/bin/env python3
"""Generate canonical dev-cli command ownership report."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

COMMAND_ROWS = [
    {"command": "dev cli status", "group": "dashboard", "visible": True},
    {"command": "dev cli parity", "group": "dashboard", "visible": True},
    {"command": "dev cli doctor", "group": "dashboard", "visible": True},
    {"command": "dev cli routes", "group": "routing", "visible": True},
    {"command": "dev cli registry", "group": "routing", "visible": True},
    {"command": "dev cli route-audit", "group": "routing", "visible": True},
    {"command": "dev cli env", "group": "runtime", "visible": True},
    {"command": "dev cli contracts", "group": "runtime", "visible": True},
    {"command": "dev cli runtime-identity", "group": "runtime", "visible": True},
    {"command": "dev cli package-health", "group": "runtime", "visible": True},
    {"command": "dev cli state-audit", "group": "runtime", "visible": True},
    {"command": "dev cli state-doctor", "group": "runtime", "visible": True},
    {"command": "dev cli plugin-health", "group": "runtime", "visible": True},
    {"command": "dev cli docs-audit", "group": "audit", "visible": True},
    {"command": "dev cli scripts", "group": "audit", "visible": True},
    {"command": "dev cli script-audit", "group": "audit", "visible": True},
    {"command": "dev cli crate-health", "group": "audit", "visible": True},
    {"command": "dev cli snapshots-audit", "group": "audit", "visible": True},
    {"command": "dev cli fixture-audit", "group": "audit", "visible": True},
    {"command": "dev cli docs", "group": "audit", "visible": False},
    {"command": "dev cli docs-prune-plan", "group": "audit", "visible": False},
    {"command": "dev cli inventory", "group": "internal", "visible": False},
    {"command": "dev cli atlas", "group": "internal", "visible": False},
    {"command": "dev cli di", "group": "internal", "visible": False},
    {"command": "dev cli list-products", "group": "internal", "visible": False},
    {"command": "dev cli list-plugins", "group": "internal", "visible": False},
]


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)

    report = {
        "namespace": "dev cli",
        "owner": "bijux-dev-cli",
        "commands": [{**row, "owner": "bijux-dev-cli"} for row in COMMAND_ROWS],
        "summary": {
            "total": len(COMMAND_ROWS),
            "visible": sum(1 for row in COMMAND_ROWS if row["visible"]),
            "internal": sum(1 for row in COMMAND_ROWS if not row["visible"]),
            "groups": sorted({row["group"] for row in COMMAND_ROWS}),
        },
    }

    json_path = STATUS / "dev_cli_ownership_report.json"
    text_path = STATUS / "dev_cli_ownership_report.txt"

    json_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    lines = [
        "Dev CLI ownership report",
        "owner: bijux-dev-cli",
        "namespace: dev cli",
        "",
    ]
    for row in COMMAND_ROWS:
        visibility = "visible" if row["visible"] else "internal"
        lines.append(f"- {row['command']} [{row['group']}, {visibility}]")
    text_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(f"wrote {json_path.relative_to(ROOT)}")
    print(f"wrote {text_path.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
