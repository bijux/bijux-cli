#!/usr/bin/env python3
"""Generate python-bridge duplicate law audit report."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "artifacts" / "status" / "bridge_duplicate_law_report.json"
BINDINGS = ROOT / "crates" / "bijux-cli-python" / "src" / "bindings.rs"


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


def main() -> int:
    text = BINDINGS.read_text(encoding="utf-8") if BINDINGS.exists() else ""

    checks = [
        {
            "area": "routing",
            "forbidden_tokens": ["parse_intent", "RouteRegistry", "root_command(", "normalize_command_path"],
        },
        {
            "area": "exit_mapping",
            "forbidden_tokens": ["map_error_category_to_exit", "USAGE_EXIT_CODE", "INTERNAL_EXIT_CODE"],
        },
        {
            "area": "output_shaping",
            "forbidden_tokens": ["render_value(", "EmitterConfig", "render_command_help("],
        },
        {
            "area": "namespace_validation",
            "forbidden_tokens": ["is_reserved_namespace(", "register_plugin_namespace(", "validate_manifest("],
        },
    ]

    duplicates: list[dict[str, object]] = []
    for check in checks:
        hits = [token for token in check["forbidden_tokens"] if token in text]
        duplicates.append(
            {
                "area": check["area"],
                "duplicate_rules": hits,
                "count": len(hits),
            }
        )

    total = sum(item["count"] for item in duplicates)
    payload = {
        "generated_at": stable_generated_at(),
        "generator": "scripts/status/generate_bridge_duplicate_law_report.py",
        "source": "crates/bijux-cli-python/src/bindings.rs",
        "checks": duplicates,
        "summary": {
            "duplicate_rule_count": total,
            "status": "clean" if total == 0 else "duplicates-found",
        },
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
