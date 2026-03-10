#!/usr/bin/env python3
"""Generate source map for maintainer diagnostics command inputs."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)

    payload = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "generator": "scripts/status/generate_dev_cli_diagnostics_source_map.py",
        "scope": "dev-cli diagnostics source map",
        "commands": [
            {
                "command": "dev cli runtime-identity",
                "presentation_owner": "bijux-dev-cli",
                "runtime_data_sources": [
                    "bijux-cli-core::install::install_health_report",
                    "bijux-cli-core::install::cargo_install_strategy",
                    "bijux-cli-core::install::pip_install_strategy",
                ],
            },
            {
                "command": "dev cli package-health",
                "presentation_owner": "bijux-dev-cli",
                "runtime_data_sources": [
                    "artifacts/status/current_rust_state.json",
                ],
            },
            {
                "command": "dev cli state-audit",
                "presentation_owner": "bijux-dev-cli",
                "runtime_data_sources": [
                    "bijux-cli-core::state_path_status",
                    "bijux-cli-core::state_diagnostics",
                ],
            },
            {
                "command": "dev cli state-doctor",
                "presentation_owner": "bijux-dev-cli",
                "runtime_data_sources": [
                    "bijux-cli-core::state_diagnostics",
                ],
            },
        ],
    }

    out = STATUS / "dev_cli_diagnostics_source_map.json"
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
