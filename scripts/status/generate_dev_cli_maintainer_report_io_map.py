#!/usr/bin/env python3
"""Generate maintainer report inputs-vs-outputs map for dev-cli report commands."""

from __future__ import annotations

import json
import pathlib
import subprocess
from datetime import datetime, timezone

ROOT = pathlib.Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

COMMANDS = [
    "dev cli env",
    "dev cli contracts",
    "dev cli parity",
    "dev cli status",
]

INPUTS = {
    "dev cli env": [
        "process environment",
        "resolved config/history/plugins paths",
    ],
    "dev cli contracts": [
        "static schema contract declarations",
        "runtime version",
    ],
    "dev cli parity": [
        "artifacts/parity/*.json",
        "artifacts/parity/*.txt",
    ],
    "dev cli status": [
        "artifacts/status/*.json",
        "artifacts/status/*.txt",
        "artifacts/parity/rust_python_parity_report.json",
        "dev-cli inventory payload",
    ],
}


def run_json(command: str) -> dict:
    args = command.split()
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-core", "--", *args, "--format", "json", "--no-pretty"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(proc.stdout or "{}")


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)

    rows = []
    for command in COMMANDS:
        payload = run_json(command)
        top_keys = sorted(payload.keys()) if isinstance(payload, dict) else []
        rows.append(
            {
                "command": command,
                "inputs": INPUTS[command],
                "output_top_level_keys": top_keys,
                "output_kind": "json-object" if isinstance(payload, dict) else "non-object",
            }
        )

    out = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "generator": "scripts/status/generate_dev_cli_maintainer_report_io_map.py",
        "scope": "dev-cli maintainer report inputs vs outputs",
        "reports": rows,
    }
    path = STATUS / "dev_cli_maintainer_report_io_map.json"
    path.write_text(json.dumps(out, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {path.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
