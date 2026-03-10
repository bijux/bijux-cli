#!/usr/bin/env python3
"""Warn CI when diagnostics text wording drifts from stable snapshots."""

from __future__ import annotations

import difflib
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SNAPSHOTS = {
    "dev cli doctor": ROOT / "crates" / "bijux-cli-core" / "tests" / "snapshots" / "dev_cli_doctor_text.txt",
    "dev cli routes": ROOT / "crates" / "bijux-cli-core" / "tests" / "snapshots" / "dev_cli_routes_text.txt",
    "dev cli registry": ROOT / "crates" / "bijux-cli-core" / "tests" / "snapshots" / "dev_cli_registry_text.txt",
    "dev cli env": ROOT / "crates" / "bijux-cli-core" / "tests" / "snapshots" / "dev_cli_env_text.txt",
}


def run_text(command: str) -> str:
    args = command.split()
    out = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-core", "--", *args, "--format", "text"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    return out.stdout if out.returncode == 0 else ""


def main() -> int:
    warned = 0
    for command, snapshot in SNAPSHOTS.items():
        if not snapshot.exists():
            print(f"::warning title=Diagnostics Wording::missing snapshot for `{command}` at {snapshot.relative_to(ROOT)}")
            warned += 1
            continue

        current = run_text(command)
        baseline = snapshot.read_text(encoding="utf-8")
        if current == baseline:
            continue

        diff = "\n".join(
            difflib.unified_diff(
                baseline.splitlines(),
                current.splitlines(),
                fromfile="snapshot",
                tofile="current",
                n=2,
            )
        )
        preview = diff[:600].replace("%", "%25").replace("\n", "%0A").replace("\r", "%0D")
        print(
            f"::warning title=Diagnostics Wording::text output drift for `{command}` against {snapshot.relative_to(ROOT)}%0A{preview}"
        )
        warned += 1

    if warned == 0:
        print("diagnostics wording warning check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
