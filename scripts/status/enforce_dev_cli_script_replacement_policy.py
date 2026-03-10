#!/usr/bin/env python3
"""Ensure remaining script-only behaviors are explicitly justified."""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ALLOWLIST = ROOT / "docs" / "architecture" / "dev_cli_script_allowlist.json"


def main() -> int:
    payload = json.loads(
        (ROOT / "artifacts" / "status" / "dev_cli_control_plane_bundle.json").read_text(
            encoding="utf-8"
        )
    )
    script_audit = payload["commands"]["dev cli script-audit"]["payload"]
    remaining = sorted(script_audit.get("remaining_script_only_behaviors", []))
    allowed = sorted(json.loads(ALLOWLIST.read_text(encoding="utf-8")).get("allowed_remaining", []))
    unexpected = [item for item in remaining if item not in allowed]

    if unexpected:
        for item in unexpected:
            print(f"script-replacement-failure: remaining behavior lacks justification: {item}")
        return 1
    print("dev cli script replacement policy satisfied")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

