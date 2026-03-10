#!/usr/bin/env python3
"""Generate release-build minimalism and reproducibility artifacts."""

from __future__ import annotations

import hashlib
import json
import subprocess
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TARGET = ROOT / "target"


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


def run(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, cwd=ROOT, check=False, capture_output=True, text=True)


def write_json(name: str, payload: dict[str, Any]) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def file_info(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {"path": str(path.relative_to(ROOT)), "exists": False}
    data = path.read_bytes()
    return {
        "path": str(path.relative_to(ROOT)),
        "exists": True,
        "size_bytes": len(data),
        "sha256": hashlib.sha256(data).hexdigest(),
    }


def cargo_tree_top() -> list[dict[str, Any]]:
    proc = run(["cargo", "tree", "-p", "bijux-cli-bin", "-e", "normal", "--prefix", "none"])
    if proc.returncode != 0:
        return []
    counter: Counter[str] = Counter()
    for raw in proc.stdout.splitlines():
        line = raw.strip()
        if not line:
            continue
        name = line.split()[0]
        if name.startswith("bijux-cli-"):
            continue
        counter[name] += 1
    return [
        {"crate": name, "hits": count}
        for name, count in counter.most_common(20)
    ]


def dependency_inventory() -> dict[str, Any]:
    proc = run(["cargo", "metadata", "--format-version", "1", "--no-deps"])
    if proc.returncode != 0:
        return {"error": proc.stderr.strip()}
    meta = json.loads(proc.stdout)
    crates = []
    for pkg in meta.get("packages", []):
        crates.append(
            {
                "name": pkg.get("name"),
                "version": pkg.get("version"),
                "manifest_path": pkg.get("manifest_path"),
            }
        )
    return {"workspace_packages": sorted(crates, key=lambda row: row["name"]) }


def license_inventory() -> dict[str, Any]:
    proc = run(["cargo", "metadata", "--format-version", "1", "--no-deps"])
    if proc.returncode != 0:
        return {"error": proc.stderr.strip()}
    meta = json.loads(proc.stdout)
    rows = []
    for pkg in meta.get("packages", []):
        rows.append(
            {
                "name": pkg.get("name"),
                "version": pkg.get("version"),
                "license": pkg.get("license") or "UNKNOWN",
            }
        )
    return {"workspace_licenses": sorted(rows, key=lambda row: row["name"]) }


def main() -> int:
    generated_at = stable_generated_at()

    release_bin = file_info(TARGET / "release" / "bijux-rs")
    debug_bin = file_info(TARGET / "debug" / "bijux-rs")

    size_top = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_release_build_reports.py",
        "top_dependency_contributors": cargo_tree_top(),
        "removed_dependencies_for_size": [
            "strsim",
            "anyhow (from bijux-cli-python)",
            "thiserror (from bijux-cli-python)",
        ],
        "disabled_default_features": [
            "clap in bijux-cli-core",
            "clap in bijux-cli-routing",
            "pyo3 in bijux-cli-python",
        ],
    }

    reproducible = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_release_build_reports.py",
        "assumptions": [
            "Cargo.lock is committed and used in CI.",
            "SOURCE_DATE_EPOCH is respected by status generators.",
            "schema snapshots and command-tree snapshots are enforced in CI.",
            "parity matrix generation is required and checked for deterministic output.",
        ],
        "non_promises": [
            "bit-for-bit reproducibility across different host toolchains is not guaranteed",
        ],
    }

    release_manifest = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_release_build_reports.py",
        "artifacts": [
            "artifacts/status/release_binary_size_report.json",
            "artifacts/status/debug_binary_size_report.json",
            "artifacts/status/release_binary_size_contributors.json",
            "artifacts/status/release_dependency_inventory.json",
            "artifacts/status/license_inventory.json",
            "artifacts/status/reproducible_build_assumptions.json",
            "artifacts/status/deterministic_generation_report.json",
            "artifacts/status/release_build_consistency_report.json",
            "artifacts/status/release_evidence_bundle.json",
            "artifacts/status/release_status_manifest.json",
        ],
    }

    write_json(
        "release_binary_size_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_release_build_reports.py",
            "binary": release_bin,
        },
    )
    write_json(
        "debug_binary_size_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_release_build_reports.py",
            "binary": debug_bin,
        },
    )
    write_json("release_binary_size_contributors.json", size_top)
    write_json(
        "release_dependency_inventory.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_release_build_reports.py",
            **dependency_inventory(),
        },
    )
    write_json(
        "license_inventory.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_release_build_reports.py",
            **license_inventory(),
        },
    )
    write_json("reproducible_build_assumptions.json", reproducible)
    write_json("release_artifact_manifest.json", release_manifest)

    print("wrote release build minimalism artifacts")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
