#!/usr/bin/env python3
"""Enforce compatibility shim/alias policy gates."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def validate_items(name: str, items: list[dict], key: str) -> list[str]:
    failures: list[str] = []
    for item in items:
        label = str(item.get(key, "<unknown>"))
        classification = str(item.get("classification", ""))
        justification = str(item.get("justification", "")).strip()
        removal_plan = str(item.get("removal_plan", "")).strip()

        if classification == "permanent":
            failures.append(f"{name}: {label} is marked permanent")
        if not justification:
            failures.append(f"{name}: {label} missing justification")
        if not removal_plan:
            failures.append(f"{name}: {label} missing removal_plan")
    return failures


def main() -> int:
    failures: list[str] = []

    shims = read_json(STATUS / "compatibility_shim_inventory.json")
    aliases = read_json(STATUS / "compatibility_alias_inventory.json")

    shim_items = shims.get("items", []) if isinstance(shims, dict) else []
    alias_items = aliases.get("items", []) if isinstance(aliases, dict) else []

    if not isinstance(shim_items, list):
        failures.append("compatibility_shim_inventory.items must be a list")
        shim_items = []
    if not isinstance(alias_items, list):
        failures.append("compatibility_alias_inventory.items must be a list")
        alias_items = []

    failures.extend(validate_items("shim", [i for i in shim_items if isinstance(i, dict)], "command"))
    failures.extend(validate_items("alias", [i for i in alias_items if isinstance(i, dict)], "alias"))

    for msg in failures:
        print(f"COMPAT POLICY FAILURE: {msg}")

    return 2 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
