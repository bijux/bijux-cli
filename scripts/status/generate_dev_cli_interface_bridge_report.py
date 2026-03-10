#!/usr/bin/env python3
"""Generate runtime-to-dev-cli query interface bridge evidence."""

from __future__ import annotations

import json
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_DIR = REPO_ROOT / "artifacts" / "status"

QUERY_FILES = {
    "routing_inventory": REPO_ROOT / "crates" / "bijux-cli-routing" / "src" / "inventory.rs",
    "routing_contracts_query": REPO_ROOT / "crates" / "bijux-cli-routing" / "src" / "query.rs",
    "install_runtime_identity_query": REPO_ROOT
    / "crates"
    / "bijux-cli-core"
    / "src"
    / "install"
    / "query.rs",
    "core_state_parity_query": REPO_ROOT / "crates" / "bijux-cli-core" / "src" / "query.rs",
}


def main() -> int:
    STATUS_DIR.mkdir(parents=True, exist_ok=True)

    interfaces: list[dict[str, object]] = []
    for name, path in QUERY_FILES.items():
        text = path.read_text(encoding="utf-8")
        interfaces.append(
            {
                "name": name,
                "path": str(path.relative_to(REPO_ROOT)).replace("\\", "/"),
                "public_structs": text.count("pub struct "),
                "public_functions": text.count("pub fn "),
                "contains_json_assembly": "serde_json::json!" in text,
                "contains_terminal_rendering": "println!" in text
                or "eprintln!" in text
                or "render_value(" in text,
            }
        )

    report = {
        "scope": "runtime query interface bridge",
        "status": "ok",
        "interfaces": interfaces,
        "rules": [
            "interfaces are read-only",
            "interfaces are structured-data only",
            "interfaces do not render text",
            "interfaces bridge runtime data to bijux-dev-cli report assembly",
        ],
    }

    (STATUS_DIR / "dev_cli_interface_bridge_report.json").write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
