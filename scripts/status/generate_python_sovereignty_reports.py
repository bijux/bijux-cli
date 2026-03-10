#!/usr/bin/env python3
"""Generate Python sovereignty and de-sovereignization artifacts."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def run_json(args: list[str]) -> dict:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli", "--", *args, "--format", "json", "--no-pretty"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout or "{}")


def run_text(args: list[str]) -> str:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli", "--", *args, "--format", "text"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return proc.stdout


def write_json(name: str, payload: dict) -> None:
    out = STATUS / name
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out.relative_to(ROOT)}")


def write_text(name: str, text: str) -> None:
    out = STATUS / name
    out.write_text(text, encoding="utf-8")
    print(f"wrote {out.relative_to(ROOT)}")


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)
    bridge = run_json(["dev", "cli", "python", "bridge-status"])
    surface = run_json(["dev", "cli", "python", "surface-status"])
    sovereignty = run_json(["dev", "cli", "python", "sovereignty-audit"])
    drift = run_json(["dev", "cli", "python", "drift"])
    packaging = run_json(["dev", "cli", "python", "packaging"])

    write_json("python_bridge_status_report.json", bridge)
    write_json("python_surface_status_report.json", surface)
    write_json("python_sovereignty_audit_report.json", sovereignty)
    write_json("python_desovereignization_report.json", sovereignty)
    write_text("python_desovereignization_report.txt", run_text(["dev", "cli", "python", "sovereignty-audit"]))
    write_json("python_drift_report.json", drift)
    write_json("python_packaging_direction_report.json", packaging)
    write_json(
        "python_surface_direction_contract.json",
        {
            "direction": "python-surface-over-rust-core",
            "status": sovereignty.get("status", "needs-work"),
            "evidence_ids": sovereignty.get("evidence_ids", []),
        },
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
