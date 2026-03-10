#!/usr/bin/env python3
"""Enforce dev-cli maintainer control-plane coverage and output stability."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(name: str) -> dict:
    path = STATUS / name
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    failures: list[str] = []
    required = [
        "maintainer_scripts_outside_dev_cli.json",
        "maintainer_control_plane_commands.json",
        "maintainer_control_plane_report.json",
    ]
    for name in required:
        if not (STATUS / name).exists():
            failures.append(f"missing maintainer artifact: artifacts/status/{name}")

    scripts = read_json("maintainer_scripts_outside_dev_cli.json")
    commands = read_json("maintainer_control_plane_commands.json")
    rows = commands.get("commands", []) if isinstance(commands, dict) else []
    for row in rows:
        if not isinstance(row, dict):
            failures.append("maintainer command row is not an object")
            continue
        command = row.get("command", "")
        if not row.get("json_sample_present"):
            failures.append(f"missing json sample for {command}")
        if not row.get("text_sample_present"):
            failures.append(f"missing text sample for {command}")
        keys = row.get("json_top_level_keys", [])
        if not isinstance(keys, list) or not keys:
            failures.append(f"json top-level keys missing for {command}")

    summary = scripts.get("summary", {}) if isinstance(scripts, dict) else {}
    if int(summary.get("remaining", 0)) < 0:
        failures.append("maintainer script summary is invalid")

    text_report = STATUS / "maintainer_control_plane_text_report.txt"
    if not text_report.exists():
        failures.append("missing maintainer text report")
    else:
        text = text_report.read_text(encoding="utf-8")
        if "Default maintainer command: bijux dev cli status" not in text:
            failures.append("maintainer text report missing default status command line")

    if failures:
        print("MAINTAINER CONTROL-PLANE POLICY VIOLATION")
        for failure in failures:
            print(f" - {failure}")
        return 1
    print("Maintainer control-plane policy passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
