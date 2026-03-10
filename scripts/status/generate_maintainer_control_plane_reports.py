#!/usr/bin/env python3
"""Generate maintainer control-plane inventory and cockpit evidence reports."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

REQUIRED_COMMANDS = [
    "dev cli status",
    "dev cli parity",
    "dev cli route-audit",
    "dev cli state-audit",
    "dev cli script-audit",
    "dev cli crate-health",
    "dev cli package-health",
    "dev cli docs-audit",
]

REPLACEMENTS = {
    "scripts/check-package-metadata.py": "bijux dev cli scripts package-metadata --format json --no-pretty",
    "scripts/check_e2e_contract.py": "bijux dev cli scripts e2e-contract --format json --no-pretty",
    "scripts/helper_pip_audit.py": "bijux dev cli scripts pip-audit --format json --no-pretty",
    "scripts/capture_python_behavior.py": "bijux dev cli scripts capture-python-behavior --format json --no-pretty",
    "scripts/generate-provenance-statement.sh": "bijux dev cli scripts provenance-statement --tag <tag> --output-dir <dir> --format json --no-pretty",
}


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


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def script_inventory() -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for path in sorted((ROOT / "scripts").rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(ROOT).as_posix()
        if "/__pycache__/" in rel or rel.endswith(".pyc"):
            continue
        if rel.startswith("scripts/status/") or rel.startswith("scripts/parity/"):
            continue
        replacement = REPLACEMENTS.get(rel, "")
        out.append(
            {
                "path": rel,
                "replacement_command": replacement,
                "status": "replaced" if replacement else "remaining",
            }
        )
    return out


def main() -> None:
    generated_at = stable_generated_at()
    base = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_maintainer_control_plane_reports.py",
    }
    command_samples = read_json(STATUS / "dev_cli_control_plane_samples.json")
    inventory_rows = script_inventory()

    write_json(
        STATUS / "maintainer_scripts_outside_dev_cli.json",
        {
            **base,
            "scripts": inventory_rows,
            "summary": {
                "total": len(inventory_rows),
                "replaced": sum(1 for row in inventory_rows if row["status"] == "replaced"),
                "remaining": sum(1 for row in inventory_rows if row["status"] == "remaining"),
            },
        },
    )

    command_rows = []
    for command in REQUIRED_COMMANDS:
        sample = command_samples.get(command, {}) if isinstance(command_samples, dict) else {}
        command_rows.append(
            {
                "command": command,
                "json_sample_present": bool(sample.get("json")),
                "text_sample_present": bool(sample.get("text")),
                "json_top_level_keys": sample.get("json_top_level_keys", []),
            }
        )
    write_json(
        STATUS / "maintainer_control_plane_commands.json",
        {
            **base,
            "required_commands": REQUIRED_COMMANDS,
            "commands": command_rows,
        },
    )

    lines = [
        "Maintainer control plane summary",
        f"Generated at: {generated_at}",
        "",
    ]
    for row in command_rows:
        keys = ", ".join(row["json_top_level_keys"]) if row["json_top_level_keys"] else "(none)"
        lines.append(f"- {row['command']}: json_keys={keys}")
    lines.extend(
        [
            "",
            "Default maintainer command: bijux dev cli status",
            "Policy: use dev cli command surfaces before creating new ad-hoc scripts.",
        ]
    )
    (STATUS / "maintainer_control_plane_text_report.txt").write_text(
        "\n".join(lines) + "\n", encoding="utf-8"
    )

    write_json(
        STATUS / "maintainer_control_plane_report.json",
        {
            **base,
            "scripts_outside_dev_cli": read_json(STATUS / "maintainer_scripts_outside_dev_cli.json"),
            "commands": read_json(STATUS / "maintainer_control_plane_commands.json"),
            "text_report": "artifacts/status/maintainer_control_plane_text_report.txt",
        },
    )

    print("wrote artifacts/status/maintainer_scripts_outside_dev_cli.json")
    print("wrote artifacts/status/maintainer_control_plane_commands.json")
    print("wrote artifacts/status/maintainer_control_plane_text_report.txt")
    print("wrote artifacts/status/maintainer_control_plane_report.json")


if __name__ == "__main__":
    main()
