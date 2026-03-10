#!/usr/bin/env python3
"""Fail CI when output/envelope and bridge-conversion fuzz hardening drifts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
REQUIRED = [
    STATUS / "output_crash_triage_artifact.json",
    STATUS / "bridge_conversion_crash_triage_artifact.json",
    STATUS / "output_fuzz_regression_artifact.json",
    STATUS / "bridge_conversion_fuzz_regression_artifact.json",
    STATUS / "output_envelope_fuzz_contract.json",
]


def main() -> int:
    missing = [path for path in REQUIRED if not path.exists()]
    if missing:
        raise SystemExit(
            "missing output/bridge fuzz artifacts: "
            + ", ".join(str(p.relative_to(ROOT)) for p in missing)
        )

    output_triage = json.loads((STATUS / "output_crash_triage_artifact.json").read_text(encoding="utf-8"))
    bridge_triage = json.loads((STATUS / "bridge_conversion_crash_triage_artifact.json").read_text(encoding="utf-8"))
    output_reg = json.loads((STATUS / "output_fuzz_regression_artifact.json").read_text(encoding="utf-8"))
    bridge_reg = json.loads((STATUS / "bridge_conversion_fuzz_regression_artifact.json").read_text(encoding="utf-8"))
    contract = json.loads((STATUS / "output_envelope_fuzz_contract.json").read_text(encoding="utf-8"))

    if output_triage.get("status") != "clean":
        raise SystemExit("output fuzz triage is not clean")
    if bridge_triage.get("status") != "clean":
        raise SystemExit("bridge conversion fuzz triage is not clean")
    if output_reg.get("status") != "clean":
        raise SystemExit("output fuzz regression drift detected")
    if bridge_reg.get("status") != "clean":
        raise SystemExit("bridge conversion fuzz regression drift detected")
    if contract.get("status") != "frozen":
        raise SystemExit(f"output/envelope fuzz contract is not frozen: {contract.get('missing_todos', [])}")

    print("output/bridge fuzz gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
