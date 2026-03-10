#!/usr/bin/env python3
"""Fail CI when route/registry fuzz regressions drift."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
REQUIRED = [
    STATUS / "route_crash_triage_artifact.json",
    STATUS / "registry_crash_triage_artifact.json",
    STATUS / "route_fuzz_regression_artifact.json",
    STATUS / "registry_fuzz_regression_artifact.json",
    STATUS / "route_registry_fuzz_contract.json",
]


def main() -> int:
    missing = [path for path in REQUIRED if not path.exists()]
    if missing:
        raise SystemExit(
            "missing route/registry fuzz artifacts: "
            + ", ".join(str(p.relative_to(ROOT)) for p in missing)
        )

    route_triage = json.loads((STATUS / "route_crash_triage_artifact.json").read_text(encoding="utf-8"))
    if not route_triage.get("regression_replay_ok", False):
        raise SystemExit("route fuzz replay failed")

    registry_triage = json.loads((STATUS / "registry_crash_triage_artifact.json").read_text(encoding="utf-8"))
    if not registry_triage.get("regression_replay_ok", False):
        raise SystemExit("registry fuzz replay failed")

    route_reg = json.loads((STATUS / "route_fuzz_regression_artifact.json").read_text(encoding="utf-8"))
    if route_reg.get("status") != "clean":
        raise SystemExit("route fuzz regression drift detected")

    registry_reg = json.loads((STATUS / "registry_fuzz_regression_artifact.json").read_text(encoding="utf-8"))
    if registry_reg.get("status") != "clean":
        raise SystemExit("registry fuzz regression drift detected")

    contract = json.loads((STATUS / "route_registry_fuzz_contract.json").read_text(encoding="utf-8"))
    if contract.get("status") != "frozen":
        raise SystemExit(f"route/registry fuzz contract not frozen: {contract.get('missing_todos', [])}")

    print("route/registry fuzz gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
