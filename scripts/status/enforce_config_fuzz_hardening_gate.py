#!/usr/bin/env python3
"""Fail CI when config fuzz hardening artifacts drift."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
REQUIRED = [
    STATUS / "config_parser_crash_triage_artifact.json",
    STATUS / "config_serializer_crash_triage_artifact.json",
    STATUS / "config_fuzz_regression_artifact.json",
    STATUS / "config_fuzz_contract.json",
]


def main() -> int:
    missing = [path for path in REQUIRED if not path.exists()]
    if missing:
        raise SystemExit(
            "missing config fuzz artifacts: " + ", ".join(str(p.relative_to(ROOT)) for p in missing)
        )

    parser = json.loads((STATUS / "config_parser_crash_triage_artifact.json").read_text(encoding="utf-8"))
    serializer = json.loads(
        (STATUS / "config_serializer_crash_triage_artifact.json").read_text(encoding="utf-8")
    )
    regression = json.loads((STATUS / "config_fuzz_regression_artifact.json").read_text(encoding="utf-8"))
    contract = json.loads((STATUS / "config_fuzz_contract.json").read_text(encoding="utf-8"))

    if parser.get("status") != "clean":
        raise SystemExit("config parser fuzz triage not clean")
    if serializer.get("status") != "clean":
        raise SystemExit("config serializer fuzz triage not clean")
    if regression.get("status") != "clean":
        raise SystemExit("config fuzz regression drift detected")
    if contract.get("status") != "frozen":
        raise SystemExit(f"config fuzz contract not frozen: {contract.get('missing_todos', [])}")

    print("config fuzz hardening gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
