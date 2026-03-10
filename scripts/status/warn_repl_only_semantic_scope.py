#!/usr/bin/env python3
"""Warn CI if new REPL-only semantic appears without explicit justification."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DRIFT = ROOT / "artifacts" / "status" / "repl_shared_law_drift_artifact.json"


def main() -> int:
    if not DRIFT.exists():
        print("::warning title=REPL Shared Law::missing artifacts/status/repl_shared_law_drift_artifact.json")
        return 0

    payload = json.loads(DRIFT.read_text(encoding="utf-8"))
    repl_only = payload.get("repl_only_semantics", [])
    if repl_only:
        print(
            "::warning title=REPL Shared Law::"
            f"REPL-only semantics detected without explicit justification: {repl_only}"
        )
    else:
        print("repl-only semantic warning check passed")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
