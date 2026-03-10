#!/usr/bin/env python3
"""Enforce maintainer automation policy for routed developer commands."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MAPPINGS = [
    ("scripts/status/generate_status_reports.py", "bijux dev cli status"),
    ("scripts/parity/generate_command_law_reports.py", "bijux dev cli parity"),
    ("scripts/status/generate_route_law_reports.py", "bijux dev cli route-audit"),
    ("scripts/status/generate_state_audit_reports.py", "bijux dev cli state-audit"),
    ("scripts/status/generate_maintainer_control_plane_reports.py", "bijux dev cli script-audit"),
    ("scripts/status/generate_crate_boundary_metrics.py", "bijux dev cli crate-health"),
    ("scripts/status/generate_install_truth_reports.py", "bijux dev cli package-health"),
    ("scripts/status/generate_docs_duplication_report.py", "bijux dev cli docs-audit"),
]
CORE_APP = ROOT / "crates" / "bijux-cli-core" / "src" / "app.rs"


def main() -> int:
    core_text = CORE_APP.read_text(encoding="utf-8")

    missing_mappings: list[str] = []
    for script, command in MAPPINGS:
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
