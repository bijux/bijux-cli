#!/usr/bin/env python3
"""Enforce maintainer automation policy for routed developer commands."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MAPPINGS = [
    ("scripts/status/generate_current_rust_state.py", "bijux dev cli status"),
    ("scripts/status/generate_crate_boundary_metrics.py", "bijux dev cli crate-health"),
    ("scripts/parity/run_rust_python_parity.py", "bijux dev cli parity"),
]
INVENTORY_SCRIPT = ROOT / "scripts" / "status" / "generate_dev_cli_inventory.py"
CORE_APP = ROOT / "crates" / "bijux-cli-core" / "src" / "app.rs"


def main() -> int:
    inventory_text = INVENTORY_SCRIPT.read_text(encoding="utf-8")
    core_text = CORE_APP.read_text(encoding="utf-8")

    missing_mappings: list[str] = []
    for script, command in MAPPINGS:
        if not (ROOT / script).exists():
            print(f"MAINTAINER POLICY FAILURE: expected script missing: {script}")
            return 1
        if script not in inventory_text or command not in inventory_text:
            missing_mappings.append(f"{script} -> {command} (inventory)")
        if script not in core_text or command not in core_text:
            missing_mappings.append(f"{script} -> {command} (core)")

    if missing_mappings:
        print("MAINTAINER POLICY FAILURE: missing dev-cli mapping declarations")
        for item in sorted(set(missing_mappings)):
            print(f" - {item}")
        return 1

    print("Maintainer automation policy check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
