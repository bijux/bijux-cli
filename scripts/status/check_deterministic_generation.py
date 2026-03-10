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
    cmd = ["cargo", "run", "-q", "-p", "bijux-cli-bin", "--", "dev", "cli", "routes", "--json", "--no-pretty"]
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
