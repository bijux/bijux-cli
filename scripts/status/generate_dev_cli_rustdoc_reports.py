#!/usr/bin/env python3
"""Generate rustdoc control-plane reports from dev-cli commands."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def run(command: list[str], fmt: str) -> str:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-core", "--", *command, "--format", fmt, "--no-pretty" if fmt == "json" else ""],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr or proc.stdout)
    return proc.stdout


def run_json(command: list[str]) -> dict:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-core", "--", *command, "--format", "json", "--no-pretty"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout or "{}")


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)

    audit_json = run_json(["dev", "cli", "rustdoc", "audit"])
    coverage_json = run_json(["dev", "cli", "rustdoc", "coverage"])

    (STATUS / "rustdoc_audit_report.json").write_text(
        json.dumps(audit_json, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    (STATUS / "rustdoc_public_api_coverage_report.json").write_text(
        json.dumps(coverage_json, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )

    audit_text = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-core", "--", "dev", "cli", "rustdoc", "audit", "--format", "text"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    (STATUS / "rustdoc_audit_report.txt").write_text(audit_text, encoding="utf-8")

    print("wrote artifacts/status/rustdoc_audit_report.json")
    print("wrote artifacts/status/rustdoc_public_api_coverage_report.json")
    print("wrote artifacts/status/rustdoc_audit_report.txt")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
