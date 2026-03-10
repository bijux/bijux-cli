#!/usr/bin/env python3
"""Enforce diagnostics taxonomy artifacts and consistency tests."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def read_json(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    failures: list[str] = []

    taxonomy = STATUS / "diagnostics_taxonomy.json"
    usefulness = STATUS / "diagnostics_usefulness_review.json"
    if not taxonomy.exists():
        failures.append("missing diagnostics taxonomy artifact")
    if not usefulness.exists():
        failures.append("missing diagnostics usefulness artifact")

    payload = read_json(taxonomy)
    rows = payload.get("taxonomy", []) if isinstance(payload, dict) else []
    required_types = {"runtime", "state", "plugin", "package", "parity", "route", "health"}
    found_types = {row.get("type") for row in rows if isinstance(row, dict)}
    missing = sorted(required_types - found_types)
    if missing:
        failures.append(f"diagnostics taxonomy missing required buckets: {', '.join(missing)}")

    test_file = ROOT / "crates/bijux-cli-bin/tests/diagnostics_contract_consistency.rs"
    if not test_file.exists():
        failures.append("diagnostics contract consistency test file is missing")

    for failure in failures:
        print(f"DIAGNOSTICS LAW FAILURE: {failure}")
    return 2 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())

