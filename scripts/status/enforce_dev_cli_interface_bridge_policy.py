#!/usr/bin/env python3
"""Enforce runtime query interface bridge boundaries."""

from __future__ import annotations

import json
from pathlib import Path


REPORT = Path("artifacts/status/dev_cli_interface_bridge_report.json")


def main() -> int:
    if not REPORT.exists():
        raise SystemExit(f"missing artifact: {REPORT}")
    payload = json.loads(REPORT.read_text(encoding="utf-8"))
    failures: list[str] = []
    interfaces = payload.get("interfaces", [])
    if not interfaces:
        failures.append("interface report must include at least one query provider")
    for row in interfaces:
        if row.get("contains_json_assembly"):
            failures.append(f"query provider assembles json payloads: {row.get('path')}")
        if row.get("contains_terminal_rendering"):
            failures.append(f"query provider renders terminal text: {row.get('path')}")
        if int(row.get("public_functions", 0)) == 0:
            failures.append(f"query provider exposes no public query function: {row.get('path')}")

    if failures:
        for failure in failures:
            print(f"interface-bridge-failure: {failure}")
        return 1
    print("dev cli interface bridge policy satisfied")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

