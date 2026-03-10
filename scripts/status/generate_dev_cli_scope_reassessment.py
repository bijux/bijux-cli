#!/usr/bin/env python3
"""Reassess runtime responsibilities against dev-cli control-plane rules."""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)
    runtime_leakage = json.loads((STATUS / "runtime_dev_leakage_report.json").read_text(encoding="utf-8"))
    interface_bridge = json.loads((STATUS / "dev_cli_interface_bridge_report.json").read_text(encoding="utf-8"))
    dispatch = json.loads((STATUS / "dev_cli_dispatch_ownership_report.json").read_text(encoding="utf-8"))

    violations = []
    if runtime_leakage.get("status") != "ok":
        violations.append("runtime leakage report is not green")
    if any(row.get("contains_json_assembly") for row in interface_bridge.get("interfaces", [])):
        violations.append("query bridge still assembles presentation json")
    if dispatch.get("checks", {}).get("bin_has_direct_dispatch_match_arms"):
        violations.append("bin owns direct dispatch match arms")

    reassessment = {
        "scope": "runtime responsibility reassessment",
        "status": "ok" if not violations else "degraded",
        "violations": violations,
        "decision": "no remaining runtime responsibilities violate the current dev-cli control-plane standard"
        if not violations
        else "runtime responsibilities still violate control-plane standard",
    }
    (STATUS / "runtime_responsibility_reassessment.json").write_text(
        json.dumps(reassessment, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

