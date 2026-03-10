#!/usr/bin/env python3
"""Generate route-law ownership, coverage, parity, and special-case artifacts."""

from __future__ import annotations

import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
PARITY = ROOT / "artifacts" / "parity"
REGISTRY = ROOT / "crates" / "bijux-cli-routing" / "src" / "registry.rs"
PARSER = ROOT / "crates" / "bijux-cli-routing" / "src" / "parser.rs"
BASELINE = ROOT / "scripts" / "status" / "route_special_cases_baseline.json"


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


def parse_builtins() -> list[str]:
    text = REGISTRY.read_text(encoding="utf-8")
    block = text.split("let built_ins = BTreeSet::from([", 1)[1].split("]);", 1)[0]
    return sorted(set(re.findall(r'"([^"]+)"\.to_string\(\)', block)))


def parse_aliases() -> list[str]:
    text = REGISTRY.read_text(encoding="utf-8")
    block = text.split("let aliases = BTreeMap::from([", 1)[1].split("]);", 1)[0]
    return sorted(set(re.findall(r'\("([^"]+)"\.to_string\(\),', block)))


def owner_for(command: str) -> str:
    if command.startswith("dev cli ") or command.startswith("cli "):
        return "bijux-cli-core"
    if command.startswith("plugins ") or command.startswith("config "):
        return "bijux-cli-core"
    if command.startswith("history") or command.startswith("memory"):
        return "bijux-cli-core"
    return "bijux-cli-core"


def command_owner_mapping(commands: list[str]) -> list[dict[str, Any]]:
    rows = []
    for command in commands:
        rows.append({
            "command": command,
            "owner_crate": owner_for(command),
            "source": "crates/bijux-cli-core/src/app.rs",
        })
    return rows


def command_test_coverage_mapping(commands: list[str]) -> list[dict[str, Any]]:
    tests = [str(p.relative_to(ROOT)) for p in (ROOT / "crates").rglob("tests/*.rs")]
    rows = []
    for command in commands:
        token = command.replace(" ", r'"\s*\.to_string\(\),\s*"')
        matched = []
        for test in tests:
            text = (ROOT / test).read_text(encoding="utf-8")
            if command in text or re.search(token, text):
                matched.append(test)
        rows.append({
            "command": command,
            "coverage_files": sorted(set(matched))[:25],
            "coverage_count": len(set(matched)),
        })
    return rows


def command_parity_status_mapping(commands: list[str]) -> list[dict[str, Any]]:
    matrix = read_json(PARITY / "command_parity_matrix.json")
    rows = matrix.get("commands", []) if isinstance(matrix, dict) else []
    by_cmd = {
        str(row.get("command", "")): row
        for row in rows
        if isinstance(row, dict) and row.get("command")
    }
    out = []
    for command in commands:
        row = by_cmd.get(command, {})
        out.append(
            {
                "command": command,
                "status": row.get("status", "unknown"),
                "owner": row.get("owner", "unknown"),
                "blocker": row.get("blocker", ""),
                "confidence": row.get("confidence", 0.0),
            }
        )
    return out


def route_special_cases() -> dict[str, Any]:
    aliases = parse_aliases()
    parser = PARSER.read_text(encoding="utf-8")

    legacy_aliases = {
        "dev routes": "dev cli routes",
        "dev registry": "dev cli registry",
    }
    legacy_hidden = {"routes", "registry"}

    live_legacy_aliases = [alias for alias in sorted(legacy_aliases) if alias in aliases]
    live_legacy_hidden = [
        name
        for name in sorted(legacy_hidden)
        if f'Command::new("{name}").hide(true)' in parser
    ]

    baseline = read_json(BASELINE)
    baseline_count = int(baseline.get("baseline_special_case_count", 0))
    current_count = len(live_legacy_aliases) + len(live_legacy_hidden)

    return {
        "legacy_route_aliases": live_legacy_aliases,
        "legacy_hidden_dev_subcommands": live_legacy_hidden,
        "summary": {
            "special_case_count": current_count,
            "baseline_special_case_count": baseline_count,
            "delta_from_baseline": current_count - baseline_count,
        },
    }


def main() -> None:
    generated_at = stable_generated_at()
    commands = parse_builtins()

    owner = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_route_law_reports.py",
        "items": command_owner_mapping(commands),
    }
    coverage = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_route_law_reports.py",
        "items": command_test_coverage_mapping(commands),
    }
    parity = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_route_law_reports.py",
        "items": command_parity_status_mapping(commands),
    }
    special_cases = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_route_law_reports.py",
        "task": 638,
        "report": route_special_cases(),
        "rule": "special-case count must trend down over releases",
    }

    write_json(STATUS / "route_command_owner_mapping.json", owner)
    write_json(STATUS / "route_command_test_coverage_mapping.json", coverage)
    write_json(STATUS / "route_command_parity_status_mapping.json", parity)
    write_json(STATUS / "route_special_cases.json", special_cases)

    print("wrote artifacts/status/route_command_owner_mapping.json")
    print("wrote artifacts/status/route_command_test_coverage_mapping.json")
    print("wrote artifacts/status/route_command_parity_status_mapping.json")
    print("wrote artifacts/status/route_special_cases.json")


if __name__ == "__main__":
    main()
