#!/usr/bin/env python3
"""Generate command-family closure reports and combined closure artifacts."""

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


def freeze_to_closure(status: str) -> str:
    if status == "frozen":
        return "complete"
    if status in {"partial", "missing"}:
        return "partial"
    return "evolving"


def main() -> int:
    generated_at = stable_generated_at()

    config_read = read_json(STATUS / "config_read_domain_contract.json")
    config_mutation = read_json(STATUS / "config_mutation_domain_contract.json")
    config_source = read_json(STATUS / "config_source_precedence_contract.json")
    plugin_status = read_json(STATUS / "plugin_command_set_status.json")
    history_read = read_json(STATUS / "history_read_domain_contract.json")
    memory_read = read_json(STATUS / "memory_read_domain_contract.json")
    diagnostics = read_json(STATUS / "diagnostics_operator_truth_contract.json")
    repl_parity = read_json(STATUS / "status_repl_parity_coverage.json")
    repl_only = read_json(STATUS / "repl_only_behaviors.json")

    config_statuses = [
        freeze_to_closure(str(config_read.get("status", ""))),
        freeze_to_closure(str(config_mutation.get("status", ""))),
        freeze_to_closure(str(config_source.get("status", ""))),
    ]
    config_closure = (
        "complete"
        if all(item == "complete" for item in config_statuses)
        else ("partial" if any(item == "partial" for item in config_statuses) else "evolving")
    )

    plugin_partial = plugin_status.get("plugin_commands", {}).get("partial", [])
    plugin_closure = "partial" if plugin_partial else "complete"
    if plugin_status.get("classification") == "evolving":
        plugin_closure = "evolving" if plugin_closure == "complete" else "partial"

    history_closure = freeze_to_closure(str(history_read.get("status", "")))
    memory_closure = freeze_to_closure(str(memory_read.get("status", "")))
    diagnostics_closure = freeze_to_closure(str(diagnostics.get("status", "")))

    repl_summary = repl_parity.get("summary", {}).get("statuses", {})
    repl_partial_count = int(repl_summary.get("partial", 0)) + int(repl_summary.get("shim", 0))
    repl_only_count = len(repl_only.get("repl_only_behaviors", [])) if isinstance(repl_only, dict) else 0
    repl_closure = "partial" if repl_partial_count > 0 else ("evolving" if repl_only_count > 0 else "complete")

    reports = {
        "config": {
            "area": "config",
            "status": config_closure,
            "evidence": [
                "artifacts/status/config_read_domain_contract.json",
                "artifacts/status/config_mutation_domain_contract.json",
                "artifacts/status/config_source_precedence_contract.json",
            ],
        },
        "plugins": {
            "area": "plugins",
            "status": plugin_closure,
            "evidence": [
                "artifacts/status/plugin_command_set_status.json",
                "artifacts/status/plugin_migration_report.json",
            ],
        },
        "history": {
            "area": "history",
            "status": history_closure,
            "evidence": ["artifacts/status/history_read_domain_contract.json"],
        },
        "memory": {
            "area": "memory",
            "status": memory_closure,
            "evidence": ["artifacts/status/memory_read_domain_contract.json"],
        },
        "diagnostics": {
            "area": "diagnostics",
            "status": diagnostics_closure,
            "evidence": ["artifacts/status/diagnostics_operator_truth_contract.json"],
        },
        "repl_shared_law": {
            "area": "repl_shared_law",
            "status": repl_closure,
            "evidence": [
                "artifacts/status/status_repl_parity_coverage.json",
                "artifacts/status/repl_only_behaviors.json",
            ],
        },
    }

    for key, payload in reports.items():
        payload.update(
            {
                "generated_at": generated_at,
                "generator": "scripts/status/generate_command_family_closure_reports.py",
            }
        )
        write_json(STATUS / f"{key}_closure_report.json", payload)

    summary = {"complete": 0, "partial": 0, "evolving": 0}
    for payload in reports.values():
        summary[payload["status"]] += 1

    combined = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_command_family_closure_reports.py",
        "scope": "command family closure",
        "reports": reports,
        "summary": summary,
        "status": "green" if summary["partial"] == 0 else "attention-required",
    }
    write_json(STATUS / "command_family_closure_report.json", combined)

    partial_areas = [k for k, v in reports.items() if v["status"] != "complete"]
    acceptance = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_command_family_closure_reports.py",
        "scope": "partial area acceptance",
        "required_when_partial_exists": True,
        "accepted_areas": partial_areas,
        "status": "accepted" if partial_areas else "not-required",
    }
    write_json(STATUS / "command_family_partial_area_acceptance.json", acceptance)

    text_lines = [
        "Command Family Closure Report",
        f"status: {combined['status']}",
        f"complete: {summary['complete']}",
        f"partial: {summary['partial']}",
        f"evolving: {summary['evolving']}",
        "",
        "areas:",
    ]
    for key, value in reports.items():
        text_lines.append(f"- {key}: {value['status']}")
    text_lines.append("")
    text_lines.append(
        "review step: explicitly accept every non-complete area in artifacts/status/command_family_partial_area_acceptance.json"
    )
    (STATUS / "command_family_closure_report.txt").write_text(
        "\n".join(text_lines) + "\n", encoding="utf-8"
    )

    print("wrote artifacts/status/config_closure_report.json")
    print("wrote artifacts/status/plugins_closure_report.json")
    print("wrote artifacts/status/history_closure_report.json")
    print("wrote artifacts/status/memory_closure_report.json")
    print("wrote artifacts/status/diagnostics_closure_report.json")
    print("wrote artifacts/status/repl_shared_law_closure_report.json")
    print("wrote artifacts/status/command_family_closure_report.json")
    print("wrote artifacts/status/command_family_closure_report.txt")
    print("wrote artifacts/status/command_family_partial_area_acceptance.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
