#!/usr/bin/env python3
"""Generate routes/registry/env/contracts consistency artifacts."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def run_json(args: list[str]) -> dict:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-core", "--", *args, "--format", "json", "--no-pretty"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout or "{}")


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    out = STATUS / name
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out.relative_to(ROOT)}")


def main() -> int:
    routes = run_json(["dev", "cli", "routes"])
    registry = run_json(["dev", "cli", "registry"])
    env = run_json(["dev", "cli", "env"])
    contracts = run_json(["dev", "cli", "contracts"])
    inspect = run_json(["inspect"])

    route_roots = {
        row["segments"][0]
        for row in routes.get("routes", [])
        if isinstance(row, dict)
        and isinstance(row.get("segments"), list)
        and row["segments"]
        and isinstance(row["segments"][0], str)
    }
    inspect_roots = {
        row["segments"][0]
        for row in inspect.get("route_sources", [])
        if isinstance(row, dict)
        and isinstance(row.get("segments"), list)
        and row["segments"]
        and isinstance(row["segments"][0], str)
    }

    checks = {
        "routes_payload_present": isinstance(routes.get("routes"), list),
        "registry_payload_present": isinstance(registry.get("registry"), list),
        "env_payload_present": isinstance(env.get("source_precedence"), list),
        "contracts_payload_present": isinstance(contracts.get("contracts"), (list, dict)),
        "routes_agree_with_inspect_roots": route_roots.issubset(inspect_roots),
        "registry_has_ownership_metadata": isinstance(registry.get("ownership"), dict),
        "env_has_active_and_precedence": isinstance(env.get("active"), dict)
        and isinstance(env.get("source_precedence"), list),
        "contracts_has_schema_runtime_versions": isinstance(contracts.get("schema_version"), str)
        and isinstance(contracts.get("runtime_version"), str),
    }
    drift = [name for name, ok in checks.items() if not ok]

    write_json(
        "route_registry_env_contracts_artifact.json",
        {
            "scope": "routes/registry/env/contracts truth",
            "generator": "scripts/status/generate_dev_cli_route_registry_env_contracts_reports.py",
            "checks": checks,
            "status": "complete" if all(checks.values()) else "partial",
        },
    )
    write_json(
        "route_registry_env_contracts_drift_artifact.json",
        {
            "scope": "routes/registry/env/contracts drift",
            "generator": "scripts/status/generate_dev_cli_route_registry_env_contracts_reports.py",
            "drift_checks": drift,
            "drift_count": len(drift),
            "status": "clean" if not drift else "drift",
        },
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
