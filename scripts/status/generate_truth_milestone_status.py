#!/usr/bin/env python3
"""Generate milestone truth status artifacts for done/left/partial/deferred/different/unproven."""

from __future__ import annotations

import json
import subprocess
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


def unproven_items(
    rows: list[dict[str, Any]],
    current_state: dict[str, Any],
    plugin_state: dict[str, Any],
) -> list[dict[str, Any]]:
    items: list[dict[str, Any]] = []

    if rows:
        uncovered = [
            row.get("command")
            for row in rows
            if row.get("status") == "partial" and float(row.get("confidence", 0.0)) < 0.5
        ]
        if uncovered:
            items.append(
                {
                    "area": "parity-confidence",
                    "item": "low-confidence partial parity coverage",
                    "reason": "partial rows still exist with low confidence scores",
                    "evidence": "artifacts/parity/command_parity_matrix.json",
                    "commands": uncovered[:40],
                }
            )

    runtime_parity = current_state.get("runtime_parity_assertions", {})
    if isinstance(runtime_parity, dict) and runtime_parity.get("violations"):
        items.append(
            {
                "area": "runtime-law",
                "item": "runtime law assertions unresolved",
                "reason": "runtime parity assertions report violations",
                "evidence": "artifacts/status/current_rust_state.json",
            }
        )

    plugin_partial = plugin_state.get("plugin_commands", {}).get("partial", [])
    if isinstance(plugin_partial, list) and plugin_partial:
        items.append(
            {
                "area": "plugin-lifecycle",
                "item": "plugin lifecycle coverage not fully proven",
                "reason": "plugin commands still marked partial",
                "evidence": "artifacts/status/plugin_state_report.json",
                "commands": plugin_partial,
            }
        )

    required = [
        ROOT / "artifacts" / "parity" / "command_parity_matrix.json",
        ROOT / "artifacts" / "status" / "runtime_unity_report.json",
        ROOT / "artifacts" / "status" / "plugin_state_report.json",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        items.append(
            {
                "area": "evidence-coverage",
                "item": "required release evidence missing",
                "reason": "one or more required evidence artifacts do not exist",
                "evidence": missing,
            }
        )

    return items


def next_two_hundred_todos(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    generated: list[dict[str, Any]] = []
    pending = [
        row
        for row in rows
        if row.get("status") in {"missing", "partial", "different-by-decision", "intentionally-different"}
    ]
    pending.sort(
        key=lambda row: (
            {"missing": 0, "partial": 1, "different-by-decision": 2, "intentionally-different": 2}.get(
                str(row.get("status", "missing")),
                3,
            ),
            str(row.get("group", "")),
            str(row.get("command", "")),
        )
    )
    for idx, row in enumerate(pending[:200], start=1):
        generated.append(
            {
                "id": idx,
                "command": row.get("command"),
                "group": row.get("group", "unknown"),
                "status": row.get("status"),
                "owner": row.get("owner", "unassigned"),
                "blocker": row.get("blocker", ""),
                "reason": row.get("reason", ""),
                "evidence": "artifacts/parity/command_parity_matrix.json",
            }
        )
    return generated


def main() -> int:
    source_date_epoch = subprocess.run(
        ["sh", "-lc", "printf %s \"${SOURCE_DATE_EPOCH:-}\""],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if source_date_epoch.isdigit():
        generated_at = datetime.fromtimestamp(int(source_date_epoch), tz=timezone.utc).isoformat()
    else:
        generated_at = "1970-01-01T00:00:00+00:00"
    rows = status_rows()
    current_state = read_json(CURRENT_STATE)
    plugin_state = read_json(PLUGIN_STATE)

    done, partial, left, intentionally_different = split_commands(rows)
    deferred = deferred_items(current_state, plugin_state)
    unproven = unproven_items(rows, current_state, plugin_state)
    next_todos = next_two_hundred_todos(rows)

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
    write_status(
        "what_is_unproven",
        {
            **shared,
            "summary": {"count": len(unproven), "scope": "areas that still lack release-grade proof"},
            "items": unproven,
        },
    )
    write_status(
        "next_200_todos",
        {
            **shared,
            "summary": {
                "count": len(next_todos),
                "scope": "prioritized from generated parity status data only",
            },
            "items": next_todos,
        },
    )
    left_lines = [
        "What is left",
        f"Generated at: {generated_at}",
        f"Missing commands: {len(left)}",
        f"Partial commands: {len(partial)}",
        f"Deferred items: {len(deferred)}",
        f"Unproven areas: {len(unproven)}",
        "",
        "Use these artifacts as source of truth:",
        "- artifacts/status/what_is_left.json",
        "- artifacts/status/what_is_partial.json",
        "- artifacts/status/what_is_deferred.json",
        "- artifacts/status/what_is_unproven.json",
        "- artifacts/parity/parity_dashboard.json",
    ]
    (STATUS_DIR / "what_is_left.txt").write_text("\n".join(left_lines) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
