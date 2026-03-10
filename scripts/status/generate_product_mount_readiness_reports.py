#!/usr/bin/env python3
"""Generate official product-mount readiness artifacts."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
REGISTRY = ROOT / "docs" / "constitution" / "official_product_namespace_registry.json"
CONTRACT = ROOT / "docs" / "constitution" / "product_mount_metadata_contract.json"
CONSTANTS = ROOT / "crates" / "bijux-cli-plugin" / "src" / "constants.rs"


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
    return json.loads(path.read_text(encoding="utf-8"))


def extract_future_namespaces(constants_text: str) -> list[str]:
    marker = "pub const FUTURE_PRODUCT_NAMESPACES: &[&str]"
    idx = constants_text.find(marker)
    if idx < 0:
        return []
    chunk = constants_text[idx : idx + 400]
    return sorted(set(part.strip('"') for part in chunk.split('"') if part.strip() and part.strip() not in {",", "= OFFICIAL_PRODUCT_NAMESPACES;"} and part.strip().isidentifier()))


def write_json(name: str, payload: dict[str, Any]) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    generated_at = stable_generated_at()
    registry = read_json(REGISTRY)
    contract = read_json(CONTRACT)

    entries = registry.get("entries", [])
    namespaces = sorted(entry.get("namespace", "") for entry in entries if entry.get("namespace"))
    placeholder_entries = registry.get("placeholder_entries", [])

    support_report = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_product_mount_readiness_reports.py",
        "supports_today": [
            "reserved namespace rejection for official mounts",
            "route-tree visibility for reserved official namespaces",
            "stable metadata contract for runtime and control binaries",
            "plugin lifecycle guardrails remain independent from product runtime binaries",
        ],
        "evidence": [
            "crates/bijux-cli-plugin/tests/plugin_namespace_regression.rs",
            "crates/bijux-cli-plugin/tests/official_namespace_registry.rs",
            "crates/bijux-cli-routing/tests/route_law_consistency.rs",
            "docs/constitution/official_product_namespace_registry.json",
            "docs/constitution/product_mount_metadata_contract.json",
        ],
    }

    missing_report = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_product_mount_readiness_reports.py",
        "not_committed": [
            "dynamic product runtime loading",
            "external ABI stability guarantee for product plugins",
            "network-distributed namespace registry",
        ],
        "why_missing": "kept intentionally out to avoid speculative core complexity",
    }

    readiness = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_product_mount_readiness_reports.py",
        "official_namespaces": namespaces,
        "placeholder_entries": placeholder_entries,
        "metadata_contract": contract,
        "freeze_rule": "future-ready via metadata and tests; no speculative runtime expansion",
    }

    machine_registry = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_product_mount_readiness_reports.py",
        "registry": registry,
    }

    write_json("official_product_mount_registry.json", machine_registry)
    write_json("product_mount_readiness_report.json", readiness)
    write_json("product_mount_support_report.json", support_report)
    write_json("product_mount_gap_report.json", missing_report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
