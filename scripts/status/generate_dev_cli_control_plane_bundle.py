#!/usr/bin/env python3
"""Generate a dedicated artifact bundle for bijux-dev-cli control-plane status."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

COMMANDS = [
    "dev cli status",
    "dev cli parity",
    "dev cli runtime-identity",
    "dev cli state-audit",
    "dev cli package-health",
    "dev cli script-audit",
    "dev cli docs-audit",
    "dev cli crate-health",
]


def run_json(command: str) -> dict:
    args = command.split()
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-bin", "--", *args, "--format", "json", "--no-pretty"],
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout or "{}")


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)
    bundle = {"scope": "bijux-dev-cli control-plane bundle", "commands": {}}
    for command in COMMANDS:
        payload = run_json(command)
        bundle["commands"][command] = {
            "top_level_keys": sorted(payload.keys()),
            "payload": payload,
        }

    (STATUS / "dev_cli_control_plane_bundle.json").write_text(
        json.dumps(bundle, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

