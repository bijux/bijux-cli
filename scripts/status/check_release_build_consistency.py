#!/usr/bin/env python3
"""Build release binary twice and report hash consistency."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
BIN = ROOT / "target" / "release" / "bijux-rs"


def run_build(env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["cargo", "build", "-p", "bijux-cli-bin", "--release"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
        env=env,
    )


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    env = os.environ.copy()
    env["SOURCE_DATE_EPOCH"] = env.get("SOURCE_DATE_EPOCH", "1")

    first = run_build(env)
    hash1 = sha(BIN) if first.returncode == 0 and BIN.exists() else ""

    second = run_build(env)
    hash2 = sha(BIN) if second.returncode == 0 and BIN.exists() else ""

    ok = first.returncode == 0 and second.returncode == 0 and hash1 == hash2
    payload = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "generator": "scripts/status/check_release_build_consistency.py",
        "ok": ok,
        "builds": [
            {"run": 1, "exit_code": first.returncode, "hash": hash1},
            {"run": 2, "exit_code": second.returncode, "hash": hash2},
        ],
        "note": "if this check fails in some environments, use it as a warning signal before release claims",
    }

    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / "release_build_consistency_report.json").write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )

    if not ok:
        print("RELEASE BUILD CONSISTENCY CHECK FAILED")
        return 1
    print("Release build consistency check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
