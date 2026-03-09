#!/usr/bin/env python3
"""Fail when generated status reports are stale in git working tree."""

from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
def run(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, cwd=ROOT, check=False, capture_output=True, text=True)


def main() -> int:
    gen = run(["python3", "scripts/status/generate_status_reports.py"])
    if gen.returncode != 0:
        print(gen.stderr.strip() or gen.stdout.strip())
        return gen.returncode

    diff = run(["git", "diff", "--name-only", "--", "artifacts/status"])
    changed = [
        line.strip()
        for line in diff.stdout.splitlines()
        if line.strip().startswith("artifacts/status/status")
        and line.strip().endswith(".json")
    ]
    if changed:
        print("STATUS REPORT STALE: regenerate and commit updated artifacts:")
        for item in changed:
            print(f" - {item}")
        return 1

    print("Status report freshness check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
