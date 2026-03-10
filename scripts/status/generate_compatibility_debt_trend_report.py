#!/usr/bin/env python3
"""Generate compatibility debt trend artifacts for shims and aliases."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def stable_generated_at() -> str:
    source_date_epoch = subprocess.run(
        ["sh", "-lc", "printf %s \"${SOURCE_DATE_EPOCH:-}\""],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if source_date_epoch.isdigit():
        return datetime.fromtimestamp(int(source_date_epoch), tz=timezone.utc).isoformat()
    return "1970-01-01T00:00:00+00:00"


def read_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    shim = read_json(STATUS / "compatibility_shim_count_report.json")
    alias = read_json(STATUS / "compatibility_alias_count_report.json")
    shim_delta = read_json(STATUS / "compatibility_shim_count_delta.json")
    alias_delta = read_json(STATUS / "compatibility_alias_count_delta.json")

    trend = {
        "generated_at": stable_generated_at(),
        "generator": "scripts/status/generate_compatibility_debt_trend_report.py",
        "scope": "compatibility debt trend",
        "series": {
            "shims": {
                "baseline_count": int(shim.get("baseline_count", 0)),
                "current_count": int(shim.get("current_count", 0)),
                "delta_vs_baseline": int(shim_delta.get("delta", 0)),
                "removed_since_baseline": int(shim.get("removed_since_baseline", 0)),
            },
            "aliases": {
                "baseline_count": int(alias.get("baseline_count", 0)),
                "current_count": int(alias.get("current_count", 0)),
                "delta_vs_baseline": int(alias_delta.get("delta", 0)),
                "removed_since_baseline": int(alias.get("removed_since_baseline", 0)),
            },
        },
    }
    trend["status"] = (
        "improving"
        if trend["series"]["shims"]["delta_vs_baseline"] <= 0
        and trend["series"]["aliases"]["delta_vs_baseline"] <= 0
        else "regressing"
    )

    write_json(STATUS / "compatibility_debt_trend_report.json", trend)

    text = [
        "Compatibility Debt Trend Report",
        f"status: {trend['status']}",
        f"shims baseline/current/delta: {trend['series']['shims']['baseline_count']}/{trend['series']['shims']['current_count']}/{trend['series']['shims']['delta_vs_baseline']}",
        f"aliases baseline/current/delta: {trend['series']['aliases']['baseline_count']}/{trend['series']['aliases']['current_count']}/{trend['series']['aliases']['delta_vs_baseline']}",
    ]
    (STATUS / "compatibility_debt_trend_report.txt").write_text("\n".join(text) + "\n", encoding="utf-8")

    print("wrote artifacts/status/compatibility_debt_trend_report.json")
    print("wrote artifacts/status/compatibility_debt_trend_report.txt")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
