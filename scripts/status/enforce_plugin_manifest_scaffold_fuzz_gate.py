#!/usr/bin/env python3
"""Fail CI when plugin manifest/scaffold fuzz hardening drifts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
REQUIRED = [
    STATUS / "plugin_manifest_crash_triage_artifact.json",
    STATUS / "plugin_scaffold_crash_triage_artifact.json",
    STATUS / "plugin_manifest_fuzz_regression_artifact.json",
    STATUS / "plugin_scaffold_fuzz_regression_artifact.json",
    STATUS / "plugin_manifest_scaffold_fuzz_contract.json",
]


def main() -> int:
    missing = [path for path in REQUIRED if not path.exists()]
    if missing:
        raise SystemExit(
            "missing plugin manifest/scaffold fuzz artifacts: "
            + ", ".join(str(p.relative_to(ROOT)) for p in missing)
        )

    manifest_triage = json.loads((STATUS / "plugin_manifest_crash_triage_artifact.json").read_text(encoding="utf-8"))
    scaffold_triage = json.loads((STATUS / "plugin_scaffold_crash_triage_artifact.json").read_text(encoding="utf-8"))
    manifest_reg = json.loads((STATUS / "plugin_manifest_fuzz_regression_artifact.json").read_text(encoding="utf-8"))
    scaffold_reg = json.loads((STATUS / "plugin_scaffold_fuzz_regression_artifact.json").read_text(encoding="utf-8"))
    contract = json.loads((STATUS / "plugin_manifest_scaffold_fuzz_contract.json").read_text(encoding="utf-8"))

    if manifest_triage.get("status") != "clean":
        raise SystemExit("plugin manifest fuzz triage is not clean")
    if scaffold_triage.get("status") != "clean":
        raise SystemExit("plugin scaffold fuzz triage is not clean")
    if manifest_reg.get("status") != "clean":
        raise SystemExit("plugin manifest fuzz regression drift detected")
    if scaffold_reg.get("status") != "clean":
        raise SystemExit("plugin scaffold fuzz regression drift detected")
    if contract.get("status") != "frozen":
        raise SystemExit(
            f"plugin manifest/scaffold fuzz contract is not frozen: {contract.get('missing_todos', [])}"
        )

    print("plugin manifest/scaffold fuzz gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
