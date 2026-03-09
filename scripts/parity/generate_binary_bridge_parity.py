#!/usr/bin/env python3
"""Generate binary-vs-python-bridge parity artifact."""

from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main() -> int:
    cmd = [
        "cargo",
        "test",
        "-q",
        "-p",
        "bijux-cli-python",
        "--test",
        "bridge_binary_parity_report",
        "--",
        "--nocapture",
    ]
    proc = subprocess.run(cmd, cwd=ROOT)
    return proc.returncode


if __name__ == "__main__":
    raise SystemExit(main())
