#!/usr/bin/env python3
"""Generate migration notes from parity and hardening artifacts."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
PARITY = ROOT / "artifacts" / "parity"


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


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT))


def command_migration_notes(generated_at: str) -> dict[str, Any]:
    matrix = read_json(PARITY / "command_parity_matrix.json")
    rows = matrix.get("commands", []) if isinstance(matrix, dict) else []
    changed = []
    for row in rows:
        if not isinstance(row, dict):
            continue
        status = str(row.get("status", "missing"))
        if status in {"partial", "intentionally-different", "different-by-decision"}:
            changed.append(
                {
                    "command": row.get("command"),
                    "status": status,
                    "reason": row.get("reason", ""),
                    "blocker": row.get("blocker", ""),
                }
            )
    changed.sort(key=lambda item: str(item.get("command", "")))
    return {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_migration_notes.py",
        "scope": "commands",
        "tasks": [574],
        "items": changed[:250],
        "source": rel(PARITY / "command_parity_matrix.json"),
    }


def packaging_migration_notes(generated_at: str) -> dict[str, Any]:
    runtime = read_json(STATUS / "runtime_unity_report.json")
    package_health = read_json(STATUS / "package_health_report.json")
    assumptions = package_health.get("payload", {}).get("install_state_assumptions", [])
    notes = [
        {
            "area": "runtime-identity",
            "note": "verify active binary and PATH shadowing behavior before cutover",
            "evidence": rel(STATUS / "runtime_unity_report.json"),
        },
        {
            "area": "install-assumptions",
            "note": "review install-state assumptions and shell completion target paths",
            "assumptions": assumptions,
            "evidence": rel(STATUS / "package_health_report.json"),
        },
    ]
    return {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_migration_notes.py",
        "scope": "packaging",
        "tasks": [575],
        "runtime_unity_ok": bool(runtime.get("ok", False)),
        "items": notes,
    }


def plugin_migration_notes(generated_at: str) -> dict[str, Any]:
    plugin_failures = read_json(STATUS / "plugin_lifecycle_failure_injection_report.json")
    rollback = read_json(STATUS / "plugin_rollback_proof_report.json")
    notes = [
        {
            "area": "plugin-install-write-path",
            "note": "validate rollback and retry behavior before enabling new plugin capabilities",
            "evidence": [
                rel(STATUS / "plugin_lifecycle_failure_injection_report.json"),
                rel(STATUS / "plugin_rollback_proof_report.json"),
            ],
        },
        {
            "area": "plugin-runtime-diagnostics",
            "note": "verify reserved-name and registry diagnostics surface expected errors",
            "evidence": rel(STATUS / "namespace_abuse_report.json"),
        },
    ]
    return {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_migration_notes.py",
        "scope": "plugin-lifecycle",
        "tasks": [576],
        "items": notes,
        "plugin_report_status": plugin_failures.get("status", "unknown"),
        "rollback_report_status": rollback.get("status", "unknown"),
    }


def state_migration_notes(generated_at: str) -> dict[str, Any]:
    config = read_json(STATUS / "config_corruption_matrix.json")
    state = read_json(STATUS / "state_resilience_summary.json")
    guidance = read_json(STATUS / "state_recovery_guidance.json")
    notes = [
        {
            "area": "config",
            "note": "backup and validate config before mutating across runtime upgrades",
            "evidence": rel(STATUS / "config_corruption_matrix.json"),
        },
        {
            "area": "history-memory",
            "note": "run state doctor when corrupted history or memory payloads are detected",
            "evidence": rel(STATUS / "state_resilience_summary.json"),
        },
        {
            "area": "recovery",
            "note": "follow machine-readable state recovery guidance for rollback paths",
            "evidence": rel(STATUS / "state_recovery_guidance.json"),
        },
    ]
    return {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_migration_notes.py",
        "scope": "state-behavior",
        "tasks": [577],
        "items": notes,
        "config_status": config.get("status", "unknown"),
        "state_status": state.get("status", "unknown"),
        "guidance_status": guidance.get("status", "unknown"),
    }


def main() -> None:
    generated_at = stable_generated_at()

    commands = command_migration_notes(generated_at)
    packaging = packaging_migration_notes(generated_at)
    plugin = plugin_migration_notes(generated_at)
    state = state_migration_notes(generated_at)

    write_json(STATUS / "migration_notes_commands.json", commands)
    write_json(STATUS / "migration_notes_packaging.json", packaging)
    write_json(STATUS / "migration_notes_plugin_lifecycle.json", plugin)
    write_json(STATUS / "migration_notes_state_behavior.json", state)

    text = [
        "Migration Notes",
        "",
        "Commands:",
        *[
            f"- {item['command']}: status={item['status']} reason={item.get('reason','')}"
            for item in commands.get("items", [])[:40]
        ],
        "",
        "Packaging:",
        *[f"- {item['area']}: {item['note']}" for item in packaging.get("items", [])],
        "",
        "Plugin lifecycle:",
        *[f"- {item['area']}: {item['note']}" for item in plugin.get("items", [])],
        "",
        "State behavior:",
        *[f"- {item['area']}: {item['note']}" for item in state.get("items", [])],
    ]
    write_text(STATUS / "migration_notes.txt", "\n".join(text) + "\n")

    print("wrote artifacts/status/migration_notes_commands.json")
    print("wrote artifacts/status/migration_notes_packaging.json")
    print("wrote artifacts/status/migration_notes_plugin_lifecycle.json")
    print("wrote artifacts/status/migration_notes_state_behavior.json")
    print("wrote artifacts/status/migration_notes.txt")


if __name__ == "__main__":
    main()
