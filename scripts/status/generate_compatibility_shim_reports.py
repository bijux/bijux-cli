#!/usr/bin/env python3
"""Generate compatibility shim and alias inventory artifacts."""

from __future__ import annotations

import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
BASELINE = ROOT / "scripts" / "status" / "compatibility_baseline.json"
REGISTRY = ROOT / "crates" / "bijux-cli-routing" / "src" / "registry.rs"


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


def alias_pairs() -> list[tuple[str, str]]:
    text = REGISTRY.read_text(encoding="utf-8")
    return re.findall(r'\("([^"]+)"\.to_string\(\),\s*"([^"]+)"\.to_string\(\)\)', text)


def shim_inventory(status_rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    items = []
    for row in status_rows:
        if row.get("status") != "shim":
            continue
        command = str(row.get("command", ""))
        matrix_status = str(row.get("matrix_status", ""))
        confidence = float(row.get("confidence", 0.0) or 0.0)
        blocker = str(row.get("blocker", ""))
        if matrix_status == "complete" and confidence >= 0.9:
            classification = "delete-now"
            justification = "parity coverage is complete and confidence is high"
            removal_plan = "remove alias and enforce canonical route"
        elif blocker:
            classification = "needed"
            justification = f"blocked by {blocker}"
            removal_plan = "remove after blocker closes and regression tests are green"
        else:
            classification = "temporary"
            justification = "compatibility bridge remains until the next cleanup window"
            removal_plan = "delete once canonical usage is stable and covered"
        items.append(
            {
                "command": command,
                "classification": classification,
                "justification": justification,
                "removal_plan": removal_plan,
                "matrix_status": matrix_status,
                "confidence": confidence,
                "blocker": blocker,
            }
        )
    items.sort(key=lambda item: (item["classification"], item["command"]))
    return items


def alias_inventory(pairs: list[tuple[str, str]]) -> list[dict[str, Any]]:
    items = []
    for alias, canonical in sorted(pairs):
        if alias.startswith("dev "):
            classification = "temporary"
            justification = "legacy developer shortcut retained during command transition"
            removal_plan = "drop after one stable release cycle with canonical-only docs"
        elif alias.startswith("config ") or alias.startswith("plugins "):
            classification = "needed"
            justification = "legacy compatibility for core operator workflows"
            removal_plan = "remove when compatibility policy no longer requires shorthand"
        else:
            classification = "temporary"
            justification = "legacy root shorthand remains for transition"
            removal_plan = "remove once canonical route adoption is complete"
        items.append(
            {
                "alias": alias,
                "canonical": canonical,
                "classification": classification,
                "justification": justification,
                "removal_plan": removal_plan,
            }
        )
    return items


def main() -> None:
    generated_at = stable_generated_at()
    baseline = read_json(BASELINE)
    status = read_json(STATUS / "status.json")
    rows = status.get("commands", []) if isinstance(status, dict) else []

    shims = shim_inventory([row for row in rows if isinstance(row, dict)])
    aliases = alias_inventory(alias_pairs())

    before_shim = int(baseline.get("baseline_shim_count", 0))
    before_alias = int(baseline.get("baseline_alias_count", 0))

    shim_payload = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_compatibility_shim_reports.py",
        "rule": "remaining shims require justification and removal plan",
        "items": shims,
        "summary": {
            "count": len(shims),
            "baseline_count": before_shim,
            "removed_since_baseline": before_shim - len(shims),
        },
    }
    alias_payload = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_compatibility_shim_reports.py",
        "rule": "remaining aliases require justification and removal plan",
        "items": aliases,
        "summary": {
            "count": len(aliases),
            "baseline_count": before_alias,
            "removed_since_baseline": before_alias - len(aliases),
        },
    }

    write_json(STATUS / "compatibility_shim_inventory.json", shim_payload)
    write_json(STATUS / "compatibility_alias_inventory.json", alias_payload)
    write_json(
        STATUS / "compatibility_shim_count_delta.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_compatibility_shim_reports.py",
            "before": before_shim,
            "after": len(shims),
            "delta": len(shims) - before_shim,
        },
    )
    write_json(
        STATUS / "compatibility_alias_count_delta.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_compatibility_shim_reports.py",
            "before": before_alias,
            "after": len(aliases),
            "delta": len(aliases) - before_alias,
        },
    )
    write_json(
        STATUS / "live_compatibility_shims.json",
        {
            "generated_at": generated_at,
            "items": shims,
        },
    )
    write_json(
        STATUS / "live_compatibility_aliases.json",
        {
            "generated_at": generated_at,
            "items": aliases,
        },
    )

    print("wrote artifacts/status/compatibility_shim_inventory.json")
    print("wrote artifacts/status/compatibility_alias_inventory.json")
    print("wrote artifacts/status/compatibility_shim_count_delta.json")
    print("wrote artifacts/status/compatibility_alias_count_delta.json")
    print("wrote artifacts/status/live_compatibility_shims.json")
    print("wrote artifacts/status/live_compatibility_aliases.json")


if __name__ == "__main__":
    main()
