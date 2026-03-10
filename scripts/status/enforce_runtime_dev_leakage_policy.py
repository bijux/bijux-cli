#!/usr/bin/env python3
"""Enforce runtime dev-leakage policy from generated report."""

from __future__ import annotations

import json
from pathlib import Path


REPORT = Path("artifacts/status/runtime_dev_leakage_report.json")


def main() -> int:
    if not REPORT.exists():
        raise SystemExit(f"missing artifact: {REPORT}")
    payload = json.loads(REPORT.read_text(encoding="utf-8"))
    failures: list[str] = []
    for row in payload.get("crates", []):
        crate = row.get("crate", "unknown")
        if int(row.get("bijux_dev_cli_imports", 0)) != 0:
            failures.append(f"{crate} imports bijux-dev-cli directly")
        if int(row.get("route_audit_assembly_calls", 0)) != 0:
            failures.append(f"{crate} still assembles route-audit report")
        if int(row.get("report_builder_calls_outside_core_exception", 0)) != 0:
            failures.append(f"{crate} still assembles maintainer report builders")
        if crate != "bijux-cli" and int(row.get("dev_cli_literals", 0)) != 0:
            failures.append(f"{crate} still contains dev cli workflow literals")

    if failures:
        for failure in failures:
            print(f"runtime-dev-leakage-failure: {failure}")
        return 1
    print("runtime dev leakage policy satisfied")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

