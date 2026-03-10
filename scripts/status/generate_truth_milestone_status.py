#!/usr/bin/env python3
"""Generate milestone truth status artifacts for done/left/partial/deferred/different."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS_DIR = ROOT / "artifacts" / "status"
PARITY_MATRIX = ROOT / "artifacts" / "parity" / "command_parity_matrix.json"
CURRENT_STATE = ROOT / "artifacts" / "status" / "current_rust_state.json"
PLUGIN_STATE = ROOT / "artifacts" / "status" / "plugin_state_report.json"


def read_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def status_rows() -> list[dict[str, Any]]:
    matrix = read_json(PARITY_MATRIX)
    rows = matrix.get("commands", []) if isinstance(matrix, dict) else []
    return [row for row in rows if isinstance(row, dict) and row.get("command")]


def write_status(name: str, payload: dict[str, Any]) -> None:
    out = STATUS_DIR / f"{name}.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def split_commands(rows: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], list[dict[str, Any]], list[dict[str, Any]], list[dict[str, Any]]]:
    done: list[dict[str, Any]] = []
    partial: list[dict[str, Any]] = []
    left: list[dict[str, Any]] = []
    intentionally_different: list[dict[str, Any]] = []

    for row in rows:
        status = str(row.get("status", "missing"))
        record = {
            "command": row.get("command"),
            "group": row.get("group", "unknown"),
            "owner": row.get("owner", "unassigned"),
            "confidence": row.get("confidence", 0.0),
            "blocker": row.get("blocker", ""),
            "reason": row.get("reason", ""),
        }
        if status == "complete":
            done.append(record)
        elif status == "partial":
            partial.append(record)
        elif status in {"intentionally-different", "different-by-decision"}:
            intentionally_different.append(record)
        else:
            left.append(record)

    for bucket in (done, partial, left, intentionally_different):
        bucket.sort(key=lambda item: str(item["command"]))

    return done, partial, left, intentionally_different


def deferred_items(current_state: dict[str, Any], plugin_state: dict[str, Any]) -> list[dict[str, Any]]:
    deferred: list[dict[str, Any]] = [
        {
            "area": "runtime-channels",
            "item": "homebrew smoke test",
            "reason": "formula publishing is not enabled yet",
            "evidence": ".github/workflows/ci.yml",
        },
        {
            "area": "plugin-lifecycle",
            "item": "full scaffold/install/uninstall parity",
            "reason": "plugin command coverage is still partial in current report",
            "evidence": "artifacts/status/plugin_state_report.json",
        },
        {
            "area": "parity",
            "item": "global complete parity claim",
            "reason": "command parity matrix still contains partial and missing commands",
            "evidence": "artifacts/parity/command_parity_matrix.json",
        },
    ]

    runtime_rules = current_state.get("runtime_identity_rules", {})
    if isinstance(runtime_rules, dict) and not runtime_rules.get("canonical_runtime_is_bijux", False):
        deferred.append(
            {
                "area": "runtime-identity",
                "item": "canonical runtime identity",
                "reason": "runtime identity is not fully converged",
                "evidence": "artifacts/status/current_rust_state.json",
            }
        )

    remaining_gaps = plugin_state.get("remaining_gaps", [])
    if isinstance(remaining_gaps, list):
        for gap in remaining_gaps[:6]:
            deferred.append(
                {
                    "area": "plugin",
                    "item": str(gap),
                    "reason": "explicit plugin gap still open",
                    "evidence": "artifacts/status/plugin_state_report.json",
                }
            )

    return deferred


def main() -> int:
    generated_at = datetime.now(timezone.utc).isoformat()
    rows = status_rows()
    current_state = read_json(CURRENT_STATE)
    plugin_state = read_json(PLUGIN_STATE)

    done, partial, left, intentionally_different = split_commands(rows)
    deferred = deferred_items(current_state, plugin_state)

    shared = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_truth_milestone_status.py",
        "source": {
            "command_matrix": str(PARITY_MATRIX.relative_to(ROOT)),
            "current_state": str(CURRENT_STATE.relative_to(ROOT)),
            "plugin_state": str(PLUGIN_STATE.relative_to(ROOT)),
        },
    }

    write_status(
        "what_is_done",
        {
            **shared,
            "summary": {"count": len(done), "scope": "commands marked complete in parity matrix"},
            "items": done,
        },
    )
    write_status(
        "what_is_partial",
        {
            **shared,
            "summary": {"count": len(partial), "scope": "commands marked partial in parity matrix"},
            "items": partial,
        },
    )
    write_status(
        "what_is_left",
        {
            **shared,
            "summary": {"count": len(left), "scope": "commands marked missing in parity matrix"},
            "items": left,
        },
    )
    write_status(
        "what_is_deferred",
        {
            **shared,
            "summary": {"count": len(deferred), "scope": "known intentionally deferred work"},
            "items": deferred,
        },
    )
    write_status(
        "what_is_intentionally_different",
        {
            **shared,
            "summary": {
                "count": len(intentionally_different),
                "scope": "commands marked intentionally-different in parity matrix",
            },
            "items": intentionally_different,
        },
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
