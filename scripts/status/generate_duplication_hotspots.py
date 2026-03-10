#!/usr/bin/env python3
"""Generate duplication hotspot report for command law, exit mapping, output mapping, and state path handling."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "artifacts" / "status" / "duplication_hotspots.json"


def run_rg(pattern: str, paths: list[str]) -> list[str]:
    cmd = ["rg", "-n", pattern, *paths]
    proc = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    if proc.returncode not in (0, 1):
        return []
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]


def main() -> int:
    hotspots = [
        {
            "name": "command-name-lists",
            "description": "command name and alias lists defined in multiple crates",
            "evidence": run_rg(
                r"(normalize_command_path|DEV_CLI_SUBCOMMANDS|CLI_PLUGINS_SUBCOMMANDS|is_known_route)",
                ["crates/bijux-cli-routing/src", "crates/bijux-cli-core/src"],
            ),
            "canonical_source": "crates/bijux-cli-routing/src/catalog.rs",
        },
        {
            "name": "exit-code-mapping",
            "description": "usage/internal exit classification logic",
            "evidence": run_rg(
                r"(map_error_category_to_exit|classify_failure\()",
                ["crates/bijux-cli-core/src", "crates/bijux-cli-python/src"],
            ),
            "canonical_source": "crates/bijux-cli-core/src/kernel.rs",
        },
        {
            "name": "output-format-mapping",
            "description": "string<->enum output format conversion branches",
            "evidence": run_rg(
                r"(OutputFormat::Json|output_format_from_name|output_format_name|parse_output_format)",
                ["crates/bijux-cli-routing/src", "crates/bijux-cli-repl/src", "crates/bijux-cli-output/src"],
            ),
            "canonical_source": "crates/bijux-cli-routing/src/parser.rs",
        },
        {
            "name": "argv-positionals-and-options",
            "description": "command positional and option extraction helpers",
            "evidence": run_rg(
                r"(command_positionals\(|command_option_value\()",
                ["crates/bijux-cli-core/src"],
            ),
            "canonical_source": "crates/bijux-cli-core/src/argv.rs",
        },
        {
            "name": "plugin-namespace-reservation",
            "description": "reserved namespace checks for plugin lifecycle",
            "evidence": run_rg(
                r"(is_reserved_namespace\(|ReservedNamespace|reserved namespace)",
                ["crates/bijux-cli-plugin/src", "crates/bijux-cli-core/src"],
            ),
            "canonical_source": "crates/bijux-cli-plugin/src/constants.rs",
        },
    ]

    payload = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "scope": "duplication hotspots across rust workspace command law",
        "hotspots": hotspots,
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
