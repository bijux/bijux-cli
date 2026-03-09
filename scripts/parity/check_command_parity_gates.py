#!/usr/bin/env python3
"""Enforce parity matrix regression and drift gates."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CURRENT = ROOT / "artifacts" / "parity" / "command_parity_matrix.json"
BASELINE = ROOT / "docs" / "architecture" / "parity" / "baseline_command_parity_matrix.json"


STATUS_RANK = {
    "complete": 4,
    "intentionally-different": 3,
    "partial": 2,
    "missing": 1,
}


def read(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def index_commands(data: dict) -> dict[str, dict]:
    return {row["command"]: row for row in data.get("commands", []) if row.get("command")}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--enforce", action="store_true")
    args = parser.parse_args()

    current = read(CURRENT)
    baseline = read(BASELINE)
    if not current:
        print("missing current parity matrix")
        return 2 if args.enforce else 0

    current_map = index_commands(current)
    baseline_map = index_commands(baseline)

    failures: list[str] = []
    warnings: list[str] = []

    for command, old in baseline_map.items():
        new = current_map.get(command)
        if not new:
            failures.append(f"command disappeared from matrix: {command}")
            continue

        old_rank = STATUS_RANK.get(old.get("status", "missing"), 0)
        new_rank = STATUS_RANK.get(new.get("status", "missing"), 0)

        if old_rank > new_rank and old.get("status") == "complete":
            failures.append(
                f"parity-covered command regressed: {command} ({old.get('status')} -> {new.get('status')})"
            )

        if old.get("status") == "partial" and new.get("status") == "partial":
            old_conf = float(old.get("confidence", 0.0))
            new_conf = float(new.get("confidence", 0.0))
            if new_conf + 0.1 < old_conf:
                warnings.append(
                    f"parity-partial command drifted further away: {command} ({old_conf:.2f} -> {new_conf:.2f})"
                )

    for msg in warnings:
        print(f"PARITY WARNING: {msg}")

    if failures:
        for msg in failures:
            print(f"PARITY FAILURE: {msg}")
        return 2 if args.enforce else 0

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
