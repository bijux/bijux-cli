#!/usr/bin/env python3
"""Check deterministic generation for command tree and parity artifacts."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
PARITY = ROOT / "artifacts" / "parity"


def run(args: list[str], env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    merged = os.environ.copy()
    if env:
        merged.update(env)
    return subprocess.run(args, cwd=ROOT, check=False, capture_output=True, text=True, env=merged)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def stable_json_digest(path: Path) -> str:
    payload = json.loads(path.read_text(encoding="utf-8"))

    def scrub(value: Any) -> Any:
        if isinstance(value, dict):
            return {
                key: scrub(item)
                for key, item in value.items()
                if key not in {"generated_at", "timestamp", "created_at"}
            }
        if isinstance(value, list):
            return [scrub(item) for item in value]
        return value

    normalized = json.dumps(scrub(payload), sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(normalized.encode("utf-8")).hexdigest()


def main() -> int:
    fixed_env = {"SOURCE_DATE_EPOCH": "1"}
    checks: list[dict[str, Any]] = []
    failures: list[str] = []

    # 754: schema generation determinism gate already via snapshots; run it here explicitly.
    schema = run(["cargo", "test", "-p", "bijux-cli-contracts", "--test", "schema_snapshots"], env=fixed_env)
    checks.append(
        {
            "name": "schema_snapshots",
            "ok": schema.returncode == 0,
            "details": "cargo test -p bijux-cli-contracts --test schema_snapshots",
        }
    )
    if schema.returncode != 0:
        failures.append("schema snapshot drift")

    # 755: command tree generation determinism.
    cmd = ["cargo", "run", "-q", "-p", "bijux-cli-core", "--", "dev", "cli", "routes", "--json", "--no-pretty"]
    first = run(cmd, env=fixed_env)
    second = run(cmd, env=fixed_env)
    command_tree_ok = first.returncode == 0 and second.returncode == 0 and first.stdout == second.stdout
    checks.append(
        {
            "name": "command_tree_generation",
            "ok": command_tree_ok,
            "details": "dev cli routes --json output is byte-identical across repeated runs",
        }
    )
    if not command_tree_ok:
        failures.append("command-tree generation is not deterministic")

    # 756: parity artifact generation determinism.
    parity_gen = ["python3", "scripts/parity/generate_command_parity_matrix.py"]
    p1 = run(parity_gen, env=fixed_env)
    parity_file = PARITY / "command_parity_matrix.json"
    hash1 = stable_json_digest(parity_file) if p1.returncode == 0 and parity_file.exists() else ""
    p2 = run(parity_gen, env=fixed_env)
    hash2 = stable_json_digest(parity_file) if p2.returncode == 0 and parity_file.exists() else ""
    parity_ok = p1.returncode == 0 and p2.returncode == 0 and hash1 == hash2
    checks.append(
        {
            "name": "parity_artifact_generation",
            "ok": parity_ok,
            "details": "command_parity_matrix.json hash is stable across repeated generation",
            "hashes": [hash1, hash2],
        }
    )
    if not parity_ok:
        failures.append("parity artifact generation is not deterministic")

    parity_law_gen = ["python3", "scripts/parity/generate_command_law_reports.py"]
    l1 = run(parity_law_gen, env=fixed_env)
    law_file = PARITY / "parity_dashboard.json"
    law_hash1 = stable_json_digest(law_file) if l1.returncode == 0 and law_file.exists() else ""
    l2 = run(parity_law_gen, env=fixed_env)
    law_hash2 = stable_json_digest(law_file) if l2.returncode == 0 and law_file.exists() else ""
    law_ok = l1.returncode == 0 and l2.returncode == 0 and law_hash1 == law_hash2
    checks.append(
        {
            "name": "parity_dashboard_generation",
            "ok": law_ok,
            "details": "parity_dashboard.json hash is stable across repeated generation",
            "hashes": [law_hash1, law_hash2],
        }
    )
    if not law_ok:
        failures.append("parity dashboard generation is not deterministic")

    migration_gen = ["python3", "scripts/status/generate_command_migration_matrix.py"]
    m1 = run(migration_gen, env=fixed_env)
    migration_file = STATUS / "command_migration_matrix.json"
    migration_hash1 = (
        stable_json_digest(migration_file) if m1.returncode == 0 and migration_file.exists() else ""
    )
    m2 = run(migration_gen, env=fixed_env)
    migration_hash2 = (
        stable_json_digest(migration_file) if m2.returncode == 0 and migration_file.exists() else ""
    )
    migration_ok = m1.returncode == 0 and m2.returncode == 0 and migration_hash1 == migration_hash2
    checks.append(
        {
            "name": "command_migration_matrix_generation",
            "ok": migration_ok,
            "details": "command_migration_matrix.json hash is stable across repeated generation",
            "hashes": [migration_hash1, migration_hash2],
        }
    )
    if not migration_ok:
        failures.append("command migration matrix generation is not deterministic")

    inventory_gen = ["python3", "scripts/status/generate_command_surface_inventory.py"]
    i1 = run(inventory_gen, env=fixed_env)
    inventory_file = STATUS / "public_python_paths_still_reachable.json"
    inventory_hash1 = (
        stable_json_digest(inventory_file) if i1.returncode == 0 and inventory_file.exists() else ""
    )
    i2 = run(inventory_gen, env=fixed_env)
    inventory_hash2 = (
        stable_json_digest(inventory_file) if i2.returncode == 0 and inventory_file.exists() else ""
    )
    inventory_ok = i1.returncode == 0 and i2.returncode == 0 and inventory_hash1 == inventory_hash2
    checks.append(
        {
            "name": "command_surface_inventory_generation",
            "ok": inventory_ok,
            "details": "public_python_paths_still_reachable.json hash is stable across repeated generation",
            "hashes": [inventory_hash1, inventory_hash2],
        }
    )
    if not inventory_ok:
        failures.append("command surface inventory generation is not deterministic")

    bridge_dup_gen = ["python3", "scripts/status/generate_bridge_duplicate_law_report.py"]
    b1 = run(bridge_dup_gen, env=fixed_env)
    bridge_dup_file = STATUS / "bridge_duplicate_law_report.json"
    bridge_hash1 = (
        stable_json_digest(bridge_dup_file) if b1.returncode == 0 and bridge_dup_file.exists() else ""
    )
    b2 = run(bridge_dup_gen, env=fixed_env)
    bridge_hash2 = (
        stable_json_digest(bridge_dup_file) if b2.returncode == 0 and bridge_dup_file.exists() else ""
    )
    bridge_ok = b1.returncode == 0 and b2.returncode == 0 and bridge_hash1 == bridge_hash2
    checks.append(
        {
            "name": "bridge_duplicate_law_report_generation",
            "ok": bridge_ok,
            "details": "bridge_duplicate_law_report.json hash is stable across repeated generation",
            "hashes": [bridge_hash1, bridge_hash2],
        }
    )
    if not bridge_ok:
        failures.append("bridge duplicate-law report generation is not deterministic")

    install_neutrality_gen = ["python3", "scripts/status/generate_install_neutrality_reports.py"]
    n1 = run(install_neutrality_gen, env=fixed_env)
    neutrality_file = STATUS / "install_neutrality_report.json"
    neutrality_hash1 = (
        stable_json_digest(neutrality_file) if n1.returncode == 0 and neutrality_file.exists() else ""
    )
    n2 = run(install_neutrality_gen, env=fixed_env)
    neutrality_hash2 = (
        stable_json_digest(neutrality_file) if n2.returncode == 0 and neutrality_file.exists() else ""
    )
    neutrality_ok = (
        n1.returncode == 0 and n2.returncode == 0 and neutrality_hash1 == neutrality_hash2
    )
    checks.append(
        {
            "name": "install_neutrality_report_generation",
            "ok": neutrality_ok,
            "details": "install_neutrality_report.json hash is stable across repeated generation",
            "hashes": [neutrality_hash1, neutrality_hash2],
        }
    )
    if not neutrality_ok:
        failures.append("install neutrality report generation is not deterministic")

    STATUS.mkdir(parents=True, exist_ok=True)
    payload = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "generator": "scripts/status/check_deterministic_generation.py",
        "checks": checks,
        "ok": not failures,
        "failures": failures,
    }
    (STATUS / "deterministic_generation_report.json").write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )

    if failures:
        for item in failures:
            print(f"DETERMINISM FAILURE: {item}")
        return 1
    print("Deterministic generation checks passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
