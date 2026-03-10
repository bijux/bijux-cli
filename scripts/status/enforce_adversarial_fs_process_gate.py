#!/usr/bin/env python3
"""Fail CI when adversarial filesystem/process hardening drifts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

REQUIRED = [
    STATUS / "adversarial_fs_process_matrix.json",
    STATUS / "adversarial_fs_process_artifact.json",
    STATUS / "adversarial_fs_process_contract.json",
]


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    missing = [p for p in REQUIRED if not p.exists()]
    if missing:
        raise SystemExit(
            "missing adversarial fs/process artifacts: "
            + ", ".join(str(p.relative_to(ROOT)) for p in missing)
        )

    matrix = read_json(STATUS / "adversarial_fs_process_matrix.json")
    artifact = read_json(STATUS / "adversarial_fs_process_artifact.json")
    contract = read_json(STATUS / "adversarial_fs_process_contract.json")

    if matrix.get("status") != "complete":
        raise SystemExit("adversarial fs/process matrix is incomplete")
    if artifact.get("status") != "complete":
        raise SystemExit("adversarial fs/process artifact is incomplete")
    if contract.get("status") != "frozen":
        raise SystemExit(f"adversarial fs/process contract is not frozen: {contract.get('missing_todos', [])}")

    print("adversarial fs/process gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
