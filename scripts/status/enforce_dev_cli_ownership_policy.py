#!/usr/bin/env python3
"""Enforce dev-cli sole maintainer ownership policy."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def main() -> int:
    failures: list[str] = []

    ownership_path = STATUS / "dev_cli_ownership_report.json"
    if not ownership_path.exists():
        failures.append("missing artifacts/status/dev_cli_ownership_report.json")
    else:
        payload = json.loads(ownership_path.read_text(encoding="utf-8"))
        if payload.get("owner") != "bijux-dev-cli":
            failures.append("dev cli ownership report must set owner to bijux-dev-cli")
        if payload.get("namespace") != "dev cli":
            failures.append("dev cli ownership report must keep namespace as dev cli")

    allowed_scope = ROOT / "docs" / "architecture" / "dev_cli_allowed_scope.md"
    denied_scope = ROOT / "docs" / "architecture" / "dev_cli_disallowed_scope.md"
    if not allowed_scope.exists():
        failures.append("missing docs/architecture/dev_cli_allowed_scope.md")
    if not denied_scope.exists():
        failures.append("missing docs/architecture/dev_cli_disallowed_scope.md")

    if failures:
        for failure in failures:
            print(f"dev-cli-ownership-failure: {failure}")
        return 1

    print("dev cli ownership policy satisfied")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
