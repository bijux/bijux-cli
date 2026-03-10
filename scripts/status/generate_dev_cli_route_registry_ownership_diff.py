#!/usr/bin/env python3
"""Generate route/registry ownership diff evidence for dev-cli extraction."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
ROUTING_REPORTS = ROOT / "crates" / "bijux-cli-routing" / "src" / "reports.rs"
CORE_APP = ROOT / "crates" / "bijux-cli" / "src" / "app.rs"
DEV_ROUTES = ROOT / "crates" / "bijux-dev-cli" / "src" / "routes.rs"
DEV_REGISTRY = ROOT / "crates" / "bijux-dev-cli" / "src" / "registry.rs"


def has_token(path: Path, token: str) -> bool:
    return token in path.read_text(encoding="utf-8")


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)
    generated_at = datetime.now(timezone.utc).isoformat()

    before = {
        "core_owned_routes_registry_presentation": has_token(CORE_APP, "routes_report(&registry)")
        or has_token(CORE_APP, "registry_report(&registry)"),
        "routing_owned_routes_registry_presentation": has_token(ROUTING_REPORTS, "pub fn routes_report")
        or has_token(ROUTING_REPORTS, "pub fn registry_report"),
    }
    after = {
        "core_delegates_routes_to_dev_cli": has_token(CORE_APP, "dev_routes::build_report"),
        "core_delegates_registry_to_dev_cli": has_token(CORE_APP, "dev_registry::build_report"),
        "dev_cli_owns_routes_presentation": has_token(DEV_ROUTES, "pub fn build_report"),
        "dev_cli_owns_registry_presentation": has_token(DEV_REGISTRY, "pub fn build_report"),
        "routing_exposes_read_only_route_inventory": has_token(
            ROOT / "crates" / "bijux-cli-routing" / "src" / "inventory.rs", "pub fn route_inventory"
        ),
        "routing_exposes_read_only_registry_inventory": has_token(
            ROOT / "crates" / "bijux-cli-routing" / "src" / "inventory.rs", "pub fn registry_inventory"
        ),
    }

    payload = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_dev_cli_route_registry_ownership_diff.py",
        "scope": "route-registry ownership shift",
        "before": before,
        "after": after,
        "summary": {
            "ownership_shift_complete": (
                not before["core_owned_routes_registry_presentation"]
                and not before["routing_owned_routes_registry_presentation"]
                and after["core_delegates_routes_to_dev_cli"]
                and after["core_delegates_registry_to_dev_cli"]
                and after["dev_cli_owns_routes_presentation"]
                and after["dev_cli_owns_registry_presentation"]
            )
        },
    }

    out = STATUS / "dev_cli_route_registry_ownership_diff.json"
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
